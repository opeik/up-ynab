use chrono::Duration;
use color_eyre::eyre::{eyre, ContextCompat, Result};
use itertools::Itertools;
use tracing::warn;

use crate::Transaction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferPair {
    pub to: Transaction,
    pub from: Transaction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferMatches {
    pub matched: Vec<TransferPair>,
    pub unmatched: Vec<Transaction>,
}
/// Matches tranfer transaction pairs together.
///
/// Unfortunately, transfers (including round ups) don't contain a reference to its other half.
/// So, we have to guess. Fun!
pub fn match_transfers(transfers: &[Transaction]) -> Result<TransferMatches> {
    let mut matched = Vec::new();
    let mut unmatched = Vec::new();

    let groups = transfers
        .iter()
        .into_group_map_by(|x| x.amount.amount.abs());

    let to_prefixes = [
        "Transfer to ",
        "Auto Transfer to ",
        "Cover to ",
        "Quick save transfer to ",
        "Forward to ",
    ];

    let from_prefixes = [
        "Transfer from ",
        "Auto Transfer from ",
        "Cover from ",
        "Quick save transfer from ",
        "Forward from ",
    ];

    // TODO: make less gross
    for group in &groups {
        let (mut tos, mut froms) = group
            .1
            .iter()
            .map(|x| Some(*x))
            .partition::<Vec<Option<&Transaction>>, _>(|x| {
                x.unwrap().amount.amount.is_sign_negative()
            });

        let mut pairs = Vec::new();
        for to in &mut tos {
            for from in &mut froms {
                if let Some(to_inner) = to
                    && let Some(from_inner) = from
                    && let Some(to_msg) = &to_inner.msg
                    && let Some(from_msg) = &from_inner.msg
                    && let Some(to_prefix_index) = to_prefixes
                        .iter()
                        .position(|prefix| to_msg.starts_with(*prefix))
                {
                    let d = (from_inner.time - to_inner.time).abs();
                    let from_prefix_index = from_prefixes
                        .iter()
                        .position(|prefix| from_msg.starts_with(*prefix));

                    if d > Duration::seconds(15) {
                        continue;
                    }

                    if let Some(from_prefix_index) = from_prefix_index
                        && to_prefix_index == from_prefix_index
                    {
                        pairs.push(TransferPair {
                            to: to_inner.clone(),
                            from: from_inner.clone(),
                        });
                        *to = None;
                        *from = None;
                    } else if to_inner.is_round_up() || from_inner.is_round_up() {
                        // For *some reason* roundups very rarely *are* given "to" and "from"
                        // transactions. But *unlike* regular transfers, the "from" transfer
                        // doesn't match the usual convention. Thanks to this phenomenon, we have
                        // to fix the "from" transfer message so it's not treated as a roundup
                        // later.
                        warn!(
                            "found sussy roundup `{}` in matched transfer pair, fixing...",
                            &to_inner.id
                        );
                        let mut x = from_inner.clone();
                        x.msg = to_inner
                            .msg
                            .as_deref()
                            .and_then(|x| x.strip_prefix(to_prefixes[to_prefix_index]))
                            .map(|x| format!("{}{x}", from_prefixes[to_prefix_index]));

                        pairs.push(TransferPair {
                            to: to_inner.clone(),
                            from: x,
                        });
                        *to = None;
                        *from = None;
                    };
                }
            }
        }

        let mut remainder = tos
            .into_iter()
            .chain(froms.into_iter())
            .flatten()
            .cloned()
            .collect::<Vec<_>>();

        matched.append(&mut pairs);
        unmatched.append(&mut remainder);
    }

    if ((matched.len() * 2) + unmatched.len()) != transfers.len() {
        return Err(eyre!(
            "expected {} total, found {} matched and {} unmatched",
            transfers.len(),
            matched.len() * 2,
            unmatched.len()
        ));
    }

    Ok(TransferMatches { matched, unmatched })
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::DateTime;
    use money2::{Currency, Money};
    use pretty_assertions::assert_eq;
    use uuid::Uuid;

    use super::*;
    use crate::{transaction::Kind, Account};

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

    #[test]
    fn match_transfers_valid() -> Result<()> {
        let from = Transaction {
            id: "from".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-02T13:44:15+11:00")?,
            amount: Money::new(57_84, 2, Currency::Aud),
            kind: Kind::Transfer {
                to: spending_account(),
                from: home_account(),
            },
            msg: "Transfer from Spending".to_string().into(),
        };
        let to = Transaction {
            id: "to".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-02T13:44:30+11:00")?,
            amount: Money::new(-57_84, 2, Currency::Aud),
            kind: Kind::Transfer {
                to: home_account(),
                from: spending_account(),
            },
            msg: "Transfer to Home".to_string().into(),
        };
        let transactions = [to.clone(), from.clone()];
        let actual = match_transfers(&transactions)?;
        let expected = TransferMatches {
            matched: Vec::from([TransferPair { to, from }]),
            unmatched: Vec::new(),
        };

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn match_transfers_dangling() -> Result<()> {
        let from = Transaction {
            id: "from".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-02T13:44:15+11:00")?,
            amount: Money::new(57_84, 2, Currency::Aud),
            kind: Kind::Transfer {
                to: spending_account(),
                from: home_account(),
            },
            msg: "Transfer from Spending".to_string().into(),
        };

        let to = Transaction {
            id: "to".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-02T13:44:17+11:00")?,
            amount: Money::new(-57_84, 2, Currency::Aud),
            kind: Kind::Transfer {
                to: home_account(),
                from: spending_account(),
            },
            msg: "Transfer to Home".to_string().into(),
        };

        let unrelated = Transaction {
            id: "unrelated".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-02T13:44:16+11:00")?,
            amount: Money::new(-14_23, 2, Currency::Aud),
            kind: Kind::Expense {
                to: home_account(),
                from_name: "Wowee".to_string(),
            },
            msg: "Unrelated transaction".to_string().into(),
        };

        let transactions = [from.clone(), unrelated.clone(), to.clone()];
        let actual = match_transfers(&transactions)?;
        let expected = TransferMatches {
            matched: Vec::from([TransferPair { to, from }]),
            unmatched: Vec::from([unrelated]),
        };

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn match_transfers_too_new() -> Result<()> {
        let from = Transaction {
            id: "from".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-02T13:44:30+11:00")?,
            amount: Money::new(57_84, 2, Currency::Aud),
            kind: Kind::Transfer {
                to: spending_account(),
                from: home_account(),
            },
            msg: "Transfer from Spending".to_string().into(),
        };

        let to = Transaction {
            id: "to".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-02T13:44:14+11:00")?,
            amount: Money::new(-57_84, 2, Currency::Aud),
            kind: Kind::Transfer {
                to: home_account(),
                from: spending_account(),
            },
            msg: "Transfer to Home".to_string().into(),
        };

        let transactions = [from.clone(), to.clone()];
        let actual = match_transfers(&transactions)?;
        let expected = TransferMatches {
            matched: Vec::new(),
            unmatched: Vec::from([to, from]),
        };

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn match_transfers_too_old() -> Result<()> {
        let from = Transaction {
            id: "from".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-02T13:44:30+11:00")?,
            amount: Money::new(57_84, 2, Currency::Aud),
            kind: Kind::Transfer {
                to: spending_account(),
                from: home_account(),
            },
            msg: "Transfer from Spending".to_string().into(),
        };

        let to = Transaction {
            id: "to".to_string(),
            time: DateTime::parse_from_rfc3339("2023-12-02T13:44:46+11:00")?,
            amount: Money::new(-57_84, 2, Currency::Aud),
            kind: Kind::Transfer {
                to: home_account(),
                from: spending_account(),
            },
            msg: "Transfer to Home".to_string().into(),
        };

        let transactions = [from.clone(), to.clone()];
        let actual = match_transfers(&transactions)?;
        let expected = TransferMatches {
            matched: Vec::new(),
            unmatched: Vec::from([to, from]),
        };

        assert_eq!(expected, actual);
        Ok(())
    }

    #[test]
    fn match_transfers_empty() -> Result<()> {
        let actual = match_transfers(&[])?;
        let expected = TransferMatches {
            matched: Vec::new(),
            unmatched: Vec::new(),
        };

        assert_eq!(expected, actual);
        Ok(())
    }
}
