# cargo-target-gc त्वरित आरंभ

cargo-target-gc Cargo के `target/` आर्टिफैक्ट के लिए garbage collector है। यह
प्रोजेक्ट या workspace की `target/` डायरेक्टरी का विश्लेषण करता है और बताता है
कि कितनी जगह वापस पाई जा सकती है। यह पुराने और सुरक्षित माने गए build
आर्टिफैक्ट केवल स्पष्ट पुष्टि के बाद हटाता है।

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
cargo target-gc clean --confirm --stale
cargo target-gc config
```

## सुरक्षा नियम

- `scan` read-only है और कभी Cargo नहीं चलाता।
- `clean` बिना ठीक एक flag, `--dry-run` या `--confirm`, के चलने से मना करता है।
- default रूप से `clean` केवल पुराने incremental cache को reclaim करता है।
- `--stale` जोड़ने पर retention अवधि से पुराने stale आर्टिफैक्ट भी reclaim होते हैं।
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
