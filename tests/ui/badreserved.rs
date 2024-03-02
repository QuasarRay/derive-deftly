// Tests of syntax we may wish to use for things in the future

use derive_adhoc::{define_derive_deftly, derive_adhoc, Deftly};

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
    $r#"template content?"#
}

derive_deftly_adhoc! {
    ReserveDriver:

    mod prevent {
        //! template doc comment?
    }
}

derive_deftly_adhoc! {
    ReserveDriver:

    $/// template doc comment?
    mod prevent {
    }
}

derive_deftly_adhoc! {
    ReserveDriver:

    ${define 0 { mod x {} }}
    $0
}

fn main() {}
