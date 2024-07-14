# derive-deftly: An ergonomic replacement for (some) proc macros

`derive-deftly` allows you to write macros which are driven
by Rust data structures, just like proc macro derive macros,
but without having to wrestle with the proc macro system.

**Stability warning**

We intend to make a 1.x release of this crate fairly soon.
We think we have made all the necessary breaking changes,
but we may be wrong.

This is the replacement for
[`derive-adhoc`](https://lib.rs/crates/derive-adhoc).
We recommend that existing `derive-adhoc` users upgrade to `derive-deftly`,
at some time that is convenient.

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
    ListVariants:

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

Why not have a look at our friendly [user guide]?

It will walk you through derive-deftly's most important features,
with a number of worked examples.

[user guide]: https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/

## Reporting issues in derive-deftly

We can't wait to hear from you!

In order to submit a bug to
[the derive-deftly issue tracker][tracker]
you will need an account on <https://gitlab.torproject.org>.

You can request an account via
[anonticket](https://anonticket.torproject.org/),
or via the [account request form].

[tracker]: https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues
[account request form]]: https://anonticket.torproject.org/user/gitlab-account/create/
