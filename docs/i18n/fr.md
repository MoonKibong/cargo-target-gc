# Démarrage rapide de cargo-target-gc

cargo-target-gc est un ramasse-miettes pour les artefacts Cargo dans `target/`.
Il analyse le répertoire `target/` d'un projet ou d'un workspace et indique
l'espace récupérable. Il supprime des artefacts de build anciens et sûrs
uniquement après confirmation explicite.

## Où l'exécuter

Exécutez-le dans le même répertoire Cargo que celui où vous lanceriez
`cargo build`.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

Si un wrapper comme `make` compile un projet Cargo imbriqué, entrez d'abord dans
ce répertoire Cargo puis lancez `cargo target-gc`. L'outil ne devine pas les
sorties de build cachées des wrappers.

## Commandes principales

```bash
cargo target-gc scan
cargo target-gc scan --json
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
cargo target-gc clean --confirm --stale
cargo target-gc config
```

## Règles de sécurité

- `scan` est en lecture seule et n'exécute jamais Cargo.
- `clean` refuse de continuer sans exactement un des deux drapeaux `--dry-run` ou `--confirm`.
- Par défaut, `clean` récupère seulement les caches incremental anciens.
- Avec `--stale`, il récupère aussi les artefacts stale plus vieux que la période de rétention.
- Si un processus Cargo/rustc actif semble utiliser le target choisi, la suppression confirmée est refusée.
- Les chemins supprimés restent limités à une racine Cargo `target/` validée.

## Configuration

Le fichier `target-gc.toml` à la racine du projet configure la rétention.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
