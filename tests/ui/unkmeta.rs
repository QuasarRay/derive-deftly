use derive_deftly::{define_derive_deftly, Deftly};

define_derive_deftly! {
    UseSomeAttrs =

    ${define BY_VALUE {
        ${tmeta(tnest(by_value))}
        ${vmeta(vnest(by_value))}
        ${fmeta(fnest(by_value))}
        ${fmeta(vfnest(by_value))}
        ${vmeta(vfnest(by_value))}
    }}

    ${defcond BOOL_ONLY all(
        tmeta(tnest(bool_only)),
        vmeta(vnest(bool_only)),
        fmeta(fnest(bool_only)),
    )}
}

#[derive(Deftly)]
#[derive_deftly(UseSomeAttrs)]
#[deftly(unknown)] // ERROR (simply not recognised)
#[deftly(tnest)] // ERROR (is only a container)
#[deftly(tnest = '1')] // (error, but suppressed)
#[deftly(tnest())] // ok
#[deftly(tnest(bool_only = 42))] // ERROR (only recognised as boolean)
#[deftly(tnest(bool_only(deeper)))] // ERROR (too deep, unrecog.. subpath)
#[deftly(tnest(by_value(deeper)))] // ERROR (too deep, wanted value instead)
#[deftly(tnest(bool_only))] // ok
#[deftly(vnest(bool_only))] // ok
#[deftly(fnest(bool_only))] // ERROR (wrong scope)
#[deftly(tnest(by_value = "t"))] // ok
#[deftly(vnest(by_value = "t"))] // ok
#[deftly(fnest(by_value = "t"))] // ERROR (wrong scope)
#[deftly(vfnest(by_value = "t"))] // ok (struct counts as V)
struct Struct {
    #[deftly(tnest(bool_only))] // ERROR (wrong scope)
    #[deftly(vnest(bool_only))] // ERROR (wrong scope)
    #[deftly(fnest(bool_only))] // ok
    #[deftly(fnest = '1')] // ERROR (is only a container)
    #[deftly(tnest(by_value = "f"))] // ERROR (wrong scope)
    #[deftly(vnest(by_value = "f"))] // ERROR (wrong scope)
    #[deftly(fnest(by_value = "f"))] // ok
    field: usize,
}

#[derive(Deftly)]
#[derive_deftly(UseSomeAttrs)]
#[deftly(tnest(bool_only))] // ok
#[deftly(vnest(bool_only))] // ERROR (wrong scope)
#[deftly(fnest(bool_only))] // ERROR (wrong scope)
#[deftly(tnest(by_value = "t"))] // ok
#[deftly(vnest(by_value = "t"))] // ERROR (wrong scope)
#[deftly(fnest(by_value = "t"))] // ERROR (wrong scope)
#[deftly(vfnest(by_value = "t"))] // ERROR (should be per-field or -variant)
enum Enum {
    #[deftly(tnest(bool_only))] // ERROR (wrong scope)
    #[deftly(vnest(bool_only))] // ok
    #[deftly(fnest(bool_only))] // ERROR (wrong scope)
    #[deftly(tnest(by_value = "v"))] // ERROR (wrong scope)
    #[deftly(vnest(by_value = "v"))] // ok
    #[deftly(fnest(by_value = "v"))] // ERROR (wrong scope)
    Variant {
        #[deftly(tnest(bool_only))] // ERROR (wrong scope)
        #[deftly(vnest(bool_only))] // ERROR (wrong scope)
        #[deftly(fnest(bool_only))] // ok
        #[deftly(tnest(by_value = "f"))] // ERROR (wrong scope)
        #[deftly(vnest(by_value = "f"))] // ERROR (wrong scope)
        #[deftly(fnest(by_value = "f"))] // ok
        field: usize,
    },
}
