# Working with structs and fields

Let's start by imagining that
we didn't have `derive-deftly`,
and we had to write Clone from scratch
for a simple structure like this:

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

Let's look at that template piece by piece.
Certain marked "expansion keywords" in the contents
(those marked with `$`)
are replaced with a value that depends on the struct
that the template is beling applied to.

You've already seen `$ttype`: it expands
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
part of our implementation
once for each field.

The `$fname` expansion means "the name of a field".
Which field?
Since `$fname` occurs inside `$( ... )`,
we will repeat the body of the `$( ... )` once for each
field, and expand `$fname`
to the name of a different field each time.

(Again, more advanced repetition is possible;
there's
[more to come](clone-enums.md#a-further-note-on-repetition).)

## An aside on naming

At this point, you've seen expansions with names like `$ttype`
and `$fname`, and you may be wondering where they come from.
If you want to know more,
you can skip ahead to the section
on [naming conventions](./expansions-naming.md).

[t:repetition]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#t:repetition
[x:fname]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#x:fname
