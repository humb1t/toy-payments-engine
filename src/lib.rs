//! Main API for `toy-payments-engine`.
//!
//! The toy payments engine processes transactions from CSV input and maintains account balances.
//! [`process_transactions`] in combination with [`State::new`] is all you need to start working.

use std::{collections::HashMap, io, result::Result as StdResult};

use redb::Database;
use redb_model::Model;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::{
    engine::{DisputedTransactionDetails, TransactionDetails, TransactionType},
    errors::Error,
};

pub mod prelude {
    pub use crate::{
        Account, Amount, ClientId, State, TransactionId, errors::Error as ToyPaymentsEngineError,
    };
}

/// Internal implementation and engine logic. Stay private.
mod engine;
pub mod errors;

/// Type alias for client ID
pub type ClientId = u16;
/// Type alias for transaction ID  
pub type TransactionId = u32;
/// Type alias for monetary amounts
pub type Amount = Decimal;
/// Result type for operations
type Result<T> = StdResult<T, Error>;

/// Process a stream of transactions from CSV reader.
///
/// This function reads all transactions from the input iterator, validates them,
/// and applies them to update account balances stored in memory. All updates are
/// persisted to the database during each transaction processing step.
pub fn process_transactions<E>(
    reader: impl Iterator<Item = StdResult<Transaction, E>>,
    state: &mut State,
) -> Result<()>
where
    E: Into<io::Error>,
{
    let database_transaction = state.database.begin_write().map_err(redb::Error::from)?;
    for result in reader {
        let transaction: Transaction =
            result.map_err(|error| Error::TransactionRead(error.into()))?;
        let mut all_transactions = database_transaction
            .open_table(TransactionDetails::DEFINITION)
            .map_err(redb::Error::from)?;
        let mut disputed_transactions = database_transaction
            .open_table(DisputedTransactionDetails::DEFINITION)
            .map_err(redb::Error::from)?;
        engine::process_transaction(
            &transaction,
            &mut state.accounts,
            &mut all_transactions,
            &mut disputed_transactions,
        )?;
    }
    database_transaction.commit().map_err(redb::Error::from)?;
    Ok(())
}

/// Engine state for maintaining account information.
///
/// This struct holds the current account balances and database connection
/// required for processing transactions.
pub struct State {
    /// Map of client accounts by client ID
    pub accounts: HashMap<ClientId, Account>,
    /// Database connection for persistent storage
    database: Database,
}

impl State {
    /// Create a new state with the given database.
    ///
    /// Note: This implementation does not persist account information across restarts.
    /// All account data is maintained only in memory during operation.
    pub fn new(database: Database) -> Self {
        Self {
            database,
            accounts: HashMap::new(),
        }
    }
}

/// Transaction type definition for CSV parsing.
///
/// This struct represents a single transaction from the input CSV file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    client: ClientId,
    tx: TransactionId,
    amount: Option<Amount>,
}

/// Account information structure.
///
/// Stores balance information for a single client including available funds,
/// held funds (in dispute), total balance, and lock status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    client: ClientId,
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}
