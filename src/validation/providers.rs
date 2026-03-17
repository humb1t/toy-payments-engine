use cgp::prelude::*;

use super::*;

#[cgp_impl(new DepositValidatorProvider)]
impl DepositValidator
where
    Self: HasTransactions,
{
    fn validate_deposit(&self, deposit: &Deposit) -> Result<(), crate::errors::Error> {
        if self.transactions().contains_key(&(deposit.client, deposit.tx)) {
            return Err(crate::errors::Error::State);
        }
        if !deposit.amount.is_pos() {
            return Err(crate::errors::Error::Input);
        }
        Ok(())
    }
}

#[cgp_impl(new WithdrawalValidatorProvider)]
impl WithdrawalValidator
where
    Self: HasTransactions + HasAccount,
{
    fn validate_withdrawal(&self, withdrawal: &Withdrawal) -> Result<(), crate::errors::Error> {
        if self.transactions().contains_key(&(withdrawal.client, withdrawal.tx)) {
            return Err(crate::errors::Error::State);
        }
        if self.account().available < withdrawal.amount {
            return Err(crate::errors::Error::Input);
        }
        Ok(())
    }
}

#[cgp_impl(new DisputeValidatorProvider)]
impl DisputeValidator
where
    Self: HasTransactions,
{
    fn validate_dispute(&self, dispute: &Dispute) -> Result<(), crate::errors::Error> {
        if !self.transactions().contains_key(&(dispute.client, dispute.tx)) {
            return Err(crate::errors::Error::Input);
        }
        Ok(())
    }
}

#[cgp_impl(new ResolveValidatorProvider)]
impl ResolveValidator
where
    Self: HasTransactions + HasDisputedTransactions,
{
    fn validate_resolve(&self, resolve: &Resolve) -> Result<(), crate::errors::Error> {
        if !self.transactions().contains_key(&(resolve.client, resolve.tx)) {
            return Err(crate::errors::Error::Input);
        }
        if !self.disputed_transactions().contains_key(&(resolve.client, resolve.tx)) {
            return Err(crate::errors::Error::Input);
        }
        Ok(())
    }
}

#[cgp_impl(new ChargebackValidatorProvider)]
impl ChargebackValidator
where
    Self: HasTransactions + HasDisputedTransactions,
{
    fn validate_chargeback(&self, chargeback: &Chargeback) -> Result<(), crate::errors::Error> {
        if !self.transactions().contains_key(&(chargeback.client, chargeback.tx)) {
            return Err(crate::errors::Error::Input);
        }
        if !self
            .disputed_transactions()
            .contains_key(&(chargeback.client, chargeback.tx))
        {
            return Err(crate::errors::Error::Input);
        }
        Ok(())
    }
}

delegate_components! {
    ValidationContext<'_> {
        DepositValidatorComponent: DepositValidatorProvider,
        WithdrawalValidatorComponent: WithdrawalValidatorProvider,
        DisputeValidatorComponent: DisputeValidatorProvider,
        ResolveValidatorComponent: ResolveValidatorProvider,
        ChargebackValidatorComponent: ChargebackValidatorProvider,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use primitive_fixed_point_decimal::fpdec;

    use super::*;
    use crate::{account::Account, difference::Difference};

    fn create_test_account() -> Account {
        Account::new(1)
    }

    fn create_test_validation_context<'a>(
        transactions: &'a HashMap<(ClientId, TransactionId), Difference>,
        disputed_transactions: &'a HashMap<(ClientId, TransactionId), Difference>,
        account: &'a Account,
    ) -> ValidationContext<'a> {
        ValidationContext {
            transactions,
            disputed_transactions,
            account,
        }
    }

    #[test]
    fn test_validate_deposit_success() {
        let account = create_test_account();
        let transactions = HashMap::new();
        let disputed_transactions = HashMap::new();
        let ctx = create_test_validation_context(&transactions, &disputed_transactions, &account);

        let deposit = Deposit::new(1, 100, fpdec!(100.0));
        assert!(ctx.validate_deposit(&deposit).is_ok());
    }

    #[test]
    fn test_validate_deposit_duplicate_transaction() {
        let account = create_test_account();
        let mut transactions = HashMap::new();
        transactions.insert((1, 100), Difference::new(fpdec!(100.0), fpdec!(0.0), false));
        let disputed_transactions = HashMap::new();
        let ctx = create_test_validation_context(&transactions, &disputed_transactions, &account);

        let deposit = Deposit::new(1, 100, fpdec!(50.0));
        assert!(ctx.validate_deposit(&deposit).is_err());
    }

    #[test]
    fn test_validate_deposit_negative_amount() {
        let account = create_test_account();
        let transactions = HashMap::new();
        let disputed_transactions = HashMap::new();
        let ctx = create_test_validation_context(&transactions, &disputed_transactions, &account);

        let deposit = Deposit::new(1, 100, fpdec!(-10.0));
        assert!(ctx.validate_deposit(&deposit).is_err());
    }

    #[test]
    fn test_validate_withdrawal_success() {
        let mut account = create_test_account();
        account.available = fpdec!(100.0);
        let transactions = HashMap::new();
        let disputed_transactions = HashMap::new();
        let ctx = create_test_validation_context(&transactions, &disputed_transactions, &account);

        let withdrawal = Withdrawal::new(1, 100, fpdec!(50.0));
        assert!(ctx.validate_withdrawal(&withdrawal).is_ok());
    }

    #[test]
    fn test_validate_withdrawal_insufficient_funds() {
        let account = create_test_account();
        let transactions = HashMap::new();
        let disputed_transactions = HashMap::new();
        let ctx = create_test_validation_context(&transactions, &disputed_transactions, &account);

        let withdrawal = Withdrawal::new(1, 100, fpdec!(50.0));
        assert!(ctx.validate_withdrawal(&withdrawal).is_err());
    }

    #[test]
    fn test_validate_dispute_success() {
        let account = create_test_account();
        let mut transactions = HashMap::new();
        transactions.insert((1, 100), Difference::new(fpdec!(100.0), fpdec!(0.0), false));
        let disputed_transactions = HashMap::new();
        let ctx = create_test_validation_context(&transactions, &disputed_transactions, &account);

        let dispute = Dispute::new(1, 100);
        assert!(ctx.validate_dispute(&dispute).is_ok());
    }

    #[test]
    fn test_validate_dispute_nonexistent_transaction() {
        let account = create_test_account();
        let transactions = HashMap::new();
        let disputed_transactions = HashMap::new();
        let ctx = create_test_validation_context(&transactions, &disputed_transactions, &account);

        let dispute = Dispute::new(1, 100);
        assert!(ctx.validate_dispute(&dispute).is_err());
    }

    #[test]
    fn test_validate_resolve_success() {
        let account = create_test_account();
        let mut transactions = HashMap::new();
        transactions.insert((1, 100), Difference::new(fpdec!(100.0), fpdec!(0.0), false));
        let mut disputed_transactions = HashMap::new();
        disputed_transactions.insert((1, 100), Difference::new(fpdec!(0.0), fpdec!(100.0), false));
        let ctx = create_test_validation_context(&transactions, &disputed_transactions, &account);

        let resolve = Resolve::new(1, 100);
        assert!(ctx.validate_resolve(&resolve).is_ok());
    }

    #[test]
    fn test_validate_resolve_no_dispute() {
        let account = create_test_account();
        let mut transactions = HashMap::new();
        transactions.insert((1, 100), Difference::new(fpdec!(100.0), fpdec!(0.0), false));
        let disputed_transactions = HashMap::new();
        let ctx = create_test_validation_context(&transactions, &disputed_transactions, &account);

        let resolve = Resolve::new(1, 100);
        assert!(ctx.validate_resolve(&resolve).is_err());
    }

    #[test]
    fn test_validate_chargeback_success() {
        let account = create_test_account();
        let mut transactions = HashMap::new();
        transactions.insert((1, 100), Difference::new(fpdec!(100.0), fpdec!(0.0), false));
        let mut disputed_transactions = HashMap::new();
        disputed_transactions.insert((1, 100), Difference::new(fpdec!(0.0), fpdec!(100.0), false));
        let ctx = create_test_validation_context(&transactions, &disputed_transactions, &account);

        let chargeback = Chargeback::new(1, 100);
        assert!(ctx.validate_chargeback(&chargeback).is_ok());
    }

    #[test]
    fn test_validate_chargeback_no_dispute() {
        let account = create_test_account();
        let mut transactions = HashMap::new();
        transactions.insert((1, 100), Difference::new(fpdec!(100.0), fpdec!(0.0), false));
        let disputed_transactions = HashMap::new();
        let ctx = create_test_validation_context(&transactions, &disputed_transactions, &account);

        let chargeback = Chargeback::new(1, 100);
        assert!(ctx.validate_chargeback(&chargeback).is_err());
    }
}
