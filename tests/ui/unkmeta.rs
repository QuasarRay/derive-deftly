use derive_deftly::{define_derive_deftly, Deftly};

define_derive_deftly! {
    UseSomeAttrs =

    ${for fields { ${ignore {
        ${if tmeta(tnest(by_value)) { ${tmeta(tnest(by_value)) as str} }}
        ${if vmeta(vnest(by_value)) { ${vmeta(vnest(by_value)) as str} }}
        ${if fmeta(fnest(by_value)) { ${fmeta(fnest(by_value)) as str} }}
        ${if fmeta(vfnest(by_value)) { ${fmeta(vfnest(by_value)) as str} }}
        ${if vmeta(vfnest(by_value)) { ${vmeta(vfnest(by_value)) as str} }}



        ${if tmeta(tnest(bool_only)) {}},
        ${if vmeta(vnest(bool_only)) {}},
        ${if fmeta(fnest(bool_only)) {}},
    }}}}
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
struct StructTop {
    field: usize, //
}

#[derive(Deftly)]
#[derive_deftly(UseSomeAttrs)]
struct StructField {
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
enum EnumTop {
    #[deftly(fnest(bool_only))] // ERROR (wrong scope, duplicate)
    Variant {
        field: usize, //
    },
}

#[derive(Deftly)]
#[derive_deftly(UseSomeAttrs)]
enum EnumVariant {
    #[deftly(tnest(bool_only))] // ERROR (wrong scope)
    #[deftly(vnest(bool_only))] // ok
    #[deftly(fnest(bool_only))] // ERROR (wrong scope)
    #[deftly(tnest(by_value = "v"))] // ERROR (wrong scope)
    #[deftly(vnest(by_value = "v"))] // ok
    #[deftly(fnest(by_value = "v"))] // ERROR (wrong scope)
    Variant {
        field: usize, //
    },
}

#[derive(Deftly)]
#[derive_deftly(UseSomeAttrs)]
enum EnumField {
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
