use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};

define_derive_deftly! {
    /// This is fundamentally misconceived
    FieldNames expect expr =
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

fn main() {}
