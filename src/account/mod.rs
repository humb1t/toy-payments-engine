use primitive_fixed_point_decimal::fpdec;
use serde::Serialize;

use crate::{difference::Difference, transaction::Balance};

pub type ClientId = u16;

#[derive(Debug, Clone, Serialize)]
pub struct Account {
    pub client: u16,
    pub available: Balance,
    pub held: Balance,
    pub total: Balance,
    pub locked: bool,
}

impl Account {
    pub fn new(client: u16) -> Self {
        Self {
            client,
            available: fpdec!(0),
            held: fpdec!(0),
            total: fpdec!(0),
            locked: false,
        }
    }

    pub fn apply(&mut self, difference: Difference) {
        self.available += difference.available;
        self.held += difference.held;
        self.total = self.available + self.held;
        self.locked = difference.lock;
    }
}
