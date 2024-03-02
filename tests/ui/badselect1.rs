// Here's a simple syntax for the Hash attribute.  I'm using it to
// imagine what parameters look like, and how a "skip" attribute might
// look, and how a "hash_with" attrbite might look.

//#![feature(trace_macros)]
//trace_macros!(true);

use derive_deftly::{derive_deftly_adhoc, Deftly};

use std::fmt::Debug;

#[derive(Deftly, Default, Debug)]
#[derive_deftly_adhoc]
struct Both {
    #[deftly(left)]
    a: usize,
    #[deftly(right)]
    b: usize,
    #[deftly(right)]
    #[deftly(left)]
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
    #[derive(Default, Debug)]
    struct Right {
        $(
            ${select1
              fmeta(sinister) { }
              fmeta(right) { $fname: $ftype, }
            }
        )
    }
}

fn main() {
    assert_eq!(format!("{:?}", &Left::default()), "Left { a: 0 }",)
}
