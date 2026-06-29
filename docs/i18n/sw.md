# Kuanza haraka na cargo-target-gc

cargo-target-gc ni garbage collector ya artefact za Cargo ndani ya `target/`.
Inachambua saraka ya `target/` ya mradi au workspace na kuripoti nafasi
inayoweza kurejeshwa. Hufuta artefact za build za zamani na zilizo salama tu
baada ya uthibitisho wa wazi.

## Kwa nini ipo

Saraka za Cargo `target/` zimekuwa zikikua kadiri muda unavyopita, lakini vibe
coding na agentic coding hufanya zikue haraka zaidi na iwe rahisi kutoona. Claude
Code, Codex, Gemini CLI na mawakala wengine wa coding wanaweza kujenga, kupima,
kurudia na kubadili kazi mara nyingi katika session moja. cargo-target-gc hutoa
mtiririko wa tahadhari: scan kwanza, hakiki kwa `--dry-run`, kisha futa tu baada
ya uthibitisho wa wazi.

## Mahali pa kuendesha

Iendeshe kwenye saraka ile ile ya mradi wa Cargo au workspace ambapo ungeendesha
`cargo build`.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

Ikiwa wrapper kama `make` inajenga mradi wa Cargo ulio ndani, ingia kwanza kwenye
saraka hiyo ya Cargo kisha endesha `cargo target-gc` hapo. Zana hii haikisi njia
za build zilizofichwa na wrapper.

## Amri kuu

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

## Kanuni za usalama

- `scan` husoma tu na haiendeshi Cargo kamwe.
- `clean` hukataa kuendelea bila moja tu kati ya `--dry-run` au `--confirm`.
- Kwa kawaida, `clean` hurejesha cache ya incremental ya zamani pekee.
- Ukiweka `--stale`, pia hurejesha artefact stale zilizozeeka kuliko muda wa kuhifadhi.
- `--profile-cache` ni hali yenye nguvu zaidi na pia hujumuisha incremental
  cache mpya na saraka mpya `deps`, `build`, `.fingerprint` na `examples`.
  Kagua kwanza kwa `--dry-run`.
- `cargo clean` bila chaguo hufuta `target/` yote; chaguo za Cargo kama
  `--package`, `--profile`, na `--target` husafisha scope yote iliyochaguliwa.
  target-gc husafisha kwa umri na aina ili kuhifadhi build cache zaidi.
- Ikiwa process hai ya Cargo/rustc inaonekana kutumia target iliyochaguliwa, ufutaji uliothibitishwa hukataliwa.
- Njia za kufuta huzuiwa ndani ya mizizi ya Cargo `target/` iliyothibitishwa.

## Usanidi

`target-gc.toml` kwenye root ya mradi huweka muda wa kuhifadhi.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
