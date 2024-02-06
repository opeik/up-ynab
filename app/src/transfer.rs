use chrono::Duration;
use color_eyre::eyre::{eyre, Result};
use itertools::Itertools;
use tracing::warn;

use crate::Transaction;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferPair<'a> {
    pub to: &'a Transaction,
    pub from: &'a Transaction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferMatches<'a> {
    pub matched: Vec<TransferPair<'a>>,
    pub unmatched: Vec<&'a Transaction>,
}

const TO_PREFIXES: &[&str] = &[
    "Transfer to ",
    "Auto Transfer to ",
    "Cover to ",
    "Quick save transfer to ",
    "Forward to ",
];

const FROM_PREFIXES: &[&str] = &[
    "Transfer from ",
    "Auto Transfer from ",
    "Cover from ",
    "Quick save transfer from ",
    "Forward from ",
];

/// Matches tranfer transaction pairs together.
///
/// Unfortunately, transfers (including round ups) don't contain a reference to its other half.
/// So, we have to guess. Fun!
pub fn match_transfers(transfers: &[Transaction]) -> Result<TransferMatches<'_>> {
    let mut matched = Vec::new();
    let mut unmatched = Vec::new();

    let groups = transfers
        .iter()
        .into_group_map_by(|x| x.amount.amount.abs());

    // TODO: make less gross
    for group in groups {
        let (mut tos, mut froms) = group
            .1
            .into_iter()
            .map(Some)
            .partition::<Vec<_>, _>(|x| x.as_deref().unwrap().amount.amount.is_sign_negative());

        let mut pairs = Vec::new();
        for to in &mut tos {
            for from in &mut froms {
                if let Some(to_inner) = to
                    && let Some(from_inner) = from
                    && let Some(pair) = match_transfer_pair(to_inner, from_inner)
                {
                    pairs.push(pair);
                    *to = None;
                    *from = None;
                }
            }
        }

        let mut remainder = tos.into_iter().chain(froms).flatten().collect::<Vec<_>>();
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

fn match_transfer_pair<'a>(to: &'a Transaction, from: &'a Transaction) -> Option<TransferPair<'a>> {
    if let Some(to_msg) = to.msg.as_deref()
        && let Some(from_msg) = from.msg.as_deref()
        && let Some(to_prefix_index) = TO_PREFIXES
            .iter()
            .position(|to_prefix| to_msg.starts_with(*to_prefix))
        && (to.time - from.time).abs() <= Duration::seconds(15)
    {
        // Normally, transfer transactions messages follow a naming convention. If they're behaving
        // this should suffice.
        if let Some(from_prefix_index) = FROM_PREFIXES
            .iter()
            .position(|prefix| from_msg.starts_with(*prefix))
            && to_prefix_index == from_prefix_index
        {
            return Some(TransferPair { to, from });
        }

        // For *some reason* roundups very rarely *are* given "to" and "from" transactions. But
        // *unlike* regular transfers, the "from" transfer doesn't match the usual convention.
        if to.is_round_up() || from.is_round_up() {
            warn!("found sussy roundup `{}` in matched transfer pair", &to.id);
            return Some(TransferPair { to, from });
        }
    }

    None
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
