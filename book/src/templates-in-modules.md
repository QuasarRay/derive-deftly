# Templates inside modules

When you define a template, derive-deftly turns it into a definition
of a `macro_rules!` macro.
In the example above, that would be `derive_deftly_template_NamePrinter`.

Unfortunately,
Rust's rules for scoping of `macro_rules!` macros are awkward and confusing.
If you want to use a macro defined in another module,
that module's `mod` statement must have `#[macro_use]`,
and it must come before the module where you use the macro.
And, the macro's name is never scoped within the module.

```
#[macro_use] // make NamePrinter visible outside the module (within the crate)
mod name_printer {
    use derive_deftly::define_derive_deftly;
    define_derive_deftly! {
        NamePrinter: /* ... */
    }
}
mod caller { // must come after mod name_printer
    use derive_deftly::Deftly;
    #[derive(Clone, Debug, Deftly)]
    #[derive_deftly(NamePrinter)] // not name_printer::NamePrinter
    pub struct MyStruct;
}
```

