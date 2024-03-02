// Tests involving driver and template options

use derive_adhoc::{define_derive_deftly, derive_adhoc, Deftly};

define_derive_deftly! {
    BadOptionsTemplate =
    broken template;
}

#[derive(Deftly)]
#[derive_adhoc(BadOptionsTemplate[dbg])]
#[derive_deftly_adhoc]
struct BadOptionsDriver;

derive_adhoc! {
    BadOptionsDriver with unknown option:
}

derive_adhoc! {
    BadOptionsDriver for wombat:
}

derive_adhoc! {
    BadOptionsDriver for union:
}

derive_adhoc! {
    BadOptionsDriver for struct:
}

derive_adhoc! {
    BadOptionsDriver dbg, expect items:
    syntax error;
}

derive_adhoc! {
    BadOptionsDriver dbg, for struct, for union:
}

define_derive_deftly! {
    #[allow(dead_code)]
    BadAttributeTemplate:
}

derive_adhoc! {
    BadOptionsDriver:

    ${define DEF {}}
    ${defcond COND true}

    $DEF
    $COND
    ${if COND {}}
    ${if DEF {}}
}

derive_adhoc! {
    BadOptionsDriver:

    ${define DEF $DEF}
    $DEF
}

fn main() {}
