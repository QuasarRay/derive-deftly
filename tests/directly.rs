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
    macro_rules! chk { { $eq_ne:ident, $a:tt, $b:tt } => { paste! {
        let a = quote! { $a };
        let b = quote! { $b };
        let ord = tokens_cmp(a.clone(), b.clone());
        [< assert_ $eq_ne >]!(
            ord, Ordering::Equal,
            "{} <=> {} was {:?}", a, b, ord
        );
    } } }

    // literals; they are compared by their string representation

    // str
    chk!(ne, "hi", "h\x69");
    chk!(ne, "hi"x, "hi");
    chk!(ne, "hi", r"hi");
    // byte str
    chk!(ne, b"hi", b"h\x69");
    chk!(ne, b"hi"x, b"hi");
    chk!(ne, b"hi", br"hi");
    // char
    chk!(ne, 'i', '\x69');
    chk!(ne, 'i'x, 'i');
    // byte
    chk!(ne, b'i', b'\x69');
    chk!(ne, b'i'x, b'i');
    // int
    chk!(ne, 1, 01);
    chk!(ne, 1, 0o1);
    chk!(ne, 1, 0b1);
    chk!(ne, 1, 0x1);
    chk!(ne, 1, 1_u32);
    chk!(ne, 1u32, 1_u32);
    // float
    chk!(ne, 32, 32.0);
    chk!(ne, 32.0_f32, 32.0f32);
}
