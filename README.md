# xlog-rs

This workspace provides a Rust-native implementation of Tencent Mars `xlog`. The public Rust release surface is `mars-xlog`; the standalone decoder is `xlog-decoder`.

## Release surface

- `mars-xlog` is the release-facing Rust crate and the intended Cargo entry point for Rust users.
- `mars-xlog-core` is the implementation-layer crate used by `mars-xlog`.
- `xlog-decoder` provides the command-line decoder for xlog files, batch folders, and feedback ZIP packages.
- The platform binding crates remain repository-local support crates and are not part of the `mars-xlog` release surface.

## Migration status
- The runtime uses Rust (`mars-xlog-core` + `mars-xlog`); the legacy C++ FFI and Python 2 comparison toolchains have been retired.
- UniFFI/JNI/Harmony wrappers are wired to the Rust backend by default.
- Protocol constants, compression, encryption, and decoding are covered by the checked-in Rust tests; they do not require a Tencent Mars source checkout.

## Workspace crates
- `mars-xlog-core`: Rust runtime core (protocol/compress/crypto/mmap/appender).
- `mars-xlog`: safe Rust wrapper API and the default Rust integration surface.
- `xlog-decoder`: command-line decoder for Mars-compatible `.xlog` files and archives.
- `mars-xlog-uniffi`: minimal UniFFI surface (Kotlin/Swift friendly).
- `mars-xlog-android-jni`: JNI bridge used by the Android example app.
- `oh-xlog`: Harmony/ohos N-API bindings.

## Flutter package
- `packages/xlog`: Flutter-native Dart FFI package using native assets and a Rust `cdylib`.
- The package includes an example diagnostics app for real logging cases, stress tests, log file inspection, and Prometheus metrics snapshots.
- The Flutter side is pinned through `fvm` in [`packages/xlog/.fvmrc`](./packages/xlog/.fvmrc).

## Build notes
- All workspace crates and the standalone Flutter native package use Rust 2024.
- `rustfmt.toml` preserves the existing formatting style independently of the language edition.
- Published Rust crates target standard stable Rust toolchain (no hardcoded version required).
- Default workspace build (`cargo build`) uses the Rust backend and does not require C++14/Boost toolchains.

## Example (Rust)
```rust
use mars_xlog::{AppenderMode, CompressMode, LogLevel, Xlog, XlogConfig};

fn main() -> anyhow::Result<()> {
    let cfg = XlogConfig::new("/tmp/xlog", "demo")
        .mode(AppenderMode::Async)
        .compress_mode(CompressMode::Zlib)
        .compress_level(6);

    let logger = Xlog::init(cfg, LogLevel::Debug)?;
    logger.log(LogLevel::Info, None, "hello from rust");
    logger.flush(true);
    Ok(())
}
```

## CLI decoder

Decode a generated `.xlog` file:

```bash
xlog-decoder decode --input app.xlog --output app.log --key <64-hex-private-key>
```

`--key` is only required for encrypted async blocks. Plaintext logs can be decoded with:

```bash
xlog-decoder --input app.xlog --output app.log
```

Install from crates.io:

```bash
cargo install xlog-decoder --locked
```

Optional binary archives produced by `scripts/xlog/package_cli_artifact.sh` are named
`xlog-decoder-v<version>-<target>.tar.gz`. The old CLI alias and npm/Homebrew wrappers have been retired.

Crates.io publishing is done locally with `cargo publish`. This repository
does not run a GitHub Actions release workflow.

## Example (tracing + tracing-subscriber)
Enable feature `tracing` and build an `XlogLayer`:
```rust
use mars_xlog::{LogLevel, Xlog, XlogConfig, XlogLayer, XlogLayerConfig};
use tracing_subscriber::prelude::*;

fn init_tracing() -> anyhow::Result<mars_xlog::XlogLayerHandle> {
    let cfg = XlogConfig::new("/tmp/xlog", "demo");
    let logger = Xlog::init(cfg, LogLevel::Info)?;

    let (layer, handle) = XlogLayer::with_config(
        logger,
        XlogLayerConfig::new(LogLevel::Info).enabled(true),
    );

    let subscriber = tracing_subscriber::registry().with(layer);
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(handle)
}
```

You can toggle the layer dynamically (mobile-friendly):
```rust
handle.set_enabled(false);
handle.set_level(LogLevel::Warn);
```

## Example (Android JNI)
An Android app example that calls the `mars-xlog` crate via JNI lives at:
`examples/android-jni`. See its README for build steps.

## Notes
- `Xlog::log`/`Xlog::write` capture caller file/line but not function name. Use the `xlog!` macros (feature `macros`) or `write_with_meta` for full metadata.
- Log files are single-writer only: the Rust backend is not multi-process safe for a shared `(name_prefix, log_dir/cache_dir)` namespace and enforces this with lock files in each storage directory.
- iOS/macOS console behavior keeps a native shim to preserve `printf`/`NSLog`/`OSLog` semantics.
- Low-level/global appender APIs are available directly in `mars-xlog` (`appender_open`/`appender_close`/`flush_all`/`appender_write_with_meta_raw`).

## License
MIT. See `LICENSE` and `NOTICE`.
