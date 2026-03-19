#![allow(unused)]

use derive_deftly::{derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct S;

derive_deftly_adhoc! {
    S beta_deftly:

    const _: [&str] = &[
        $"unclosed ${",
        $"unclosed $(",
        $"unclosed $[",
        $"unclosed $<",
        $"mismatched ${ )",
        $"mismatched $( ]",
        $"mismatched $[ }",
        $"nest mismatched $( ${ ] )",
        $"nest mismatched $( $( } )",
        $"nest mismatched $( $[ ) )",
        $r#"forbid $# wrong-expansion"#,
        $r#"forbid ${concat ">"} quotes"#,
        $r#"forbid ${define X { \ }} backslash"#,
        $r#"forbid ${define X { ' }} apostrophe"#,
        $r#"un-lexable ${define X { let 🦀 = 1; }}."#,
        $r#"un-parseable ${invalid_expansion}."#,
    ];
}

fn main() {}
