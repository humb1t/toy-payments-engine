use rayon::prelude::*;

pub use crate::{
    account::Account,
    context::{Shard, ShardedState, State},
    transaction::Transaction,
};
use crate::{context::TransactionContext, errors::Error, transaction::CanProcessTransaction};

pub mod account;
pub mod context;
pub mod difference;
pub mod errors;
pub mod transaction;
pub mod validation;

pub fn process_transactions(
    reader: impl Iterator<Item = Result<Transaction, Error>>,
    state: &mut State,
) -> Result<(), Error> {
    for transaction in reader {
        apply_transaction(state, transaction?)?;
    }
    Ok(())
}

pub fn process_transactions_parallel(transactions: Vec<Transaction>, shard_count: usize) -> Result<State, Error> {
    let mut sharded_state = ShardedState::new(shard_count);
    let mut shards_transactions: Vec<Vec<Transaction>> = (0..shard_count).map(|_| Vec::new()).collect();
    for transaction in transactions {
        let shard_id = sharded_state.shard_index(transaction.client());
        shards_transactions[shard_id].push(transaction);
    }
    sharded_state.shards = sharded_state
        .shards
        .into_par_iter()
        .zip(shards_transactions.into_par_iter())
        .map(|(mut shard, transactions)| {
            for transaction in transactions {
                apply_transaction(shard.as_mut(), transaction)?;
            }
            Ok(shard)
        })
        .collect::<Result<Vec<_>, Error>>()?;
    Ok(sharded_state.merge_into_state())
}

fn apply_transaction(state: &mut State, transaction: Transaction) -> Result<(), Error> {
    let client_id = transaction.client();
    let account = state
        .accounts
        .entry(client_id)
        .or_insert_with(|| Account::new(client_id));
    if account.locked {
        return Err(Error::State);
    }
    let mut ctx = TransactionContext {
        account,
        transactions: &mut state.transactions,
        disputed_transactions: &mut state.disputed_transactions,
    };
    match &transaction {
        Transaction::Deposit(tx) => ctx.process_transaction(tx),
        Transaction::Withdrawal(tx) => ctx.process_transaction(tx),
        Transaction::Dispute(tx) => ctx.process_transaction(tx),
        Transaction::Resolve(tx) => ctx.process_transaction(tx),
        Transaction::Chargeback(tx) => ctx.process_transaction(tx),
    }
}
