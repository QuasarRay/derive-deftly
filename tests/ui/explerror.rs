use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};

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

define_derive_deftly! {
    Throws beta_deftly:
    ${error message="just from Throws"}
    $( ${error message="from Throw at ftype" span=$ftype} )
}

#[derive(Deftly)]
#[derive_deftly(Throws)]
struct Struct2 {
    field2: u32,
}

fn main() {}
