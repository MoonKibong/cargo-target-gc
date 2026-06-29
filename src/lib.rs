//! cargo-target-gc — a read-only Cargo target-artifact garbage collector.
//!
//! `scan` is a pure filesystem analysis of `target/` directories: it NEVER
//! invokes cargo and creates no build artifacts. `clean` reuses the same walk
//! to remove only reclaimable artifacts, and only with explicit confirmation.

pub mod active;
pub mod clean;
pub mod cli;
pub mod config;
pub mod discovery;
pub mod report;
pub mod scan;
pub mod target;

#[cfg(test)]
mod test_support;
