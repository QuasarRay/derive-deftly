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

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct Unit;

#[derive(Deftly)]
#[derive_deftly_adhoc]
struct ConstOnly<const N: u32 = 0>;

#[derive(Deftly)]
#[derive_deftly_adhoc]
enum Enum<T = (), const N: usize = 1> {
    Unit,
    Tuple([T; N]),
}

trait Trait<T> {}

derive_deftly_adhoc! {
    Enum beta_deftly, dbg:

    $tvis $tdefkwd ${paste $tdeftype Copy}
    ${tdefvariants $(
        ${vdefbody $vname $(
            $fvis ${fdefine $fname} $ftype,
        ) }
    ) }

    impl<$tgens> From<$ttype> for ${paste $ttype Copy} {
        fn from(orig: $ttype) -> Self {
            match orig { $(
                ${vpat} => ${vtype self=${paste $ttype Copy}} { $(
                    $fname: $fpatname,
                ) },
            ) }
        }
    }

    ${impl<T: Trait<T>> Trait<()> for Box<Box<T>> where T: Sync} {}

    ${impl<>
      Trait::<(for <'x> fn(&'x u32))>
      for (for <'x> fn(&'x u32))
      where Box::<(for <'x> fn(&'x u32))>: Sync
    } {}

    ${define GENS_MIXTURE {
        , ZD=(), const KD: usize = 23,, 'l,
        $tdefgens
        'z: 'l, , const KM: usize,, ZM,
    }}

    struct Defs<${gens_collate $GENS_MIXTURE}> {
        l: &'l &'z (),
        d: [ZD; KD],
        m: [ZM; KM],
        t: $ttype,
    }

    struct Nodefs<${gens_nodefs , $GENS_MIXTURE ,}> {
        d: Defs<${gens_names , $GENS_MIXTURE}>,
    }
}

fn main() {
    let _: Option<Enum> = None;
    let _ = Enum::Unit::<u8, 3>;
    let _ = Enum::Tuple([42; 2]);

    let assert = |ok: bool| {
        if !ok {
            panic!();
        }
    };

    assert(!derive_deftly_adhoc! {
        Unit:
        ${if tgens { true } else { false }}
    });
    assert(derive_deftly_adhoc! {
        ConstOnly:
        ${if tgens { true } else { false }}
    });
}
