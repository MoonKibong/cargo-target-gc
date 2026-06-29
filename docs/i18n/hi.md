# cargo-target-gc त्वरित आरंभ

cargo-target-gc Cargo के `target/` आर्टिफैक्ट के लिए garbage collector है। यह
प्रोजेक्ट या workspace की `target/` डायरेक्टरी का विश्लेषण करता है और बताता है
कि कितनी जगह वापस पाई जा सकती है। यह पुराने और सुरक्षित माने गए build
आर्टिफैक्ट केवल स्पष्ट पुष्टि के बाद हटाता है।

## यह क्यों उपयोगी है

Cargo की `target/` डायरेक्टरी समय के साथ पहले भी बढ़ती थी, लेकिन vibe coding और
agentic coding में यह बढ़ोतरी तेज और कम दिखाई देने वाली हो जाती है। Claude Code,
Codex, Gemini CLI और दूसरे coding agents एक ही session में कई बार build, test,
retry और task switch कर सकते हैं। cargo-target-gc एक conservative cleanup flow
देता है: पहले scan करें, `--dry-run` से preview देखें, और केवल स्पष्ट पुष्टि के
बाद delete करें।

## कहाँ चलाएँ

इसे उसी Cargo प्रोजेक्ट या workspace डायरेक्टरी में चलाएँ जहाँ आप `cargo build`
चलाते हैं।

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

अगर `make` जैसा wrapper किसी nested Cargo प्रोजेक्ट को build करता है, तो पहले
उस Cargo डायरेक्टरी में जाएँ और वहीं `cargo target-gc` चलाएँ। यह टूल छिपे हुए
wrapper build paths का अनुमान नहीं लगाता।

## मुख्य commands

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

## सुरक्षा नियम

- `scan` read-only है और कभी Cargo नहीं चलाता।
- `clean` बिना ठीक एक flag, `--dry-run` या `--confirm`, के चलने से मना करता है।
- default रूप से `clean` केवल पुराने incremental cache को reclaim करता है।
- `--stale` जोड़ने पर retention अवधि से पुराने stale आर्टिफैक्ट भी reclaim होते हैं।
- `--profile-cache` अधिक मजबूत mode है; यह fresh incremental cache और हाल के
  `deps`, `build`, `.fingerprint` और `examples` directories को भी शामिल करता
  है। पहले `--dry-run` से देखें।
- बिना options के `cargo clean` पूरा `target/` हटाता है; Cargo options जैसे
  `--package`, `--profile`, और `--target` पूरे चुने हुए scope को साफ करते हैं।
  target-gc age और category के आधार पर साफ करता है ताकि अधिक build cache बचा
  रहे।
- अगर कोई active Cargo/rustc process चुने हुए target root का उपयोग करता दिखे, confirmed delete रोका जाता है।
- delete paths केवल validated Cargo `target/` root के अंदर सीमित रहते हैं।

## Configuration

प्रोजेक्ट root में `target-gc.toml` retention settings देता है।

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
