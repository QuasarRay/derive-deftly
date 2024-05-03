### Debugging templates

When writing complex templates,
it can sometimes be hard to figure out compiler errors.
derive-deftly has some features which can help:

#### Syntax checking the expansion

You can
[tell derive-deftly what your macro is supposed to produce](crate::doc_reference#expect-items-expect-expr--syntax-check-the-expansion):

```
# use derive_deftly::{Deftly, define_derive_deftly};
define_derive_deftly! {
   Constructor for struct, expect items =

   impl<$tgens> $ttype where $twheres {
      // ...
   }
}
# #[derive(Deftly)]
# #[derive_deftly(Constructor)]
# struct Test;
```

If the expansion fails to syntax check,
you'll get not only an error pointing at
the part of the template or structure which seems wrong,
but also error messages pointing into a *copy of the actual expansion*.
Hopefully you'll be able to see what's wrong, there.

If your macro is supposed to expand to an expression,
you can write `expect expr` instead of `expect items`.

Alternatively,
you can request this syntax check
[when you invoke the macro](Deftly#expansion-options):

```
# use derive_deftly::{Deftly, define_derive_deftly};
# define_derive_deftly! { Constructor = }
#[derive(Deftly)]
#[derive_deftly(Constructor[expect items])]
struct MyStruct { /* ... */ }
```

#### Seeing a copy of the expansion

Sometimes, the expansion syntax checks,
but is wrong in some other way.

You can ask derive-deftly to simply
[print a copy of the expansion](crate::doc_reference#dbg--print-the-expansion-to-stderr-for-debugging).

```
# use derive_deftly::{Deftly, define_derive_deftly};
# define_derive_deftly! { Constructor = }
#[derive(Deftly)]
#[derive_deftly(Constructor[dbg])]
struct MyStruct { /* ... */ }
```

You'll see the compiler print something like this:

```ignore
---------- derive-deftly expansion of Constructor for MyStruct (start) ----------
impl<> MyStruct where {
    pub fn new(...) { ... }
}
---------- derive-deftly expansion of Constructor for MyStruct (end) ----------
```

Like the `dbg!` macro, you don't want to
leave this in your production code.
