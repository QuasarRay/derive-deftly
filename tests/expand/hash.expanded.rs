//! Simple example, including use of an #[deftly(...)] attribute
//!
//! Also demonstrates use of field type trait bounds.
use derive_adhoc::define_derive_deftly;
use derive_adhoc::{derive_deftly_adhoc, Deftly};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
#[derive_deftly(MyHash)]
struct DataType {
    foo: u8,
    #[deftly(hash(skip))]
    bar: Vec<String>,
}
impl Hash for DataType
where
    u8: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.foo.hash(state);
    }
}
#[derive_deftly(MyHash)]
struct Pair<S, T: Debug>
where
    S: Debug,
{
    first: S,
    second: T,
}
impl<S, T: Debug> Hash for Pair<S, T>
where
    S: Debug,
    S: Hash,
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.first.hash(state);
        self.second.hash(state);
    }
}
#[derive_deftly(MyHash)]
struct IntPair(usize, usize);
impl Hash for IntPair
where
    usize: Hash,
    usize: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}
fn main() {
    let v = DataType {
        foo: 23,
        bar: <[_]>::into_vec(#[rustc_box] ::alloc::boxed::Box::new(["hi".into()])),
    };
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut hasher);
    {
        ::std::io::_print(format_args!("{0:x}\n", hasher.finish()));
    };
}
