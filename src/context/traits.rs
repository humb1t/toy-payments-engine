use std::collections::HashMap;

use cgp::prelude::*;

use crate::{
    account::{Account, ClientId},
    difference::Difference,
    transaction::TransactionId,
};

#[cgp_auto_getter]
pub trait HasAccountMut {
    fn account_mut(&mut self) -> &mut Account;
}

#[cgp_auto_getter]
pub trait HasTransactionsMut {
    fn transactions_mut(&mut self) -> &mut HashMap<(ClientId, TransactionId), Difference>;
}

#[cgp_auto_getter]
pub trait HasDisputedTransactionsMut {
    fn disputed_transactions_mut(&mut self) -> &mut HashMap<(ClientId, TransactionId), Difference>;
}
