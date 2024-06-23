use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct DataType {
    foo: u8,
    #[deftly(hash(skip))]
    bar: Vec<String>,
}

derive_deftly_adhoc! {
    DataType:

    fn $unknown() { }

    $(
    )

    ${for fields {
        f ${fname junk}() { }
    }}

    ${if false { } else { } forbidden}

    ${if tmeta(some_path) as lit { }}
}

define_derive_deftly! {
    Broken:

    type Alias = ${ttype $junk};
}

#[derive(Deftly)]
#[derive_deftly(Broken)]
#[derive_deftly_adhoc]
struct ForBroken;

derive_deftly_adhoc! {
    DataType:

    ${if approx_equal(1,2,3) {}}
}

fn main() {}
