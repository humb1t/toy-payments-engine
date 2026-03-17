use std::collections::HashMap;

use cgp::prelude::*;
use derive_more::{AsMut, AsRef, Into};

use crate::{ClientId, TransactionId, account::Account, difference::Difference};

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
