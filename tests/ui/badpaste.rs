use derive_adhoc::{derive_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[adhoc(something = "Box<char>")]
struct DataType {
    foo: u8,
    bar: Vec<String>,
}

derive_deftly_adhoc! {
    DataType:

    ${for fields {
        ${paste $ttype _ $ftype}
    }}
}

derive_deftly_adhoc! {
    DataType:

    struct ${paste $tname _ 42};
}

derive_deftly_adhoc! {
    DataType:

    struct ${paste $ttype ${tmeta(something) as ty}};
}

derive_deftly_adhoc! {
    DataType:

    // This expands to "r#struct Broken { }"
    ${paste tdefkwed} Broken { }
}

fn main() {}
