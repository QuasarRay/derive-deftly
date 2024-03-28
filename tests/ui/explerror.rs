use derive_deftly::{derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct Struct {
    field: usize,
}

derive_deftly_adhoc! {
    Struct:
    ${for fields { ${error "basic"}} }
}

fn main() {}
