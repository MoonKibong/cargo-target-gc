# Краткое руководство cargo-target-gc

cargo-target-gc — это сборщик мусора для артефактов Cargo в `target/`. Он
анализирует каталог `target/` проекта или workspace и показывает, сколько места
можно освободить. Старые и безопасные для удаления артефакты сборки удаляются
только после явного подтверждения.

## Зачем это нужно

Каталоги Cargo `target/` всегда растут со временем, но vibe coding и агентное
кодирование ускоряют этот рост и делают его менее заметным. Claude Code, Codex,
Gemini CLI и другие агенты могут много раз запускать build, test, retry и
переключаться между задачами в одной сессии. cargo-target-gc дает осторожный
цикл очистки: сначала scan, затем предварительный просмотр через `--dry-run`,
и удаление только после явного подтверждения.

## Где запускать

Запускайте инструмент в том же каталоге Cargo-проекта или workspace, где вы
запускаете `cargo build`.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

Если wrapper вроде `make` собирает вложенный Cargo-проект, сначала перейдите в
этот Cargo-каталог и запустите `cargo target-gc` там. Инструмент не угадывает
скрытые пути сборки wrapper-скриптов.

## Основные команды

```bash
cargo target-gc scan
cargo target-gc scan --json
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
cargo target-gc clean --dry-run --profile-cache
cargo target-gc clean --confirm --stale
cargo target-gc config
cargo target-gc install-agent-skills
```

## Правила безопасности

- `scan` работает только на чтение и никогда не запускает Cargo.
- `clean` отказывается работать без ровно одного флага: `--dry-run` или `--confirm`.
- По умолчанию `clean` освобождает только старый incremental-кэш.
- С `--stale` также удаляются stale-артефакты старше срока хранения.
- `--profile-cache` — более сильный режим: он включает свежий incremental cache
  и свежие каталоги `deps`, `build`, `.fingerprint` и `examples`. Сначала
  проверьте через `--dry-run`.
- `cargo clean` без опций удаляет весь `target/`; опции Cargo вроде
  `--package`, `--profile` и `--target` очищают всю выбранную область.
  target-gc очищает по возрасту и категории, чтобы сохранить больше кэша сборки.
- Если активный процесс Cargo/rustc, похоже, использует выбранный target, подтвержденное удаление отклоняется.
- Пути удаления ограничены проверенными корнями Cargo `target/`.

## Конфигурация

Файл `target-gc.toml` в корне проекта задает политику хранения.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
