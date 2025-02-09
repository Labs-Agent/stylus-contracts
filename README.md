# Stylus Contracts

This project provides Rust-based contracts for storing and retrieving statistics, that can be compiled to run on Arbitrum using the Stylus SDK. It includes:

- **`src/lib.rs`** with a user stats contract in Sol-like Rust (using `alloy_sol_types` and `stylus_sdk`).
- **`src/main.rs`** used to build or export ABI.
- **Tests** (in `src/lib.rs`) to verify contract functionality.

## Features

- Uses the `export-abi` feature to generate Solidity ABIs.
- Implements a ring buffer with a maximum of 50 entries for each user’s stats.
- Exposes convenience functions to retrieve recent stats.

## License

This project is licensed under the terms specified in [Cargo.toml](Cargo.toml): **MIT OR Apache-2.0**.