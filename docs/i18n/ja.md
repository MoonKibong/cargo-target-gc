# cargo-target-gc クイックスタート

cargo-target-gc は Cargo の `target/` アーティファクト用ガベージコレクタです。
プロジェクトまたはワークスペースの `target/` ディレクトリを解析し、回収可能な
容量を報告します。明示的に確認した場合だけ、安全と判定した古いビルド
アーティファクトを削除します。

## なぜ必要か

Cargo の `target/` ディレクトリは以前から時間とともに大きくなりますが、
vibe coding や agentic coding では、より速く増え、見落としやすくなります。
Claude Code、Codex、Gemini CLI などのコーディングエージェントは、1 回の
セッションで build、test、retry、タスク切り替えを何度も行うことがあります。
cargo-target-gc は、まず scan し、`--dry-run` で確認し、明示的な確認後だけ
削除する保守的なクリーンアップ手順を提供します。

## 実行する場所

`cargo build` を実行するのと同じ Cargo プロジェクトまたはワークスペースの
ディレクトリで実行してください。

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

`make` などのラッパーが入れ子の Cargo プロジェクトをビルドする場合は、その
Cargo ディレクトリに移動してから `cargo target-gc` を実行してください。この
ツールは隠れたラッパーのビルド先を推測しません。

## 主なコマンド

```bash
cargo target-gc scan
cargo target-gc scan --json
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
cargo target-gc clean --confirm --stale
cargo target-gc config
cargo target-gc install-agent-skills
```

## 安全ルール

- `scan` は読み取り専用で、Cargo を実行しません。
- `clean` は `--dry-run` または `--confirm` のどちらか一方だけがないと拒否します。
- 既定の `clean` は古い incremental キャッシュだけを回収します。
- `--stale` を追加すると、保持期間より古い stale アーティファクトも回収します。
- 選択した target ルートを Cargo/rustc が使用中に見える場合、確認済み削除を拒否します。
- 削除パスは検証済みの Cargo `target/` ルート内に制限されます。

## 設定

プロジェクトルートの `target-gc.toml` で保持期間を設定できます。

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
