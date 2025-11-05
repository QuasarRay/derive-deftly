#![allow(unused)]

use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct DataType;

define_derive_deftly! {
    Break:

    $require_beta
}

derive_deftly_adhoc! {
    DataType:

    $require_beta
}

fn main() {}
