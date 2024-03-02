use derive_adhoc::{define_derive_deftly, derive_adhoc, Adhoc};

#[derive(Adhoc)]
#[derive_truly_adhoc]
struct DataType {
    foo: u8,
    #[adhoc(hash(skip))]
    bar: Vec<String>,
}

derive_adhoc! {
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
    Broken =

    type Alias = ${ttype $junk};
}

#[derive(Adhoc)]
#[derive_adhoc(Broken)]
#[derive_truly_adhoc]
struct ForBroken;

derive_adhoc! {
    DataType:

    ${if approx_equal(1,2,3) {}}
}

fn main() {}
