//! Tests for Xmeta etc. edge cases
#![allow(dead_code, unused_variables)]
use derive_adhoc::{derive_adhoc, Adhoc};
struct IgnoredThings {
    #[adhoc(prefix = "pfx")]
    #[adhoc(multi("42", hi, there(wombat), suffix = "sfx"))]
    f: String,
}
fn just_pfx_prefix() {}
fn multi_hi_pfx() {}
fn just_suffix_sfx() {}
#[adhoc("forbidden")]
#[adhoc(value = wrong)]
struct MetasNotCheckedIfNoTemplatesExpanded;
fn main() {}
