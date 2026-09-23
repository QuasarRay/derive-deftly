//! Test errors from missing cargo features
//!
//! This is exercised only
//! *with* the `ui` cargo feature but *without* `full`.

use derive_deftly::{Deftly, derive_deftly_adhoc};

#[derive(Deftly)]
#[deftly(e = "1")]
#[deftly(i = "mod m {}")]
#[derive_deftly_adhoc]
struct S;

derive_deftly_adhoc! {
    S expect items:
}
derive_deftly_adhoc! {
    S:
    const X: () = ${tmeta(d) as expr};
}
derive_deftly_adhoc! {
    S:
    ${tmeta(i) as items};
}
derive_deftly_adhoc! {
    S beta_deftly:
}

fn main() {}
