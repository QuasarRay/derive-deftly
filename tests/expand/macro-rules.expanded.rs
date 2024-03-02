//! Define and use macro_rules macros in a template!
use derive_adhoc::{define_derive_deftly, derive_adhoc, Deftly};
#[derive_adhoc(DefineAndUseMacroRulesMacro[dbg])]
#[derive_truly_adhoc]
struct S(usize);
impl S {
    fn forget(self) {
        std::mem::forget(self);
    }
}
impl S {
    fn display(&self) -> String {
        self.0.to_string()
    }
}
fn main() {
    S(1).forget();
    let _: String = S(2).display();
}
