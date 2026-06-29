//! derust — read-only Rust project health and refactoring-readiness scan library.
//!
//! All modules are read-only by design: probes never mutate the target project.
//! Auto-fix is intentionally out of scope (see README "Future work").

pub mod cli;
pub mod config;
pub mod discovery;
pub mod probe;
pub mod report;
pub mod scan;
