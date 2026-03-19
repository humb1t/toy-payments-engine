mod providers;

use std::collections::HashMap;

use cgp::prelude::*;
pub use providers::*;

use crate::{
    account::ClientId,
    difference::Difference,
    transaction::{Chargeback, Deposit, Dispute, Resolve, TransactionId, Withdrawal},
};

#[cgp_auto_getter]
pub trait HasTransactions {
    fn transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference>;
}

#[cgp_auto_getter]
pub trait HasDisputedTransactions {
    fn disputed_transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference>;
}

#[cgp_auto_getter]
pub trait HasAccount {
    fn account(&self) -> &crate::account::Account;
}

#[cgp_component(DepositValidator)]
pub trait CanValidateDeposit {
    fn validate_deposit(&self, deposit: &Deposit) -> Result<(), crate::errors::Error>;
}

#[cgp_component(WithdrawalValidator)]
pub trait CanValidateWithdrawal {
    fn validate_withdrawal(&self, withdrawal: &Withdrawal) -> Result<(), crate::errors::Error>;
}

#[cgp_component(DisputeValidator)]
pub trait CanValidateDispute {
    fn validate_dispute(&self, dispute: &Dispute) -> Result<(), crate::errors::Error>;
}

#[cgp_component(ResolveValidator)]
pub trait CanValidateResolve {
    fn validate_resolve(&self, resolve: &Resolve) -> Result<(), crate::errors::Error>;
}

#[cgp_component(ChargebackValidator)]
pub trait CanValidateChargeback {
    fn validate_chargeback(&self, chargeback: &Chargeback) -> Result<(), crate::errors::Error>;
}
