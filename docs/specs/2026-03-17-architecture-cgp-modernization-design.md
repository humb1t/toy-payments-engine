# Architecture + CGP Modernization Design

## Overview

Refactor the toy-payments-engine to use a modular architecture with modern CGP 0.7 patterns. This addresses three issues identified in the code review:

1. **Flat structure with mixed concerns** - `src/lib.rs` contains both `State` and `process_transactions`, `src/engine.rs` contains both domain types and validators
2. **Outdated CGP patterns** - Manual trait implementations instead of `#[derive(HasField)]`, scattered `delegate_components!` blocks
3. **State is not a CGP context** - Plain struct with no field accessors

## Target Architecture

### Module Structure

```
src/
├── lib.rs                    # Re-exports, public API, process_transactions
├── context.rs                # State as CGP context with HasField
├── transaction/
│   ├── mod.rs               # Transaction enum, TransactionIterator
│   └── types.rs             # Deposit, Withdrawal, Dispute, Resolve, Chargeback
├── account/
│   └── mod.rs               # Account struct, Difference
├── validation/
│   ├── mod.rs               # Validation CGP components
│   └── providers.rs         # Validator implementations
└── difference/
    └── mod.rs               # Difference calculation CGP components
```

### Module Responsibilities

| Module | Responsibility |
|--------|----------------|
| `lib.rs` | Public API, `prelude` module, `process_transactions` function |
| `context.rs` | `State` struct with CGP field accessors |
| `transaction/` | Transaction types, `Balance` type alias, CSV parsing |
| `account/` | Account struct |
| `difference/` | Difference struct and calculation CGP components |
| `validation/` | Validation CGP components and providers |

### Type Locations After Refactoring

| Type | Current Location | New Location |
|------|------------------|--------------|
| `Balance` | `engine.rs:7` | `transaction/types.rs` |
| `Deposit`, `Withdrawal`, `Dispute`, `Resolve`, `Chargeback` | `engine.rs` | `transaction/types.rs` |
| `Account` | `engine.rs` | `account/mod.rs` |
| `Difference` | `engine.rs` | `difference/mod.rs` |
| `Transaction` | `transaction.rs` | `transaction/mod.rs` |
| `TransactionIterator` | `transaction.rs` | `transaction/mod.rs` |
| `State` | `lib.rs` | `context.rs` |
| Validation traits | `engine/gcp.rs` | `validation/mod.rs` |
| Validation providers | `engine/gcp.rs` | `validation/providers.rs` |
| Difference traits | `engine/gcp.rs` | `difference/mod.rs` |

### File Disposition

| Current File | Action |
|-------------|--------|
| `src/engine.rs` | Delete after contents moved |
| `src/engine/gcp.rs` | Delete after contents moved |
| `src/transaction.rs` | Convert to `src/transaction/mod.rs` |

### Import Paths After Refactoring

```rust
// In lib.rs
use crate::account::Account;
use crate::context::State;
use crate::difference::Difference;
use crate::transaction::{Transaction, TransactionIterator};
use crate::validation::ValidationContext;

// In prelude (lib.rs)
pub mod prelude {
    pub use crate::account::Account;
    pub use crate::context::State;
    pub use crate::errors::Error;
    pub use crate::transaction::{Transaction, TransactionIterator};
    pub use crate::{ClientId, TransactionId};
}
```

## CGP Pattern Changes

### 1. Replace `#[cgp_auto_getter]` with `#[derive(HasField)]`

**Before:**
```rust
#[cgp_auto_getter]
pub trait HasTransactions {
    fn transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference>;
}

impl<'a> HasTransactions for ValidationContext<'a> {
    fn transactions(&self) -> &HashMap<(ClientId, TransactionId), Difference> {
        self.transactions
    }
}
```

**After:**
```rust
#[derive(HasField)]
pub struct ValidationContext<'a> {
    pub transactions: &'a HashMap<(ClientId, TransactionId), Difference>,
    pub disputed_transactions: &'a HashMap<(ClientId, TransactionId), Difference>,
    pub account: &'a Account,
}
// HasField generates transactions(), disputed_transactions(), account() automatically
```

### 2. Consolidate `delegate_components!`

**Before:** 5 separate blocks for transaction types + 1 for ValidationContext

**After:** Single consolidated block:
```rust
delegate_components! {
    Deposit { TransactionDifferenceComponent: DepositProvider }
    Withdrawal { TransactionDifferenceComponent: WithdrawalProvider }
    Dispute { DisputeDifferenceComponent: DisputeProvider }
    Resolve { ResolveChargebackDifferenceComponent: ResolveProvider }
    Chargeback { ResolveChargebackDifferenceComponent: ChargebackProvider }
    ValidationContext<'_> {
        DepositValidatorComponent: DepositValidatorProvider,
        WithdrawalValidatorComponent: WithdrawalValidatorProvider,
        DisputeValidatorComponent: DisputeValidatorProvider,
        ResolveValidatorComponent: ResolveValidatorProvider,
        ChargebackValidatorComponent: ChargebackValidatorProvider,
    }
}
```

### 3. State as CGP Context

**Before:**
```rust
pub struct State {
    pub accounts: HashMap<ClientId, Account>,
    pub transactions: HashMap<(ClientId, TransactionId), Difference>,
    pub disputed_transactions: HashMap<(ClientId, TransactionId), Difference>,
}
```

**After:**
```rust
#[derive(HasField)]
pub struct State {
    pub accounts: HashMap<ClientId, Account>,
    pub transactions: HashMap<(ClientId, TransactionId), Difference>,
    pub disputed_transactions: HashMap<(ClientId, TransactionId), Difference>,
}
```

## Implementation Phases

### Phase 1: CGP Modernization

**Goal:** Modernize CGP patterns without changing module structure

**Changes:**
1. Add `#[derive(HasField)]` to `ValidationContext`
2. Remove manual `HasTransactions`, `HasDisputedTransactions`, `HasAccount` implementations
3. Remove `#[cgp_auto_getter]` trait definitions
4. Consolidate `delegate_components!` blocks into one

**Risk:** Low - internal refactoring only, no API changes

**Verification:** All existing tests pass

### Phase 2: Module Extraction

**Goal:** Create modular directory structure

**Changes:**
1. Create `src/transaction/mod.rs` and `src/transaction/types.rs`
   - Convert `src/transaction.rs` to `src/transaction/mod.rs`
   - Move `Transaction` enum to `mod.rs`
   - Move `TransactionIterator` to `mod.rs`
   - Move `Deposit`, `Withdrawal`, `Dispute`, `Resolve`, `Chargeback` from `engine.rs` to `types.rs`
   - Move `Balance` type alias from `engine.rs:7` to `types.rs`
   - Move tests from `engine.rs:76-139` to `types.rs`
2. Create `src/difference/mod.rs`
   - Move `Difference` struct from `engine.rs` to `mod.rs`
   - Move difference calculation CGP components from `engine/gcp.rs`
   - Move `TransactionDifference`, `DisputeDifference`, `ResolveChargebackDifference` traits
   - Move provider implementations (`DepositProvider`, `WithdrawalProvider`, etc.)
   - Move tests from `engine/gcp.rs:293-350` to `mod.rs`
3. Create `src/account/mod.rs`
   - Move `Account` struct from `engine.rs`
   - Note: `Account` depends on `Difference` from `difference/` module
4. Create `src/validation/mod.rs` and `src/validation/providers.rs`
   - Move validation CGP components to `mod.rs`
   - Move validator implementations to `providers.rs`
   - Move tests from `engine/gcp.rs:351-495` to `providers.rs`
5. Update `src/lib.rs`
   - Update `prelude` module to use new import paths
   - Update imports in `process_transactions`
6. Delete `src/engine.rs` and `src/engine/gcp.rs`

**Risk:** Medium - significant file moves, import updates

**Verification:** All existing tests pass, `cargo clippy` clean

### Phase 3: State as Context

**Goal:** Make `State` a proper CGP context

**Changes:**
1. Move `State` from `src/lib.rs` to `src/context.rs`
2. Add `#[derive(HasField)]` to `State`
3. Update `process_transactions` to use field accessors (optional, for consistency)

**Risk:** Low - single struct move with derive

**Verification:** All existing tests pass

### Phase 4: Cleanup

**Goal:** Finalize and document

**Changes:**
1. Remove any dead code
2. Update module documentation
3. Ensure consistent import ordering
4. Run `cargo fmt`

**Risk:** Low - cosmetic changes

**Verification:** All tests pass, `cargo clippy` clean, `cargo fmt --check` passes

## Success Criteria

1. All existing tests pass
2. `cargo clippy` reports no warnings
3. `cargo fmt --check` passes
4. Module structure matches target architecture
5. CGP patterns use modern 0.7 idioms
6. Single consolidated `delegate_components!` block
7. `State` uses `#[derive(HasField)]`

## Rollback Strategy

Each phase should be committed separately to enable easy rollback:

1. **Phase 1 commit:** `refactor: modernize CGP patterns`
2. **Phase 2 commit:** `refactor: extract modules from engine.rs`
3. **Phase 3 commit:** `refactor: move State to context.rs`
4. **Phase 4 commit:** `refactor: cleanup and documentation`

If any phase fails verification, use `git reset --hard HEAD~1` to rollback to the previous phase.

## Out of Scope

- Throughput improvements (sharding) - deferred to future work
- Adding new transaction types
- Changing public API
- Performance optimizations
