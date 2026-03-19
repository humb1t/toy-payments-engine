# AGENTS.md

This file contains guidelines for agentic coding agents operating in this repository.

## Build, Lint, and Test Commands

### Building
```bash
cargo build
```

### Running Tests
```bash
cargo test
```

To run a single test:
```bash
cargo test <test_name>
```

### Linting
```bash
cargo clippy
```

### Formatting
```bash
cargo fmt
```

## Code Style Guidelines

### General
- This is a Rust project using edition 2024
- All code must compile successfully with `cargo build`
- All code must pass clippy checks with `cargo clippy`
- All code must be formatted with `cargo fmt` using the settings in .rustfmt.toml

### Imports and Formatting
- Use `imports_layout = "HorizontalVertical"` from .rustfmt.toml
- Use `imports_granularity = "Crate"`
- Group imports with `group_imports = "StdExternalCrate"`
- Reorder imports automatically with `reorder_imports = true`
- Reorder modules automatically with `reorder_modules = true`

### Naming Conventions
- Use snake_case for functions, variables and module names
- Use PascalCase for structs and enums
- Use SCREAMING_SNAKE_CASE for constants
- Use descriptive names that indicate the purpose

### Types
- Prefer explicit typing over type inference where it improves readability
- Use `Option<T>` and `Result<T, E>` appropriately for error handling
- Use proper error types instead of generic Error or panic

### Error Handling
- Avoid `unwrap()` and `expect()` in production code
- Use `?` operator for propagating errors
- Use `match` or `if let` for error handling where appropriate
- Prefer returning `Result<T, E>` over panicking when errors are possible

### Documentation
- Add documentation comments to all public functions, structs and enums
- Document the purpose, parameters and return values
- Use rustdoc compatible format

### Testing
- Write unit tests for all logic
- Use integration tests where necessary
- Test both success and error cases
- Name tests descriptively with `test_` prefix

### Security
- Avoid unsafe code (`unsafe_code = "forbid"` in Cargo.toml)
- All `unwrap()` and `expect()` usage is forbidden by clippy
- All `todo!()` and `unimplemented!()` usage is forbidden by clippy

### Dependencies
- Use `cgp` crate for context-generic programming
- Use `derive_more` for convenient derive macros
- Use `serde` for serialization
- Use `primitive_fixed_point_decimal` for decimal arithmetic
- Use `csv` for CSV processing