# CGP Transaction Processing Implementation Plan

> **For agentic workers:** REQUIRED: Use subagent-driven-development (if subagents available) or executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor transaction processing to use CGP, enabling easy addition of new transaction types.

**Architecture:** Create `CanProcessTransaction<Tx>` component with `UseDelegate<Tx>` dispatch. Each transaction type gets a processor provider that handles validation, difference calculation, state mutation, and account application.

**Tech Stack:** Rust, CGP (cgp crate), existing validation/difference traits

---

## Chunk 1: Context Traits and TransactionContext

### Task 0: Convert context.rs to context module directory

**Files:**
- Move: `src/context.rs` → `src/context/mod.rs`

- [ ] **Step 1: Create context directory and move file**

```bash
mkdir -p src/context
mv src/context.rs src/context/mod.rs
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Success (no functional change)

- [ ] **Step 3: Commit**

```bash
git add src/context
git commit -m "refactor: convert context.rs to context module directory"
```

### Task 1: Create HasAccountMut trait

**Files:**
- Create: `src/context/traits.rs`
- Modify: `src/context/mod.rs`

- [ ] **Step 1: Create context traits module**

Create `src/context/traits.rs`:

```rust
use cgp::prelude::*;
use crate::{account::Account, ClientId, TransactionId, difference::Difference};
use std::collections::HashMap;

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

- [ ] **Step 2: Export traits from context module**

Add to the top of `src/context/mod.rs` (these are additions to the existing file):

```rust
mod traits;

pub use traits::*;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Success

- [ ] **Step 4: Commit**

```bash
git add src/context/traits.rs src/context/mod.rs
git commit -m "feat: add mutable context traits for transaction processing"
```

### Task 2: Create TransactionContext struct

**Files:**
- Modify: `src/context/mod.rs`

- [ ] **Step 1: Add TransactionContext struct**

Add to `src/context/mod.rs` (at the top of the file, add imports; add struct after existing State/Shard definitions):

```rust
use std::collections::HashMap;
use crate::{ClientId, TransactionId, account::Account, difference::Difference};

pub struct TransactionContext<'a> {
    pub account: &'a mut Account,
    pub transactions: &'a mut HashMap<(ClientId, TransactionId), Difference>,
    pub disputed_transactions: &'a mut HashMap<(ClientId, TransactionId), Difference>,
}
```

- [ ] **Step 2: Implement context traits for TransactionContext**

Add to `src/context/mod.rs`:

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
```

- [ ] **Step 3: Implement reused validation traits for TransactionContext**

Add to `src/context/mod.rs`:

```rust
use crate::validation::{HasTransactions, HasDisputedTransactions};

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

- [ ] **Step 4: Verify compilation**

Run: `cargo check`
Expected: Success

- [ ] **Step 5: Commit**

```bash
git add src/context/mod.rs
git commit -m "feat: add TransactionContext with trait implementations"
```

---

## Chunk 2: CanProcessTransaction Component

### Task 3: Create CanProcessTransaction component

**Files:**
- Create: `src/transaction/component.rs`
- Modify: `src/transaction/mod.rs`

- [ ] **Step 1: Create transaction component module**

Create `src/transaction/component.rs`:

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

- [ ] **Step 2: Export component from transaction module**

Modify `src/transaction/mod.rs` to add:

```rust
mod component;

pub use component::*;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Success

- [ ] **Step 4: Commit**

```bash
git add src/transaction/component.rs src/transaction/mod.rs
git commit -m "feat: add CanProcessTransaction CGP component"
```

---

## Chunk 3: Transaction Processors

### Task 4: Create DepositProcessor

**Files:**
- Create: `src/transaction/processors.rs`
- Modify: `src/transaction/mod.rs`

- [ ] **Step 1: Create processors module with DepositProcessor**

Create `src/transaction/processors.rs`:

```rust
use cgp::prelude::*;
use crate::{
    context::{HasAccountMut, HasTransactionsMut},
    difference::CanCalculateDifference,
    errors::Error,
    transaction::{CanProcessTransaction, Deposit, TransactionProcessor},
    validation::CanValidateDeposit,
};

pub struct DepositProcessor;

#[cgp_provider]
impl<Context> TransactionProcessor<Context, Deposit> for DepositProcessor
where
    Context: CanValidateDeposit + HasAccountMut + HasTransactionsMut,
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
```

- [ ] **Step 2: Export processors from transaction module**

Modify `src/transaction/mod.rs` to add:

```rust
mod processors;

pub use processors::*;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Success

- [ ] **Step 4: Commit**

```bash
git add src/transaction/processors.rs src/transaction/mod.rs
git commit -m "feat: add DepositProcessor"
```

### Task 5: Create WithdrawalProcessor

**Files:**
- Modify: `src/transaction/processors.rs`

- [ ] **Step 1: Add WithdrawalProcessor**

Add to `src/transaction/processors.rs`:

```rust
use crate::validation::CanValidateWithdrawal;
use crate::transaction::Withdrawal;

pub struct WithdrawalProcessor;

#[cgp_provider]
impl<Context> TransactionProcessor<Context, Withdrawal> for WithdrawalProcessor
where
    Context: CanValidateWithdrawal + HasAccountMut + HasTransactionsMut,
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
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Success

- [ ] **Step 3: Commit**

```bash
git add src/transaction/processors.rs
git commit -m "feat: add WithdrawalProcessor"
```

### Task 6: Create DisputeProcessor

**Files:**
- Modify: `src/transaction/processors.rs`

- [ ] **Step 1: Add DisputeProcessor**

Add to `src/transaction/processors.rs`:

```rust
use crate::{
    context::{HasDisputedTransactionsMut, HasTransactions},
    difference::CanCalculateDisputeDifference,
    transaction::Dispute,
    validation::CanValidateDispute,
};

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
            context
                .disputed_transactions_mut()
                .insert((dispute.client, dispute.tx), diff);
            context.account_mut().apply(diff);
        }
        Ok(())
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Success

- [ ] **Step 3: Commit**

```bash
git add src/transaction/processors.rs
git commit -m "feat: add DisputeProcessor"
```

### Task 7: Create ResolveProcessor

**Files:**
- Modify: `src/transaction/processors.rs`

- [ ] **Step 1: Add ResolveProcessor**

Add to `src/transaction/processors.rs`:

```rust
use crate::{
    context::HasDisputedTransactions,
    difference::CanCalculateResolveChargebackDifference,
    transaction::Resolve,
    validation::CanValidateResolve,
};

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

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Success

- [ ] **Step 3: Commit**

```bash
git add src/transaction/processors.rs
git commit -m "feat: add ResolveProcessor"
```

### Task 8: Create ChargebackProcessor

**Files:**
- Modify: `src/transaction/processors.rs`

- [ ] **Step 1: Add ChargebackProcessor**

Add to `src/transaction/processors.rs`:

```rust
use crate::{
    transaction::Chargeback,
    validation::CanValidateChargeback,
};

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

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Success

- [ ] **Step 3: Commit**

```bash
git add src/transaction/processors.rs
git commit -m "feat: add ChargebackProcessor"
```

---

## Chunk 4: Component Wiring

### Task 9: Wire TransactionProcessors delegate table

**Files:**
- Modify: `src/transaction/processors.rs`

- [ ] **Step 1: Add TransactionProcessors delegate table**

Add to `src/transaction/processors.rs`:

```rust
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
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Success

- [ ] **Step 3: Commit**

```bash
git add src/transaction/processors.rs
git commit -m "feat: add TransactionProcessors delegate table"
```

### Task 10: Wire TransactionContext components

**Files:**
- Modify: `src/context/mod.rs`

- [ ] **Step 1: Add component wiring for TransactionContext**

Add to `src/context/mod.rs`:

```rust
use cgp::prelude::*;
use crate::transaction::{TransactionProcessors, TransactionProcessorComponent};
use crate::validation::{
    DepositValidatorComponent, DepositValidatorProvider,
    WithdrawalValidatorComponent, WithdrawalValidatorProvider,
    DisputeValidatorComponent, DisputeValidatorProvider,
    ResolveValidatorComponent, ResolveValidatorProvider,
    ChargebackValidatorComponent, ChargebackValidatorProvider,
};

delegate_components! {
    TransactionContext<'_> {
        // Validation providers
        DepositValidatorComponent: DepositValidatorProvider,
        WithdrawalValidatorComponent: WithdrawalValidatorProvider,
        DisputeValidatorComponent: DisputeValidatorProvider,
        ResolveValidatorComponent: ResolveValidatorProvider,
        ChargebackValidatorComponent: ChargebackValidatorProvider,
        
        // Transaction processing
        TransactionProcessorComponent: UseDelegate<TransactionProcessors>,
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check`
Expected: Success

- [ ] **Step 3: Commit**

```bash
git add src/context/mod.rs
git commit -m "feat: wire TransactionContext components"
```

---

## Chunk 5: Refactor apply_transaction

### Task 11: Refactor apply_transaction to use TransactionContext

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Update apply_transaction function**

Replace the `apply_transaction` function in `src/lib.rs` with:

```rust
use crate::context::TransactionContext;
use crate::transaction::CanProcessTransaction;

fn apply_transaction(state: &mut State, transaction: Transaction) -> Result<(), errors::Error> {
    let client_id = transaction.client();
    let account = state
        .accounts
        .entry(client_id)
        .or_insert_with(|| Account::new(client_id));
    
    if account.locked {
        return Err(errors::Error::State);
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

- [ ] **Step 2: Remove unused imports**

Remove from `src/lib.rs`:
- `CanCalculateDifference`
- `CanCalculateDisputeDifference`
- `CanCalculateResolveChargebackDifference`
- `CanValidateChargeback`, `CanValidateDeposit`, `CanValidateDispute`, `CanValidateResolve`, `CanValidateWithdrawal`
- `ValidationContext`

- [ ] **Step 3: Verify compilation**

Run: `cargo check`
Expected: Success

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs
git commit -m "refactor: use TransactionContext for transaction processing"
```

### Task 12: Run full test suite

- [ ] **Step 1: Run all tests**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 2: Run clippy**

Run: `cargo clippy`
Expected: No warnings

- [ ] **Step 3: Run format check**

Run: `cargo fmt --check`
Expected: No changes needed

- [ ] **Step 4: Final commit if needed**

```bash
git add -A
git commit -m "chore: final cleanup for CGP transaction processing"
```

---

## Summary

This plan creates:
1. Context traits for mutable state access
2. `TransactionContext` struct with trait implementations
3. `CanProcessTransaction<Tx>` CGP component
4. Five processor providers (Deposit, Withdrawal, Dispute, Resolve, Chargeback)
5. Component wiring for `TransactionContext`
6. Refactored `apply_transaction` using the new CGP structure

The result enables adding new transaction types by:
1. Creating a processor struct
2. Adding to `TransactionProcessors` delegate table
3. Adding a match arm in `apply_transaction`
