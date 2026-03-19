use cgp::prelude::*;

use crate::{
    context::{HasAccountMut, HasDisputedTransactionsMut, HasTransactionsMut},
    difference::{CanCalculateDifference, CanCalculateDisputeDifference, CanCalculateResolveChargebackDifference},
    errors::Error,
    transaction::{Chargeback, Deposit, Dispute, ProcessTransaction, ProcessTransactionComponent, Resolve, Withdrawal},
    validation::{
        CanValidateChargeback,
        CanValidateDeposit,
        CanValidateDispute,
        CanValidateResolve,
        CanValidateWithdrawal,
        HasDisputedTransactions,
        HasTransactions,
    },
};

pub struct TransactionProcessors;

delegate_components! {
    TransactionProcessors {
        Deposit: DepositProcessor,
        Withdrawal: WithdrawalProcessor,
        Dispute: DisputeProcessor,
        Resolve: ResolveProcessor,
        Chargeback: ChargebackProcessor,
    }
}

pub struct DepositProcessor;
pub struct WithdrawalProcessor;
pub struct DisputeProcessor;
pub struct ResolveProcessor;
pub struct ChargebackProcessor;

#[cgp_provider]
impl<Context> ProcessTransaction<Context, Deposit> for DepositProcessor
where Context: CanValidateDeposit + HasAccountMut + HasTransactionsMut
{
    fn process_transaction(context: &mut Context, deposit: &Deposit) -> Result<(), Error> {
        context.validate_deposit(deposit)?;
        let difference = deposit.difference();
        context
            .transactions_mut()
            .insert((deposit.client, deposit.tx), difference);
        context.account_mut().apply(difference);
        Ok(())
    }
}

#[cgp_provider]
impl<Context> ProcessTransaction<Context, Withdrawal> for WithdrawalProcessor
where Context: CanValidateWithdrawal + HasAccountMut + HasTransactionsMut
{
    fn process_transaction(context: &mut Context, withdrawal: &Withdrawal) -> Result<(), Error> {
        context.validate_withdrawal(withdrawal)?;
        let difference = withdrawal.difference();
        context
            .transactions_mut()
            .insert((withdrawal.client, withdrawal.tx), difference);
        context.account_mut().apply(difference);
        Ok(())
    }
}

#[cgp_provider]
impl<Context> ProcessTransaction<Context, Dispute> for DisputeProcessor
where Context: CanValidateDispute + HasAccountMut + HasTransactions + HasDisputedTransactionsMut
{
    fn process_transaction(context: &mut Context, dispute: &Dispute) -> Result<(), Error> {
        context.validate_dispute(dispute)?;
        if let Some(original) = context.transactions().get(&(dispute.client, dispute.tx)) {
            let diff = dispute.dispute_difference(original);
            context
                .disputed_transactions_mut()
                .insert((dispute.client, dispute.tx), diff);
            context.account_mut().apply(diff);
        }
        Ok(())
    }
}

#[cgp_provider]
impl<Context> ProcessTransaction<Context, Resolve> for ResolveProcessor
where Context: CanValidateResolve + HasAccountMut + HasTransactions + HasDisputedTransactions
{
    fn process_transaction(context: &mut Context, resolve: &Resolve) -> Result<(), Error> {
        context.validate_resolve(resolve)?;
        if let Some(original) = context.transactions().get(&(resolve.client, resolve.tx)) &&
            let Some(dispute) = context.disputed_transactions().get(&(resolve.client, resolve.tx))
        {
            let diff = resolve.resolve_chargeback_difference(original, dispute);
            context.account_mut().apply(diff);
        }
        Ok(())
    }
}

#[cgp_provider]
impl<Context> ProcessTransaction<Context, Chargeback> for ChargebackProcessor
where Context: CanValidateChargeback + HasAccountMut + HasTransactions + HasDisputedTransactions
{
    fn process_transaction(context: &mut Context, chargeback: &Chargeback) -> Result<(), Error> {
        context.validate_chargeback(chargeback)?;
        if let Some(original) = context.transactions().get(&(chargeback.client, chargeback.tx)) &&
            let Some(dispute) = context.disputed_transactions().get(&(chargeback.client, chargeback.tx))
        {
            let diff = chargeback.resolve_chargeback_difference(original, dispute);
            context.account_mut().apply(diff);
        }
        Ok(())
    }
}
