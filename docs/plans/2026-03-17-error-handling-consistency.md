# Error Handling Consistency Implementation Plan

> **For agentic workers:** REQUIRED: Use subagent-driven-development (if subagents available) or executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace all panic!(), unwrap(), and expect() calls in production code with proper Result types and error handling, ensuring consistent and safe error handling throughout the toy-payments-engine project.

**Architecture:** This plan implements a comprehensive error handling strategy by:
1. Converting all functions that currently use panic!() to return Result types
2. Using the existing Error enum consistently throughout the codebase
3. Updating transaction parsing to return Result<Transaction, Error> instead of Result<Transaction, String>
4. Updating main() to properly handle Result<(), Error>
5. Ensuring all existing tests continue to pass

**Tech Stack:** Rust, Cargo, derive_more, csv, redb, primitive_fixed_point_decimal

---
## Chunk 1: Update Error Handling in Transaction Parsing

### Task 1: Modify TransactionIterator to return Result<Transaction, Error>

**Files:**
- Modify: `src/transaction.rs:18`
- Modify: `src/transaction.rs:21`
- Modify: `src/transaction.rs:23`
- Modify: `src/transaction.rs:49`
- Modify: `src/transaction.rs:65`
- Modify: `src/transaction.rs:52`
- Modify: `src/transaction.rs:68`

- [ ] **Step 1: Write the failing test**

```rust
// Add to tests/e2e.rs
#[test]
fn test_transaction_parsing_error_handling() {
    // This test will be added to verify that parsing errors are properly handled
    // We'll test with invalid CSV data that should produce an Error instead of panic
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_transaction_parsing_error_handling -v`
Expected: Test not yet implemented

- [ ] **Step 3: Write minimal implementation**

```rust
// In src/transaction.rs, modify the TransactionIterator implementation
use crate::errors::Error;

impl<R: io::Read> Iterator for TransactionIterator<R> {
    type Item = Result<Transaction, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.reader.records().next().map(|result| {
            result.map_err(|e| Error::TransactionRead(e))
                .and_then(|row| {
                    let transaction_type = row.get(0).ok_or(Error::Input)?; // Changed from expect

                    let client: u16 = row
                        .get(1)
                        .ok_or(Error::Input)? // Changed from expect
                        .parse()
                        .map_err(|e| Error::Input)?; // Changed from format!("Invalid client: {}", e)
                    let tx: u32 = row
                        .get(2)
                        .ok_or(Error::Input)? // Changed from expect
                        .parse()
                        .map_err(|e| Error::Input)?; // Changed from format!("Invalid tx: {}", e)

                    let transaction_type_str = transaction_type;

                    match transaction_type_str {
                        "deposit" => {
                            let amount: f64 = row
                                .get(3)
                                .ok_or(Error::Input)? // Changed from expect
                                .parse()
                                .map_err(|e| Error::Input)?; // Changed from format!("Invalid amount: {}", e)
                            // Check for negative amounts
                            if amount < 0.0 {
                                return Err(Error::Input); // Changed from error string
                            }
                            Ok(Transaction::Deposit(Deposit {
                                client,
                                tx,
                                amount: amount.try_into().map_err(|_| Error::Calculation)?, // Changed from unwrap()
                            }))
                        }
                        "withdrawal" => {
                            let amount: f64 = row
                                .get(3)
                                .ok_or(Error::Input)? // Changed from expect
                                .parse()
                                .map_err(|e| Error::Input)?; // Changed from format!("Invalid amount: {}", e)
                            // Check for negative amounts
                            if amount < 0.0 {
                                return Err(Error::Input); // Changed from error string
                            }
                            Ok(Transaction::Withdrawal(Withdrawal {
                                client,
                                tx,
                                amount: amount.try_into().map_err(|_| Error::Calculation)?, // Changed from unwrap()
                            }))
                        }
                        "dispute" => Ok(Transaction::Dispute(Dispute { client, tx })),
                        "resolve" => Ok(Transaction::Resolve(Resolve { client, tx })),
                        "chargeback" => Ok(Transaction::Chargeback(Chargeback { client, tx })),
                        _ => Err(Error::Input), // Changed from error string
                    }
                })
        })
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_transaction_parsing_error_handling -v`
Expected: Test should pass with new implementation

- [ ] **Step 5: Commit**

```bash
git add src/transaction.rs tests/e2e.rs
git commit -m "feat: update transaction parsing to return Result<Transaction, Error>"
```

## Chunk 2: Update process_transactions to Return Result<(), Error>

### Task 2: Modify process_transactions function to return Result<(), Error>

**Files:**
- Modify: `src/lib.rs:26`
- Modify: `src/lib.rs:31`
- Modify: `src/lib.rs:45`
- Modify: `src/lib.rs:50`
- Modify: `src/lib.rs:53`
- Modify: `src/lib.rs:61`
- Modify: `src/lib.rs:64`
- Modify: `src/lib.rs:73`
- Modify: `src/lib.rs:86`
- Modify: `src/lib.rs:96`
- Modify: `src/lib.rs:103`
- Modify: `src/lib.rs:113`

- [ ] **Step 1: Write the failing test**

```rust
// Add to tests/e2e.rs
#[test]
fn test_process_transactions_error_handling() {
    // This test will verify that process_transactions properly handles errors
    // by returning Result<(), Error> instead of panicking
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_process_transactions_error_handling -v`
Expected: Test not yet implemented

- [ ] **Step 3: Write minimal implementation**

```rust
// In src/lib.rs, modify the process_transactions function
pub fn process_transactions(
    reader: impl Iterator<Item = Result<Transaction, Error>>,
    state: &mut State,
) -> Result<(), Error> {
    for transaction in reader {
        let transaction = transaction?; // Changed from expect
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
            return Err(Error::State); // Changed from panic
        }
        match transaction {
            Transaction::Deposit(d) => {
                if state.transactions.contains_key(&(d.client, d.tx)) {
                    return Err(Error::State); // Changed from panic
                }
                if !d.amount.is_pos() {
                    return Err(Error::Input); // Changed from panic
                }
                let difference = d.difference();
                state.transactions.insert((d.client, d.tx), difference);
                account.apply(difference);
            }
            Transaction::Withdrawal(w) => {
                if state.transactions.contains_key(&(w.client, w.tx)) {
                    return Err(Error::State); // Changed from panic
                }
                if account.available < w.amount {
                    return Err(Error::Input); // Changed from panic
                }
                let difference = w.difference();
                state.transactions.insert((w.client, w.tx), difference);
                account.apply(difference);
            }
            Transaction::Dispute(d) => {
                // If the transaction doesn't exist, fail fast
                if !state.transactions.contains_key(&(d.client, d.tx)) {
                    return Err(Error::Input); // Changed from panic
                }
                if let Some(original_difference) = state.transactions.get(&(d.client, d.tx)) {
                    let dispute_difference = d.dispute_difference(original_difference);
                    state
                        .disputed_transactions
                        .insert((d.client, d.tx), dispute_difference);
                    account.apply(dispute_difference);
                }
            }
            Transaction::Resolve(r) => {
                // If the transaction doesn't exist or wasn't disputed, fail fast
                if !state.transactions.contains_key(&(r.client, r.tx)) {
                    return Err(Error::Input); // Changed from panic
                }
                if let Some(original_difference) = state.transactions.get(&(r.client, r.tx)) {
                    if let Some(dispute_difference) =
                        state.disputed_transactions.get(&(r.client, r.tx))
                    {
                        let resolve_difference = r
                            .resolve_chargeback_difference(original_difference, dispute_difference);
                        account.apply(resolve_difference);
                    } else {
                        return Err(Error::Input); // Changed from panic
                    }
                }
            }
            Transaction::Chargeback(c) => {
                // If the transaction doesn't exist or wasn't disputed, fail fast
                if !state.transactions.contains_key(&(c.client, c.tx)) {
                    return Err(Error::Input); // Changed from panic
                }
                if let Some(original_difference) = state.transactions.get(&(c.client, c.tx)) {
                    if let Some(dispute_difference) =
                        state.disputed_transactions.get(&(c.client, c.tx))
                    {
                        let chargeback_difference = c
                            .resolve_chargeback_difference(original_difference, dispute_difference);
                        account.apply(chargeback_difference);
                    } else {
                        return Err(Error::Input); // Changed from panic
                    }
                }
            }
        };
    }
    Ok(())
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_process_transactions_error_handling -v`
Expected: Test should pass with new implementation

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs tests/e2e.rs
git commit -m "feat: update process_transactions to return Result<(), Error>"
```

## Chunk 3: Update main() to Handle Result<(), Error>

### Task 3: Modify main() function to properly handle Result<(), Error>

**Files:**
- Modify: `src/main.rs:11`
- Modify: `src/main.rs:25`

- [ ] **Step 1: Write the failing test**

```rust
// Add to tests/e2e.rs
#[test]
fn test_main_error_handling() {
    // This test will verify that main() properly handles errors from process_transactions
    // and returns appropriate exit codes
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_main_error_handling -v`
Expected: Test not yet implemented

- [ ] **Step 3: Write minimal implementation**

```rust
// In src/main.rs, modify the main function
use std::{
    fs::File,
    io::{self, BufReader},
};

use csv::{ReaderBuilder, Trim, Writer};
use derive_more::{Display, Error, From};
use toy_payments_engine::prelude::*;

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err(Error::Input); // Changed from Error::Arguments
    }
    let input_path = &args[1];
    let file = File::open(input_path)?;
    let reader = ReaderBuilder::new()
        .trim(Trim::All)
        .from_reader(BufReader::new(file));
    let mut state = State::new();
    toy_payments_engine::process_transactions(TransactionIterator::new(reader), &mut state)?;
    let accounts = state.accounts.values();
    let mut writer = Writer::from_writer(io::stdout());
    for account in accounts {
        writer.serialize(account)?;
    }
    writer.flush()?;
    Ok(())
}

/// Possible errors of executable CLI.
/// Categories based, please add new variants based on category of errors.
#[derive(Debug, Error, Display, From)]
enum Error {
    #[display("Usage: cargo run -- sample.csv")]
    Arguments, // This will be replaced with Input error
    Io(io::Error),
    Csv(csv::Error),
    TransactionRead(io::Error),
    Database(redb::Error),
    Programmer(ProgrammerError),
    #[display(
        "incorrect math calculation, like attempt to substract with overflow, code changes may be needed"
    )]
    Calculation,
    #[display("incorrect input data, like attempt to withdraw with insufficient balance")]
    Input,
    #[display("incorrect state data, some inconsistent changes were made")]
    State,
}

// This will be moved to errors.rs
#[derive(Debug, Error, Display, From)]
pub enum ProgrammerError {
    WrongCallForConvertation,
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_main_error_handling -v`
Expected: Test should pass with new implementation

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "feat: update main() to properly handle Result<(), Error>"
```

## Chunk 4: Update Error Enum and Remove Unused Code

### Task 4: Update error enum and remove unused code

**Files:**
- Modify: `src/errors.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Write the failing test**

```rust
// Add to tests/e2e.rs
#[test]
fn test_error_enum_consistency() {
    // This test will verify that all error handling uses the same Error enum
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_error_enum_consistency -v`
Expected: Test not yet implemented

- [ ] **Step 3: Write minimal implementation**

```rust
// In src/errors.rs, update the Error enum
use std::io;

use derive_more::{Display, Error, From};

#[derive(Debug, Error, Display, From)]
pub enum Error {
    TransactionRead(io::Error),
    Database(redb::Error),
    Programmer(ProgrammerError),
    #[display(
        "incorrect math calculation, like attempt to substract with overflow, code changes may be needed"
    )]
    Calculation,
    #[display("incorrect input data, like attempt to withdraw with insufficient balance")]
    Input,
    #[display("incorrect state data, some inconsistent changes were made")]
    State,
}

#[derive(Debug, Error, Display, From)]
pub enum ProgrammerError {
    WrongCallForConvertation,
}

// In src/main.rs, remove the local Error enum and use the one from errors.rs
// Remove the local Error enum definition and add:
use toy_payments_engine::errors::Error;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test test_error_enum_consistency -v`
Expected: Test should pass with new implementation

- [ ] **Step 5: Commit**

```bash
git add src/errors.rs src/main.rs
git commit -m "feat: update error enum and remove unused code"
```

## Chunk 5: Verify All Tests Pass

### Task 5: Run all tests to ensure no regressions

**Files:**
- Test: All existing tests in `tests/`

- [ ] **Step 1: Write the failing test**

```rust
// Add to tests/e2e.rs
#[test]
fn test_all_existing_functionality_preserved() {
    // This test will verify that all existing functionality still works
    // after the error handling changes
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test test_all_existing_functionality_preserved -v`
Expected: Test not yet implemented

- [ ] **Step 3: Write minimal implementation**

```rust
// This is just a placeholder - the actual test will be run by cargo test
// All existing tests should pass with our changes
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test`
Expected: All existing tests should pass

- [ ] **Step 5: Commit**

```bash
git add tests/e2e.rs
git commit -m "test: verify all existing functionality preserved"
```

## Chunk 6: Final Verification and Cleanup

### Task 6: Final verification and cleanup

**Files:**
- Verify: All files in the project
- Test: All existing tests

- [ ] **Step 1: Run all tests**

Run: `cargo test`
Expected: All tests should pass

- [ ] **Step 2: Run linter**

Run: `cargo clippy`
Expected: No clippy warnings

- [ ] **Step 3: Run formatter**

Run: `cargo fmt`
Expected: Code should be formatted correctly

- [ ] **Step 4: Verify no unwrap/expect/panic calls remain**

Run: `grep -r "unwrap\|expect\|panic" src/`
Expected: Only the allowed usage in tests should remain

- [ ] **Step 5: Commit**

```bash
git add .
git commit -m "chore: final verification and cleanup"
```

Plan complete and saved to `docs/plans/2026-03-17-error-handling-consistency.md`. Ready to execute?