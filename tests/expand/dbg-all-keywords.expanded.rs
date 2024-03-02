use derive_adhoc::{define_derive_deftly, Deftly};
#[allow(dead_code)]
#[derive_deftly(Dbg)]
enum Enum {
    Unit,
    Tuple(usize),
    Struct { field: String },
}
#[allow(dead_code)]
#[derive_deftly(Dbg)]
struct Unit;
#[allow(dead_code)]
#[derive_deftly(Dbg)]
struct Tuple(usize);
#[allow(dead_code)]
#[derive_deftly(Dbg)]
struct Struct {
    field: String,
}
fn main() {}
