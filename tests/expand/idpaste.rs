//! Examples / test cases for identifier pasting.
//!
//! Refer to `idpaste.expanded.rs` to see what this generates.

#![allow(dead_code, unused_variables)]

use derive_deftly::{derive_deftly_adhoc, Deftly};

type FieldType = ();

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct TypeNames {
    /// We use std::slice::Chunks here because that way we can test
    /// identifier pasting with a whole path.  The macro `TypeNames`
    /// will generate a field with type `RChunksMut.
    error: std::slice::Chunks<'static, ()>,

    // Test `${paste }` of a `(...)`-grouped ty
    #[allow(unused_parens)]
    parens: (std::slice::Chunks<'static, ()>),
}

derive_deftly_adhoc! {
    TypeNames:

    struct ${paste Pre $tdeftype Post} {
        $(
            $fname: ${paste R $ftype Mut},
        )
    }
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct TopName<F> {
    top_name_field: F,
}

derive_deftly_adhoc! {
    TopName:

    #[allow(non_snake_case)]
    struct ${paste Pre $tdeftype Post} {
        $( ${pascal_case $fname}: $ftype )
    }

    impl<$tgens> ::std::panic::RefUnwindSafe for ${paste Pre $ttype Post} {}
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(t_frag = "TFrag")]
struct ExpandName {
    #[deftly(prefix = "attr", suffix = "24")]
    f: FieldType,

    k: String,

    #[deftly(prefix_i = "pub", suffix = "await")]
    p: u32,

    #[deftly(prefix_i = "r#pub", suffix = "")]
    r: u32,
}

derive_deftly_adhoc! {
    ExpandName:

    ${define VIA_DEFINE ${paste ${tmeta(t_frag) as ident}}}
    ${define CONCAT_DEFINE ${concat "$ttype = " $ttype "."}}

    struct ${paste "Pre" $tdeftype "Post"} {
        $(
            ${paste
              ${if fmeta(prefix) { ${fmeta(prefix)} _ } }
              ${if fmeta(prefix_i) { ${fmeta(prefix_i) as ident} _ } }
              $fname
              ${if fmeta(suffix) { _ ${fmeta(suffix) as str} } }
            }: $ftype,
        )
    }

    struct ${pascal_case ${paste wom_ ${shouty_snake_case $tname}
                           ${if true { _bat } else { _noise }}}};

    struct $<ViaDefine $VIA_DEFINE>;

    const HUMAN_STR: &str = ${concat $CONCAT_DEFINE};
    const COMPUTER_STR: &str = ${concat $VIA_DEFINE};
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
enum EdgeCases {
    Tuple(u32),
    Struct { r#for: u32, r#unneeded: u32 },
}

derive_deftly_adhoc! {
    EdgeCases:

    impl $ttype {
        fn $<edge_ $<"0" _end>>() {}
        fn $<edge_ $( $fname )>() {}
        fn $<$tdefkwd>() {}
        $(
            fn $<binding_ $<$fname> _end>() {}
            // `: `ok_0_tail`, `ok_field_tail`
        )
        fn body() {
            $(
                ${when v_is_named}
                let $fname = ();
                let $<$fname> = ();
            )
        }
    }
}

fn main() {}
