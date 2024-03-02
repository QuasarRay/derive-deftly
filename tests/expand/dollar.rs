#![allow(dead_code)]

use derive_adhoc::{define_derive_deftly, derive_adhoc, Deftly};

define_derive_deftly! {
    ReuseableTemplate =

    $tattrs struct ${paste ReusedWith $tname};
}

#[derive(Deftly)]
#[derive_adhoc(ReuseableTemplate)]
#[derive_deftly_adhoc]
#[doc=stringify!(one $ dollar $template end)]
struct OneDollar;

derive_adhoc! {
    OneDollar:

    $tattrs struct AdhocOneDollar;
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[derive_adhoc(ReuseableTemplate)]
#[doc=stringify!(two $$ dollars)]
struct TwoDollars;

derive_adhoc! {
    TwoDollars:

    $tattrs struct AdhocTwoDollars;
}

fn main() {}
