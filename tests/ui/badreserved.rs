// Tests of syntax we may wish to use for things in the future

use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct ReserveDriver;

derive_deftly_adhoc! {
    ReserveDriver:
    $[ future ]
}

derive_deftly_adhoc! {
    ReserveDriver:
    $<ident:upper>
    ${paste ident:upper}
}

derive_deftly_adhoc! {
    ReserveDriver:

    mod prevent {
        //! template doc comment?
    }
}

derive_deftly_adhoc! {
    ReserveDriver:

    mod prevent {
        $//! string template inner attr?
    }
}

derive_deftly_adhoc! {
    ReserveDriver:

    ${define 0 { mod x {} }}
    $0
}

define_derive_deftly! {
    lowercase:
}

define_derive_deftly! {
    __lowercase:
}

define_derive_deftly! {
    _0unused:
}

define_derive_deftly! {
    _Unused:
}

fn main() {}
