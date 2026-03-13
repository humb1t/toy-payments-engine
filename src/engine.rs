use std::{collections::HashMap, str::FromStr};

use redb::Table;
use redb_model::Model;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{errors::ProgrammerError, *};

mod db;

/// Main engine logic.
/// As enum variant types are not in Rust yet, we use simple pattern match to
/// choose strategy based on transaction type.
/// All in one function to do not spread small main business logic into separate
/// spaces, as we assume no more transaction types will come in future.
///
/// Some assumptions that were made:
/// - we fail in cases of big issues, like negative amounts or debit/credit conflicts;
/// - we less agressive against smaller issues, but my understanding may differ from real project requirements, for example I decided not to fail in case of transaction duplicate and just skip action if deposit or withdraw transaction with this ID already took place.
pub fn process_transaction(
    transaction: &Transaction,
    accounts: &mut HashMap<ClientId, Account>,
    all_transactions: &mut Table<
        '_,
        <engine::TransactionDetails as redb_model::ModelExt>::RedbKey,
        <engine::TransactionDetails as redb_model::ModelExt>::RedbValue,
    >,
    disputed_transactions: &mut Table<
        '_,
        <engine::DisputedTransactionDetails as redb_model::ModelExt>::RedbKey,
        <engine::DisputedTransactionDetails as redb_model::ModelExt>::RedbValue,
    >,
) -> Result<()> {
    if transaction
        .amount
        .is_some_and(|decimal| decimal.is_sign_negative() || decimal.is_zero())
    {
        return Err(Error::Input);
    }
    match transaction.transaction_type {
        TransactionType::Deposit => {
            let amount = transaction.amount.ok_or(Error::Input)?;
            if db::load_transaction(transaction.tx, all_transactions)?.is_some() {
                return Err(Error::Input);
            } else {
                let account = accounts
                    .entry(transaction.client)
                    .or_insert_with(|| Account::new(transaction.client));
                if account.locked {
                    return Err(Error::Input);
                }
                account.update_available(amount)?;
                db::store_transaction(transaction, all_transactions)?;
            }
        }
        TransactionType::Withdrawal => {
            let amount = transaction.amount.ok_or(Error::Input)?;
            if db::load_transaction(transaction.tx, all_transactions)?.is_some() {
                return Err(Error::Input);
            } else {
                let account = accounts.get_mut(&transaction.client).ok_or(Error::Input)?;
                if account.locked {
                    return Err(Error::Input);
                }
                if account.available < amount {
                    return Err(Error::Input);
                }
                account.update_available(-amount)?;
                db::store_transaction(transaction, all_transactions)?;
            }
        }
        TransactionType::Dispute => {
            if db::load_disputed_transaction(transaction.tx, disputed_transactions)?.is_some() {
                return Err(Error::Input);
            } else {
                let tx_details =
                    db::load_transaction(transaction.tx, all_transactions)?.ok_or(Error::Input)?;
                if transaction.client != tx_details.client {
                    return Err(Error::Input);
                }
                let account = accounts.get_mut(&transaction.client).ok_or(Error::Input)?;
                if account.locked {
                    return Err(Error::Input);
                }
                let dispute_amount = tx_details.amount;
                // Can go negative, see `test_dispute_with_already_withdrawn_funds`
                account.update_available(-dispute_amount)?;
                account.update_held(dispute_amount)?;
                db::store_disputed_transaction(disputed_transactions, tx_details)?;
            }
        }
        TransactionType::Resolve => {
            let tx_details = db::load_disputed_transaction(transaction.tx, disputed_transactions)?
                .ok_or(Error::Input)?;
            if transaction.client != tx_details.client {
                return Err(Error::Input);
            }
            let account = accounts.get_mut(&transaction.client).ok_or(Error::Input)?;
            if account.locked {
                return Err(Error::Input);
            }
            if account.held < tx_details.amount {
                return Err(Error::State);
            }
            account.update_held(-tx_details.amount)?;
            account.update_available(tx_details.amount)?;
            db::remove_disputed_transaction(transaction.tx, disputed_transactions)?;
        }
        TransactionType::Chargeback => {
            let tx_details = db::load_disputed_transaction(transaction.tx, disputed_transactions)?
                .ok_or(Error::Input)?;
            if transaction.client != tx_details.client {
                return Err(Error::Input);
            }
            let account = accounts.get_mut(&transaction.client).ok_or(Error::Input)?;
            if account.locked {
                return Err(Error::Input);
            }
            if account.held < tx_details.amount {
                return Err(Error::State);
            }
            account.update_held(-tx_details.amount)?;
            account.locked = true;
            db::remove_disputed_transaction(transaction.tx, disputed_transactions)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Clone, Model)]
#[model(impl_ext)]
pub struct TransactionDetails {
    #[entry(position = "key")]
    transaction_id: TransactionId,
    #[entry(position = "value")]
    client: ClientId,
    /// Usage of [`Decimal`] is not optimal, but gives additional security in calculations.
    #[entry(
        position = "value",
        redb_type = "String",
        from = "Decimal::from_str(&amount).expect(\"migration should be done with care\")",
        into = "amount.to_string()"
    )]
    amount: Amount,
}

#[derive(Debug, Clone, Model)]
#[model(impl_ext)]
pub struct DisputedTransactionDetails {
    #[entry(position = "key")]
    transaction_id: TransactionId,
    #[entry(position = "value")]
    client: ClientId,
    /// Usage of [`Decimal`] is not optimal, but gives additional security in calculations.
    #[entry(
        position = "value",
        redb_type = "String",
        from = "Decimal::from_str(&amount).expect(\"migration should be done with care\")",
        into = "amount.to_string()"
    )]
    amount: Amount,
}

impl TryFrom<&Transaction> for TransactionDetails {
    type Error = Error;

    fn try_from(value: &Transaction) -> StdResult<Self, Self::Error> {
        Ok(TransactionDetails {
            transaction_id: value.tx,
            client: value.client,
            amount: value
                .amount
                .ok_or(ProgrammerError::WrongCallForConvertation)?,
        })
    }
}

impl From<TransactionDetails> for DisputedTransactionDetails {
    fn from(value: TransactionDetails) -> Self {
        Self {
            transaction_id: value.transaction_id,
            client: value.client,
            amount: value.amount,
        }
    }
}

impl Account {
    fn new(client: ClientId) -> Self {
        Account {
            client,
            available: Decimal::ZERO,
            held: Decimal::ZERO,
            total: Decimal::ZERO,
            locked: false,
        }
    }

    fn update_available(&mut self, amount: Amount) -> Result<()> {
        self.available = self
            .available
            .checked_add(amount)
            .ok_or(Error::Calculation)?;
        self.total = self
            .available
            .checked_add(self.held)
            .ok_or(Error::Calculation)?;
        Ok(())
    }

    fn update_held(&mut self, amount: Amount) -> Result<()> {
        self.held = self.held.checked_add(amount).ok_or(Error::Calculation)?;
        self.total = self
            .available
            .checked_add(self.held)
            .ok_or(Error::Calculation)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use std::str::FromStr;

    use csv::Reader;
    use rust_decimal::Decimal;

    use crate::Transaction;

    #[test]
    fn given_amount_has_4_point_precision_when_read_then_no_errors() {
        Decimal::from_str(&Decimal::new(1000000000000000001, 4).to_string()).unwrap();
        assert_eq!(
            Decimal::new(1000000000000000001, 4),
            Reader::from_reader(
                "type,client,tx,amount\ndeposit,1,1,100000000000000.0001".as_bytes()
            )
            .deserialize::<Transaction>()
            .next()
            .unwrap()
            .unwrap()
            .amount
            .unwrap()
        );
    }

    #[test]
    fn to_string_and_back() {
        Decimal::from_str(&Decimal::new(1000000000000000001, 4).to_string()).unwrap();
    }

    #[test]
    fn decimals_go_below_zero() {
        assert_eq!(
            Decimal::new(-5, 1),
            Decimal::new(10, 1)
                .checked_add(-Decimal::new(15, 1))
                .unwrap()
        );
    }
}
