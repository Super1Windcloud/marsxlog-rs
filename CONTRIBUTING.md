# Contributing

Thanks for your interest in improving mars-xlog.

## Development
- Format Rust code with `cargo fmt` and address warnings from `cargo clippy`.
- Run tests with `cargo test` where applicable.
- If you touch native build logic, mention the toolchains you used in the PR.

## Protocol and decoder validation
The workspace uses the Rust backend and does not require external C++ sources or Python 2.
Keep protocol and decoder fixtures covered when changing the on-disk format:

```bash
cargo test -p mars-xlog-core --locked --test protocol_compat
cargo test -p xlog-decoder --locked --test decoder
```

## Reporting issues
Please include reproduction steps, platform/toolchain details, and logs.
