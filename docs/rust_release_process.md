# Rust 与 CLI 发布流程

本文定义 Rust crate 与 CLI 的版本号和本地发布方式。

crates.io 发布在本机执行 `cargo publish`。仓库里不再保留 GitHub Actions 发布工作流，推送 `v*` tag 不会触发发布。

## 1. 版本号策略

当前 Rust crate、CLI 二进制、Homebrew cask 与 npm package manifest 仍使用同一个 release version。

需要保持一致的位置：

1. `crates/xlog-core/Cargo.toml`
2. `crates/xlog/Cargo.toml`
3. `crates/xlog-cli/Cargo.toml`
4. `packages/mars-xlog-cli-npm/package.json`
5. `Casks/mars-xlog.rb`
6. workspace 内对 `mars-xlog-core` / `mars-xlog` 的 path 依赖版本

使用统一版本的原因是：crate、npm 包和 Homebrew cask 按同一个 release version 发布。`xlog-decoder` 自带解码实现，不依赖 `mars-xlog-core`。

版本更新命令：

```bash
scripts/xlog/set_rust_release_version.sh <version>
```

tag 校验命令：

```bash
scripts/xlog/check_rust_release_tag.sh --tag v<version>
```

## 2. Tag 规则

正式 release tag 使用：

```text
v<version>
```

示例：

```text
v0.1.0-preview.3
v0.1.0
```

要求：

1. tag 必须指向 `main` 上已经合入的 release commit
2. tag 中的版本号必须与所有 manifest、npm package 和 cask 版本一致
3. 不要把同一个版本号重新打到不同 commit 上

版本 tag 只用于标记已经合入 `main` 的 release commit，推送 tag 不会发布。

```bash
git tag -a v0.1.0 -m "Rust GA 0.1.0"
git push origin v0.1.0
```

## 3. 发布

在 `main` 上完成版本号更新，并且 `rust-ci` 通过之后，于本机执行：

```bash
cargo publish -p mars-xlog-core --locked
cargo publish -p mars-xlog --locked
cargo publish -p xlog-decoder --locked
```

`xlog-decoder` 不再依赖 `mars-xlog-core`，可以单独发布。`mars-xlog` 依赖已发布的 `mars-xlog-core`，需要等该版本在 crates.io 可见后再发布。

发布前可以跑：

```bash
scripts/xlog/check_rust_release_tag.sh --tag v<version>
scripts/xlog/check_mars_xlog_core_release.sh
scripts/xlog/check_mars_xlog_release.sh --skip-crates-io-check
scripts/xlog/check_mars_xlog_cli_release.sh
```

crates.io 上已经存在的版本不能再次发布。release commit 需要变化时，递增版本号并打新 tag。
