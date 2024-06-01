# More complicated conditionals

Frequently, we'd like our template
to behave in different ways different fields.
For example, let's suppose that we want our template
to be able to set fields to their default values,
and not take them as arguments.

We could do this with an explicit conditional for each field:
```
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
   Constructor:

   impl<$tgens> $ttype where $twheres {
     pub fn new
     ( $(
          ${when not(fmeta(Constructor(default))) } // (1)
          $fname: $ftype ,
        ) ) -> Self {
          Self {
              $( $fname:
                  ${if fmeta(Constructor(default)) { Default::default() }
                  else { $fname } }
                 , )
          }
      }
   }
}

use derive_deftly::Deftly;
#[derive(Deftly)]
#[derive_deftly(Constructor)]
struct Foo {
    #[deftly(Constructor(default))]
    s: Vec<String>,
    n: u32,
}
```

Here we're using a new construct:
[`$when`][x:when]
It's only valid inside a loop like `$( ... )`.
It causes the output of the loop to be surpressed
whenever the condition is not true.

The condition in this cases is `not(fmeta(Constructor(default)))`.
You've seen `fmeta` before;
[`not`][c:not]
is just how we express negation.
All together, this `$when` keyword causes each field
that has `#[deftly(Constructor(default))]` applied to it
to be omitted from the list of arguments
to the `new()` function.

You can use other boolean operators in conditions too:
there is an
[`any(...)`][c:any]
that is true
whenever at least one of its arguments is true,
and an
[`all(...)`][c:all]
that is true
when _all_ of its arguments are true.


[c:all]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:all
[c:any]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:any
[c:not]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#c:not
[x:when]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:when

