//! Examples / test cases for identifier pasting.
//!
//! Refer to `idpaste.expanded.rs` to see what this generates.

#![allow(dead_code, unused_variables)]

use derive_deftly::{derive_deftly_adhoc, Deftly};

type FieldType = ();

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct DataType {
    #[deftly(lit = "42")]
    a: (),
    #[deftly(lit = "042")]
    b: (),
    #[deftly(lit = "04\x32")]
    c: (),
    #[deftly(lit = r"42")]
    d: (),
    #[deftly(lit = "42"x)]
    e: (),
}

fn main() {
    let p = |s: &str| print!(" {}", s);
    derive_deftly_adhoc! {
        DataType: $(

        print!("{}", stringify!($fname));
        ${if approx_equal("42",     ${fmeta(lit) as str}) { p("simple"); }}
        ${if approx_equal("042",    ${fmeta(lit) as str}) { p("leftpad"); }}
        ${if approx_equal("04\x32", ${fmeta(lit) as str}) { p("hex"); }}
        ${if approx_equal(r"42",    ${fmeta(lit) as str}) { p("raw"); }}
        ${if approx_equal("42"x,    ${fmeta(lit) as str}) { p("suffix"); }}
        println!(".");
    ) }
}
