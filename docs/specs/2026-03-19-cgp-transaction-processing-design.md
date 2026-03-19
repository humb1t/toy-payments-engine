# CGP Transaction Processing Design

## Overview

Refactor the transaction processing logic in `apply_transaction` to use Context-Generic Programming (CGP), enabling easy addition of new transaction types without modifying the core processing function.

## Goals

- Enable adding new transaction types without modifying `apply_transaction`
- Keep all logic for a transaction type co-located
- Reuse existing validation and difference calculation traits

## Architecture

### Core Component: CanProcessTransaction<Tx>

```rust
use cgp::prelude::*;
use crate::errors::Error;

#[cgp_component {
    provider: TransactionProcessor,
    derive_delegate: UseDelegate<Tx>,
}]
pub trait CanProcessTransaction<Tx> {
    fn process_transaction(&mut self, transaction: &Tx) -> Result<(), Error>;
}
```

The component is generic over `Tx` (the transaction type). The `derive_delegate: UseDelegate<Tx>` syntax generates a `UseDelegate` provider that dispatches to the correct processor based on the concrete `Tx` type.

**Generated provider trait:**
```rust
pub trait TransactionProcessor<Context, Tx>: IsProviderFor<TransactionProcessorComponent, Context, Tx> {
    fn process_transaction(context: &mut Context, transaction: &Tx) -> Result<(), Error>;
}
```

The provider trait takes `Context` as the first generic parameter and `Tx` as the second. The provider struct (e.g., `DepositProcessor`) implements this trait.

### Reused Traits

The following traits from `src/validation/mod.rs` are reused for immutable access:

```rust
// From src/validation/mod.rs - reused
#[cgp_auto_getter]
pub trait HasTransactions {
    fn transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference>;
}

#[cgp_auto_getter]
pub trait HasDisputedTransactions {
    fn disputed_transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference>;
}
```

### New Context Traits

```rust
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
```

These traits provide mutable access to the state required by transaction processors.

### TransactionContext

```rust
pub struct TransactionContext<'a> {
    pub account: &'a mut Account,
    pub transactions: &'a mut HashMap<(ClientId, TransactionId), Difference>,
    pub disputed_transactions: &'a mut HashMap<(ClientId, TransactionId), Difference>,
}
```

A mutable context that holds references to all state needed for transaction processing.

#### Trait Implementations

```rust
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

// Reuse immutable accessors from validation module
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
```

## Provider Implementations

Each provider struct implements `TransactionProcessor<Context, Tx>` where `Context` is the transaction context and `Tx` is the transaction type.

**Note on difference calculation:** The existing difference calculation traits (`CanCalculateDifference`, `CanCalculateDisputeDifference`, `CanCalculateResolveChargebackDifference`) are implemented on the transaction types themselves (Deposit, Withdrawal, etc.), not on TransactionContext. The processors call these methods directly on the transaction objects.

### Deposit Processor

```rust
pub struct DepositProcessor;

#[cgp_provider]
impl<Context> TransactionProcessor<Context, Deposit> for DepositProcessor
where
    Context: CanValidateDeposit + HasAccountMut + HasTransactionsMut,
{
    fn process_transaction(context: &mut Context, deposit: &Deposit) -> Result<(), Error> {
        context.validate_deposit(deposit)?;
        let difference = deposit.difference();
        context.transactions_mut().insert((deposit.client, deposit.tx), difference);
        context.account_mut().apply(difference);
        Ok(())
    }
}
```

### Withdrawal Processor

```rust
pub struct WithdrawalProcessor;

#[cgp_provider]
impl<Context> TransactionProcessor<Context, Withdrawal> for WithdrawalProcessor
where
    Context: CanValidateWithdrawal + HasAccountMut + HasTransactionsMut,
{
    fn process_transaction(context: &mut Context, withdrawal: &Withdrawal) -> Result<(), Error> {
        context.validate_withdrawal(withdrawal)?;
        let difference = withdrawal.difference();
        context.transactions_mut().insert((withdrawal.client, withdrawal.tx), difference);
        context.account_mut().apply(difference);
        Ok(())
    }
}
```

### Dispute Processor

```rust
pub struct DisputeProcessor;

#[cgp_provider]
impl<Context> TransactionProcessor<Context, Dispute> for DisputeProcessor
where
    Context: CanValidateDispute + HasAccountMut + HasTransactions + HasDisputedTransactionsMut,
{
    fn process_transaction(context: &mut Context, dispute: &Dispute) -> Result<(), Error> {
        context.validate_dispute(dispute)?;
        if let Some(original) = context.transactions().get(&(dispute.client, dispute.tx)) {
            let diff = dispute.dispute_difference(original);
            context.disputed_transactions_mut().insert((dispute.client, dispute.tx), diff);
            context.account_mut().apply(diff);
        }
        Ok(())
    }
}
```

### Resolve Processor

```rust
pub struct ResolveProcessor;

#[cgp_provider]
impl<Context> TransactionProcessor<Context, Resolve> for ResolveProcessor
where
    Context: CanValidateResolve + HasAccountMut + HasTransactions + HasDisputedTransactions,
{
    fn process_transaction(context: &mut Context, resolve: &Resolve) -> Result<(), Error> {
        context.validate_resolve(resolve)?;
        if let Some(original) = context.transactions().get(&(resolve.client, resolve.tx))
            && let Some(dispute) = context.disputed_transactions().get(&(resolve.client, resolve.tx))
        {
            let diff = resolve.resolve_chargeback_difference(original, dispute);
            context.account_mut().apply(diff);
        }
        Ok(())
    }
}
```

### Chargeback Processor

```rust
pub struct ChargebackProcessor;

#[cgp_provider]
impl<Context> TransactionProcessor<Context, Chargeback> for ChargebackProcessor
where
    Context: CanValidateChargeback + HasAccountMut + HasTransactions + HasDisputedTransactions,
{
    fn process_transaction(context: &mut Context, chargeback: &Chargeback) -> Result<(), Error> {
        context.validate_chargeback(chargeback)?;
        if let Some(original) = context.transactions().get(&(chargeback.client, chargeback.tx))
            && let Some(dispute) = context.disputed_transactions().get(&(chargeback.client, chargeback.tx))
        {
            let diff = chargeback.resolve_chargeback_difference(original, dispute);
            context.account_mut().apply(diff);
        }
        Ok(())
    }
}
```

## State Cleanup Behavior

**Note:** The current implementation does not clean up entries from `transactions` or `disputed_transactions` maps after Resolve or Chargeback. This design preserves that behavior. If cleanup is desired in the future, it can be added to the respective processors without affecting other transaction types.

## Component Wiring

The wiring for `TransactionContext` wires validation and transaction processing components. Difference calculation remains wired to the transaction types themselves (existing wiring in `src/difference/mod.rs`).

```rust
// Define the processor delegate table
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

// Wire all components for TransactionContext
delegate_components! {
    TransactionContext<'_> {
        // Validation providers (reuse existing)
        DepositValidatorComponent: DepositValidatorProvider,
        WithdrawalValidatorComponent: WithdrawalValidatorProvider,
        DisputeValidatorComponent: DisputeValidatorProvider,
        ResolveValidatorComponent: ResolveValidatorProvider,
        ChargebackValidatorComponent: ChargebackValidatorProvider,
        
        // Transaction processing - dispatch based on transaction type
        TransactionProcessorComponent: UseDelegate<TransactionProcessors>,
    }
}
```

**Note:** Difference calculation components (`TransactionDifferenceComponent`, `DisputeDifferenceComponent`, `ResolveChargebackDifferenceComponent`) are wired to the transaction types in `src/difference/mod.rs`, not to `TransactionContext`. The processors call these methods directly on the transaction objects.

## Simplified apply_transaction

```rust
use crate::errors::Error;

fn apply_transaction(state: &mut State, transaction: Transaction) -> Result<(), Error> {
    let client_id = transaction.client();
    let account = state
        .accounts
        .entry(client_id)
        .or_insert_with(|| Account::new(client_id));
    
    if account.locked {
        return Err(Error::State);
    }
    
    let mut ctx = TransactionContext {
        account,
        transactions: &mut state.transactions,
        disputed_transactions: &mut state.disputed_transactions,
    };
    
    match &transaction {
        Transaction::Deposit(tx) => ctx.process_transaction(tx),
        Transaction::Withdrawal(tx) => ctx.process_transaction(tx),
        Transaction::Dispute(tx) => ctx.process_transaction(tx),
        Transaction::Resolve(tx) => ctx.process_transaction(tx),
        Transaction::Chargeback(tx) => ctx.process_transaction(tx),
    }
}
```

## Adding a New Transaction Type

To add a new transaction type (e.g., `Refund`):

1. Define the transaction struct in `src/transaction/types.rs`
2. Add variant to `Transaction` enum in `src/transaction/mod.rs`
3. Create a validator provider implementing `RefundValidator`
4. Create a difference provider if needed (wired to the transaction type)
5. Create a processor struct implementing `TransactionProcessor<Context, Refund>`
6. Add to the `TransactionProcessors` delegate table
7. Add the match arm in `apply_transaction`

## File Structure

```
src/
├── transaction/
│   ├── mod.rs           # Transaction enum, CanProcessTransaction trait
│   ├── types.rs         # Transaction structs (existing)
│   └── processors.rs    # TransactionProcessor implementations (new)
├── validation/
│   ├── mod.rs           # Validation traits (existing)
│   └── providers.rs     # Validation providers (existing)
├── difference/
│   └── mod.rs           # Difference traits and providers (existing)
├── context.rs           # State, Shard, TransactionContext (modified)
└── lib.rs               # apply_transaction (simplified)
```

## Testing Strategy

- Unit tests for each processor in `processors.rs`
- Integration tests for `apply_transaction` with all transaction types
- Verify existing tests continue to pass

## Migration Path

1. Create `TransactionContext` and context traits
2. Create `CanProcessTransaction` component
3. Implement processors for each transaction type
4. Wire components for `TransactionContext`
5. Refactor `apply_transaction` to use the new structure
6. Run tests to verify correctness
