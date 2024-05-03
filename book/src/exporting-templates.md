### Exporting templates

But now suppose that you want to expose your template,
so that people can use it from any crate.

To do this, you use
[`pub` before the name of your macro](define_derive_deftly#exporting-a-template-for-use-by-other-crates).
```
pub trait NamePrinter {
    fn print_name();
}

derive_deftly::define_derive_deftly! {
    /// Derives `NamePrinter`, providing `print_name`
    pub NamePrinter =
    impl $crate::NamePrinter for $ttype {
        fn print_name() {
            println!("The name of this type is {}", stringify!($ttype));
        }
    }
}
```

Note that this time,
we've defined `NamePrinter` as a trait,
and we've changed our template to refer to that trait as
`$crate::NamePrinter`.
The
[`$crate`](crate::doc_reference#crate--root-of-template-crate)
syntax will expand to the name of the crate
in which our template was defined,
so that when later we expand this template,
it can find the right template.

We've also added a
[doc comment](define_derive_deftly#docs-in-define),
which will appear in the public API documentation for our crate.

Additionally,
[we need to re-export `derive_deftly`](define_derive_deftly#you-must-re-export-derive_deftly-semver-implications)
from our crate, so that users get the correct version.
And you should invoke
[`derive_deftly::pub_template_semver_check`](pub_template_semver_check)
once,
somewhere in your crate:
this will tell you specially about any implications for your crate's semver, 
when you change your `Cargo.toml` to upgrade `derive-deftly`.


```rust,ignore
// you might want to apply #[doc(hidden)] to this.
pub use derive_deftly;
derive_deftly::pub_template_semver_check!("0.9.0");
```

Now, when somebody wants to use our template,
they can do it like this:

```rust,ignore
// Let's pretend our crate is called name_printer.
use name_printer::{
    // This is the trait we defined...
    NamePrinter,
    // This is the macro that makes our template work.
    // (We might come up with a better syntax for this later).
    derive_deftly_template_NamePrinter.
};
use derive_deftly::Deftly;

#[derive(Deftly)]
#[derive_deftly(NamePrinter)]
struct TheirStructure {
    // ...
}
```

Exporting a template to other crates with `pub`
doesn't affect its visibility *within your crate*.
You may still need `#[macro_use]`.

