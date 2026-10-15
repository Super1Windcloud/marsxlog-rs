# xlog-decoder

High-performance, pure-Rust command-line decoder for Tencent Mars `xlog` files, batch log directories, and Feedback diagnostic ZIP archives.

Provides both `xlog-decoder` and `mars-xlog` binary aliases for full drop-in compatibility.

## Installation

```bash
cargo install xlog-decoder
```

Or run via `cargo`:

```bash
cargo run -p xlog-decoder -- <args>
```

## Features

- **Single file decoding**: Decrypt and unpack Tencent Mars `.xlog` files to `.log`.
- **Directory batch processing**: Recursively traverse folder trees, decode all `.xlog` files while mirroring directory structure.
- **Feedback ZIP unpacking**: Automatically extract feedback diagnostic `.zip` packages, decode all nested `.xlog` files, and clean up extraction artifacts.
- **ECDH Cryptography**: Support ECDH secp256k1 private keys via CLI flags (`-k`, `-p`, `--key`) or environment variable (`MARS_XLOG_KEY`).
- **Flexible CLI ergonomics**: Direct file arguments without subcommands (`xlog-decoder input.xlog output.log`) as well as explicit flags (`-i`, `-o`).
- **Key pair generator**: Built-in `gen-key` subcommand to generate compatible ECDH public/private key pairs.

## Usage Examples

### 1. Direct Single-File Decoding

```bash
# Decrypt encrypted xlog file
xlog-decoder app.xlog app.log -k <64_HEX_PRIVATE_KEY>

# Plaintext non-encrypted xlog file
xlog-decoder app.xlog app.log
```

You can also use flag syntax or the `mars-xlog` alias:

```bash
mars-xlog -i app.xlog -o app.log -k <64_HEX_PRIVATE_KEY>
```

### 2. Batch Directory Processing

Recursively decode all `.xlog` files in a folder into a target directory:

```bash
xlog-decoder logs/ decoded_logs/ -k <64_HEX_PRIVATE_KEY>
```

### 3. Feedback ZIP Archive Extraction & Decoding

Automatically unpack user diagnostic ZIP files and decode all internal `.xlog` files:

```bash
xlog-decoder feedback_dump.zip decoded_feedback/ -k <64_HEX_PRIVATE_KEY>
```

### 4. Key Pair Generation

Generate a secp256k1 public/private key pair suitable for Tencent Mars log encryption:

```bash
xlog-decoder gen-key
```

Outputs:
```text
Public key (hex): 04...
Private key (hex): ...
```

## License

MIT

