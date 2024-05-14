//! Test various grouping behaviours, including some with None-grouping
//!
//! None-grouping is very hard to test for,
//! especially since rustc tends to ignore it - rust-lang/rust#67062.
#![allow(dead_code)]
use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};
use std::fmt::Display;
fn assert(ok: bool) {
    if !ok {
        {
            #[cold]
            #[track_caller]
            #[inline(never)]
            const fn panic_cold_explicit() -> ! {
                ::core::panicking::panic_explicit()
            }
            panic_cold_explicit();
        }
    }
}
#[derive_deftly_adhoc]
struct OperatorPrecedence {
    #[deftly(value = "2")]
    f1: (),
    #[deftly(value = "3 + 4")]
    f2: (),
}
#[derive_deftly_adhoc]
enum OurOption {
    None,
}
impl OurOption {
    fn is_none(&self) -> bool {
        true
    }
}
#[derive_deftly(UseFieldsLikeOption)]
#[rustfmt::skip]
struct UseFieldsLikeOption {
    ours: OurOption,
    stds: Option<()>,
}
impl UseFieldsLikeOption {
    fn construct_each_field() {
        let none = OurOption::None;
        assert(OurOption::is_none(&none));
        let none = Option::<()>::None;
        assert(Option::<()>::is_none(&none));
    }
}
#[derive_deftly_adhoc]
struct TypePrecedenceDyn {
    parens: (dyn Display + 'static),
}
fn main() {
    let product = 2 * 3 + 4 * 1;
    assert(product == 10);
    let product = 2 * (3 + 4);
    assert(product == 14);
    struct TypePrecedenceDynStaticRefs {
        parens: &'static (dyn Display + 'static),
    }
}
