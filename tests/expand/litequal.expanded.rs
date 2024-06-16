//! Examples / test cases for identifier pasting.
//!
//! Refer to `idpaste.expanded.rs` to see what this generates.
#![allow(dead_code, unused_variables)]
use derive_deftly::{derive_deftly_adhoc, Deftly};
type FieldType = ();
#[derive_deftly_adhoc(export)]
struct DataType {
    #[deftly(lit = "42")]
    a: (),
    #[deftly(lit = "042")]
    b: (),
    #[deftly(lit = "04\x32")]
    c: (),
    #[deftly(lit = r"42")]
    d: (),
}
fn main() {
    let p = |s: &str| {
        ::std::io::_print(format_args!("{0}", s));
    };
    p("a");
    p(" simple");
    p(" raw");
    p(".\n");
    p("b");
    p(" leftpad");
    p(" hex");
    p(".\n");
    p("c");
    p(" leftpad");
    p(" hex");
    p(".\n");
    p("d");
    p(" simple");
    p(" raw");
    p(".\n");
}
