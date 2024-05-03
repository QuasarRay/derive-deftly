use derive_deftly::{define_derive_deftly, Deftly};

define_derive_deftly! {
    ShortCircuit =

    $(
        ${if all(fmeta(value), fmeta(short_circuit))  {
            ${ignore ${fmeta(value)}}
        }}
    )
}

define_derive_deftly! {
    Require =

    $(
        ${if fmeta(require) {
            ${ignore ${fmeta(value)}}
        }}
    )
}

#[derive(Deftly)]
#[derive_deftly(ShortCircuit, Require)]
struct Data {
    none: (),

    #[deftly(value = "ignored")] // ERROR, tested only as bool by ShortCircuit
    not_enabled: (),

    #[deftly(require, value = "used")] // ok
    required_used: (),

    #[deftly(short_circuit, value = "used")] // ok
    short_circuit_used: (),

    #[deftly(short_circuit)] // ERROR, not reached in the all()
    short_circuit_uselessly_enabled: (),

    #[deftly(require, short_circuit, value = "used")] // ok
    used_both_ways: (),
}

fn main() {}

compile_error!("XXXX this currently produces no other errors");
