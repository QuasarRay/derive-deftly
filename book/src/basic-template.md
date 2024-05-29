# Getting started with `derive_deftly`

There are two parts to using `derive_deftly`:
specifying _templates_ that you can use to derive new features for your
structs and enums
and then _applying_ those templates to your types.

To define a template, you use
[`define_derive_deftly!`](https://docs.rs/derive-deftly/latest/derive_deftly//macro.define_derive_deftly.html), as in

```
use derive_deftly::define_derive_deftly;

define_derive_deftly! {
    NamePrinter:

    impl $ttype {
        pub fn print_name() {
            println!("The name of this type is {}", stringify!($ttype));
        }
    }
}
```

This is a very simple template: it uses a single expansion:
[`$ttype`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#ftype-vtype-ttype-tdeftype--types).
(We'll get into more expansions, and more useful examples, later on.
For now, all you need to know
is that `$ttype` expands to the type
on which you're applying your template.)


Later on, you can apply `NamePrinter` to your own type,
with
[`#[derive(Deftly)]`](https://docs.rs/derive-deftly/latest/derive_deftly/derive.Deftly.html),
as in:

```
# use derive_deftly::define_derive_deftly;
# define_derive_deftly! {
#    NamePrinter:
#
#    impl $ttype {
#        pub fn print_name() {
#            println!("The name of this type is {}", stringify!($ttype));
#        }
#    }
# }
use derive_deftly::Deftly;

#[derive(Clone, Debug, Deftly)]
#[derive_deftly(NamePrinter)]
pub struct MyStruct;

MyStruct::print_name();
```

