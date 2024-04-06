//! Examples / test cases for identifier pasting.
//!
//! Refer to `idpaste.expanded.rs` to see what this generates.

#![allow(dead_code, unused_variables)]

use derive_deftly::{derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct DataType {
    field: String,
}

const FIELD_NAMES: &[&str] = derive_deftly_adhoc! {
    DataType expect expr:
    &[ $( stringify!($fname), ) ]
};

fn main() {}
