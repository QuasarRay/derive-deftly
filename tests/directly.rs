//! Arrangements for testing derive-deftly code directly
//!
//! We don't want to include all the exciting test code in
//! `derive-deftly-macros`'s `cargo test`.
//!
//! Instead, we re-import the same source files here.
//!
//! Currently, this makes a testing version of derive-deftly without any
//! of the optional features.

use macros::framework::*;
use macros::Concatenated;

use super::*;

use syn::parse_quote;

#[allow(dead_code)]
#[path = "../macros/macros.rs"]
pub(super) mod macros;

#[cfg(all(
    feature = "recent", // examples may not work with old compiler
    feature = "case",
))]
mod check_examples;

#[cfg(all(
    feature = "recent", // we want toml parser etc.
))]
mod check_semver_check;
