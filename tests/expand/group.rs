//! Test various grouping behaviours, including some with None-grouping
//!
//! None-grouping is very hard to test for,
//! especially since rustc tends to ignore it - rust-lang/rust#67062.

#![allow(dead_code)]

use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};

use std::fmt::Display;

fn assert(ok: bool) {
    if !ok {
        panic!()
    }
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct OperatorPrecedence {
    #[deftly(value = "2")]
    f1: (),
    #[deftly(value = "3 + 4")]
    f2: (),
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
enum OurOption {
    None,
}

impl OurOption {
    fn is_none(&self) -> bool {
        true
    }
}

define_derive_deftly! {
    UseFieldsLikeOption =

    impl $ttype {
        fn construct_each_field() {
            $(
                let none = $ftype::None;
                assert($ftype::is_none(&none));
            )
        }
    }
}

#[derive(Deftly)]
#[derive_deftly(UseFieldsLikeOption)]
#[rustfmt::skip] // TODO #74 rustfmt wants to remove the :: in `stds`
struct UseFieldsLikeOption {
    ours: OurOption,
    stds: Option::<()>,
    // Test that inherent types, and methods, can be made from `$ftype`
    // even if in the struct the type is specified without colons.
    /* BUT TODO #74
    std_no_colon: Option<()>,
     */
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct TypePrecedenceDyn {
    parens: (dyn Display + 'static),
    // Test that $ftype's None group is effective to disambiguate
    /* BUT we can't do that right now, because
     * rust-lang/rust/issues/124817
     * rust-lang/rust/issues/67062
     * "error: ambiguous `+` in a type"
    bare: dyn Display + 'static, //
     */
}

fn main() {
    // Test that `... as expr` gets precedence right.
    let product = derive_deftly_adhoc!(
        OperatorPrecedence:

        $(
            ${fmeta(value) as expr} *
        ) 1
    );
    assert(product == 14);

    // Test that just `${Xmeta()}` gets precedence right for an expression.
    /* BUT it doesn't right now, because
     * https://github.com/rust-lang/rust/issues/124974
     * https://github.com/rust-lang/rust/issues/67062
    let product = derive_deftly_adhoc!(
        OperatorPrecedence:

        $(
            ${fmeta(value) /* future default: None-Grouped tokens */ } *
        ) 1
    );
    assert!(product == 14);
     */

    // Test showing raw tokens *wrong* answer, due to unexpected precedence
    let product = derive_deftly_adhoc!(
        OperatorPrecedence:

        $(
            ${fmeta(value) as token_stream} *
        ) 1
    );
    assert(product == 10);

    // Test that there's an implicit group around an expression proc macro
    let product = 2 * derive_deftly_adhoc!(OperatorPrecedence: 3 + 4);
    assert(product == 14);

    // Invokes the test cases in `TypePrecedenceDyn`
    derive_deftly_adhoc! {
        TypePrecedenceDyn:

        struct $<$ttype StaticRefs> {
            $(
                $fname: &'static $ftype,
            )
        }
    }
}
