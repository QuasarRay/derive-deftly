#![allow(dead_code)]
use derive_adhoc::{define_derive_deftly, derive_adhoc, Deftly};
#[derive_deftly(ReuseableTemplate)]
#[derive_deftly_adhoc]
///one $dollar $template end
struct OneDollar;
///one $dollar $template end
struct ReusedWithOneDollar;
///one $dollar $template end
struct AdhocOneDollar;
#[derive_deftly_adhoc]
#[derive_deftly(ReuseableTemplate)]
///two $$dollars
struct TwoDollars;
///two $$dollars
struct ReusedWithTwoDollars;
///two $$dollars
struct AdhocTwoDollars;
fn main() {}
