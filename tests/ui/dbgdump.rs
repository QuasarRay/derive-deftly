use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};

define_derive_deftly! {
    BasicDbgs:

    ${dbg { fn $<$tdefkwd _fn>() {} }}
    ${dbg "noted" { fn $<$tdefkwd _noted>() {} }}

    ${if dbg(is_struct) {}}
    ${if dbg("noted", is_enum) {}}

    // errors
    ${dbg { $fname }}
    ${dbg "error" { $fname }}
    ${if dbg(is_empty($fname)) {}}
    ${if dbg("error", is_empty($fname)) {}}
}

define_derive_deftly! {
    Contexts:

    ${ignore
        $( ${dbg { $vname }} )
        $( ${dbg "noted vtype" { $vtype }} )
        $( ${dbg "noted fname" { $fname }} )
    }
}

define_derive_deftly! {
    Pastes:

    fn $<${dbg "paste" { "pasting_" $ttype "_pasted" }}>() {}

    $<${dbg "paste error" { "x" $fname "y" }}>
}

#[derive(Deftly)]
#[derive_deftly(BasicDbgs)]
#[derive_deftly_adhoc]
struct DataType;

#[derive(Deftly)]
#[derive_deftly(Contexts)]
struct Tuple(i32);
#[derive(Deftly)]
#[derive_deftly(Contexts)]
struct Struct {
    f: i32,
}
#[derive(Deftly)]
#[derive_deftly(Contexts)]
enum Enum {
    Tuple(i32),
    Struct { f: i32 },
}

#[derive(Deftly)]
#[derive_deftly(Pastes)]
struct WithGenerics<T>(T);

derive_deftly_adhoc! {
    DataType:

    ${dbg { fn $<$tdefkwd _adhoc>() {} }}
    ${dbg "adhoc" { fn $<tdefkwed _adhoc2>() {} }}
    ${if dbg("adhoc", v_is_unit) {} }
}

fn main() {}
