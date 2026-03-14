# Toy Payments Engine

A simple payments engine implementation that processes transactions from CSV files and outputs account balances.

## Features

- Processes deposit, withdrawal, dispute, resolve, and chargeback transactions
- Handles client account tracking with available, held, and total funds
- Implements proper locking mechanism for charged back accounts

## Input Format

The input CSV file must have columns:
- `type` - Transaction type (deposit, withdrawal, dispute, resolve, chargeback)
- `client` - Client ID (u16)
- `tx` - Transaction ID (u32) 
- `amount` - Amount for deposit/withdrawal transactions (positive decimal)

## Output Format

The output will be written to stdout in CSV format with columns:
- `client` - Client ID (u16)
- `available` - Available funds for trading/staking/withdrawal
- `held` - Funds held due to dispute
- `total` - Total funds (available + held)
- `locked` - Whether account is locked due to chargeback

## Implementation Details

### Core Data Structures
// TODO:

### Transaction Processing
1. **Deposit**: Increases available and total funds
2. **Withdrawal**: Decreases available and total funds if sufficient balance exists
3. **Dispute**: Moves funds from available to held (when client has sufficient balance) 
4. **Resolve**: Moves funds from held back to available
5. **Chargeback**: Deducts held funds, locks account (cannot be reversed)

### Special Cases Handled
- Insufficient funds for withdrawal result in no change
- Disputes that reference non-existent transactions are ignored
- Resolves that reference non-existent or non-disputed transactions are ignored
- Chargebacks that reference non-existent or non-disputed transactions are ignored
- Locked accounts cannot have any operations performed on them

## Testing

The program can be tested with:

```bash
cargo test
```

Or by using sample CSV files with the command:

```bash
cargo run -- sample.csv | head -n 10
```
