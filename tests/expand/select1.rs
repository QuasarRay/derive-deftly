//! Test case / example for `${select }`

use std::fmt::Debug;

use derive_deftly::{derive_deftly_adhoc, Deftly};

use derive_adhoc_tests::*;

#[derive(Deftly, Default, Debug)]
#[derive_deftly_adhoc]
struct Both {
    #[deftly(left)]
    a: usize,
    #[deftly(right)]
    b: usize,
    #[deftly(right)]
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
