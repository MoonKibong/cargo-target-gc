# cargo-target-gc snelstart

cargo-target-gc is een garbage collector voor Cargo-artefacten in `target/`.
Het analyseert de `target/`-map van een project of workspace en rapporteert
hoeveel ruimte kan worden teruggewonnen. Het verwijdert oude build-artefacten
die als veilig zijn beoordeeld alleen na expliciete bevestiging.

## Waar uitvoeren

Voer het uit in dezelfde Cargo-project- of workspace-map waar u `cargo build`
zou uitvoeren.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

Als een wrapper zoals `make` een genest Cargo-project bouwt, ga dan eerst naar
die Cargo-map en voer daar `cargo target-gc` uit. Het hulpmiddel raadt geen
verborgen buildpaden van wrappers.

## Belangrijkste opdrachten

```bash
cargo target-gc scan
cargo target-gc scan --json
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
cargo target-gc clean --confirm --stale
cargo target-gc config
```

## Veiligheidsregels

- `scan` is alleen-lezen en voert Cargo nooit uit.
- `clean` weigert zonder precies één van `--dry-run` of `--confirm`.
- Standaard ruimt `clean` alleen oude incremental-cache op.
- Met `--stale` worden ook stale-artefacten ouder dan de bewaartermijn opgeruimd.
- Als een actief Cargo/rustc-proces de gekozen target-root lijkt te gebruiken, wordt bevestigde verwijdering geweigerd.
- Verwijderpaden blijven beperkt tot gevalideerde Cargo `target/`-roots.

## Configuratie

`target-gc.toml` in de projectroot stelt de bewaartermijn in.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
