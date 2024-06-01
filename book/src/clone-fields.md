# Simple templates: fields and repetition

Let's imagine we had to write Clone from scratch for a simple structure
like this:

```
struct GiftBasket {
   n_apples: u8,
   n_oranges: u8,
   given_from: Option<String>,
   given_to: String,
}
```

We'd probably write something like this:
```
# struct GiftBasket {
#   n_apples: u8,
#   n_oranges: u8,
#   given_from: Option<String>,
#   given_to: String,
# }

impl Clone for GiftBasket {
    fn clone(&self) -> Self {
        Self {
            n_apples: self.n_apples.clone(),
            n_oranges: self.n_oranges.clone(),
            given_from: self.given_from.clone(),
            given_to: self.given_to.clone()
        }
    }
}
```

(In reality,
since `n_apples` and `n_oranges` both implement `Copy`,
you wouldn't actually call `clone()` on them.
But since the compiler knows their types,
it should be smart enough to
optimize the Clone away for you.)

If you imagine generalizing this
to any simple struct struct with named fields,
you might come up with a pseudocode template
like this one:

```text,ignore
impl Clone for ⟪Your struct⟫ {
    fn clone(&self) -> Self {
        Self {
            for each field:
                ⟪field name⟫: self.⟪field name⟫.clone(),
        }
    }
}
```

And here's how that pseudocode translates into
a derive-deftly template:

```
use derive_deftly::define_derive_deftly;

define_derive_deftly! {
    MyClone:

    impl Clone for $ttype {
        fn clone(&self) -> Self {
            Self {
                $( $fname : self.$fname.clone() , )
            }
        }
    }
}
```

Let's look at that template.  You've already seen `$ttype`: it expands
to the type on which you are applying the macro.  There are two new
pieces of syntax here, though:
[`$( ... )`][t:repetition]
and
[`$fname`][x:fname].


In derive-deftly templates, `$( ... )`  denotes repetition:
it repeats what is inside it
an "appropriate" number of times.
(We'll give a definition of "appropriate" later on.)
Since we want to clone every field in our struct,
we are repating the `field: self.field.clone() ,`
part of our implementation.

The `$fname` expansion means "the name of a field".
Which field?
Since `$fname` occurs inside `$( ... )`,
we will repeat the body of the `$( ... )` once for each
field, and expand `$fname`
to the name of a different field each time.

(Again, more advanced repetition is possible;
there's
[more to come](clone-enums.md#a-further-note-on-reptition).)


> ### On naming
>
> Many derive-deftly expansions'
> [names start with][keyword-initial-letters]
> `t` for **top-level**
> (whatever you are applying the template to),
> `v` for **variant**
> (a variant of an `enum`),
> or `f` for **field**
> (a single field of a struct or variant).
>
> So far, you've seen `$ttype` for "top-level type" and `$fname` for
> "field name".
>
> (We say "top-level" instead of "struct":
> later on, we'll be showing you how to apply
> derive_deftly to `enums`.)
>
> Many derive-deftly expansions end with
> a short identifier for what they contain.
> For example, `$tname` is the name of a top-level type,
> `$vname` is the name of a variant,
> and `$fname` is the name of a field.
> Whenever possible, we have tried to use the same
> identifier for the `t`, `v`, and `f` cases,
> whenever it is logical.


#### Will MyClone apply to other kinds of struct?

Rust defines several kinds of struct:
structs with fields (`struct Foo {...};`),
tuple structs (`struct Foo(...);`),
and unit structs (`struct Foo;`).

If you try to apply the `MyClone` template above
to a struct with fields,
it will work just fine.
But with a tuple struct, or a unit struct,
you might expect it to fail.

Surprisingly, it will still work fine!
This isn't because of any clever trickery
from derive-deftly:
it's just how Rust works.
When you use it on tuple or unit structs,
the `MyClone` template we wrote above will expand
to code like this...
which happens to be valid syntax!

```
struct TupleStruct(String, Option<String>);
impl Clone for TupleStruct {
    fn clone(&self) -> Self {
        Self {
            0: self.0.clone(),
            1: self.1.clone(),
        }
    }
}

struct UnitStruct;
impl Clone for UnitStruct {
    fn clone(&self) -> Self {
        Self {
        }
    }
}
```

This will be a common theme in what follows:
Rust often lets you use a slightly unidiomatic syntax
so that you can handle many different cases
in the same way.

[t:repetition]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#t:repetition
[x:fname]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:fname
[keyword-initial-letters]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#keyword-initial-letters
