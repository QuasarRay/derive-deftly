//! Define and use macro_rules macros in a template!

use derive_adhoc::{define_derive_deftly, derive_adhoc, Deftly};

define_derive_deftly! {
    DefineAndUseMacroRulesMacro =

    impl $ttype {
        fn forget(self) {
            macro_rules! forget { { $$v:expr } => { std::mem::forget($$v) } }
            forget!(self);
        }
    }
}

#[derive(Deftly)]
#[derive_deftly(DefineAndUseMacroRulesMacro[dbg])]
#[derive_deftly_adhoc]
struct S(usize);

derive_adhoc! {
    S:

    impl $ttype {
        fn display(&self) -> String {
            macro_rules! display { { $$v:expr } => { $$v.to_string() } }
            display!(self.0)
        }
    }
}

fn main() {
    S(1).forget();
    let _: String = S(2).display();
}
