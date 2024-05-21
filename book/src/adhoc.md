# If you're only deriving once... 

If you want, you can
[apply a template to an existing type](https://docs.rs/derive-deftly/latest/derive_deftly/macro.derive_deftly_adhoc.html)
without having to name that template.
You might want to do this if you have a template
that you only want to apply to a single struct,
and so you don't want to bother naming it.

Supposing that you wanted to apply the template above to `MyStruct`
and `MyStruct` alone,
you could have said:

```
use derive_deftly::{Deftly, derive_deftly_adhoc};

#[derive(Clone, Debug, Deftly)]
#[derive_deftly_adhoc]
pub struct MyStruct;

derive_deftly_adhoc!{
    MyStruct:

    impl $ttype {
        pub fn print_name() {
            println!("The name of this type is {}", stringify!($ttype));
        }
    }
}
```

Of course, that's not so useful yet.
In this case, it would have been easier just to write
`impl MyStruct { pub fn print_name() { ... } }`.
But soon, we'll see how to write more interesting templates,
and how to use them to create much more interesting code.


The rest of this document will focus
on how to use derive_deftly's template features
to your fullest advantage.
If you're the kind of person who wants to skip
straight to the reference manual,
you can find it [over here](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html).

