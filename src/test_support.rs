use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

const SECONDS_PER_DAY: u64 = 86_400;

pub fn temp_dir(scope: &str, tag: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!(
        "cargo-target-gc-{scope}-{tag}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

pub fn cargo_project(scope: &str, tag: &str) -> PathBuf {
    let root = temp_dir(scope, tag);
    write_manifest(&root, "x");
    let target = root.join("target");
    fs::create_dir_all(&target).expect("target");
    write_cachedir_tag(&target);
    root
}

pub fn write_manifest(dir: &Path, name: &str) {
    fs::create_dir_all(dir).expect("manifest parent");
    fs::write(
        dir.join("Cargo.toml"),
        format!("[package]\nname=\"{name}\"\n"),
    )
    .expect("manifest");
}

pub fn write_cachedir_tag(target: &Path) {
    fs::write(target.join("CACHEDIR.TAG"), "Signature").expect("tag");
}

pub fn write_aged(path: &Path, len: usize, age_days: u64) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parents");
    }
    fs::write(path, vec![b'x'; len]).expect("write");
    let when = SystemTime::now()
        .checked_sub(Duration::from_secs(age_days * SECONDS_PER_DAY))
        .expect("aged");
    File::options()
        .write(true)
        .open(path)
        .expect("open")
        .set_modified(when)
        .expect("mtime");
}
