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

derive_deftly_adhoc! {
    Struct beta_deftly:
    ${error message="just message"}
    $( ${error message=${concat "spanned at ftype for " $fname} span=$ftype} )
}

fn main() {}
