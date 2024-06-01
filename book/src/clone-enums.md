# Deriving for enumerations

At this point, you've probably noticed
that we've defined `MyClone` to apply to `struct`s only,
but it won't (yet) work on `enum`s.
Let's fix that!

Suppose that we have enumeration defined like this:

```
enum AllTypes {
    NoData,
    Tuple(u8, u16),
    Struct { a: String, b: String }
}
```
We want to make sure that
MyClone can recognize and re-construct
each of the three variants.

We can do that as follow
(For simplicity, we're going to ignore generics for now.)
```
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
    MyClone:

    impl Clone for $ttype
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
```

Note that now we have two levels of nested repetition.
First, we match once for each variant.
(This is at the `$vpat` and `$vtype` level.)
Then we match once for each field of each variant.
(This is at the `$fname` and `$fpatname` level.)

Let's go over the new expansions here.
First, we have
[`$vpat`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#vpat-fpatname--pattern-matching-and-value-deconstruction):
that expands to a pattern that can match and deconstruct
a single variant.
Then, we have
[`$vtype`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#ftype-vtype-ttype-tdeftype--types):
that's the type of the variant,
suitable for use as a constructor.
Then, inside the variant, we have `$fname`:
that's our field name, which we've seen it before.
Finally, we have
[`$fpatname`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#vpat-fpatname--pattern-matching-and-value-deconstruction):
that is the name of the variable that we used for this field
in the pattern that deconstructed it.

When we apply `MyClone` to our enumeration,
we get something like this:
```
# enum AllTypes { NoData, Tuple(u8, u16), Struct { a: String, b: String } }
impl Clone for AllTypes {
    fn clone(&self) -> Self {
        match self {
            AllTypes::NoData {} => AllTypes::NoData {},
            AllTypes::Tuple {
                0: f_0,
                1: f_1,
            } => AllTypes::Tuple {
                0: f_0.clone(),
                1: f_1.clone()
            },
            AllTypes::Struct {
                a: f_a,
                b: f_b,
            } => AllTypes::Struct {
                a: f_a.clone(),
                b: f_b.clone()
            },
        }
    }
}
```

Note that our template above will still work fine on a regular struct,
even though it's written for an `enum`.
If we apply `MyClone` above
to `struct Example { a: u8, b: String }`,
we get this:

```
# struct Example { a: u8, b: String }
impl Clone for Example {
    fn clone(&self) -> Self {
        match self {
            Example {
                a: f_a,
                b: f_b,
            } => Example {
                a: f_a.clone(),
                b: f_b.clone(),
            }
        }
    }
}
```

So (in this case at least)
we were able to write a single template expansion
that worked for both `struct`s and enum`s.

#### Putting the generics back into our enumeration-friendly template

Now let's see how it works when we try to handle generics again.
(It's surprisingly straightforward!)

```
# use derive_deftly::{Deftly, define_derive_deftly};
define_derive_deftly! {
    MyClone:

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

##### A further note on reptition

Note that when we define our additional `where` clauses,
we don't have to specify separate of repetition
for variants and fields:
if we just have `$ftype` in a top-level repetition,
`derive_deftly` will iterate over all fields in all variants.

Sometimes, if you do something subtle,
derive-deftly may not be able to figure
out what you're trying to repeat over.
You can use
[`${for ...}`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#repetition-and-nesting)
to specifiy explicitly:
so, above, instead, we could have written
```
# use derive_deftly::{Deftly, derive_deftly_adhoc};
# #[derive(Deftly)]
# #[derive_deftly_adhoc]
# enum TestCase<A> { Variant(A) }
# derive_deftly_adhoc! { TestCase:
#     fn testcase<$tgens>() where
${for fields { $ftype: Clone, }}
#     {}
# }
```

You can also use `${for ...}` rather than `$(...)`
in cases where you feel
it makes your macro code clearer.

