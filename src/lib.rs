use crate::{
    account::Account,
    context::State,
    difference::{
        CanCalculateDifference, CanCalculateDisputeDifference,
        CanCalculateResolveChargebackDifference,
    },
    transaction::Transaction,
    validation::{
        CanValidateChargeback, CanValidateDeposit, CanValidateDispute, CanValidateResolve,
        CanValidateWithdrawal, ValidationContext,
    },
};

pub mod account;
pub mod context;
pub mod difference;
pub mod errors;
pub mod transaction;
pub mod validation;

pub mod prelude {
    pub use crate::account::Account;
    pub use crate::context::State;
    pub use crate::errors::Error;
    pub use crate::transaction::{Transaction, TransactionIterator};
    pub use crate::{ClientId, TransactionId};
}

pub type ClientId = u16;
pub type TransactionId = u32;

pub fn process_transactions(
    reader: impl Iterator<Item = Result<Transaction, errors::Error>>,
    state: &mut State,
) -> Result<(), errors::Error> {
    for transaction in reader {
        let transaction = transaction?;
        let client_id = match &transaction {
            Transaction::Deposit(d) => d.client,
            Transaction::Withdrawal(w) => w.client,
            Transaction::Dispute(d) => d.client,
            Transaction::Resolve(r) => r.client,
            Transaction::Chargeback(c) => c.client,
        };

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

        match transaction {
            Transaction::Deposit(d) => {
                validation_ctx.validate_deposit(&d)?;
                let difference = d.difference();
                state.transactions.insert((d.client, d.tx), difference);
                account.apply(difference);
            }
            Transaction::Withdrawal(w) => {
                validation_ctx.validate_withdrawal(&w)?;
                let difference = w.difference();
                state.transactions.insert((w.client, w.tx), difference);
                account.apply(difference);
            }
            Transaction::Dispute(d) => {
                validation_ctx.validate_dispute(&d)?;
                if let Some(original_difference) = state.transactions.get(&(d.client, d.tx)) {
                    let dispute_difference = d.dispute_difference(original_difference);
                    state
                        .disputed_transactions
                        .insert((d.client, d.tx), dispute_difference);
                    account.apply(dispute_difference);
                }
            }
            Transaction::Resolve(r) => {
                validation_ctx.validate_resolve(&r)?;
                if let Some(original_difference) = state.transactions.get(&(r.client, r.tx))
                    && let Some(dispute_difference) =
                        state.disputed_transactions.get(&(r.client, r.tx))
                {
                    let resolve_difference =
                        r.resolve_chargeback_difference(original_difference, dispute_difference);
                    account.apply(resolve_difference);
                }
            }
            Transaction::Chargeback(c) => {
                validation_ctx.validate_chargeback(&c)?;
                if let Some(original_difference) = state.transactions.get(&(c.client, c.tx))
                    && let Some(dispute_difference) =
                        state.disputed_transactions.get(&(c.client, c.tx))
                {
                    let chargeback_difference =
                        c.resolve_chargeback_difference(original_difference, dispute_difference);
                    account.apply(chargeback_difference);
                }
            }
        };
    }
    Ok(())
}
