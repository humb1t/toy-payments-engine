# Architecture + CGP Modernization Implementation Plan

> **For agentic workers:** REQUIRED: Use subagent-driven-development (if subagents available) or executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor toy-payments-engine to use modular architecture with modern CGP 0.7 patterns.

**Architecture:** Incremental refactoring in 4 phases: CGP modernization → module extraction → State as context → cleanup. Each phase produces working, testable state.

**Tech Stack:** Rust 2024, CGP 0.7, derive_more, primitive_fixed_point_decimal, csv, serde

---

## Chunk 1: Phase 1 - CGP Modernization

### Task 1.1: Add HasField derive to ValidationContext

**Files:**
- Modify: `src/engine/gcp.rs:259-281`

- [ ] **Step 1: Add HasField derive to ValidationContext**

In `src/engine/gcp.rs`, add `HasField` to the derive macro:

```rust
#[derive(HasField)]
pub struct ValidationContext<'a> {
    pub transactions: &'a HashMap<(ClientId, TransactionId), Difference>,
    pub disputed_transactions: &'a HashMap<(ClientId, TransactionId), Difference>,
    pub account: &'a Account,
}
```

- [ ] **Step 2: Remove manual trait implementations**

Delete lines 265-281 (the manual `HasTransactions`, `HasDisputedTransactions`, `HasAccount` implementations):

```rust
// DELETE THESE:
impl<'a> HasTransactions for ValidationContext<'a> {
    fn transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference> {
        self.transactions
    }
}

impl<'a> HasDisputedTransactions for ValidationContext<'a> {
    fn disputed_transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference> {
        self.disputed_transactions
    }
}

impl<'a> HasAccount for ValidationContext<'a> {
    fn account(&self) -> &Account {
        self.account
    }
}
```

- [ ] **Step 3: Replace cgp_auto_getter with cgp_getter**

Replace `#[cgp_auto_getter]` with `#[cgp_getter]` on the trait definitions (lines 14-28):

```rust
// Replace #[cgp_auto_getter] with #[cgp_getter]
#[cgp_getter]
pub trait HasTransactions {
    fn transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference>;
}

#[cgp_getter]
pub trait HasDisputedTransactions {
    fn disputed_transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference>;
}

#[cgp_getter]
pub trait HasAccount {
    fn account(&self) -> &Account;
}
```

`#[cgp_getter]` is the modern CGP 0.7 replacement for `#[cgp_auto_getter]`. It generates blanket implementations for types that derive `HasField`, so the manual implementations in Step 2 are no longer needed.

- [ ] **Step 4: Verify validator implementations remain unchanged**

The validator implementations (lines 57-157) continue to use `self.transactions()`, `self.disputed_transactions()`, `self.account()` method calls. No changes needed - `#[cgp_getter]` generates the necessary blanket implementations.

- [ ] **Step 5: Run cargo check to verify compilation**

Run: `cargo check`
Expected: Compilation succeeds

- [ ] **Step 6: Run tests to verify behavior**

Run: `cargo test`
Expected: All tests pass

---

### Task 1.2: Consolidate delegate_components blocks

**Files:**
- Modify: `src/engine/gcp.rs:228-256`

- [ ] **Step 1: Replace 5 separate delegate_components blocks with one**

Replace lines 228-256 with a single consolidated block:

```rust
delegate_components! {
    Deposit {
        TransactionDifferenceComponent: DepositProvider,
    }
    Withdrawal {
        TransactionDifferenceComponent: WithdrawalProvider,
    }
    Dispute {
        DisputeDifferenceComponent: DisputeProvider,
    }
    Resolve {
        ResolveChargebackDifferenceComponent: ResolveProvider,
    }
    Chargeback {
        ResolveChargebackDifferenceComponent: ChargebackProvider,
    }
    ValidationContext<'_> {
        DepositValidatorComponent: DepositValidatorProvider,
        WithdrawalValidatorComponent: WithdrawalValidatorProvider,
        DisputeValidatorComponent: DisputeValidatorProvider,
        ResolveValidatorComponent: ResolveValidatorProvider,
        ChargebackValidatorComponent: ChargebackValidatorProvider,
    }
}
```

- [ ] **Step 2: Run cargo check to verify compilation**

Run: `cargo check`
Expected: Compilation succeeds

- [ ] **Step 3: Run tests to verify behavior**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 4: Commit Phase 1 changes**

```bash
git add src/engine/gcp.rs
git commit -m "refactor: modernize CGP patterns - use HasField, consolidate delegate_components"
```

---

## Chunk 2: Phase 2 - Module Extraction

### Task 2.1: Create transaction module

**Files:**
- Create: `src/transaction/mod.rs`
- Create: `src/transaction/types.rs`
- Delete: `src/transaction.rs` (after conversion)

- [ ] **Step 1: Create src/transaction/ directory**

```bash
mkdir -p src/transaction
```

- [ ] **Step 2: Create src/transaction/types.rs with transaction types**

```rust
use derive_more::Constructor;
use primitive_fixed_point_decimal::ConstScaleFpdec;

pub type Balance = ConstScaleFpdec<i64, 4>;

#[derive(Clone, Debug, PartialEq, Constructor)]
pub struct Deposit {
    pub client: u16,
    pub tx: u32,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Constructor)]
pub struct Withdrawal {
    pub client: u16,
    pub tx: u32,
    pub amount: Balance,
}

#[derive(Clone, Debug, PartialEq, Constructor)]
pub struct Dispute {
    pub client: u16,
    pub tx: u32,
}

#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct Resolve {
    pub client: u16,
    pub tx: u32,
}

#[derive(Debug, Clone, PartialEq, Constructor)]
pub struct Chargeback {
    pub client: u16,
    pub tx: u32,
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};

    use primitive_fixed_point_decimal::fpdec;

    use super::*;

    #[test]
    fn deposit_withdrawal() {
        let client = 1;
        let deposit_amount = fpdec!(50.1234);
        let withdrawal_amount = fpdec!(30.1234);
        let expected_balance: Balance = fpdec!(20.0000);
        let tx_seq = AtomicU32::new(0);
        let deposit = Deposit {
            client,
            tx: tx_seq.fetch_add(1, Ordering::Relaxed),
            amount: deposit_amount,
        };
        let withdrawal = Withdrawal {
            client,
            tx: tx_seq.fetch_add(1, Ordering::Relaxed),
            amount: withdrawal_amount,
        };
        assert_eq!(deposit.amount, deposit_amount);
        assert_eq!(withdrawal.amount, withdrawal_amount);
        assert!(deposit.amount.is_pos());
        assert!(withdrawal.amount.is_pos());
    }

    #[test]
    fn deposit_withdrawal_dispute() {
        let client = 1;
        let deposit_amount = fpdec!(50.1234);
        let withdrawal_amount = fpdec!(30.1234);
        let tx_seq = AtomicU32::new(0);
        let deposit = Deposit {
            client,
            tx: tx_seq.fetch_add(1, Ordering::Relaxed),
            amount: deposit_amount,
        };
        let withdrawal = Withdrawal {
            client,
            tx: tx_seq.fetch_add(1, Ordering::Relaxed),
            amount: withdrawal_amount,
        };
        let dispute = Dispute {
            client,
            tx: tx_seq.load(Ordering::Relaxed),
        };
        assert_eq!(deposit.amount, deposit_amount);
        assert_eq!(withdrawal.amount, withdrawal_amount);
        assert_eq!(dispute.tx, withdrawal.tx);
    }
}
```

- [ ] **Step 3: Create src/transaction/mod.rs with Transaction enum and TransactionIterator**

```rust
mod types;

pub use types::*;

use std::io;

use crate::errors::Error;

pub struct TransactionIterator<R> {
    reader: csv::Reader<R>,
}

impl<R> TransactionIterator<R> {
    pub fn new(reader: csv::Reader<R>) -> Self {
        Self { reader }
    }
}

impl<R: io::Read> Iterator for TransactionIterator<R> {
    type Item = Result<Transaction, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.records().next().map(|result| {
            result.map_err(Error::Csv).and_then(|row| {
                let transaction_type = row.get(0).ok_or(Error::Input)?;

                let client: u16 = row
                    .get(1)
                    .ok_or(Error::Input)?
                    .parse()
                    .map_err(|_| Error::Input)?;
                let tx: u32 = row
                    .get(2)
                    .ok_or(Error::Input)?
                    .parse()
                    .map_err(|_| Error::Input)?;

                match transaction_type {
                    "deposit" => {
                        let amount: f64 = row
                            .get(3)
                            .ok_or(Error::Input)?
                            .parse()
                            .map_err(|_| Error::Input)?;
                        if amount < 0.0 {
                            return Err(Error::Input);
                        }
                        Ok(Transaction::Deposit(Deposit {
                            client,
                            tx,
                            amount: amount.try_into().map_err(|_| Error::Calculation)?,
                        }))
                    }
                    "withdrawal" => {
                        let amount: f64 = row
                            .get(3)
                            .ok_or(Error::Input)?
                            .parse()
                            .map_err(|_| Error::Input)?;
                        if amount < 0.0 {
                            return Err(Error::Input);
                        }
                        Ok(Transaction::Withdrawal(Withdrawal {
                            client,
                            tx,
                            amount: amount.try_into().map_err(|_| Error::Calculation)?,
                        }))
                    }
                    "dispute" => Ok(Transaction::Dispute(Dispute { client, tx })),
                    "resolve" => Ok(Transaction::Resolve(Resolve { client, tx })),
                    "chargeback" => Ok(Transaction::Chargeback(Chargeback { client, tx })),
                    _ => Err(Error::Input),
                }
            })
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
}
```

- [ ] **Step 4: Run cargo check to verify compilation**

Run: `cargo check`
Expected: Compilation errors due to duplicate types (expected - we haven't removed old files yet)

---

### Task 2.2: Create difference module

**Files:**
- Create: `src/difference/mod.rs`

- [ ] **Step 1: Create src/difference/ directory**

```bash
mkdir -p src/difference
```

- [ ] **Step 2: Create src/difference/mod.rs with Difference struct and CGP components**

```rust
use cgp::prelude::*;
use derive_more::Constructor;
use primitive_fixed_point_decimal::fpdec;

use crate::transaction::{Balance, Chargeback, Deposit, Dispute, Resolve, Withdrawal};

#[derive(Clone, Copy, Constructor)]
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
    Withdrawal {
        TransactionDifferenceComponent: WithdrawalProvider,
    }
    Dispute {
        DisputeDifferenceComponent: DisputeProvider,
    }
    Resolve {
        ResolveChargebackDifferenceComponent: ResolveProvider,
    }
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
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check`
Expected: Compilation errors due to duplicate types (expected)

---

### Task 2.3: Create account module

**Files:**
- Create: `src/account/mod.rs`

- [ ] **Step 1: Create src/account/ directory**

```bash
mkdir -p src/account
```

- [ ] **Step 2: Create src/account/mod.rs with Account struct**

```rust
use primitive_fixed_point_decimal::fpdec;
use serde::Serialize;

use crate::difference::Difference;
use crate::transaction::Balance;

#[derive(Debug, Serialize)]
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
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check`
Expected: Compilation errors due to duplicate types (expected)

---

### Task 2.4: Create validation module

**Files:**
- Create: `src/validation/mod.rs`
- Create: `src/validation/providers.rs`

- [ ] **Step 1: Create src/validation/ directory**

```bash
mkdir -p src/validation
```

- [ ] **Step 2: Create src/validation/mod.rs with validation CGP components**

```rust
mod providers;

pub use providers::*;

use cgp::prelude::*;
use std::collections::HashMap;

use crate::difference::Difference;
use crate::transaction::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
use crate::{ClientId, TransactionId};

#[cgp_getter]
pub trait HasTransactions {
    fn transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference>;
}

#[cgp_getter]
pub trait HasDisputedTransactions {
    fn disputed_transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference>;
}

#[cgp_getter]
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

#[derive(HasField)]
pub struct ValidationContext<'a> {
    pub transactions: &'a HashMap<(ClientId, TransactionId), Difference>,
    pub disputed_transactions: &'a HashMap<(ClientId, TransactionId), Difference>,
    pub account: &'a crate::account::Account,
}
```

- [ ] **Step 3: Create src/validation/providers.rs with validator implementations**

```rust
use cgp::prelude::*;
use std::collections::HashMap;

use super::*;
use crate::difference::Difference;
use crate::transaction::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
use crate::{ClientId, TransactionId};

#[cgp_impl(new DepositValidatorProvider)]
impl DepositValidator
where
    Self: HasTransactions,
{
    fn validate_deposit(&self, deposit: &Deposit) -> Result<(), crate::errors::Error> {
        if self
            .transactions()
            .contains_key(&(deposit.client, deposit.tx))
        {
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
        if self
            .transactions()
            .contains_key(&(withdrawal.client, withdrawal.tx))
        {
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
        if !self
            .transactions()
            .contains_key(&(dispute.client, dispute.tx))
        {
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
        if !self
            .transactions()
            .contains_key(&(resolve.client, resolve.tx))
        {
            return Err(crate::errors::Error::Input);
        }
        if !self
            .disputed_transactions()
            .contains_key(&(resolve.client, resolve.tx))
        {
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
        if !self
            .transactions()
            .contains_key(&(chargeback.client, chargeback.tx))
        {
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

    use super::*;
    use crate::account::Account;
    use crate::difference::Difference;
    use primitive_fixed_point_decimal::fpdec;

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
```

- [ ] **Step 4: Run cargo check**

Run: `cargo check`
Expected: Compilation errors due to duplicate types (expected)

---

### Task 2.5: Update lib.rs and delete old files

**Files:**
- Modify: `src/lib.rs`
- Delete: `src/engine.rs`
- Delete: `src/engine/gcp.rs`
- Delete: `src/transaction.rs`

- [ ] **Step 1: Update src/lib.rs with new module structure and imports**

Replace entire `src/lib.rs`:

```rust
use std::collections::HashMap;

use crate::{
    account::Account,
    difference::{
        CanCalculateDifference, CanCalculateDisputeDifference,
        CanCalculateResolveChargebackDifference, Difference,
    },
    transaction::{Transaction, TransactionIterator},
    validation::{
        CanValidateChargeback, CanValidateDeposit, CanValidateDispute, CanValidateResolve,
        CanValidateWithdrawal, ValidationContext,
    },
};

pub mod account;
pub mod difference;
pub mod errors;
pub mod transaction;
pub mod validation;

pub mod prelude {
    pub use crate::account::Account;
    pub use crate::errors::Error;
    pub use crate::transaction::{Transaction, TransactionIterator};
    pub use crate::{ClientId, TransactionId};
}

pub type ClientId = u16;
pub type TransactionId = u32;

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

pub fn process_transactions(
    reader: impl Iterator<Item = Result<Transaction, errors::Error>>,
    state: &mut State,
) -> Result<(), errors::Error> {
    for transaction in reader {
        let transaction = transaction?;
        let client_id = match &transaction {
            Transaction::Deposit(d) => d.client,
            Transaction::Withdrawal(w) => w.client,
            Transaction::Dispute(d) => d.client,
            Transaction::Resolve(r) => r.client,
            Transaction::Chargeback(c) => c.client,
        };

        let account = state
            .accounts
            .entry(client_id)
            .or_insert_with(|| Account::new(client_id));
        if account.locked {
            return Err(errors::Error::State);
        }

        let validation_ctx = ValidationContext {
            transactions: &state.transactions,
            disputed_transactions: &state.disputed_transactions,
            account,
        };

        match transaction {
            Transaction::Deposit(d) => {
                validation_ctx.validate_deposit(&d)?;
                let difference = d.difference();
                state.transactions.insert((d.client, d.tx), difference);
                account.apply(difference);
            }
            Transaction::Withdrawal(w) => {
                validation_ctx.validate_withdrawal(&w)?;
                let difference = w.difference();
                state.transactions.insert((w.client, w.tx), difference);
                account.apply(difference);
            }
            Transaction::Dispute(d) => {
                validation_ctx.validate_dispute(&d)?;
                if let Some(original_difference) = state.transactions.get(&(d.client, d.tx)) {
                    let dispute_difference = d.dispute_difference(original_difference);
                    state
                        .disputed_transactions
                        .insert((d.client, d.tx), dispute_difference);
                    account.apply(dispute_difference);
                }
            }
            Transaction::Resolve(r) => {
                validation_ctx.validate_resolve(&r)?;
                if let Some(original_difference) = state.transactions.get(&(r.client, r.tx))
                    && let Some(dispute_difference) =
                        state.disputed_transactions.get(&(r.client, r.tx))
                {
                    let resolve_difference = r
                        .resolve_chargeback_difference(original_difference, dispute_difference);
                    account.apply(resolve_difference);
                }
            }
            Transaction::Chargeback(c) => {
                validation_ctx.validate_chargeback(&c)?;
                if let Some(original_difference) = state.transactions.get(&(c.client, c.tx))
                    && let Some(dispute_difference) =
                        state.disputed_transactions.get(&(c.client, c.tx))
                {
                    let chargeback_difference = c
                        .resolve_chargeback_difference(original_difference, dispute_difference);
                    account.apply(chargeback_difference);
                }
            }
        };
    }
    Ok(())
}
```

- [ ] **Step 2: Delete old files**

```bash
rm -rf src/engine
rm src/transaction.rs
```

- [ ] **Step 3: Run cargo check**

Run: `cargo check`
Expected: Compilation succeeds

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 5: Run clippy**

Run: `cargo clippy`
Expected: No warnings

- [ ] **Step 6: Commit Phase 2 changes**

```bash
git add -A
git commit -m "refactor: extract modules from engine.rs - create transaction/, account/, difference/, validation/"
```

---

## Chunk 3: Phase 3 - State as Context

### Task 3.1: Create context.rs with State

**Files:**
- Create: `src/context.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create src/context.rs**

```rust
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
```

- [ ] **Step 2: Update src/lib.rs to use context module**

Update `src/lib.rs` to:
1. Add `pub mod context;`
2. Add `use crate::context::State;`
3. Remove the `State` struct definition and its impl blocks
4. Update prelude to re-export `State` from context

The updated `src/lib.rs` should have:

```rust
use std::collections::HashMap;

use crate::{
    account::Account,
    context::State,
    difference::{
        CanCalculateDifference, CanCalculateDisputeDifference,
        CanCalculateResolveChargebackDifference, Difference,
    },
    transaction::{Transaction, TransactionIterator},
    validation::{
        CanValidateChargeback, CanValidateDeposit, CanValidateDispute, CanValidateResolve,
        CanValidateWithdrawal, ValidationContext,
    },
};

pub mod account;
pub mod context;
pub mod difference;
pub mod errors;
pub mod transaction;
pub mod validation;

pub mod prelude {
    pub use crate::account::Account;
    pub use crate::context::State;
    pub use crate::errors::Error;
    pub use crate::transaction::{Transaction, TransactionIterator};
    pub use crate::{ClientId, TransactionId};
}

pub type ClientId = u16;
pub type TransactionId = u32;

pub fn process_transactions(
    // ... rest of function unchanged
)
```

Remove the `State` struct and its `impl` blocks from `lib.rs` (they are now in `context.rs`).

- [ ] **Step 3: Run cargo check**

Run: `cargo check`
Expected: Compilation succeeds

- [ ] **Step 4: Run tests**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 5: Commit Phase 3 changes**

```bash
git add -A
git commit -m "refactor: move State to context.rs with HasField derive"
```

---

## Chunk 4: Phase 4 - Cleanup

### Task 4.1: Format and verify

**Files:**
- All source files

- [ ] **Step 1: Run cargo fmt**

Run: `cargo fmt`

- [ ] **Step 2: Run cargo fmt --check**

Run: `cargo fmt --check`
Expected: No output (all files formatted)

- [ ] **Step 3: Run cargo clippy**

Run: `cargo clippy`
Expected: No warnings

- [ ] **Step 4: Run all tests**

Run: `cargo test`
Expected: All tests pass

- [ ] **Step 5: Commit Phase 4 changes**

```bash
git add -A
git commit -m "refactor: cleanup and formatting"
```

---

## Verification

- [ ] **Final verification: All success criteria met**

Run:
```bash
cargo test && cargo clippy && cargo fmt --check
```

Expected: All commands succeed with no errors or warnings.
