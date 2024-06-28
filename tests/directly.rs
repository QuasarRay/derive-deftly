//! Arrangements for testing derive-deftly code directly
//!
//! We don't want to include all the exciting test code in
//! `derive-deftly-macros`'s `cargo test`.
//!
//! Instead, we re-import the same source files here.
//!
//! Currently, this makes a testing version of derive-deftly without any
//! of the optional features.

use macros::approx_equal::flatten_none_groups;
use macros::framework::*;
use macros::Concatenated;

use super::*;

use syn::parse_quote;

#[allow(dead_code)]
#[path = "../macros/macros.rs"]
pub(super) mod macros;

mod tscompare;
use tscompare::*;

// PreprocessedTree contains a Cell which shouldn't be Clone,
// since that duplicates the interior mutability, which can lead to bugs.
assert_not_impl_any!(macros::meta::PreprocessedTree: Clone);

mod check_approx_equal;
mod check_details;
mod check_meta_coding;

#[cfg(all(
    feature = "recent", // examples may not work with old compiler
    feature = "case",
))]
mod check_examples;

#[cfg(all(
    feature = "recent", // we want toml parser etc.
))]
mod check_semver_check;
