### Simple templates: fields and repetition

Let's start by imagining that
we didn't have `derive-deftly`,
and we had to write Clone from scratch
for a simple structure like this:

```rust
struct GiftBasket {
   n_apples: u8,
   n_oranges: u8,
   given_from: Option<String>,
   given_to: String,
}
```

We would probably write something like this:

```rust
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

> (In reality,
> since `n_apples` and `n_oranges` both implement `Copy`,
> you wouldn't actually call `clone()` on them.
> But since the compiler knows their types,
> it should be smart enough to
> optimize the Clone away for you.)

If you imagine generalizing this
to any simple struct with named fields,
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
a `derive-deftly` template:

```rust
use derive_deftly::{define_derive_deftly, Deftly};

define_derive_deftly! {
    MyClone =

    impl Clone for $ttype {
        fn clone(&self) -> Self {
            Self {
                $( $fname : self.$fname.clone() , )
            }
        }
    }
}

#[derive(Deftly)]
#[derive_deftly(MyClone)]
struct GiftBasket {
    // ...
}
```

Let's look at that template piece by piece.

As in our ["hello world" example](./hello-world.md),
`define_derive_deftly!` creates a template that we later apply
with `#[derive(Deftly)]`.
When applied, the template expands to its contents—that is,
everything after `MyClone =`.
Certain marked "expansion keywords" in the contents
(those marked with `$`)
are replaced with a value that depends on the struct
that the template is beling applied to.

You've already seen `$ttype`: it expands
to the type on which you are applying the macro.  There are two new
pieces of syntax here, though:
[`$( ... )`][ref:repetition-and-nesting]
and
[`$fname`][ref:fname-vname-tname--names]

The `$( ... )` syntax denotes repetition:
it repeats what is inside it
an "appropriate" number of times.
(This syntax will be familiar if you already know `macro_rules!`.)
Since we want to clone every field in our struct,
we are repating the `field: self.field.clone() ,`
part of our implementation
once for each field.

The `$fname` expansion means "the name of a field".
It expands to the name of a different field for each repetition
of the `$( ... )` block.
Since we have four fields in `GiftBasket`,
the `$( ... )` block will repeat four times,
and `$fname` will expand to a different field each time.

<!-- TODO: Re-add this.
   (Again, more advanced repetition is possible;
   there's
   [more to come](#a-further-note-on-reptition).)
-->


> ### On naming
>
> Many `derive-deftly` expansions'
> names [start with][ref:keyword-initial-letters]
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
> After this character,
> many `derive_deftly` expansions end with
> a short identifier for what they contain.
> For example, `$tname` is the _name_ of a top-level type,
> `$vname` is the _name_ of a variant,
> and `$fname` is the _name_ of a field.
> Whenever it is logical,
> we have tried to use the same
> identifier for the `t`, `v`, and `f` cases.

[ref:repetition-and-nesting]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#repetition-and-nesting
[ref:fname-vname-tname--names]: https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#fname-vname-tname--names
[ref:keyword-initial-letters] keyword-initial-letters#keyword-initial-letters
