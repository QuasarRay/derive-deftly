//! Arrangements for testing derive-deftly code directly
//!
//! We don't want to include all the exciting test code in
//! `derive-deftly-macros`'s `cargo test`.
//!
//! Instead, we re-import the same source files here.
//!
//! Currently, this makes a testing version of derive-deftly without any
//! of the optional features.

use macros::approx_equal::flatten_none_groups;
use macros::framework::*;
use macros::Concatenated;

use super::*;

use syn::parse_quote;

#[allow(dead_code)]
#[path = "../macros/macros.rs"]
pub(super) mod macros;

// PreprocessedTree contains a Cell which shouldn't be Clone,
// since that duplicates the interior mutability, which can lead to bugs.
assert_not_impl_any!(macros::meta::PreprocessedTree: Clone);

mod check_meta_coding;

#[cfg(all(
    feature = "recent", // examples may not work with old compiler
    feature = "case",
))]
mod check_examples;

#[cfg(all(
    feature = "recent", // we want toml parser etc.
))]
mod check_semver_check;

#[test]
fn check_approx_equal() {
    // Testing all the corner cases for literals is quite hard.

    // negative literals are deeply weird, see
    //   https://doc.rust-lang.org/reference/procedural-macros.html#declarative-macro-tokens-and-procedural-macro-tokens
    // so we must test this in a crazy way too.
    //
    // We can make single-token negative literals with
    // the constructors on proc_macro2::Literal, or with the FromStr impl.
    // Empirically, though, a proc_macro2::TokenStream *cannot*
    // contain negative literals!  They seem to be converted on input.

    // So, anyway, we test comparisons of tokenstreams constructed
    //  - by parsing a string
    //  - by putting a single literal into a TS
    // (as returned by versions()), in all four combinations,
    // for each test case.

    fn versions(s: &str) -> impl IntoIterator<Item = (&str, TokenStream)> {
        let s = s.trim();

        let ts_direct: TokenStream = syn::parse_str(s).expect(s);
        if s.starts_with('-') {
            match ts_direct.clone().into_iter().next() {
                Some(TT::Punct(p)) if p.as_char() == '-' => {}
                other => panic!("{:?} => {:?}", s, other),
            }
        }

        let lit = proc_macro2::Literal::from_str(s).expect(s);
        let tt = TT::Literal(lit.clone());
        let ts_from_tt: TokenStream = iter::once(tt).collect();
        vec![
            ("parse2::<TS>", ts_direct),
            ("Literal::from_str", ts_from_tt),
        ]
    }

    fn chk(s: &str) {
        for t in s.split('\n').map(|t| t.trim()).filter(|t| !t.is_empty()) {
            eprintln!("{}", t);
            let (l, r) = t.rsplit_once('=').unwrap();
            let mut l = l.chars();
            let o = l.next_back().unwrap();
            let l = l.as_str();
            let exp = match o {
                '=' => Ok(Equality::Equal),
                '!' => Ok(Equality::Different),
                '?' => Err(()),
                other => panic!("{:?}", other),
            };
            for (lv, l) in versions(l) {
                for (rv, r) in versions(r) {
                    eprintln!("{} {} {}", t, lv, rv);
                    let got = tokens_cmpeq(l.clone(), r).map_err(|_| ());
                    assert_eq!(got, exp);
                }
            }
        }
    }

    // literals; they are compared by their string representation

    // str
    chk(r#"
        "hi"    != "ho"
        "hi"    != "h\x69"
        "hi"x   != "hi"
        "hi"    != r"hi"
        "hi"    != b"hi"
    "#);

    // byte str
    chk(r#"
        b"hi"   != b"ho"
        b"hi"   != b"h\x69"
        b"hi"x  != b"hi"
        b"hi"   != br"hi"
        b"i"    !=  'i'
        b"i"    != b'i'
    "#);

    // char
    chk(r#"
        'i'     != 'o'
        'i'     == 'i'
        'i'     != '\x69'
        'i'x    != 'i'
        'i'     != b'i'
    "#);

    // byte
    chk(r#"
        b'i'    != b'o'
        b'i'    == b'i'
        b'i'    != b'\x69'
        b'i'x   != b'i'
        b'i'    != 0x69
    "#);

    // int
    chk(r#"
        1       != 2
        1       != -1
        1       != 01
        1       != 0o1
        1       != 0b1
        1       != 0x1
        1       != 1_u32
        1u32    != 1_u32
        -1u32   != -1_u32

        0xffff_ffff_ffff_ffff == 0xffff_ffff_ffff_ffff
        -0xffff_ffff_ffff_ffff == -0xffff_ffff_ffff_ffff

        0x7777_1234_5678_1234_5678_1234_5678_1234_5678 != 1
        0x1234_5678_1234_5678_1234_5678_1234_5678 == 0x1234_5678_1234_5678_1234_5678_1234_5678
    "#);

    // float
    // Rust has no non-finite floating point literals,
    // so we don't need to test NaN or Inf
    chk(r#"
        32      != 32.0
        32.0    == 32.0
        -32.0   == -32.0
        32.     != 32.0
        32.0_f32 != 32.0f32
        32.0    != 32.0_f32
        -32.0   != -32.
        0.0     != 0.00
        0.0     != -0.00
    "#);
}
