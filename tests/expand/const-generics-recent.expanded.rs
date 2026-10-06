//! Test/example for const generics, including default values
//!
//! This is a separate file rather than (say) integrated into
//! `partial-ord.rs` or `ref-version.rs` because this feature is new
//! in Rust and our MSRV can't compile it.
//!
//! `recent` in the filename arranges for it not to be run by the MSRV
//! compiler.
#![forbid(unused_parens)]
#![allow(dead_code)]
use derive_deftly::{derive_deftly_adhoc, Deftly};
#[derive_deftly_adhoc]
struct Unit;
#[derive_deftly_adhoc]
struct ConstOnly<const N: u32 = 0>;
#[derive_deftly_adhoc]
enum Enum<T = (), const N: usize = 1> {
    Unit,
    Tuple([T; N]),
}
trait Trait<T> {}
enum EnumCopy<T = (), const N: usize = 1> {
    Unit,
    Tuple([T; N]),
}
impl<T, const N: usize> From<Enum<T, N>> for EnumCopy<T, N> {
    fn from(orig: Enum<T, N>) -> Self {
        match orig {
            Enum::Unit {} => EnumCopy::Unit::<T, N> {},
            Enum::Tuple { 0: f_0 } => EnumCopy::Tuple::<T, N> { 0: f_0 },
        }
    }
}
impl<T: Trait<T>> Trait<()> for Box<Box<T>>
where
    T: Sync,
{}
impl Trait<(for<'x> fn(&'x u32))> for (for<'x> fn(&'x u32))
where
    Box<(for<'x> fn(&'x u32))>: Sync,
{}
struct Defs<
    'l,
    'z: 'l,
    const KM: usize,
    ZM,
    ZD = (),
    const KD: usize = 23,
    T = (),
    const N: usize = 1,
> {
    l: &'l &'z (),
    d: [ZD; KD],
    m: [ZM; KM],
    t: Enum<T, N>,
}
struct Nodefs<'l, 'z: 'l, const KM: usize, ZM, ZD, const KD: usize, T, const N: usize> {
    d: Defs<'l, 'z, KM, ZM, ZD, KD, T, N>,
}
struct NoGens;
fn main() {
    let _: Option<Enum> = None;
    let _ = Enum::Unit::<u8, 3>;
    let _ = Enum::Tuple([42; 2]);
    let assert = |ok: bool| {
        if !ok {
            ::core::panicking::panic("explicit panic");
        }
    };
    assert(!false);
    assert(true);
}
