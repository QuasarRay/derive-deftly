# Making MyClone apply to generics

But here's a structure where our current `MyClone` implementation
will fall flat:

```rust
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

That isn't valid!  For our expansion to compile,
we will need to use the generic parameters
and their "where" constraints, like so:
```rust,ignore
impl<T:Clone, U> Clone for MyItems<T,U>
    where U: Clone+Debug
{ ... }
```

Therefore, to get our `MyClone` template working with generic types,
we can expand it to add the same parameters and constraints.

```rust
# use derive_deftly::{define_derive_deftly,Deftly};
define_derive_deftly! {
    MyClone:

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
# use std::fmt::Debug;
# #[derive(Deftly)]
# #[derive_deftly(MyClone)]
# struct MyItems<T:Clone, U>
#    where U: Clone + Debug
# {
#     things: Vec<T>,
#     items: Vec<U>
# }
```

Here we meet two new expansions.
[`$tgens`][x:tgens]
("top-level generics") becomes
the generic parameters as declared on the top-level type.
(In our case, that's `T:Clone, U`.)
[`$twheres`][x:twheres]
("top-level where clauses") becomes
the `where` constraints as declared on the top-level type.
(In our case, that's `U: Clone+Debug`.)

Note that `$ttype` expands to the top-level _type_:
that's now `MyItems<T,U>`,
which is what we want.
If we had wanted only `MyItems`,
we would say [`$tname`][x:tname] instead.

<!-- TODO: Forward-reference a discussion of $tdefgens. -->


---

Will this template still work for non-parameterized types?
Indeed, it will!
To Rust, this syntax is perfectly fine:
```rust
struct Simple {
    a: String
}
// Note empty <> and "where" list:
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

[x:tgens]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:tgens
[x:twheres]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:twheres
[x:tname]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:tname

