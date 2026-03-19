use std::collections::HashMap;

use cgp::prelude::*;
use derive_more::{AsMut, AsRef, Into};
pub use traits::*;

use crate::{
    account::{Account, ClientId},
    difference::Difference,
    transaction::{ProcessTransactionComponent, TransactionId, TransactionProcessors},
    validation::{
        ChargebackValidatorComponent,
        ChargebackValidatorProvider,
        DepositValidatorComponent,
        DepositValidatorProvider,
        DisputeValidatorComponent,
        DisputeValidatorProvider,
        HasAccount,
        HasDisputedTransactions,
        HasTransactions,
        ResolveValidatorComponent,
        ResolveValidatorProvider,
        WithdrawalValidatorComponent,
        WithdrawalValidatorProvider,
    },
};

mod traits;

delegate_components! {
    TransactionContext<'_> {
        // Validation providers
        DepositValidatorComponent: DepositValidatorProvider,
        WithdrawalValidatorComponent: WithdrawalValidatorProvider,
        DisputeValidatorComponent: DisputeValidatorProvider,
        ResolveValidatorComponent: ResolveValidatorProvider,
        ChargebackValidatorComponent: ChargebackValidatorProvider,

        // Transaction processing
        ProcessTransactionComponent: UseDelegate<TransactionProcessors>,
    }
}

#[derive(HasField, Default)]
pub struct State {
    pub accounts: HashMap<ClientId, Account>,
    pub transactions: HashMap<(ClientId, TransactionId), Difference>,
    pub disputed_transactions: HashMap<(ClientId, TransactionId), Difference>,
}

#[derive(Default, Into, AsRef, AsMut, HasField)]
pub struct Shard(State);

pub struct ShardedState {
    pub shards: Vec<Shard>,
    shard_count: usize,
}

impl ShardedState {
    pub fn new(shard_count: usize) -> Self {
        let shards = (0..shard_count).map(|_| Shard::default()).collect();
        Self { shards, shard_count }
    }

    pub fn shard_index(&self, client: ClientId) -> usize {
        client as usize % self.shard_count
    }

    pub fn merge_into_state(self) -> State {
        let mut state = State::default();
        for shard in self.shards {
            state.accounts.extend(shard.0.accounts);
            state.transactions.extend(shard.0.transactions);
            state.disputed_transactions.extend(shard.0.disputed_transactions);
        }
        state
    }
}

pub struct TransactionContext<'a> {
    pub account: &'a mut Account,
    pub transactions: &'a mut HashMap<(ClientId, TransactionId), Difference>,
    pub disputed_transactions: &'a mut HashMap<(ClientId, TransactionId), Difference>,
}

impl<'a> HasAccount for TransactionContext<'a> {
    fn account(&self) -> &Account {
        self.account
    }
}

impl<'a> HasAccountMut for TransactionContext<'a> {
    fn account_mut(&mut self) -> &mut Account {
        self.account
    }
}

impl<'a> HasTransactionsMut for TransactionContext<'a> {
    fn transactions_mut(&mut self) -> &mut HashMap<(ClientId, TransactionId), Difference> {
        self.transactions
    }
}

impl<'a> HasDisputedTransactionsMut for TransactionContext<'a> {
    fn disputed_transactions_mut(&mut self) -> &mut HashMap<(ClientId, TransactionId), Difference> {
        self.disputed_transactions
    }
}

impl<'a> HasTransactions for TransactionContext<'a> {
    fn transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference> {
        self.transactions
    }
}

impl<'a> HasDisputedTransactions for TransactionContext<'a> {
    fn disputed_transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference> {
        self.disputed_transactions
    }
}
