# cargo-target-gc 快速开始

cargo-target-gc 是 Cargo `target/` 构建产物的垃圾回收工具。它会分析项目或
workspace 的 `target/` 目录，报告可回收空间。只有在用户明确确认后，它才会
删除被判定为安全且过期的构建产物。

## 在哪里运行

请在运行 `cargo build` 的同一个 Cargo 项目或 workspace 目录中运行。

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

如果 `make` 等包装脚本会构建嵌套的 Cargo 项目，请先进入那个 Cargo 目录再运行
`cargo target-gc`。本工具不会猜测隐藏的包装脚本构建路径。

## 常用命令

```bash
cargo target-gc scan
cargo target-gc scan --json
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
cargo target-gc clean --confirm --stale
cargo target-gc config
```

## 安全规则

- `scan` 只读，不会运行 Cargo。
- `clean` 必须且只能带有 `--dry-run` 或 `--confirm` 之一，否则会拒绝执行。
- 默认 `clean` 只回收过期的 incremental 缓存。
- 添加 `--stale` 后，也会回收超过保留期的 stale 产物。
- 如果检测到 Cargo/rustc 似乎正在使用选中的 target 根目录，确认删除会被拒绝。
- 删除路径会被限制在验证过的 Cargo `target/` 根目录内。

## 配置

可以在项目根目录的 `target-gc.toml` 中设置保留策略。

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
