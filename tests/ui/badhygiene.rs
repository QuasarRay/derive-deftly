use derive_deftly::{
    define_derive_deftly, define_derive_deftly_module, Deftly,
};

define_derive_deftly! {
    DefineA:
    let a = ();
}

define_derive_deftly_module! {
    DefineC:
    ${define DEFINE_C {
        let c = ();
    }}
}

define_derive_deftly! {
    use DefineC;
    DefineB:
    let b = ();
    $DEFINE_C
}

fn main() {
    #[derive(Deftly)]
    #[derive_deftly(DefineA, DefineB)]
    struct Data;

    let () = a;
    let () = b;
    let () = c;
}
