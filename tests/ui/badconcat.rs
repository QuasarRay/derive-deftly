use derive_deftly::{derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(something = "Box<char>")]
struct DataType {
    foo: u8,
    bar: Vec<String>,
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    ${concat forbidden}
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    ${concat ${tmeta(something) as token_stream}}
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    ${define NOT_CONCAT $tgens}
    ${concat $NOT_CONCAT}
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    ${paste ${concat "ok_so_far"}}
}

fn main() {}
