# cargo-target-gc pika-aloitus

cargo-target-gc on Cargo `target/` -artefaktien roskienkerääjä. Se analysoi
projektin tai workspacen `target/`-hakemiston ja raportoi vapautettavissa olevan
tilan. Se poistaa vanhoja ja turvallisiksi arvioituja build-artefakteja vain
selkeän vahvistuksen jälkeen.

## Missä suoritetaan

Suorita työkalu samassa Cargo-projektin tai workspacen hakemistossa, jossa
ajaisit `cargo build`.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

Jos wrapper kuten `make` rakentaa sisäkkäisen Cargo-projektin, siirry ensin
siihen Cargo-hakemistoon ja aja `cargo target-gc` siellä. Työkalu ei arvaa
piilotettuja wrapper-buildien kohteita.

## Tärkeimmät komennot

```bash
cargo target-gc scan
cargo target-gc scan --json
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
cargo target-gc clean --confirm --stale
cargo target-gc config
```

## Turvasäännöt

- `scan` on vain lukuoperaatio eikä koskaan käynnistä Cargoa.
- `clean` kieltäytyy ilman täsmälleen yhtä lippua: `--dry-run` tai `--confirm`.
- Oletuksena `clean` vapauttaa vain vanhaa incremental-välimuistia.
- `--stale` vapauttaa myös säilytysajan ylittäneet stale-artefaktit.
- Jos aktiivinen Cargo/rustc-prosessi näyttää käyttävän valittua target-juurta, vahvistettu poisto estetään.
- Poistopolut rajataan validoidun Cargo `target/` -juuren sisälle.

## Asetukset

Projektin juuressa oleva `target-gc.toml` määrittää säilytyksen.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
