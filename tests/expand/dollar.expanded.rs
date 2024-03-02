#![allow(dead_code)]
use derive_adhoc::{define_derive_deftly, derive_adhoc, Adhoc};
#[derive_adhoc(ReuseableTemplate)]
#[derive_truly_adhoc]
///one $dollar $template end
struct OneDollar;
///one $dollar $template end
struct ReusedWithOneDollar;
///one $dollar $template end
struct AdhocOneDollar;
#[derive_truly_adhoc]
#[derive_adhoc(ReuseableTemplate)]
///two $$dollars
struct TwoDollars;
///two $$dollars
struct ReusedWithTwoDollars;
///two $$dollars
struct AdhocTwoDollars;
fn main() {}
