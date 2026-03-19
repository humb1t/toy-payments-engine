pub use cgp::prelude::*;
pub use derive_more::Constructor;
pub use primitive_fixed_point_decimal::fpdec;

pub use crate::transaction::{Balance, Chargeback, Deposit, Dispute, Resolve, Withdrawal};

#[derive(Clone, Copy, Constructor, Debug, PartialEq)]
pub struct Difference {
    pub available: Balance,
    pub held: Balance,
    pub lock: bool,
}

#[cgp_component(TransactionDifference)]
pub trait CanCalculateDifference {
    fn difference(&self) -> Difference;
}

#[cgp_component(DisputeDifference)]
pub trait CanCalculateDisputeDifference {
    fn dispute_difference(&self, original_difference: &Difference) -> Difference;
}

#[cgp_component(ResolveChargebackDifference)]
pub trait CanCalculateResolveChargebackDifference {
    fn resolve_chargeback_difference(
        &self,
        original_difference: &Difference,
        dispute_difference: &Difference,
    ) -> Difference;
}

#[cgp_impl(new DepositProvider)]
impl TransactionDifference for Deposit {
    fn difference(&self) -> Difference {
        Difference::new(self.amount, fpdec!(0.0), false)
    }
}

#[cgp_impl(new WithdrawalProvider)]
impl TransactionDifference for Withdrawal {
    fn difference(&self) -> Difference {
        Difference::new(-self.amount, fpdec!(0.0), false)
    }
}

#[cgp_impl(new DisputeProvider)]
impl DisputeDifference for Dispute {
    fn dispute_difference(&self, original_difference: &Difference) -> Difference {
        Difference {
            available: -original_difference.available,
            held: original_difference.available,
            lock: false,
        }
    }
}

#[cgp_impl(new ResolveProvider)]
impl ResolveChargebackDifference for Resolve {
    fn resolve_chargeback_difference(
        &self,
        _original_difference: &Difference,
        dispute_difference: &Difference,
    ) -> Difference {
        Difference {
            available: dispute_difference.held,
            held: -dispute_difference.held,
            lock: false,
        }
    }
}

#[cgp_impl(new ChargebackProvider)]
impl ResolveChargebackDifference for Chargeback {
    fn resolve_chargeback_difference(
        &self,
        _original_difference: &Difference,
        dispute_difference: &Difference,
    ) -> Difference {
        Difference {
            available: fpdec!(0),
            held: -dispute_difference.held,
            lock: true,
        }
    }
}

delegate_components! {
    Deposit {
        TransactionDifferenceComponent: DepositProvider,
    }
}

delegate_components! {
    Withdrawal {
        TransactionDifferenceComponent: WithdrawalProvider,
    }
}

delegate_components! {
    Dispute {
        DisputeDifferenceComponent: DisputeProvider,
    }
}

delegate_components! {
    Resolve {
        ResolveChargebackDifferenceComponent: ResolveProvider,
    }
}

delegate_components! {
    Chargeback {
        ResolveChargebackDifferenceComponent: ChargebackProvider,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deposit_difference() {
        let deposit = Deposit::new(1, 100, fpdec!(100.0));
        let diff = deposit.difference();
        assert_eq!(diff.available, fpdec!(100.0));
        assert_eq!(diff.held, fpdec!(0.0));
        assert!(!diff.lock);
    }

    #[test]
    fn test_withdrawal_difference() {
        let withdrawal = Withdrawal::new(1, 100, fpdec!(50.0));
        let diff = withdrawal.difference();
        assert_eq!(diff.available, fpdec!(-50.0));
        assert_eq!(diff.held, fpdec!(0.0));
        assert!(!diff.lock);
    }

    #[test]
    fn test_dispute_difference() {
        let original_diff = Difference::new(fpdec!(100.0), fpdec!(0.0), false);
        let dispute = Dispute::new(1, 100);
        let diff = dispute.dispute_difference(&original_diff);
        assert_eq!(diff.available, fpdec!(-100.0));
        assert_eq!(diff.held, fpdec!(100.0));
        assert!(!diff.lock);
    }

    #[test]
    fn test_resolve_difference() {
        let original_diff = Difference::new(fpdec!(100.0), fpdec!(0.0), false);
        let dispute_diff = Difference::new(fpdec!(0.0), fpdec!(100.0), false);
        let resolve = Resolve::new(1, 1);
        let diff = resolve.resolve_chargeback_difference(&original_diff, &dispute_diff);
        assert_eq!(diff.available, fpdec!(100.0));
        assert_eq!(diff.held, fpdec!(-100.0));
        assert!(!diff.lock);
    }

    #[test]
    fn test_chargeback_difference() {
        let original_diff = Difference::new(fpdec!(10.0), fpdec!(0.0), false);
        let dispute_diff = Difference::new(fpdec!(0.0), fpdec!(10.0), false);
        let chargeback = Chargeback::new(1, 1);
        let diff = chargeback.resolve_chargeback_difference(&original_diff, &dispute_diff);
        assert_eq!(diff.available, fpdec!(0.0));
        assert_eq!(diff.held, fpdec!(-10.0));
        assert!(diff.lock);
    }
}
