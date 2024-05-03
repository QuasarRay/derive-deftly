### Making MyClone apply to generics

But here's a structure where our current `MyClone` implementation
will fall flat:

```
# use std::fmt::Debug;
struct MyItems<T:Clone, U>
    where U: Clone + Debug
{
    things: Vec<T>,
    items: Vec<U>
}
```

When we go to expand the template, it will generate something like:
```rust,ignore
impl Clone for MyItems { ... }
```

That isn't valid!  We need to use the generic parameters, like so:
```rust,ignore
impl<T:Clone, U> Clone for MyItems<T,U> 
    where U: Clone+Debug
{ ... }
```

We can expand our `MyClone` definition to look that way:

```
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
    MyClone =

    impl<$tgens> Clone for $ttype
    where $twheres
    {
        fn clone(&self) -> Self {
            Self {
                $( $fname : self.$fname.clone() , )
            }
        }
    }
}
```

Here we meet two new expansions.
[`$tgens`](crate::doc_reference#tgens-tgnames-twheres-tdefgens--generics)
("top-level generics") becomes
the generic parameters as declared on the top-level type.
(In our case, that's `$T:Clone, U`.)
[`$twheres`](crate::doc_reference#tgens-tgnames-twheres-tdefgens--generics)
("top-level where clauses") becomes
the `where` constraints as declared on the top-level type.
(In our case, that's `U: Clone+Debug`.)

Note that `$ttype` expands to the top-level _type_:
that's now `MyItems<T,U>`,
which is what we want.
If we had wanted only `MyItems`,
we would say [`$tname`](crate::doc_reference#fname-vname-tname--names) instead.

Will this template still work for non-parameterized types?
Again, yes!
To Rust, this syntax is perfectly fine:
```rust
struct Simple {
    a: String
}
impl<> Clone for Simple
where
{
    fn clone(&self) -> Self {
        Self {
            a: self.a.clone(),
        }
    }
}
```


#### Putting the generics back into our enumeration-friendly template

Now let's see how it works when we try to handle generics again.
(It's surprisingly straightforward!)

```
# use derive_deftly::{Deftly, define_derive_deftly};
define_derive_deftly! {
    MyClone =

    impl<$tgens> Clone for $ttype
    where $( $ftype: Clone, )
          $twheres
    {
        fn clone(&self) -> Self {
            match self {
                $(
                    $vpat => $vtype {
                        $(
                            $fname: $fpatname.clone(),
                        )
                    },
                )
            }
        }
    }
}
# #[derive(Deftly)]
# #[derive_deftly(MyClone)]
# enum TestCase<A> { Variant(A) }
```

