use derive_deftly::{
    define_derive_deftly, define_derive_deftly_module, Deftly,
};

// Test hygiene respanning.  The spans are not visible in the expanded text
// but without the correct spans, this file *does* cause the tests fo fail
// with "cannot find value `variable` in this scope"

define_derive_deftly_module! {
    LetVariable:

    ${define LET_PASTED {
        $(
          let $<pasted_ $fname> = ();
        )
    }}

    ${define F_PASTED $<pasted_ $fname>}

    ${define USE_PASTED {
        let () = $<pasted_ $fname>;
    }}

    ${define LET_VARIABLE {
        let variable: i32 = 0;
    }}
    ${define CHECK_CALLER_VAR {
        let _: i32 = caller_var;
    }}
}

#[macro_use]
// We don't know that this being a `mod` is important, but it seems a good idea
mod y {
    use super::*;
    define_derive_deftly_module! { FromY: }
}

define_derive_deftly! {
    use FromY; // induces additional plumbing, causing more excitement
    use LetVariable;
    UseVariable:

    impl $ttype {
        pub fn with_pasted() {
            $LET_PASTED
            ${for fields {
                let () = $F_PASTED;
                $USE_PASTED
            }}
        }

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
struct Data {
    #[allow(unused)]
    field: (),
}

fn main() {
    Data::with_pasted();
    std::process::exit(Data::use_variable());
}
