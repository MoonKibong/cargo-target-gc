# cargo-target-gc snabbstart

cargo-target-gc är en skräpinsamlare för Cargo-artefakter i `target/`. Den
analyserar ett projekts eller workspaces `target/`-katalog och rapporterar hur
mycket utrymme som kan återvinnas. Den tar bara bort gamla byggartefakter som
bedöms säkra efter uttrycklig bekräftelse.

## Varför det finns

Cargo-`target/`-kataloger har alltid vuxit över tid, men vibe coding och
agentisk kodning gör att de växer snabbare och är lättare att missa. Claude
Code, Codex, Gemini CLI och andra kodningsagenter kan bygga, testa, försöka igen
och byta uppgift många gånger i en session. cargo-target-gc ger ett försiktigt
flöde: skanna först, förhandsgranska med `--dry-run` och radera bara efter
uttrycklig bekräftelse.

## Var ska det köras

Kör verktyget i samma Cargo-projekt- eller workspace-katalog där du skulle köra
`cargo build`.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

Om en wrapper som `make` bygger ett nästlat Cargo-projekt, gå först till den
Cargo-katalogen och kör `cargo target-gc` där. Verktyget gissar inte dolda
byggmål från wrappers.

## Viktiga kommandon

```bash
cargo target-gc scan
cargo target-gc scan --json
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
cargo target-gc clean --confirm --stale
cargo target-gc config
cargo target-gc install-agent-skills
```

## Säkerhetsregler

- `scan` är skrivskyddad och startar aldrig Cargo.
- `clean` vägrar köra utan exakt en av `--dry-run` eller `--confirm`.
- Som standard återvinner `clean` bara gammal incremental-cache.
- Med `--stale` återvinns även stale-artefakter äldre än retentionstiden.
- Om en aktiv Cargo/rustc-process verkar använda vald target-rot vägras bekräftad borttagning.
- Borttagningsvägar begränsas till validerade Cargo `target/`-rötter.

## Konfiguration

`target-gc.toml` i projektroten styr retention.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
