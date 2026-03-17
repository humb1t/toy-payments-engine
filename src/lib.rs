use rayon::prelude::*;

pub use crate::{
    account::Account,
    context::{Shard, ShardedState, State},
    transaction::Transaction,
};
use crate::{
    difference::{CanCalculateDifference, CanCalculateDisputeDifference, CanCalculateResolveChargebackDifference},
    validation::{
        CanValidateChargeback, CanValidateDeposit, CanValidateDispute, CanValidateResolve, CanValidateWithdrawal,
        ValidationContext,
    },
};

pub mod account;
pub mod context;
pub mod difference;
pub mod errors;
pub mod transaction;
pub mod validation;

pub mod prelude {
    pub use crate::{
        ClientId, TransactionId,
        account::Account,
        context::{Shard, ShardedState, State},
        errors::Error,
        transaction::{Balance, Chargeback, Deposit, Dispute, Resolve, Transaction, TransactionIterator, Withdrawal},
    };
}

pub type ClientId = u16;
pub type TransactionId = u32;

pub fn process_transactions(
    reader: impl Iterator<Item = Result<Transaction, errors::Error>>,
    state: &mut State,
) -> Result<(), errors::Error> {
    for transaction in reader {
        apply_transaction(state, transaction?)?;
    }
    Ok(())
}

pub fn process_transactions_parallel(
    transactions: Vec<Transaction>,
    shard_count: usize,
) -> Result<State, errors::Error> {
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
        .collect::<Result<Vec<_>, errors::Error>>()?;
    Ok(sharded_state.merge_into_state())
}

fn apply_transaction(state: &mut State, transaction: Transaction) -> Result<(), errors::Error> {
    let client_id = transaction.client();
    let account = state
        .accounts
        .entry(client_id)
        .or_insert_with(|| Account::new(client_id));
    if account.locked {
        return Err(errors::Error::State);
    }
    let validation_ctx = ValidationContext {
        transactions: &state.transactions,
        disputed_transactions: &state.disputed_transactions,
        account,
    };
    // TODO: using CGP - extract generic validation here.
    match transaction {
        Transaction::Deposit(deposit_tx) => {
            validation_ctx.validate_deposit(&deposit_tx)?;
            let difference = deposit_tx.difference();
            state
                .transactions
                .insert((deposit_tx.client, deposit_tx.tx), difference);
            account.apply(difference);
        },
        Transaction::Withdrawal(withdrawal_tx) => {
            validation_ctx.validate_withdrawal(&withdrawal_tx)?;
            let difference = withdrawal_tx.difference();
            state
                .transactions
                .insert((withdrawal_tx.client, withdrawal_tx.tx), difference);
            account.apply(difference);
        },
        Transaction::Dispute(dispute_tx) => {
            validation_ctx.validate_dispute(&dispute_tx)?;
            if let Some(original_difference) = state.transactions.get(&(dispute_tx.client, dispute_tx.tx)) {
                let dispute_difference = dispute_tx.dispute_difference(original_difference);
                state
                    .disputed_transactions
                    .insert((dispute_tx.client, dispute_tx.tx), dispute_difference);
                account.apply(dispute_difference);
            }
        },
        Transaction::Resolve(resolve_tx) => {
            validation_ctx.validate_resolve(&resolve_tx)?;
            if let Some(original_difference) = state.transactions.get(&(resolve_tx.client, resolve_tx.tx))
                && let Some(dispute_difference) = state.disputed_transactions.get(&(resolve_tx.client, resolve_tx.tx))
            {
                let resolve_difference =
                    resolve_tx.resolve_chargeback_difference(original_difference, dispute_difference);
                account.apply(resolve_difference);
            }
        },
        Transaction::Chargeback(chargeback_tx) => {
            validation_ctx.validate_chargeback(&chargeback_tx)?;
            if let Some(original_difference) = state.transactions.get(&(chargeback_tx.client, chargeback_tx.tx))
                && let Some(dispute_difference) = state
                    .disputed_transactions
                    .get(&(chargeback_tx.client, chargeback_tx.tx))
            {
                let chargeback_difference =
                    chargeback_tx.resolve_chargeback_difference(original_difference, dispute_difference);
                account.apply(chargeback_difference);
            }
        },
    }
    // TODO: using CGP - extract difference application here.
    Ok(())
}
