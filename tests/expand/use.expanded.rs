//! Test `use` of template to allow use before definition
#![allow(dead_code)]
use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};
#[derive_deftly(Macro)]
#[derive_deftly_adhoc]
struct Struct {}
use derive_deftly_driver_Struct;
use derive_deftly_template_Macro;
fn main() {}
