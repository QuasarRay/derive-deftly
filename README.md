# derive-adhoc: An ergonomic replacement for (some) proc macros

`derive-adhoc` allows you to write macros which are driven
by Rust data structures, just like proc macro derive macros,
but without having to wrestle with the proc macro system.

**Maintenance - status of this crate**

This library has now been renamed to
[`derive-deftly`](https://lib.rs/crates/derive-deftly),
which has some significant improvements.

We recommend that users upgrade to that crate,
when it is convenient to do so.
This crate, `derive-adhoc`, will get important bugfixes,
but not new features etc.

Everything is renamed and there are other breaking changes;
consult
[derive-deftly's changelog](https://docs.rs/derive-deftly/latest/derive_deftly/doc_changelog/index.html#changelog)
for information about how to upgrade.

As of derive-deftly 0.10.0,
the template syntax does *not* have breaking changes.

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
derive-adhoc parses the input data structure for you,
and makes the pieces available via predefined expansion variables.

Further documentation is available in the `doc_` module(s)
and the docs for the individual proc macros.

## Simple example - providing `Vec` containing enum variant names

```
use derive_adhoc::{define_derive_adhoc, Adhoc};

define_derive_adhoc! {
    ListVariants =

    impl $ttype {
        fn list_variants() -> Vec<&'static str> {
            vec![ $( stringify!( $vname ) , ) ]
        }
    }
}

#[derive(Adhoc)]
#[derive_adhoc(ListVariants)]
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

It will walk you through derive-adhoc's most important features,
with a number of worked examples,
