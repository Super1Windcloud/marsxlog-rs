# Xlog Rust 迁移计划（历史归档）

> 原文日期: 2026-03-16
> 归档日期: 2026-11-08

当前状态见仓库 [README](../README.md) 的 Migration status：运行时使用 Rust（`mars-xlog-core` + `mars-xlog`），遗留 C++ FFI 与 Python 2 对比工具链已经退役。

本文归档 2026-03-16 的阶段性计划。其中“不能进入移除 C++ 依赖阶段”已经过期。优化边界的现行表述见 [rust_semantic_redlines.md](rust_semantic_redlines.md)。

## 1. 历史状态（2026-03-16）

截至当时代码：

1. Rust backend 已是主实现路径
2. Rust 侧 active semantic blocker 已清零
3. 当时判断仍不能进入移除 C++ 依赖阶段。该判断已经过期

## 2. 当时的下一步计划

### P0

1. 维持 `FileManager` 单写者约束闭环
2. 持续保持 recovery / oneshot contiguous append 语义
3. 基于已有 metrics 解释 async 剩余差距

### P1

1. 定向治理 `async_4t_zstd3` tail latency
2. 定向治理 async 小消息 `bytes/msg`
3. 拆分 `FileManager` 降低维护成本

## 3. 当时记录的约束

后续优化不得跨越以下边界：

1. 不改协议兼容性
2. 不弱化恢复语义
3. 不把 sync / fatal 写成更强语义
4. 不移除 single-writer guardrails
5. 不引入面向单机型 benchmark 的 runtime 特调

这些边界的现行表述见 [rust_semantic_redlines.md](rust_semantic_redlines.md)。
