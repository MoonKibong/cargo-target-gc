# cargo-target-gc Schnellstart

cargo-target-gc ist ein Garbage Collector für Cargo-Artefakte in `target/`.
Das Werkzeug analysiert das `target/`-Verzeichnis eines Projekts oder Workspace
und meldet, wie viel Speicher zurückgewonnen werden kann. Es löscht alte,
als sicher eingestufte Build-Artefakte nur nach ausdrücklicher Bestätigung.

## Warum es das gibt

Cargo-`target/`-Verzeichnisse wachsen schon immer mit der Zeit, aber Vibe Coding
und agentisches Coding machen das Wachstum schneller und leichter zu übersehen.
Claude Code, Codex, Gemini CLI und andere Coding-Agenten können in einer Sitzung
viele Builds, Tests, Wiederholungen und Aufgabenwechsel ausführen.
cargo-target-gc bietet dafür einen vorsichtigen Ablauf: zuerst scannen, mit
`--dry-run` prüfen und erst nach ausdrücklicher Bestätigung löschen.

## Wo ausführen

Führen Sie es im selben Cargo-Projekt- oder Workspace-Verzeichnis aus, in dem Sie
auch `cargo build` ausführen würden.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

Wenn ein Wrapper wie `make` ein verschachteltes Cargo-Projekt baut, wechseln Sie
zuerst in dieses Cargo-Verzeichnis und starten Sie dort `cargo target-gc`. Das
Werkzeug errät keine versteckten Build-Ziele von Wrappern.

## Wichtige Befehle

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

## Sicherheitsregeln

- `scan` ist nur lesend und startet Cargo nie.
- `clean` verweigert den Lauf ohne genau eines von `--dry-run` oder `--confirm`.
- Standardmäßig entfernt `clean` nur alte incremental-Caches.
- Mit `--stale` werden auch Artefakte entfernt, die älter als die Aufbewahrungszeit sind.
- `--profile-cache` ist ein stärkerer Modus und umfasst auch frische
  Incremental-Caches sowie aktuelle `deps`-, `build`-, `.fingerprint`- und
  `examples`-Verzeichnisse. Prüfen Sie zuerst mit `--dry-run`.
- `cargo clean` ohne Optionen entfernt das ganze `target/`; Cargo-Optionen wie
  `--package`, `--profile` und `--target` reinigen jeweils den ganzen gewählten
  Bereich. target-gc räumt nach Alter und Kategorie auf, um mehr Build-Cache zu
  erhalten.
- Wenn ein aktiver Cargo/rustc-Prozess das gewählte target zu nutzen scheint, wird bestätigtes Löschen verweigert.
- Löschpfade bleiben auf validierte Cargo-`target/`-Wurzeln begrenzt.

## Konfiguration

Die Datei `target-gc.toml` im Projektstamm konfiguriert die Aufbewahrung.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
