# derive-deftly: An ergonomic replacement for (some) proc macros

`derive-deftly` allows you to write macros which are driven
by Rust data structures, just like proc macro derive macros,
but without having to wrestle with the proc macro system.

**Stability warning**

We intend to make a 1.x release of this crate fairly soon,
but right now we have some breaking changes planned.

We aren't planning changes to the template language before 1.0.

This is the new name for
[`derive-adhoc`](https://lib.rs/crates/derive-adhoc).
If you are aready using that, please continue to use it for now:
We intend to make several further breaking changes,
besides the renamings.

## Overview

You can write an ad-hoc template,
which can speak about the fields and types in the data structure.
You can also define named templates and apply them to multiple structures:
effectively, you can define your own derive macro.

You **don't** need to make a separate proc macro crate,
write to the `syn` and `proc_macro` APIs.
take care to properly propagate compile errors,
or, generally, do any of the things that
make writing proc macros so complicated.

The template language resembles the "expander" part
of a `macro_rules` macro,
but you don't have to write the "matcher" part:
derive-deftly parses the input data structure for you,
and makes the pieces available via predefined expansion variables.

Further documentation is available in the `doc_` module(s)
and the docs for the individual proc macros.

## Simple example - providing `Vec` containing enum variant names

```
use derive_deftly::{define_derive_deftly, Deftly};

define_derive_deftly! {
    ListVariants =

    impl $ttype {
        fn list_variants() -> Vec<&'static str> {
            vec![ $( stringify!( $vname ) , ) ]
        }
    }
}

#[derive(Deftly)]
#[derive_deftly(ListVariants)]
enum Enum {
    UnitVariant,
    StructVariant { a: u8, b: u16 },
    TupleVariant(u8, u16),
}

assert_eq!(
    Enum::list_variants(),
    ["UnitVariant", "StructVariant", "TupleVariant"],
);
```

## Next steps

Why not have a look at our [friendly introduction](doc_introduction)?

It will walk you through derive-deftly's most important features,
with a number of worked examples,
