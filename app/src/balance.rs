use std::{collections::BTreeMap, fmt, fs::File, path::Path};

use color_eyre::eyre::Result;
use indoc::{formatdoc, writedoc};
use itertools::Itertools;
use money2::Money;

use crate::{transaction, Account, Transaction};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Balance<'a> {
    pub values: BTreeMap<Account, Money>,
    pub transaction: &'a Transaction,
}

pub fn running_balance(transactions: &[Transaction]) -> Vec<Balance> {
    let mut balances = Vec::<Balance>::new();
    let transactions = transactions
        .iter()
        .sorted_by(|a, b| Ord::cmp(&a.time, &b.time));

    for transaction in transactions {
        let last_balance = balances.last().cloned().unwrap_or(Balance {
            values: BTreeMap::new(),
            transaction,
        });
        let mut new_values = last_balance.values;

        let to = match &transaction.kind {
            transaction::Kind::Expense { to, from_name: _ } => to,
            transaction::Kind::Transfer { to, from } => {
                new_values
                    .entry(from.clone())
                    .and_modify(|x| *x -= transaction.amount)
                    .or_insert(transaction.amount);
                to
            }
        };

        new_values
            .entry(to.clone())
            .and_modify(|x| *x += transaction.amount)
            .or_insert(transaction.amount);

        let new_balance = Balance {
            values: new_values.clone(),
            transaction,
        };

        balances.push(new_balance);
    }

    balances
}

pub fn write_balance_csv<P: AsRef<Path>>(balances: &[Balance], path: P) -> Result<()> {
    let accounts = balances
        .last()
        .cloned()
        .unwrap()
        .values
        .into_keys()
        .sorted_by(|a, b| Ord::cmp(&a.name, &b.name))
        .collect::<Vec<_>>();
    let accounts_str = accounts.iter().map(|x| x.name.clone()).collect::<Vec<_>>();
    let headers = ["time", "id", "amount", "msg", "kind", "to", "from"]
        .into_iter()
        .map(|x| x.to_owned())
        .chain(accounts_str)
        .collect::<Vec<_>>();

    let mut wtr = csv::Writer::from_writer(File::create(path.as_ref())?);
    wtr.write_record(headers)?;

    for balance in balances {
        let time = Some(balance.transaction.time.to_rfc3339());
        let id = Some(balance.transaction.id.clone());
        let amount = Some(balance.transaction.amount.to_string());
        let msg = balance.transaction.msg.clone();
        let kind = Some(
            match &balance.transaction.kind {
                transaction::Kind::Expense {
                    to: _,
                    from_name: _,
                } => "expense",
                transaction::Kind::Transfer { to: _, from: _ } => "transfer",
            }
            .to_owned(),
        );

        let to = Some(balance.transaction.to().name.clone());
        let from = Some(balance.transaction.from_name().to_owned());

        let account_balances = &accounts
            .iter()
            .map(|k| balance.values.get(k).map(|x| x.amount.to_string()))
            .collect::<Vec<_>>();

        let row = [time, id, amount, msg, kind, to, from]
            .into_iter()
            .chain(account_balances.clone())
            .map(|x| match x {
                None => "".to_string(),
                Some(x) => x,
            })
            .collect::<Vec<_>>();
        wtr.write_record(&row)?;
    }

    wtr.flush()?;
    Ok(())
}

impl<'a> fmt::Display for Balance<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let accounts = self
            .values
            .iter()
            .map(|(k, v)| format!(" • {}: {}", k.name, v.amount))
            .join("\n");

        let kind = match &self.transaction.kind {
            transaction::Kind::Expense {
                to: _,
                from_name: _,
            } => "expense",
            transaction::Kind::Transfer { to: _, from: _ } => "transfer",
        };

        let time = self.transaction.time.to_rfc3339();
        let amount = self.transaction.amount;
        let to = self.transaction.to_name();
        let from = self.transaction.from_name();

        let transaction = match self.transaction.msg.as_deref() {
            Some(x) => formatdoc! {"
                • amount: {amount}
                • kind: {kind}
                • msg: {x}
                • {to} →  {from}"
            },
            None => formatdoc! {"
                • amount: {amount}
                • kind: {kind}
                • {to} →  {from}"
            },
        };

        writedoc! {
            f,
            "
            Balance at {time}:
            Transaction:
            {transaction}
            Accounts:
            {accounts}"
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, str::FromStr};

    use money2::Currency;
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::{normalize_up_transactions, Account, UpTransaction};

    fn spending_account() -> Account {
        Account {
            name: "Spending".to_owned(),
            up_id: "2be1c9de-7a89-4e8f-8077-f535150b588d".to_owned(),
            ynab_id: Uuid::from_str("f6ca888b-327a-45d0-9775-830abdaa3a04").unwrap(),
            ynab_transfer_id: Uuid::from_str("89ddd9ef-2510-4b42-a889-e7a68cae291c").unwrap(),
        }
    }

    fn home_account() -> Account {
        Account {
            name: "Home".to_owned(),
            up_id: "328160b1-d7bc-41ee-9d7b-c7da4f2484b0".to_owned(),
            ynab_id: Uuid::from_str("2b00a77e-9b3c-4277-9c6c-6944f7696705").unwrap(),
            ynab_transfer_id: Uuid::from_str("f9b0b92f-70f7-4015-b885-4e5807a78a44").unwrap(),
        }
    }

    fn accounts() -> Vec<Account> {
        Vec::from([home_account(), spending_account()])
    }

    #[test]
    fn up_round_up_balance() -> Result<()> {
        let payload = fs::read_to_string("test/data/up_round_up_balance.json")?;
        let accounts = accounts();
        let up_transactions = serde_json::from_str::<Vec<UpTransaction>>(&payload)?;
        let transactions = normalize_up_transactions(&up_transactions, &accounts)?;

        let actual = running_balance(&transactions);
        let expected = Vec::from([
            Balance {
                values: BTreeMap::from([(
                    spending_account(),
                    Money::new(50_00, 2, Currency::from_str("AUD")?),
                )]),
                transaction: &transactions[0],
            },
            Balance {
                values: BTreeMap::from([(
                    spending_account(),
                    Money::new(20_00, 2, Currency::from_str("AUD")?),
                )]),
                transaction: &transactions[1],
            },
            Balance {
                values: BTreeMap::from([
                    (
                        spending_account(),
                        Money::new(19_00, 2, Currency::from_str("AUD")?),
                    ),
                    (
                        home_account(),
                        Money::new(1_00, 2, Currency::from_str("AUD")?),
                    ),
                ]),
                transaction: &transactions[2],
            },
        ]);

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn up_transfer_balance() -> Result<()> {
        let payload = fs::read_to_string("test/data/up_transfer_balance.json")?;
        let accounts = accounts();
        let up_transactions = serde_json::from_str::<Vec<UpTransaction>>(&payload)?;
        let transactions = normalize_up_transactions(&up_transactions, &accounts)?;

        let actual = running_balance(&transactions);
        let expected = Vec::from([
            Balance {
                values: BTreeMap::from([(
                    spending_account(),
                    Money::new(90000, 2, Currency::from_str("AUD")?),
                )]),
                transaction: &transactions[0],
            },
            Balance {
                values: BTreeMap::from([
                    (
                        spending_account(),
                        Money::new(40000, 2, Currency::from_str("AUD")?),
                    ),
                    (
                        home_account(),
                        Money::new(50000, 2, Currency::from_str("AUD")?),
                    ),
                ]),
                transaction: &transactions[1],
            },
        ]);

        assert_eq!(expected, actual);
        Ok(())
    }
}
