use cgp::prelude::*;
use std::collections::HashMap;

use crate::account::Account;
use crate::difference::Difference;
use crate::{ClientId, TransactionId};

#[derive(HasField)]
pub struct State {
    pub accounts: HashMap<ClientId, Account>,
    pub transactions: HashMap<(ClientId, TransactionId), Difference>,
    pub disputed_transactions: HashMap<(ClientId, TransactionId), Difference>,
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            transactions: HashMap::new(),
            disputed_transactions: HashMap::new(),
        }
    }
}
