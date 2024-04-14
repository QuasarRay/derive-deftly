// Tests that cause internal macros to generate their "incompatible versions"
// advice.  Actually generating this advice in a more realistic scenario,
// rather than just directly invoking the macros, is quite hard.  So this
// test is not particularly realistic.  But it does exercise the error
// generation code paths.

use derive_deftly::{define_derive_deftly, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct Driver;

define_derive_deftly! {
    Template =
}

derive_deftly_driver_Driver! { GARBAGE -> DRIVER }
derive_deftly_template_Template! { GARBAGE -> TEMPLATE }
derive_deftly::derive_deftly_engine! { GARBAGE -> INNARDS }

// OpCompatVersions 1.0, ie our own, and the earliest released
derive_deftly::derive_deftly_engine! {
    { pub struct StructName {} }
    [1 0]
    ( )
    { pub struct OptOk0; }
    ( crate; [] Template; )
    // XXXX
    RANDOM STUFF
}

// OpCompatVersions 1.200, some imaginary future compatible one
derive_deftly::derive_deftly_engine! {
    { pub struct StructName {} }
    [1 200]
    ( )
    { pub struct OptOk200; }
    ( crate; [] Template; )
    // XXXX
    RANDOM STUFF
}

// OpCompatVersions 200.1, some imaginary future incompatible one
derive_deftly::derive_deftly_engine! {
    { pub struct StructName {} }
    [200 0]
    ( )
    { OUGHT NOT TO BE EXPANDED; }
    ( crate; [] Template; )
    // XXXX
    RANDOM STUFF
}

fn main() {
    let _ = OptOk0;
    let _ = OptOk200;
}
