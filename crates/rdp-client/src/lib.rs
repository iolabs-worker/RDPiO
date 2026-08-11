//! RDPiO client library facade.
//!
//! The `rdpio` binary lives in this crate (`src/main.rs`); the library target
//! exists so the shared persistence types — [`connections::ConnectionProfile`]
//! and [`connections::ConnectionStore`] — are usable by other crates and by
//! tests without pulling in the binary's platform modules.
//!
//! The binary itself keeps its module tree private to `main.rs` (see
//! `mod connect`/`mod connections` there); this library re-exposes only the
//! self-contained, cross-platform pieces.

pub mod connections;
