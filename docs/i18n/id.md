# Mulai cepat cargo-target-gc

cargo-target-gc adalah garbage collector untuk artefak Cargo di `target/`. Alat
ini menganalisis direktori `target/` milik proyek atau workspace dan melaporkan
berapa banyak ruang yang bisa diklaim kembali. Artefak build lama yang dianggap
aman hanya dihapus setelah konfirmasi eksplisit.

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
cargo target-gc clean --confirm --stale
cargo target-gc config
```

## Aturan keamanan

- `scan` hanya membaca dan tidak pernah menjalankan Cargo.
- `clean` menolak berjalan tanpa tepat satu dari `--dry-run` atau `--confirm`.
- Secara default, `clean` hanya mengklaim kembali cache incremental lama.
- Dengan `--stale`, artefak stale yang lebih lama dari masa retensi juga diklaim kembali.
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
