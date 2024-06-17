# **Notes and plans (`NOTES.md`)**

# Future improvements which are (currently) difficult

(Should this section be tickets instead?)

 * Ideally we would check the case of reuseable derive names.
   These should be in Pascal case.
   But we only want to *lint* for this (and, suppressably), not fail,
   and that's hard.


# Future template features

## String concatenation `${string ...}`
		
Argument can contain any expansions (will be expanded into tokens and
stringified), and string literals (in `" "`).  No un-quoted literal
text is allowed.  Expansion is a single string literal.  FTAOD
`${string ${string "y"}}` expands to `r#""y""#`

Do we need this?  Perhaps we should just expect people
to use `stringify!` from `std`, which works well with derive-deftly.

Would we support case conversion ?


## Splitting off fields and handling subsets of the generics

Syntax and semantics TBD.  Some notes:

```text
   For things that need to split off fields        struct Foo as above {
   and talk only about subsets of the generics         field: Box<T>,
      generic parameter uses (for fields)
      		$fgens					T,
		$fgens_omitted				'l, C
      For explicit iteration, within ${for tgens ...} or $( ... )
		$tgname					'l   	T   	C
		$tgbounds ???

   Something for being able to handle structs/unions/enums
   equally in template, whatever that means.  We need to expand
   something to struct/union/enum, and possibly the brackets and
   commas in enum { ..., ..., }, and so on.
```

# Future plans wrt macro namespace questions

## Deriving from things other than data structures

It would be nice to be able to eventually support deriving from
items (traits, fns, ...).  This would have to be an attribute proc macro.  Attribute proc macros get to modify their applicand, but we would not do that.

Ideally that attribute macro would be `#[derive_deftly]`.  However:

 * We are already using that as an inert helper attribute for `#[derive(Deftly)]`.  Possibly we could experiment to see how that would interact with a non-inert attribute macro, except that:

 * It is not possible to have an attribute macro and a function-like macro with the same name; even though the invocation syntaxes (and implementing macro function signatures) are different.

## Proposed taxonomy of macros and attributes

We won't implement all of this right away,
but it is good to have a plan to make sure the names we take now
won't get in the way.

 * **`#[derive(Deftly)]`**:
   invokes the from-struct derivation machinery; enables:
    1. use of `#[derive_deftly(ReuseableMacro)]` on this very struct
    2. later use of `derive_deftly_adhoc!` of the same struct
	   (if `#[derive_defly_adhoc]` specified)
    3. `#[deftly(...)]` attributes on bits of the data structure
	   (checked via chaining technique).

 * **`define_derive_deftly!{ [export] MACNAME: TEMPLATE }`**:
   define a reusable template, which may be invoked as
   `#[derive_deftly(MACNAME)]`
   (within a struct annotated with `#[derive(Deftly)]` or
   `#[item_derive_deftly(MACNAME)]`.

 * **`derive_deftly_adhoc!{ DRIVERNAME: TEMPLATE }`**:
   adhoc derivation from something previously annotated with
   `#[derive(Deftly)]` or `#[item_derive_deftly]`.
   `DRIVERNAME` is an item path; we conflate the type and value namespaces.

 * **`#[derive_defly_adhoc]`**:
   Inert helper attribute to enable use of `derive_deftly_adhoc!`.

 * **`#[item_derive_deftly(MACNAME)]`**:
   attribute macro to be applied to items.
   The item is reproduced unchanged, except that
   `#[deftly]` attributes *in places where we would look for them*
   are filtered out.
   `#[item_derive_deftly]` will look forward to see if there are
   further `#[item_derive_deftly]` attributes,
   so that they can be combined and processed together
   (this is necessary for correctness of meta attr handling).
   Template *must* use `for ...` option.
   `#[derive_deftly_adhoc]` works as usual.
   It's an error to have `#[item_derive_deftly]` without the `()`.

 * **`#[deftly]`**:
   Inert helper attribute for `#[derive(Deftly)]`.
   Filtered-out attribute for `#[item_derive_deftly]`.
   Contents available via `$Xmeta`.

 * **`#[only_derive_deftly]`**:
   attribute macro to be applied to items;
   like `#[item_derive_deftly]` but *consumes and does not emit* the item.
   (We don't really need to be sure about this name;
   this will be an unusual case and we can give it whatever name seems good,
   later.)

## consume and not emit:

### Composition problem with `#[deftly]` attributes

You should be able to compose mutually-ignorant derive-deftly templates.
In particular you should be able to chain transforming templates,
and transforming templates should be able to
output invocations of normal deriving templates.

This won't work without doing "something",
because the outer invocation (the first to process the input)
will see a bunch of unrecognised `#[deftly]` attributes.

I'm not sure what the answer is,
but perhaps a template option for accepting `#[deftly]` attrs
and `${attr}` for transforming them.

Then the caller could `#[deftly(inner(foo))]`
and the inner template would receive `#[deftly(foo)]`.

Perhaps.

### Possible alternative syntax/naming

 * **`#[transform_deftly]`**:
   attribute macro to be applied to items.

 * d-d option `transform`.
   Insists that this template is for `#[transform_deftly]` only.

## `for ...` d-d options

Currently, we have `for enum`, `for struct`, `for union`.
These are fine.

We want also want ways to say:

 * `struct` or `enum`, not `union`: `for data`?
 * Particular kind of item: `fn`, `trait`, `mod`, `const`.
 * Any item: `item` (with `#[item_derive_adhoc]` for non-data items).
 * Combinations of the above: eg `for fn/const`?

Outstanding questions, therefore:

 * Does `for any` mean anything?
 * What keyword is "`struct`/`enum`"?
 * Do we need a keyword for `struct`/`enum`/`union`? 
   Probably, since this is going to be the default!
 * Is `/` the right separator for "or"?

### Internals

 * **`derive_deftly_engine!`**: Proc macro that does all the work.

 * **`derive_deftly_driver_DRIVERNAME!`**:
   `macro_rules` macro generated by `#[derive(Deftly)]` and
   `#[item_derive_deftly]`, embodying a driver.

 * **`derive_deftly_template_MACNAME!`**:
   `macro_rules` macro generated by `define_derive_deftly!`,
   embodying a template.

# Things to check before declaring 1.0

None!

But we should get some experience with the renamed crate,
probably by upgrading arti to it.
