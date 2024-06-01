# A first example: derive_deftly(NamePrinter)

Here we'll make an example project
with a simple `derive-deftly` template.
Our template won't do much,
but it will get us started using the system.

To begin with, we'll create a new project:

> ```shell
> $ cargo init --lib dd-hello-world
> $ cd dd-hello-world
> ```

Next, we'll add the `derive-deftly` crate as a new dependency:

> Note: You could also add the dependency
> by adding a line like this to the `[dependencies]` section
> in your `Cargo.toml`:
>
> ```toml
> derive-deftly = "0.10"
> ```
>
> (You should replace `0.10` with the real latest version.)

Now we can edit `src/lib.rs` in our project
to define and use a new template.

## Writing NamePrinter

There are two parts to using derive-deftly:
specifying _templates_ that you can use to derive new features for your
structs and enums
and then _applying_ those templates to your types.

To define a template, you use
[`define_derive_deftly!`], as in

```rust
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

This is a very simple template: it uses a single expansion: [`$ttype`].
(We'll get into more expansions, and more useful examples, later on.
For now, all you need to know
is that `$ttype` expands to the type
on which you're applying your template.)

Later on, you can apply `NamePrinter` to your own type
with [`#[derive(Deftly)]`[derive-Deftly], as in:

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

[`$ttype`]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:ttype
[`define_derive_deftly!`]: https://docs.rs/derive-deftly/latest/derive_deftly//macro.define_derive_deftly.html
[derive-Deftly]: https://docs.rs/derive-deftly/latest/derive_deftly/derive.Deftly.html



