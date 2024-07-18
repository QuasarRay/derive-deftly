//! Tests for Xmeta etc. edge cases

#![allow(dead_code, unused_variables)]

use derive_deftly::{derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct IgnoredThings {
    #[deftly(prefix = "pfx")]
    #[deftly(multi(hi, there(wombat), path::suffix = "sfx"))]
    #[deftly(mixed(inner = "yes"), mixed = "no")]
    f: String,
}

derive_deftly_adhoc! {
    IgnoredThings beta_deftly:

  $(
    // normal fmeta parsing
    fn $<just_  ${fmeta(prefix)} _prefix>() {}

    // we can pick out "hi" even though the list contains other stuff
    ${if fmeta(multi(hi)) {
      fn $<multi_hi_ ${fmeta(prefix)}>() {}
    }}

    // we can pick out "path::suffix" too
    fn $<just_suffix_ ${fmeta(multi(path::suffix))}>() {}

    // Ignores the mixed(inner) = "yes" and finds the "no"
    fn $<mixed_ ${fmeta(mixed)}>() {}
    // Ignores the mixed = "no" and finds the inner
    fn $<mixed_ ${fmeta(mixed(inner))}>() {}
  )

    fn ${tmeta(absent) as ident, default { defaulted }}() {}
}

fn main() {}
