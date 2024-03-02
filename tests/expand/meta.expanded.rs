//! Tests for Xmeta etc. edge cases
#![allow(dead_code, unused_variables)]
use derive_adhoc::{derive_deftly_adhoc, Deftly};
#[derive_deftly_adhoc]
struct IgnoredThings {
    #[deftly(prefix = "pfx")]
    #[deftly(multi(hi, there(wombat), path::suffix = "sfx"))]
    #[deftly(mixed(inner = "yes"), mixed = "no")]
    f: String,
}
fn just_pfx_prefix() {}
fn multi_hi_pfx() {}
fn just_suffix_sfx() {}
fn mixed_no() {}
fn mixed_yes() {}
#[deftly("forbidden")]
#[deftly(value = wrong)]
struct MetasNotCheckedIfNoTemplatesExpanded;
fn main() {}
