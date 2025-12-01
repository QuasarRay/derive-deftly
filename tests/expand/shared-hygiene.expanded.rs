use derive_deftly::{define_derive_deftly, define_derive_deftly_module, Deftly};
#[macro_use]
mod y {
    use super::*;
}
#[derive_deftly(UseVariable)]
struct Data {
    #[allow(unused)]
    field: (),
}
impl Data {
    pub fn with_pasted() {
        let pasted_field = ();
        let () = pasted_field;
        let () = pasted_field;
    }
    fn use_variable() -> i32 {
        let caller_var = 42;
        let _: i32 = caller_var;
        let variable: i32 = 0;
        variable
    }
}
fn main() {
    Data::with_pasted();
    std::process::exit(Data::use_variable());
}
