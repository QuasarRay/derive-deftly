// Tests involving driver and template options

use derive_adhoc::{define_derive_deftly, derive_deftly_adhoc, Deftly};

define_derive_deftly! {
    BadOptionsTemplate =
    broken template;
}

#[derive(Deftly)]
#[derive_deftly(BadOptionsTemplate[dbg])]
#[derive_deftly_adhoc]
struct BadOptionsDriver;

derive_deftly_adhoc! {
    BadOptionsDriver with unknown option:
}

derive_deftly_adhoc! {
    BadOptionsDriver for wombat:
}

derive_deftly_adhoc! {
    BadOptionsDriver for union:
}

derive_deftly_adhoc! {
    BadOptionsDriver for struct:
}

derive_deftly_adhoc! {
    BadOptionsDriver dbg, expect items:
    syntax error;
}

derive_deftly_adhoc! {
    BadOptionsDriver dbg, for struct, for union:
}

define_derive_deftly! {
    #[allow(dead_code)]
    BadAttributeTemplate:
}

derive_deftly_adhoc! {
    BadOptionsDriver:

    ${define DEF {}}
    ${defcond COND true}

    $DEF
    $COND
    ${if COND {}}
    ${if DEF {}}
}

derive_deftly_adhoc! {
    BadOptionsDriver:

    ${define DEF $DEF}
    $DEF
}

fn main() {}
