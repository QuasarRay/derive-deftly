#![allow(dead_code)]

use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};

define_derive_deftly! {
    ReuseableTemplate:

    $tattrs struct ${paste ReusedWith $tname};
}

#[derive(Deftly)]
#[derive_deftly(ReuseableTemplate)]
#[derive_deftly_adhoc]
#[doc=stringify!(one $ dollar $template end)]
struct OneDollar;

derive_deftly_adhoc! {
    OneDollar:

    $tattrs struct AdhocOneDollar;
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[derive_deftly(ReuseableTemplate)]
#[doc=stringify!(two $$ dollars)]
struct TwoDollars;

derive_deftly_adhoc! {
    TwoDollars:

    $tattrs struct AdhocTwoDollars;
}

fn main() {}
