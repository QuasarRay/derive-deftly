//! Test shared modules
#![allow(unused)]
use derive_deftly::{define_derive_deftly, define_derive_deftly_module, Deftly};
#[rustfmt::skip]
mod t {
    use super::*;
    #[derive_deftly(U, UMM)]
    struct S0;
    pub const CS0: &[&str] = &["D $M1", "D $M2", "S M2", "SHADOW_COND_M2"];
    pub const CUS0: &[&str] = &[
        "D $M1",
        "D $M2",
        "D $MM12",
        "S MM12",
        "SHADOW_COND_M2",
        "SHADOW_COND_MM12",
    ];
    #[deftly(m1)]
    #[derive_deftly(U, UMM)]
    struct S1;
    pub const CS1: &[&str] = &["D $M1", "D $M2", "S M2", "M1_COND", "SHADOW_COND_M2"];
    pub const CUS1: &[&str] = &[
        "D $M1",
        "D $M2",
        "D $MM12",
        "S MM12",
        "M1_COND",
        "SHADOW_COND_M2",
        "SHADOW_COND_MM12",
    ];
    #[deftly(m2)]
    #[derive_deftly(U, UMM)]
    struct S2;
    pub const CS2: &[&str] = &["D $M1", "D $M2", "S M2", "M2_COND", "SHADOW_COND_M2"];
    pub const CUS2: &[&str] = &[
        "D $M1",
        "D $M2",
        "D $MM12",
        "S MM12",
        "M2_COND",
        "SHADOW_COND_M2",
        "SHADOW_COND_MM12",
    ];
    #[deftly(m1, m2)]
    #[derive_deftly(U, UMM)]
    struct S12;
    pub const CS12: &[&str] = &[
        "D $M1",
        "D $M2",
        "S M2",
        "M1_COND",
        "M2_COND",
        "SHADOW_COND_M2",
    ];
    pub const CUS12: &[&str] = &[
        "D $M1",
        "D $M2",
        "D $MM12",
        "S MM12",
        "M1_COND",
        "M2_COND",
        "SHADOW_COND_M2",
        "SHADOW_COND_MM12",
    ];
    #[deftly(mm12)]
    #[derive_deftly(U, UMM)]
    struct SMM12;
    pub const CSMM12: &[&str] = &["D $M1", "D $M2", "S M2", "SHADOW_COND_M2"];
    pub const CUSMM12: &[&str] = &[
        "D $M1",
        "D $M2",
        "D $MM12",
        "S MM12",
        "MM12_COND",
        "SHADOW_COND_M2",
        "SHADOW_COND_MM12",
    ];
}
fn main() {
    {
        ::std::io::_print(format_args!("{0:?}\n", t::CS0));
    };
    {
        ::std::io::_print(format_args!("{0:?}\n", t::CS1));
    };
    {
        ::std::io::_print(format_args!("{0:?}\n", t::CS2));
    };
    {
        ::std::io::_print(format_args!("{0:?}\n", t::CS12));
    };
    {
        ::std::io::_print(format_args!("{0:?}\n", t::CUS0));
    };
    {
        ::std::io::_print(format_args!("{0:?}\n", t::CUS1));
    };
    {
        ::std::io::_print(format_args!("{0:?}\n", t::CUS2));
    };
    {
        ::std::io::_print(format_args!("{0:?}\n", t::CUS12));
    };
}
