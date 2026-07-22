#![allow(unused)]

use derive_deftly::{derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct Struct;

derive_deftly_adhoc! {
    Struct beta_deftly:

    ${define C ${concat "c1 c2"}}
    ${define T $"t1 t2"}

    $/// Doc generated from `$ttype`.
    const _: &[&str] = &[
        $"direct",
        $"C $C.",
        $"T $T.",
        $"dollar $$ and paste $<Before $tname After>.",
        $"cond ${defcond T approx_equal({<}, $<>)} so empty.",
        $r#"quotes " apostrophes ' and backslashes \ ok in lit parts."#,
    ];

    ${define JUST_STRING "just string"}
    const _: &str = $JUST_STRING;
}

fn main() {}
