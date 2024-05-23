# Working with visibility 

Our `Constructor` template above doesn't really make sense
if it's applied to a non-public type:
Rust may even complain that we're declaring
a public function that ruturns a private type!

Let's fix this, and have it give our constructor
the same visibility as the type itself:

```
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
   Constructor for struct:

   impl<$tgens> $ttype where $twheres {
      $tvis fn new( $( $fname: $ftype , ) ) -> Self {
          Self {
              $( $fname , )
          }
      }
   }
}
```

Here instead of saying `pub fn new`,
we said `$tvis fn new`.
The
[`$tvis`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#fvis-tvis-fdefvis--visibility)
keyword will expand
to the visibility of the top-level type.

There is a similar similar
[`$fvis`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#fvis-tvis-fdefvis--visibility)
that expands to the visibility of the current field.

(Since enums variants are always visible, there is no `$vvis`.)

