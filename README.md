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

- **State** - Central state with accounts, transactions, and disputed transactions tracking
- **Shard** - Independent processing shard mirroring state for parallel execution
- **ShardedState** - Manages multiple shards, distributing accounts by client ID hash
- **Account** - Client account with available, held, and total funds
- **Difference** - Atomic account change representation (available, held, lock state)
- **ValidationContext** - CGP context for component-based validation logic

### Context-Generic Programming (CGP)

The project uses CGP (`cgp` crate) for composition and context-based operations:

- **Transaction Differences**: Lazy computation of account changes via trait-based providers
- **Validation Components**: Isolated validation logic tied to transaction context
- **Delegation**: Dynamic component composition and resolution via `delegate_components!`
- **Trait-based Validation**: `CanValidateDeposit`, `CanValidateWithdrawal`, etc. with contexts

### Transaction Processing
1. **Deposit**: Increases available and total funds
2. **Withdrawal**: Decreases available and total funds if sufficient balance exists
3. **Dispute**: Moves funds from available to held (when client has sufficient balance) 
4. **Resolve**: Moves funds from held back to available
5. **Chargeback**: Deducts held funds, locks account (cannot be reversed)

### Parallel Transactions Processing

Parallel processing shows benefit at scale (10,000+ clients) with ~55% improvement (on AMD Ryzen 9 7900X3D 12-Core).
For smaller workloads (around 1,000 clients), sequential is faster due to parallelization overhead.

## Testing

The program can be tested with:

```bash
cargo test
```

Or by using sample CSV files with the command:

```bash
cargo run -- sample.csv | head -n 10
```

## Development Tools

This project was developed with assistance from AI-powered tools:
- **Opencode** - Code editor and development environment providing intelligent code assistance
- **GLM Model** - Provided guidance on CGP architecture and Rust best practices
- **Qwen Model** - Assisted with code optimization and architectural decisions

These tools helped implement many aspects of the codebase including CGP-based component composition, error handling patterns, and parallel processing strategies.
