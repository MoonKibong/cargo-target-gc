# Bắt đầu nhanh với cargo-target-gc

cargo-target-gc là công cụ dọn rác cho artifact Cargo trong `target/`. Công cụ
phân tích thư mục `target/` của project hoặc workspace và báo cáo dung lượng có
thể thu hồi. Nó chỉ xóa artifact build cũ được xem là an toàn sau khi có xác
nhận rõ ràng.

## Chạy ở đâu

Chạy trong cùng thư mục project Cargo hoặc workspace nơi bạn thường chạy
`cargo build`.

```bash
cd path/to/cargo-project
cargo build
cargo target-gc scan
```

Nếu wrapper như `make` build một project Cargo lồng bên trong, hãy vào thư mục
Cargo đó trước rồi chạy `cargo target-gc`. Công cụ không đoán các đường dẫn
build ẩn của wrapper.

## Lệnh chính

```bash
cargo target-gc scan
cargo target-gc scan --json
cargo target-gc clean --dry-run
cargo target-gc clean --dry-run --stale
cargo target-gc clean --confirm --stale
cargo target-gc config
```

## Quy tắc an toàn

- `scan` chỉ đọc và không bao giờ chạy Cargo.
- `clean` từ chối chạy nếu không có đúng một trong `--dry-run` hoặc `--confirm`.
- Theo mặc định, `clean` chỉ thu hồi cache incremental cũ.
- Với `--stale`, artifact stale cũ hơn thời gian giữ lại cũng được thu hồi.
- Nếu process Cargo/rustc đang hoạt động có vẻ dùng target đã chọn, thao tác xóa có xác nhận sẽ bị từ chối.
- Đường dẫn xóa bị giới hạn trong root Cargo `target/` đã được xác thực.

## Cấu hình

`target-gc.toml` ở root project cấu hình thời gian giữ lại.

```toml
retention_days = 14
incremental_retention_hours = 24
# max_reclaim_bytes = 1073741824
# crate_path = "crates/core"
```
