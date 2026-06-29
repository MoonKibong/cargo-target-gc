# Mulai cepat cargo-target-gc

cargo-target-gc adalah garbage collector untuk artefak Cargo di `target/`. Alat
ini menganalisis direktori `target/` milik proyek atau workspace dan melaporkan
berapa banyak ruang yang bisa diklaim kembali. Artefak build lama yang dianggap
aman hanya dihapus setelah konfirmasi eksplisit.

## Mengapa ini ada

Direktori Cargo `target/` memang selalu bertambah besar seiring waktu, tetapi
vibe coding dan agentic coding membuat pertumbuhan itu lebih cepat dan lebih
mudah luput dari perhatian. Claude Code, Codex, Gemini CLI, dan agen coding lain
dapat berkali-kali build, test, retry, dan berpindah tugas dalam satu sesi.
cargo-target-gc memberi alur pembersihan konservatif: scan dulu, lihat pratinjau
dengan `--dry-run`, lalu hapus hanya setelah konfirmasi eksplisit.

## Tempat menjalankan

Jalankan di direktori proyek Cargo atau workspace yang sama dengan tempat Anda
menjalankan `cargo build`.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

Jika wrapper seperti `make` membangun proyek Cargo yang bersarang, masuk dulu ke
direktori Cargo tersebut lalu jalankan `cargo target-gc` di sana. Alat ini tidak
menebak lokasi build tersembunyi milik wrapper.

## Perintah utama

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

## Aturan keamanan

- `scan` hanya membaca dan tidak pernah menjalankan Cargo.
- `clean` menolak berjalan tanpa tepat satu dari `--dry-run` atau `--confirm`.
- Secara default, `clean` hanya mengklaim kembali cache incremental lama.
- Dengan `--stale`, artefak stale yang lebih lama dari masa retensi juga diklaim kembali.
- `--profile-cache` adalah mode yang lebih kuat dan juga mencakup cache
  incremental baru serta direktori baru seperti `deps`, `build`, `.fingerprint`,
  dan `examples`. Periksa dulu dengan `--dry-run`.
- `cargo clean` tanpa opsi menghapus seluruh `target/`; opsi Cargo seperti
  `--package`, `--profile`, dan `--target` membersihkan seluruh scope yang
  dipilih. target-gc membersihkan berdasarkan umur dan kategori agar lebih
  banyak cache build tetap tersimpan.
- Jika proses Cargo/rustc aktif tampak memakai target yang dipilih, penghapusan terkonfirmasi ditolak.
- Jalur penghapusan dibatasi ke root Cargo `target/` yang sudah divalidasi.

## Konfigurasi

`target-gc.toml` di root proyek mengatur retensi.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
