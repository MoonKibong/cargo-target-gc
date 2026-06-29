# Início rápido do cargo-target-gc

cargo-target-gc é um coletor de lixo para artefatos do Cargo em `target/`. Ele
analisa o diretório `target/` de um projeto ou workspace e informa quanto espaço
pode ser recuperado. Ele só remove artefatos de build antigos e considerados
seguros depois de confirmação explícita.

## Por que isto existe

Diretórios Cargo `target/` sempre cresceram com o tempo, mas vibe coding e
programação agentic tornam esse crescimento mais rápido e fácil de ignorar.
Claude Code, Codex, Gemini CLI e outros agentes de código podem compilar,
testar, tentar novamente e trocar de tarefa muitas vezes em uma sessão.
cargo-target-gc oferece um fluxo conservador: primeiro scan, depois prévia com
`--dry-run`, e remoção somente após confirmação explícita.

## Onde executar

Execute no mesmo diretório do projeto Cargo ou workspace em que você rodaria
`cargo build`.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

Se um wrapper como `make` compila um projeto Cargo aninhado, entre primeiro
nesse diretório Cargo e execute `cargo target-gc` ali. A ferramenta não tenta
adivinhar destinos de build ocultos de wrappers.

## Comandos principais

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

## Regras de segurança

- `scan` é somente leitura e nunca executa Cargo.
- `clean` recusa executar sem exatamente uma das opções `--dry-run` ou `--confirm`.
- Por padrão, `clean` recupera apenas caches incremental antigos.
- Com `--stale`, também recupera artefatos stale mais antigos que o período de retenção.
- `--profile-cache` é um modo mais forte que também inclui cache incremental
  recente e diretórios recentes `deps`, `build`, `.fingerprint` e `examples`.
  Verifique primeiro com `--dry-run`.
- `cargo clean` sem opções remove todo o `target/`; opções do Cargo como
  `--package`, `--profile` e `--target` limpam todo o escopo selecionado.
  target-gc limpa por idade e categoria para preservar mais cache de build.
- Se um processo Cargo/rustc ativo parecer usar o target escolhido, a remoção confirmada é recusada.
- Os caminhos removidos ficam limitados a raízes Cargo `target/` validadas.

## Configuração

`target-gc.toml` na raiz do projeto configura a retenção.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
