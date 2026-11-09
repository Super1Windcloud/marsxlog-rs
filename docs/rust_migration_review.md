# Rust Migration Review（历史归档）

> 原文日期: 2026-03-16
> 归档日期: 2026-11-08

当前状态见仓库 [README](../README.md) 的 Migration status：运行时使用 Rust（`mars-xlog-core` + `mars-xlog`），遗留 C++ FFI 与 Python 2 对比工具链已经退役。

本文归档 2026-03-16 的评审结论。其中“尚未进入可移除 C++ 参考实现阶段”已经过期。

## 1. 历史状态（2026-03-16）

Rust 迁移主体在当时已经完成。当时保留的结论：

1. Rust backend 已覆盖协议、压缩、加密、formatter、mmap、文件管理和 bindings 主路径
2. benchmark 结果支持“Rust 已具备受控接入和持续优化价值”
3. Rust 侧 active semantic blocker 已清零
4. 当时判断这不等价于已经进入 GA，也不等价于可移除 C++ 参考实现。该判断已经过期

## 2. 当时的主线

当时的主线是消费已经落地的迁移结果：

1. 消费 async observability
2. 维持 single-writer guardrails
3. 定向优化 async 剩余 tail / `bytes/msg` 差距

## 3. 配套文档

1. 性能与正确性审查见 [rust_core_performance_review.md](rust_core_performance_review.md)
2. 代码架构与可维护性审查见 [rust_core_code_review.md](rust_core_code_review.md)
3. 语义护栏见 [rust_semantic_redlines.md](rust_semantic_redlines.md)
4. benchmark 体系见 [benchmark_strategy.md](benchmark_strategy.md)
5. 发布计划见 [rust_release_plan.md](rust_release_plan.md)
6. 同期计划归档见 [xlog_rust_migration_plan.md](xlog_rust_migration_plan.md)
