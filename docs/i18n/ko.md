# cargo-target-gc 빠른 시작

cargo-target-gc는 Cargo `target/` 아티팩트 가비지 컬렉터입니다. 프로젝트나
워크스페이스의 `target/` 디렉터리를 분석하고, 회수할 수 있는 공간을 보고합니다.
명시적으로 확인한 경우에만 안전하다고 판단한 오래된 빌드 아티팩트를 삭제합니다.

## 왜 필요한가요

Cargo `target/` 디렉터리는 원래 시간이 지나면 커지지만, vibe coding과 agentic
coding에서는 더 빨리 커지고 놓치기 쉽습니다. Claude Code, Codex, Gemini CLI 같은
코딩 에이전트는 한 세션에서 build, test, retry, task 전환을 여러 번 수행할 수
있습니다. cargo-target-gc는 먼저 scan하고, `--dry-run`으로 미리 확인한 뒤,
명시적 확인이 있을 때만 삭제하는 보수적인 정리 흐름을 제공합니다.

## 어디에서 실행하나요

`cargo build`를 실행하는 것과 같은 Cargo 프로젝트 또는 워크스페이스 디렉터리에서
실행하세요.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

`make` 같은 래퍼가 중첩된 Cargo 프로젝트를 빌드한다면, 그 Cargo 디렉터리로
이동한 뒤 `cargo target-gc`를 실행하세요. 이 도구는 숨겨진 래퍼 빌드 경로를
추측하지 않습니다.

## 주요 명령

```bash
cargo target-gc scan
cargo target-gc scan --json
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
cargo target-gc clean --confirm --stale
cargo target-gc config
cargo target-gc install-agent-skills
```

## 안전 규칙

- `scan`은 읽기 전용이며 Cargo를 실행하지 않습니다.
- `clean`은 `--dry-run` 또는 `--confirm` 중 정확히 하나가 없으면 거부합니다.
- 기본 `clean`은 오래된 incremental 캐시만 회수합니다.
- `--stale`을 추가하면 보존 기간보다 오래된 stale 아티팩트도 회수합니다.
- 활성 Cargo/rustc 프로세스가 선택된 target 루트를 사용하는 것처럼 보이면
  확인된 삭제를 거부합니다.
- 삭제 경로는 검증된 Cargo `target/` 루트 안으로 제한됩니다.

## 설정

프로젝트 루트의 `target-gc.toml`에서 보존 기간을 설정할 수 있습니다.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
