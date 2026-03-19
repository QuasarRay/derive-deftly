#![allow(unused)]
use derive_deftly::{derive_deftly_adhoc, Deftly};
#[derive_deftly_adhoc]
struct Struct;
/// Doc generated from `Struct`.
const _: &[&str] = &[
    "direct",
    "C c1 c2.",
    "T t1 t2.",
    "dollar $ and paste BeforeStructAfter.",
    "cond  so empty.",
    "quotes \" apostrophes ' and backslashes \\ ok in lit parts.",
];
fn main() {}
