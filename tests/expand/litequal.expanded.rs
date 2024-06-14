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
    #[deftly(lit = "42"x)]
    e: (),
}
fn main() {
    let p = |s: &str| {
        ::std::io::_print(format_args!(" {0}", s));
    };
    {
        ::std::io::_print(format_args!("{0}", "a"));
    };
    p("simple");
    {
        ::std::io::_print(format_args!(".\n"));
    };
    {
        ::std::io::_print(format_args!("{0}", "b"));
    };
    p("leftpad");
    {
        ::std::io::_print(format_args!(".\n"));
    };
    {
        ::std::io::_print(format_args!("{0}", "c"));
    };
    p("hex");
    {
        ::std::io::_print(format_args!(".\n"));
    };
    {
        ::std::io::_print(format_args!("{0}", "d"));
    };
    p("raw");
    {
        ::std::io::_print(format_args!(".\n"));
    };
    {
        ::std::io::_print(format_args!("{0}", "e"));
    };
    p("suffix");
    {
        ::std::io::_print(format_args!(".\n"));
    };
}
