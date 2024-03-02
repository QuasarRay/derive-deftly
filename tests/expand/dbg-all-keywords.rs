use derive_deftly::{define_derive_deftly, Deftly};

define_derive_deftly! { Dbg = $dbg_all_keywords }

#[allow(dead_code)]
#[derive(Deftly)]
#[derive_deftly(Dbg)]
enum Enum {
    Unit,
    Tuple(usize),
    Struct { field: String },
}

#[allow(dead_code)]
#[derive(Deftly)]
#[derive_deftly(Dbg)]
struct Unit;

#[allow(dead_code)]
#[derive(Deftly)]
#[derive_deftly(Dbg)]
struct Tuple(usize);

#[allow(dead_code)]
#[derive(Deftly)]
#[derive_deftly(Dbg)]
struct Struct {
    field: String,
}

fn main() {}
