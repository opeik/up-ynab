use std::str::FromStr;

use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime};
use color_eyre::eyre::{Context, ContextCompat, Result};
use money2::{Currency, Money};
use ynab_client::models::TransactionClearedStatus;

use crate::{Account, NewYnabTransaction, UpTransaction, YnabBudget, YnabTransaction};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Expense { to: Account, from_name: String },
    Transfer { to: Account, from: Account },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Transaction {
    pub id: String,
    pub time: DateTime<FixedOffset>,
    pub amount: Money,
    pub msg: Option<String>,
    pub kind: Kind,
}

impl Transaction {
    pub fn to(&self) -> &Account {
        match &self.kind {
            Kind::Expense { to, from_name: _ } => to,
            Kind::Transfer { to, from: _ } => to,
        }
    }

    pub fn to_name(&self) -> &str {
        match &self.kind {
            Kind::Expense { to, from_name: _ } => &to.name,
            Kind::Transfer { to, from: _ } => &to.name,
        }
    }

    pub fn from(&self) -> Option<&Account> {
        match &self.kind {
            Kind::Expense {
                to: _,
                from_name: _,
            } => None,
            Kind::Transfer { to: _, from } => Some(from),
        }
    }

    pub fn from_name(&self) -> &str {
        match &self.kind {
            Kind::Expense { to: _, from_name } => from_name,
            Kind::Transfer { to: _, from } => &from.name,
        }
    }

    pub fn is_round_up(&self) -> bool {
        self.msg
            .as_ref()
            .map(|x| x == "Round Up")
            .unwrap_or_default()
    }

    pub fn is_transfer(&self) -> bool {
        match &self.kind {
            Kind::Expense {
                to: _,
                from_name: _,
            } => false,
            Kind::Transfer { to: _, from: _ } => true,
        }
    }

    pub fn is_expense(&self) -> bool {
        match &self.kind {
            Kind::Expense {
                to: _,
                from_name: _,
            } => true,
            Kind::Transfer { to: _, from: _ } => false,
        }
    }

    pub fn from_up(value: UpTransaction, accounts: &[Account]) -> Result<Self> {
        let to_id =
            Some(value.relationships.account.data.id.as_str()).wrap_err("missing `to` account")?;
        let from_id = value
            .relationships
            .transfer_account
            .data
            .as_ref()
            .map(|transfer_account| transfer_account.id.as_str());

        let to = accounts
            .iter()
            .find(|account| account.up_id == to_id)
            .map(|account| account.to_owned())
            .wrap_err(format!("failed to match incoming up account: `{to_id}`",))?;

        let from = match from_id {
            Some(from_id) => Some(
                accounts
                    .iter()
                    .find(|account| account.up_id == from_id)
                    .map(|account| account.to_owned())
                    .wrap_err(format!("failed to match incoming up account: `{from_id}`",))?,
            ),
            None => None,
        };

        let kind = if let Some(from) = from {
            Kind::Transfer { to, from }
        } else {
            Kind::Expense {
                to,
                from_name: value.attributes.description.clone(),
            }
        };

        let msg = match kind {
            Kind::Expense {
                to: _,
                from_name: _,
            } => value.attributes.message,
            Kind::Transfer { to: _, from: _ } => Some(value.attributes.description),
        };

        let mut amount = Money::new(
            i64::from(value.attributes.amount.value_in_base_units),
            2,
            Currency::from_str(&value.attributes.amount.currency_code)?,
        );

        if let Some(cashback) = value.attributes.cashback {
            amount = amount
                .checked_add(Money::new(
                    i64::from(cashback.amount.value_in_base_units),
                    2,
                    Currency::from_str(&cashback.amount.currency_code)?,
                ))
                .wrap_err("failed to add cashback amount")?;
        };

        Ok(Self {
            id: value.id,
            amount,
            msg,
            kind,
            time: DateTime::parse_from_rfc3339(&value.attributes.created_at)?,
        })
    }

    pub fn from_ynab(
        value: YnabTransaction,
        budget: &YnabBudget,
        accounts: &[Account],
    ) -> Result<Self> {
        let to = accounts
            .iter()
            .find(|account| account.ynab_id == value.account_id)
            .map(|account| account.to_owned())
            .wrap_err("failed to match incoming ynab account")?;

        let from = match &value.transfer_account_id {
            Some(Some(transfer_account)) => Some(
                accounts
                    .iter()
                    .find(|account| account.ynab_id == *transfer_account)
                    .map(|account| account.to_owned())
                    .wrap_err("failed to match outgoing ynab account")?,
            ),
            _ => None,
        };

        let kind = if let Some(from) = from {
            Kind::Transfer { to, from }
        } else {
            Kind::Expense {
                to,
                from_name: value
                    .payee_name
                    .clone()
                    .wrap_err("missing payee name")?
                    .wrap_err("missing payee name")?,
            }
        };

        let msg = match &kind {
            Kind::Expense {
                to: _,
                from_name: _,
            } => value.memo,
            Kind::Transfer { to: _, from: _ } => value.memo,
        }
        .wrap_err("missing memo")?;

        let amount = Money::new(
            value.amount / 10,
            2,
            Currency::from_str(
                &budget
                    .currency_format
                    .as_ref()
                    .wrap_err("missing currency format")?
                    .as_ref()
                    .wrap_err("missing currency format")?
                    .iso_code,
            )?,
        );

        Ok(Self {
            id: value.id,
            amount,
            msg,
            kind,
            time: NaiveDate::parse_from_str(&value.date, "%Y-%m-%d")?
                .and_time(NaiveTime::MIN)
                .and_utc()
                .into(),
        })
    }

    pub fn to_ynab(&self) -> Result<NewYnabTransaction> {
        let amount = i64::try_from(self.amount.amount.mantissa() * 10)
            .wrap_err("failed to convert amount")?;

        let mut transaction = NewYnabTransaction {
            date: Some(self.time.to_rfc3339()),
            amount: Some(amount),
            memo: self.msg.clone().map(Some),
            cleared: Some(TransactionClearedStatus::Cleared),
            approved: Some(true),
            account_id: None,
            payee_id: None,
            payee_name: None,
            category_id: None,
            flag_color: None,
            import_id: None,
            subtransactions: None,
        };

        match &self.kind {
            Kind::Expense { to, from_name } => {
                transaction.account_id = Some(to.ynab_id);
                transaction.payee_name = Some(Some(from_name.clone()));
            }
            Kind::Transfer { to, from } => {
                transaction.account_id = Some(to.ynab_id);
                transaction.payee_id = Some(Some(from.ynab_transfer_id));
            }
        }

        Ok(transaction)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::{Account, NewYnabTransaction, UpTransaction};

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
    fn up_expense() -> Result<()> {
        let payload = fs::read_to_string("test/data/up_expense.json")?;
        let up_transaction = serde_json::from_str::<UpTransaction>(&payload)?;
        let accounts = accounts();
        let actual = Transaction::from_up(up_transaction, &accounts)?;
        let expected = Transaction {
            id: "5ce7c223-0188-4b68-8d19-227a7cc3464d".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-02T13:44:15+11:00")?,
            amount: Money::new(-57_84, 2, Currency::Aud),
            kind: Kind::Expense {
                to: spending_account(),
                from_name: "7-Eleven".to_string(),
            },
            msg: None,
        };

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn up_income() -> Result<()> {
        let payload = fs::read_to_string("test/data/up_income.json")?;
        let up_transaction = serde_json::from_str::<UpTransaction>(&payload)?;
        let accounts = accounts();
        let actual = Transaction::from_up(up_transaction, &accounts)?;
        let expected = Transaction {
            id: "9f08959d-51d2-43a8-a45a-154373870094".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-27T05:08:06+11:00")?,
            amount: Money::new(10_95, 2, Currency::Aud),
            kind: Kind::Expense {
                to: spending_account(),
                from_name: "Z KIDD-SMITH".to_string(),
            },
            msg: Some("pizza".to_string()),
        };

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn up_transfer() -> Result<()> {
        let payload = fs::read_to_string("test/data/up_transfer.json")?;
        let up_transaction = serde_json::from_str::<UpTransaction>(&payload)?;
        let accounts = accounts();
        let actual = Transaction::from_up(up_transaction, &accounts)?;
        let expected = Transaction {
            id: "f1b6981f-94d2-42b6-9cae-304dae08a480".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-07T22:35:56+11:00")?,
            amount: Money::new(37_94, 2, Currency::Aud),
            kind: Kind::Transfer {
                to: spending_account(),
                from: home_account(),
            },
            msg: Some("Transfer from Home".to_string()),
        };

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn up_transfer_invalid_account_id() -> Result<()> {
        let payload = fs::read_to_string("test/data/up_transfer_invalid_account_id.json")?;
        let up_transaction = serde_json::from_str::<UpTransaction>(&payload)?;
        let accounts = accounts();
        let transaction = Transaction::from_up(up_transaction, &accounts);
        assert!(transaction.is_err());

        Ok(())
    }

    #[test]
    fn up_transfer_invalid_transfer_account_id() -> Result<()> {
        let payload = fs::read_to_string("test/data/up_transfer_invalid_transfer_account_id.json")?;
        let up_transaction = serde_json::from_str::<UpTransaction>(&payload)?;
        let accounts = accounts();
        let transaction = Transaction::from_up(up_transaction, &accounts);
        assert!(transaction.is_err());

        Ok(())
    }

    #[test]
    fn up_round_up() -> Result<()> {
        let payload = fs::read_to_string("test/data/up_round_up.json")?;
        let up_transaction = serde_json::from_str::<UpTransaction>(&payload)?;
        let accounts = accounts();
        let actual = Transaction::from_up(up_transaction, &accounts)?;
        let expected = Transaction {
            id: "a0f9976c-d0ac-4cef-afd6-91bbc0033730".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-28T22:49:40+11:00")?,
            amount: Money::new(-12_99, 2, Currency::Aud),
            kind: Kind::Expense {
                to: spending_account(),
                from_name: "Amazon".to_string(),
            },
            msg: None,
        };

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn up_round_up_transfer() -> Result<()> {
        let payload = fs::read_to_string("test/data/up_round_up_transfer.json")?;
        let up_transaction = serde_json::from_str::<UpTransaction>(&payload)?;
        let accounts = accounts();
        let actual = Transaction::from_up(up_transaction, &accounts)?;
        let expected = Transaction {
            id: "66e3f7f3-e766-4095-adbb-19f3e1271646".to_string(),
            time: DateTime::parse_from_rfc3339("2023-08-03T13:07:33+10:00")?,
            amount: Money::new(1_00, 2, Currency::Aud),
            kind: Kind::Transfer {
                to: home_account(),
                from: spending_account(),
            },
            msg: Some("Round Up".to_string()),
        };

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn to_ynab_expense() -> Result<()> {
        let expected = NewYnabTransaction {
            account_id: Some(spending_account().ynab_id),
            date: Some("2023-12-02T13:44:15+11:00".to_string()),
            amount: Some(-57_840),
            payee_name: Some(Some("7-Eleven".to_string())),
            cleared: Some(TransactionClearedStatus::Cleared),
            payee_id: None,
            category_id: None,
            memo: None,
            approved: Some(true),
            flag_color: None,
            import_id: None,
            subtransactions: None,
        };

        let actual = Transaction {
            id: "hi".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-02T13:44:15+11:00")?,
            amount: Money::new(-57_84, 2, Currency::Aud),
            kind: Kind::Expense {
                to: spending_account(),
                from_name: "7-Eleven".to_string(),
            },
            msg: None,
        }
        .to_ynab()?;

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn to_ynab_transfer() -> Result<()> {
        let expected = NewYnabTransaction {
            account_id: Some(spending_account().ynab_id),
            date: Some("2023-12-07T22:35:56+11:00".to_string()),
            amount: Some(37_940),
            payee_id: Some(Some(home_account().ynab_transfer_id)),
            cleared: Some(TransactionClearedStatus::Cleared),
            memo: Some(Some("Transfer from Home".to_string())),
            payee_name: None,
            category_id: None,
            approved: Some(true),
            flag_color: None,
            import_id: None,
            subtransactions: None,
        };

        let actual = Transaction {
            id: "hi".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-07T22:35:56+11:00")?,
            amount: Money::new(37_94, 2, Currency::Aud),
            kind: Kind::Transfer {
                to: spending_account(),
                from: home_account(),
            },
            msg: Some("Transfer from Home".to_string()),
        }
        .to_ynab()?;

        assert_eq!(expected, actual);
        Ok(())
    }
}
