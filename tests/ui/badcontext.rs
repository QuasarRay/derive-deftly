use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};

define_derive_deftly! {
    /// This is fundamentally misconceived
    FieldNames expect expr:
    [ $( stringify!($fname), ) ]
}

#[derive(Deftly)]
#[derive_deftly(FieldNames)]
#[derive_deftly_adhoc]
struct DataType {}

derive_deftly_adhoc! {
    // This is going to be a syntax error, since it expands to an expr
    DataType expect expr:
    ${for fields { 1 + }} 0
}

derive_deftly_adhoc! {
    DataType expect items:
    ${ignore $fname}
}

define_derive_deftly! {
    FieldOutsideRepeatGroup:
    [ stringify!($fname), ] // not in a repeat group at all
}
define_derive_deftly! {
    FieldInVariantsRepeatGroup:
    [ ${for variants { stringify!($fname), }} ] // insufficient repeat depth
}

#[derive(Deftly)]
#[derive_deftly(FieldOutsideRepeatGroup, FieldInVariantsRepeatGroup)]
struct Struct;

#[derive(Deftly)]
#[derive_deftly(FieldOutsideRepeatGroup, FieldInVariantsRepeatGroup)]
enum Enum {
    Variant,
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
enum NotExactly1<T> {
    Unit,
    Singleton(T),
}

derive_deftly_adhoc! {
    NotExactly1 beta_deftly:

    $(
        ${when v_is_unit}
        ${loop_exactly_1 "just right"}
        $vname
    )
    $(
        ${when v_is_named}
        ${loop_exactly_1 "too few"}
        $vname
    )
    $(
        ${loop_exactly_1 "too many"}
        $vname
    )
    $(
        ${loop_exactly_1 message="too few, spanned" span=$tgens}
        ${when v_is_unit}
        $vname
    )
    $(
        ${loop_exactly_1 message="too many, spanned" span=$tgens}
        $vname
    )
}

fn main() {}
