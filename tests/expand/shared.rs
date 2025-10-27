//! Test shared modules
#![allow(unused)]

use derive_deftly::{
    define_derive_deftly, define_derive_deftly_module, Deftly,
};

define_derive_deftly_module! {
    /// M1-docs
    M1:
    ${define M1_DEF { stringify!(D $$ M1) }}
    ${defcond M1_COND tmeta(m1) }
    ${define SHADOW { "S M1" }}
    ${defcond SHADOW_COND_M2 false}
    ${defcond SHADOW_COND_MM12 false}
    ${define M1_DOC_A {
        /// M1-doc-A
    }}
    ${define M1_DOC_B {
        /// M1-doc-B
        $M1_DOC_B2
    }}
    ${define M1_DOC_B2 {
        /// M1-doc-B2
    }}
}

define_derive_deftly_module! {
    /// M2 docs
    M2:
    ${define M2_DEF { stringify!(D $$ M2) }}
    ${defcond M2_COND tmeta(m2) }
    ${define SHADOW { "S M2" }}
    ${defcond SHADOW_COND_M2 true}
    ${defcond SHADOW_COND_MM12 false}
}

define_derive_deftly! {
    use M1;
    use M2;

    /// Using12-docs
    U:
    pub const $<C $tname>: &[&str] = &[
        $M1_DEF, $M2_DEF, $SHADOW,
        ${if M1_COND { "M1_COND", }}
        ${if M2_COND { "M2_COND", }}
        ${if SHADOW_COND_M2 { "SHADOW_COND_M2", }}
    ];
}

define_derive_deftly_module! {
    /// MM12 docs
    MM12:
    use M1;
    use M2;
    ${define MM12_DEF { stringify!(D $$ MM12) }}
    ${defcond MM12_COND tmeta(mm12)}
    ${define SHADOW { "S MM12" }}
}

define_derive_deftly! {
    use MM12;

    /// UsingU-docs
    ///
    $M1_DOC_A
    /// interleave
        ${select1 approx_equal({ "S MM12" }, $SHADOW) { $M1_DOC_B }}
    /// interleave again
    $M1_DOC_B2

    UMM:

    ${defcond SHADOW_COND_MM12 true}

    pub const $<CU $tname>: &[&str] = &[
        $M1_DEF, $M2_DEF, $MM12_DEF, $SHADOW,
        ${if M1_COND { "M1_COND", }}
        ${if M2_COND { "M2_COND", }}
        ${if MM12_COND { "MM12_COND", }}
        ${if SHADOW_COND_M2 { "SHADOW_COND_M2", }}
        ${if SHADOW_COND_MM12 { "SHADOW_COND_MM12", }}
    ];
}

#[rustfmt::skip]
mod t {
    use super::*;

    #[derive(Deftly)] #[derive_deftly(U, UMM)] struct S0;
    #[derive(Deftly)] #[deftly(m1)] #[derive_deftly(U, UMM)] struct S1;
    #[derive(Deftly)] #[deftly(m2)] #[derive_deftly(U, UMM)] struct S2;
    #[derive(Deftly)] #[deftly(m1, m2)] #[derive_deftly(U, UMM)] struct S12;
    #[derive(Deftly)] #[deftly(mm12)] #[derive_deftly(U, UMM)] struct SMM12;
}
fn main() {
    println!("{:?}", t::CS0);
    println!("{:?}", t::CS1);
    println!("{:?}", t::CS2);
    println!("{:?}", t::CS12);
    println!("{:?}", t::CUS0);
    println!("{:?}", t::CUS1);
    println!("{:?}", t::CUS2);
    println!("{:?}", t::CUS12);
}
