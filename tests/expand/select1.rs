//! Test case / example for `${select }`

use std::fmt::Debug;

use derive_adhoc::{derive_adhoc, Deftly};

use derive_adhoc_tests::*;

#[derive(Deftly, Default, Debug)]
#[derive_deftly_adhoc]
struct Both {
    #[adhoc(left)]
    a: usize,
    #[adhoc(right)]
    b: usize,
    #[adhoc(right)]
    c: usize,
}

derive_deftly_adhoc! {
    Both:

    #[derive(Default, Debug)]
    struct Left {
        $(
            ${select1
              fmeta(left) { $fname: $ftype, }
              fmeta(right) { }
            }
        )
    }
    struct Right {
        $(
            ${select1
              fmeta(left) { }
              fmeta(right) { $fname: $ftype, }
            }
        )
    }
}

fn main() {
    assert_eq!(Left::default().to_debug(), "Left { a: 0 }",)
}
