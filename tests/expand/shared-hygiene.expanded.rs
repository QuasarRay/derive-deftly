use derive_deftly::{define_derive_deftly, define_derive_deftly_module, Deftly};
#[macro_use]
mod y {
    use super::*;
}
#[derive_deftly(UseVariable)]
struct Data;
impl Data {
    fn use_variable() -> i32 {
        let caller_var = 42;
        let _: i32 = caller_var;
        let variable: i32 = 0;
        variable
    }
}
fn main() {
    std::process::exit(Data::use_variable());
}
