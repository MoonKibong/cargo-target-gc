# Inicio rápido de cargo-target-gc

cargo-target-gc es un recolector de basura para artefactos de Cargo en
`target/`. Analiza el directorio `target/` de un proyecto o workspace e informa
cuánto espacio se puede recuperar. Solo elimina artefactos de compilación
antiguos y considerados seguros después de una confirmación explícita.

## Por qué existe

Los directorios Cargo `target/` siempre han crecido con el tiempo, pero el vibe
coding y la programación agentic hacen que crezcan más rápido y sea más fácil no
notarlo. Claude Code, Codex, Gemini CLI y otros agentes de código pueden compilar,
probar, reintentar y cambiar de tarea muchas veces en una sola sesión.
cargo-target-gc ofrece un flujo conservador: primero scan, luego vista previa
con `--dry-run`, y eliminación solo después de confirmación explícita.

## Dónde ejecutarlo

Ejecútalo en el mismo directorio del proyecto Cargo o workspace donde ejecutarías
`cargo build`.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

Si un wrapper como `make` compila un proyecto Cargo anidado, entra primero en
ese directorio Cargo y ejecuta allí `cargo target-gc`. La herramienta no adivina
rutas de build ocultas de wrappers.

## Comandos principales

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

## Reglas de seguridad

- `scan` es de solo lectura y nunca ejecuta Cargo.
- `clean` se niega a ejecutarse sin exactamente una de `--dry-run` o `--confirm`.
- Por defecto, `clean` recupera solo cachés incremental antiguos.
- Con `--stale`, también recupera artefactos stale más antiguos que el periodo de retención.
- `--profile-cache` es un modo más fuerte que también incluye caché incremental
  reciente y directorios recientes `deps`, `build`, `.fingerprint` y `examples`.
  Revisa primero con `--dry-run`.
- `cargo clean` sin opciones elimina todo `target/`; las opciones de Cargo como
  `--package`, `--profile` y `--target` limpian todo el ámbito seleccionado.
  target-gc limpia por edad y categoría para conservar más caché de build.
- Si un proceso Cargo/rustc activo parece usar el target seleccionado, se rechaza la eliminación confirmada.
- Las rutas eliminadas se limitan a raíces Cargo `target/` validadas.

## Configuración

`target-gc.toml` en la raíz del proyecto configura la retención.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
