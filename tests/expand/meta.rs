//! Tests for Xmeta etc. edge cases

#![allow(dead_code, unused_variables)]

use derive_adhoc::{derive_adhoc, Adhoc};

#[derive(Adhoc)]
struct IgnoredThings {
    #[adhoc(prefix = "pfx")]
    #[adhoc(multi("42", hi, there(wombat), suffix = "sfx"))]
    #[adhoc(mixed(inner = "yes"), mixed = "no")]
    f: String,
}

derive_adhoc! {
    IgnoredThings:

  $(
    // normal fmeta parsing
    fn $<just_  ${fmeta(prefix)} _prefix>() {}

    // we can pick out "hi" even though the list contains other stuff
    ${if fmeta(multi(hi)) {
      fn $<multi_hi_ ${fmeta(prefix)}>() {}
    }}

    // we can pick out "suffix" too
    fn $<just_suffix_ ${fmeta(multi(suffix))}>() {}

    // TODO: there is no way to get access to the multi("42"),
    // it probably ought to be rejected.

    // Ignores the mixed(inner) = "yes" and finds the "no"
    fn $<mixed_ ${fmeta(mixed)}>() {}
    // Ignores the mixed = "no" and finds the inner
    fn $<mixed_ ${fmeta(mixed(inner))}>() {}
  )
}

// Arguably it is wrong that this is accepted.  However, rejecting it
// would have complex semver implications.  Maybe the template is
// from a newer derive-adhoc which handles this case, for example.
#[derive(Adhoc)]
#[adhoc("forbidden")]
#[adhoc(value = wrong)]
struct MetasNotCheckedIfNoTemplatesExpanded;

fn main() {}
