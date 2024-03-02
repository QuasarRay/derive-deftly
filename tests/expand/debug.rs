//! More complex example with multiple, complex, attributes etc.
//!
//! Includes an example of an attribute containing Rust syntax.
//!
//! Read "clone.rs" and "hash.rs" first for simpler examples.

use std::fmt::{self, Debug, Formatter};

use derive_deftly::define_derive_deftly;
use derive_deftly::{derive_deftly_adhoc, Deftly};

use derive_deftly_tests::*;

define_derive_deftly! {
    MyDebug =

    impl Debug for $ttype
    where $(
        ${if fmeta(debug(skip)) {
        } else if fmeta(debug(into)) {
            for <'x> &'x $ftype : Into< ${fmeta(debug(into)) as ty} >,
                ${fmeta(debug(into)) as ty} : Debug,
        } else {
            $ftype: Debug,
        }}
      )
    {
        fn fmt(&self, f: &mut Formatter<'_>)
                 -> Result<(), fmt::Error>
        {
            #[allow(unused_mut)]
            let mut ds = f.debug_struct(stringify!($tname));
                $(
                    ${if fmeta(debug(skip)) {
                    } else if fmeta(debug(call)) {
       ${fmeta(debug(call))} (&mut ds, stringify!($fname), &self.$fname)?;
                    } else if fmeta(debug(into)) {
              ds.field(stringify!($fname),
       &< ${fmeta(debug(into)) as ty} as From<&$ftype> >::from(&self.$fname)
              );
                    } else {
              ds.field(stringify!($fname), &self.$fname);
                    }}
                )
                ds.finish()
        }
    }
}

mod custom {
    use super::*;

    pub fn fmt(
        ds: &mut fmt::DebugStruct,
        name: &'static str,
        value: &char,
    ) -> fmt::Result {
        ds.field(name, &(*value as u32));
        Ok(())
    }
}

#[derive(Debug)]
struct PrettyVec<T>(Vec<T>);

impl<T: Clone> From<&Vec<T>> for PrettyVec<T> {
    fn from(v: &Vec<T>) -> Self {
        PrettyVec(v.clone())
    }
}

struct Opaque;

#[derive(Deftly)]
#[derive_deftly(MyDebug)]
#[allow(dead_code)]
struct DataType {
    foo: u8,
    #[deftly(debug(into = "PrettyVec<String>"))]
    bar: Vec<String>,
    #[deftly(debug(skip))]
    opaque: Opaque,
    #[deftly(debug(call = "custom::fmt"))]
    custom: char,
}

fn main() {
    let dt = DataType {
        foo: 42,
        bar: ["a", "bar"].iter().map(|s| s.to_string()).collect(),
        opaque: Opaque,
        custom: 'y',
    };

    assert_eq!(
        DebugExt::to_debug(&dt),
        "DataType { foo: 42, bar: PrettyVec([\"a\", \"bar\"]), custom: 121 }",
    );
}
