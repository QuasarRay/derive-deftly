use derive_deftly::{define_derive_deftly, define_derive_deftly_module, Deftly};
#[derive_deftly(UseVariable)]
struct Data;
impl Data {
    fn use_variable() -> i32 {
        let variable: i32 = 0;
        variable
    }
}
fn main() {
    std::process::exit(Data::use_variable());
}
