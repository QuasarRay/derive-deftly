//! Example which fully and precisely derives Clone
//!
//! This gives a basic demonstration of how to handle an enum.

use derive_adhoc::{define_derive_deftly, Deftly};

define_derive_deftly! {
    PreciseClone =

    impl<$tgens> Clone for $ttype
    // We don't need to $( $( ) ) (ie, twice); it automatically descends.
    where $( $ftype: Clone, )
          $twheres
    {
        fn clone(&self) -> Self {
            match self { $(
                $vpat => $vtype { $(
                    $fname: $fpatname.clone(),
                ) },
            ) }
        }
    }
}

#[derive(Deftly)]
#[derive_deftly(PreciseClone)]
struct Unit;

#[derive(Deftly)]
#[derive_deftly(PreciseClone)]
struct Tuple<F>(F);

#[derive(Deftly)]
#[derive_deftly(PreciseClone)]
struct Struct<F> {
    field: F,
}

#[derive(Deftly)]
#[derive_deftly(PreciseClone)]
enum Enum<F> {
    Unit,
    Tuple(F),
    Struct { field: F },
}

#[derive(Deftly)]
#[derive_deftly(PreciseClone)]
enum AllTypes {
    NoData,
    Tuple(u16, u32),
    Struct { a: String, b: String },
}

fn test<T: Clone>(value: &T) {
    let ours = value.clone();
    drop(ours);
}

fn main() {
    test(&Unit);
    test(&Tuple(String::new()));
    test(&Struct { field: 42 });
    test(&Enum::<()>::Unit);
    test(&Enum::Tuple(String::new()));
    test(&Enum::Struct { field: 66 });
}
