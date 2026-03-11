//! Place for isolated database interactions.

use redb::{ReadableTable, Table};
use redb_model::ModelExt;

use crate::*;

pub fn load_transaction(
    transaction_id: TransactionId,
    all_transactions: &mut Table<'_, u32, (u16, String)>,
) -> Result<Option<TransactionDetails>> {
    Ok(all_transactions
        .get(&transaction_id)
        .map_err(redb::Error::from)?
        .map(|guard| TransactionDetails::from_key_and_guard((transaction_id, &guard))))
}

pub fn store_transaction(
    transaction: &Transaction,
    all_transactions: &mut Table<'_, u32, (u16, String)>,
) -> Result<()> {
    let transaction_details = TransactionDetails::try_from(transaction)?;
    let (k, v) = transaction_details.as_key_and_value();
    all_transactions.insert(k, v).map_err(redb::Error::from)?;
    Ok(())
}

pub fn load_disputed_transaction(
    transaction_id: TransactionId,
    disputed_transactions: &mut Table<'_, u32, (u16, String)>,
) -> Result<Option<DisputedTransactionDetails>> {
    let maybe_dipute = disputed_transactions
        .get(&transaction_id)
        .map_err(redb::Error::from)?
        .map(|guard| DisputedTransactionDetails::from_key_and_guard((transaction_id, &guard)));
    Ok(maybe_dipute)
}

pub fn store_disputed_transaction(
    disputed_transactions: &mut Table<'_, u32, (u16, String)>,
    tx_details: TransactionDetails,
) -> Result<()> {
    let (k, v) = tx_details.as_key_and_value();
    disputed_transactions
        .insert(k, v)
        .map_err(redb::Error::from)?;
    Ok(())
}

pub fn remove_disputed_transaction(
    transaction_id: TransactionId,
    disputed_transactions: &mut Table<'_, u32, (u16, String)>,
) -> Result<()> {
    disputed_transactions
        .remove(&transaction_id)
        .map_err(redb::Error::from)?;
    Ok(())
}
