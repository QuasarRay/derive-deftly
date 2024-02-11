//! Tests for Xmeta etc. edge cases
#![allow(dead_code, unused_variables)]
use derive_adhoc::{derive_adhoc, Adhoc};
#[derive_truly_adhoc]
struct IgnoredThings {
    #[adhoc(prefix = "pfx")]
    #[adhoc(multi(hi, there(wombat), path::suffix = "sfx"))]
    #[adhoc(mixed(inner = "yes"), mixed = "no")]
    f: String,
}
fn just_pfx_prefix() {}
fn multi_hi_pfx() {}
fn just_suffix_sfx() {}
fn mixed_no() {}
fn mixed_yes() {}
#[adhoc("forbidden")]
#[adhoc(value = wrong)]
struct MetasNotCheckedIfNoTemplatesExpanded;
fn main() {}
