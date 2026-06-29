# cargo-target-gc snelstart

cargo-target-gc is een garbage collector voor Cargo-artefacten in `target/`.
Het analyseert de `target/`-map van een project of workspace en rapporteert
hoeveel ruimte kan worden teruggewonnen. Het verwijdert oude build-artefacten
die als veilig zijn beoordeeld alleen na expliciete bevestiging.

## Waarom dit bestaat

Cargo-`target/`-mappen groeiden altijd al na verloop van tijd, maar vibe coding
en agentic coding maken die groei sneller en makkelijker te missen. Claude Code,
Codex, Gemini CLI en andere coding agents kunnen in één sessie vaak builden,
testen, opnieuw proberen en van taak wisselen. cargo-target-gc biedt een
voorzichtige opruimlus: eerst scannen, bekijken met `--dry-run`, en alleen
verwijderen na expliciete bevestiging.

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
cargo target-gc clean --dry-run --profile-cache
cargo target-gc clean --confirm --stale
cargo target-gc config
cargo target-gc install-agent-skills
```

## Veiligheidsregels

- `scan` is alleen-lezen en voert Cargo nooit uit.
- `clean` weigert zonder precies één van `--dry-run` of `--confirm`.
- Standaard ruimt `clean` alleen oude incremental-cache op.
- Met `--stale` worden ook stale-artefacten ouder dan de bewaartermijn opgeruimd.
- `--profile-cache` is een sterkere modus die ook verse incremental cache en
  recente `deps`, `build`, `.fingerprint` en `examples` meeneemt. Controleer
  eerst met `--dry-run`.
- `cargo clean` zonder opties verwijdert heel `target/`; Cargo-opties zoals
  `--package`, `--profile` en `--target` ruimen de hele gekozen scope op.
  target-gc ruimt op basis van leeftijd en categorie op, zodat meer buildcache
  behouden blijft.
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
