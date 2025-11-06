use derive_deftly::{
    define_derive_deftly, define_derive_deftly_module, Deftly,
};

// Test hygiene respanning.  The spans are not visible in the expanded text
// but without the correct spans, this file *does* cause the tests fo fail
// with "cannot find value `variable` in this scope"

define_derive_deftly_module! {
    LetVariable:
    ${define LET_VARIABLE {
        let variable: i32 = 0;
    }}
    ${define CHECK_CALLER_VAR {
        let _: i32 = caller_var;
    }}
}

define_derive_deftly! {
    use LetVariable;
    UseVariable:

    impl $ttype {
        fn use_variable() -> i32 {
            let caller_var = 42;
            $CHECK_CALLER_VAR
            ${LET_VARIABLE}
            variable
        }
    }
}

#[derive(Deftly)]
#[derive_deftly(UseVariable)]
struct Data;

fn main() {
    std::process::exit(Data::use_variable());
}
