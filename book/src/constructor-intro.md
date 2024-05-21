# Another example: Defining a constructor function. 

In this section,
we'll be using another example to demonstrate
more of what `derive_deftly` can do.

We'll be building a `Constructor` template
to define a `new()` function for a struct,
without having to write out all of its arguments.

Let's start with the following (struct-only) template:

```
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
   Constructor =

   impl<$tgens> $ttype where $twheres {
      pub fn new( $( $fname: $ftype , ) ) -> Self {
          Self {
              $( $fname , )
          }
      }
   }
}
```

When you apply the above template to a type like this:

```
# use derive_deftly::define_derive_deftly;
# define_derive_deftly! {
#   Constructor =
#
#   impl<$tgens> $ttype where $twheres {
#       pub fn new( $( $fname: $ftype , ) ) -> Self {
#           Self {
#               $( $fname , )
#           }
#       }
#   }
# }
use derive_deftly::Deftly;
#[derive(Deftly)]
#[derive_deftly(Constructor)]
struct Ex<A> {
  a: f64,
  b: A,
  c: String
}
```

You'll get a constructor like this:
```
# struct Ex<A> { a: f64, b: A, c: String }
impl<A> Ex<A> {
    pub fn new( a: f64, b: A, c: String ) -> Self {
        Self { a, b, c }
    }
}
```

So far, there aren't any new techniques at work here.
We'll add some more down below.


