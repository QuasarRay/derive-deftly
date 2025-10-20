# **Notes and plans (`NOTES.md`)**

# Future template features

## Stringification enhancements

### `${stringify ...}` - re-stringification

**Draft docs**

Usually, use `${concat ..}`.

`${stringify }` expands the content,
displays it (converting Rust tokens to their string representations)
and expands to a single string literal whose value is that content.

String literals in the input,
including the expansions of `${concat }` and `${Xmeta as str}`
are *re-quoted*,
not simply concatenated with
the string representation of non-string tokens,
resulting in double-quoting.

The precise representation is neither defined nor stable!
The expansion can be used in documentation or messages
but should not reinterpreted as Rust code,
nor compared for equality.

**Add to list of things allowed in `${concat ..}`**

 * `${stringify }` expansions:
   The value of the `${stringify }` string literal is used.
 
#### Examples

 * `${stringify $fvis}`: `"pub(crate)"`
 * `${concat $%{stringify $fvis}}`: `"pub(crate)"`
 * `${stringify "requoted"}`: `"\"requoted\""`
 * `${stringify ${concat $fvis}}`: `"\"pub(crate)\""`

### String templating `$"..."`

This is a convenient syntax for specifying a string template.
It is equivalent to a use of `${concat }`.
(Counts for `${define }` too.)

The leading `$` is followed by a (possbily raw) string literal.

The string literal contents is lexed as follows:

 * Literal text
 * Expansions in the forms
   `$keyword`, `${keyword ...}`, `$<...>`,
   but `"`, `'` and ``\`` are completely forbidden within the `...`.
   If you need this, use a `${define ...}`.
 * Doubled dollars `$$`

Possibly in the future we might support
`${"template with $1 positional arguments" $< blarky $fname >}`
or similar.

### Doc attributes

We could support `$///` and `$//!`.
The compiler turns these into `#[doc(...)]` and `#![doc(...)]`.

So we would have to recognise `$#[doc` and `$#![doc`.
(This means we would find it hard (or maybe impossible)
to use `$#` for anything other than attributes.)

The literal would be processed as for `$"..."`.

### Byte strings

We could support byte literals maybe.
`${string }` doesn't have a way to request them;
there would probably have to be `${byte_string }`.
`$"..."` does have a natural way: `$b"..."`.

Supporting byte strings would mean allowing `b"..."`
within (the relevant) string context,
posing the possibility of
`${string b"non-utf8 bytes"}` which has to be an error.

When is this error checked?
Is there one kind of string context, or two?
Do we allow `${string b"valid utf8"}`?

We could detect the invalid utf-8 at expansion time.
But perhaps we should declare
that we reserve the right to check this statically,
and that wouldn't be a breaking change?

## Scopes (for meta, and loops)

See [discussion in #36](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/36#note_3015721)

### Demo - cross product, allowing access to outer repetition

```rust,ignore
 ${for let a = fields {
   ${for let b = fields {
     self_cross_product_contains(${a.fname} ${b.fname});
   }}
 }}
```

### Fixing awkwardness re optional meta attributes

[#40](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/40)

```rust,ignore
 ${if let m = fmeta(call) {
   ${m.meta as expr} (&self.$fname);
 }}
```

### Handling meta attribute multiplicity

[#36](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/36)

```rust,ignore
 ${for let m = fmeta(call) {
   ${m.meta as expr} (&self.$fname);
 }}
```

### Looking for a thing in various places:

```rust,ignore
 ${if let m = any(fmeta(call), vmeta(fields(call)), tmeta(fields(call))) {..}}
 ${for let m = all(fmeta(call), vmeta(fields(call)), tmeta(fields(call))) {..}}
 ${if let m = select1(fmeta(call), fmeta(obsolete_name_for_call)) {..}}
```

### Meta scope referring to nonexistent meta item?

Can a meta item scope refer to a putative,
but actually nonexistent, item?
Not sure if we need this.
[#62](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/62)
I suggest not, in the first instance.

```rust,ignore
 ${let m = fmeta(call) {
   ${if ${m.meta} { ... }}
 }}
```

### Details of binding in `if ... any` etc.

What about mixed binding and non-binding conditions?
Options are
(a) reject this
(b) on the non-binding arms, bind to nonexistent meta.
I think I prefer (a).
It's the more cautious approach, certainly.

```rust,ignore
 ${if let m = any(fmeta(call), true) {
   ${if ${m.meta} { ... }}
}}
```

### Meta scopes vs repeat scopes

Every scope is either a meta scope or a repeat scope.
`${scope.meta}` is only allowed for meta scopes.
Other expansions, including notably `${scope.Xmeta}`,
are only allowed for repeat scopes.

```rust,ignore
 ${for let m = fmeta(call) {
   .. ${m.fmeta(call)} .. // ERROR, user wrote `fmeta`, wanted `meta`
   .. ${m.meta(call)} .. // OK
 }}
 ${for let m = fields {
   .. ${m.fmeta(call)} .. // OK
   .. ${m.meta(call)} .. // ERROR, m isn't a meta scope
                         // user must write ${m.fmeta}
 }}
```

#### Alternative

With a meta scope `m`,
the only legal expansion using it is `${m.meta((call)}`
or whatever.

(Right now;
But expansions other than `$Xmeta` might be OK to support.
Not `$Xmeta` because it has too much scope for error:
since `${if let m = fmeta(call) { .. ${m.fmeta(..)} .. }}`
doesn't do at all what the user might expect.)

So, instead, we could say that you don't need to write `.meta`
and have it be just `${m(call)}`.  But:

 * Now it lives in the same namespace as keywords,
   and is a user-defined name, so it must be uppercase.
   `${M(call)}`.
 * This reduces the similarity between meta scopes and normal scopes.
 * It would prevent us supporting eg `${m.fname}` in the future,
   which might be useful with something like
   `${if let m = find1(field, fmeta(call)) { $m.fname ... }}`.
   (meaning "find the precisely one field with `#[deftly(call)]`,
   and then expand stuff with it),
   or other things I haven't thought of yet.
 * If we support arguments to user-defined meta items,
   the syntax for passing them wouldn't look like the meta syntax,
   so `${M(call)}` is syntactically weird.

### Binding and checking

Binding is dynamic (like `${define }`)
(despite the use of `let` which is often lexical
in other languages including Rust).

(Meta attribute checking is dynamic and precise, naturally.)

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

# Sharing template code between multiple derives

### Impetus and xrefs

 * [\#39](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/39)
   Way to define multiple templates sharing `${define ..}` and `${defcond }`
 * [\#35](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/35)
   Can derive_deftly macros be modular?
 * [\#69](https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/69)
   proposed syntax change to make #39 easier: braces around top-level of definition.

### Proposed approach

```rust,ignore
define_derive_deftly_module! {
    ///DOCS
    [export] ModuleName:

    use SubModule;
    ${define ...}
    ${defcnd ...}
}
```

Generates `macro_rules! derive_deftly_module_module_name`
which captures just those definitions.

```rust,ignore
define_derive_deftly! {
    use [crate::]ModuleName;

    ///DOCS
    Template:
}
```

This meets all (or almost all) of the use cases, I think.

### Questions

 * Is "module" the right name?  I have chosen "module" rather than
   `mod` because these things are rather unlike normal Rust modules.
   ("defines" or "definitions" would be hopelessly confusing.)

 * `define_derive_deftly! { mod ...` would be a possible module
   definition syntax but it seems more magical than having a totally
   different top-level definition macro.  (And using `mod` this way
   makes the syntax overlap more between different uses.)

 * Casing of the module name.  I have chosen `ModuleName` mostly to
   avoid clashing with future d-d keywords (eg here we have `export`).
   We would forbid module names starting with lowercase.

### Fine points

 * The module can only contain `${define}`, `${defcond}` and `use`.
   So it cannot define derives as-such, but it could contain
   the whole *body* of a derive as a `${define}`.

 * Definitions from modules can be shadowed by definitions in
   subsequently-included modules, or in the main template.

 * There is no namespacing, and scope is (still) dynamic, not lexical.
   This plus shadowing means that a module's "internal" defines might
   be redefined by the module's user.  So definitions in modules
   should have a namespace component within their name - including
   internal definitions.

 * `use` statements in `define_derive_deftly!` appear before the
   name of the template being defined.  In `define_derive_deftly_module!`
   they appare after the module name.  This is partly related to docs
   processing, see below.  It also makes it clear that `use`
   in a module makes the definitions available to the caller.

 * Module references in `use` statements are resolved and their contents
   syntax-checked at the point where the `use` is encountered.
   (So `define_derive_deftly_module` is a proc macro.)

 * There is no way to specify per-dd-module template options.
   (Doing that would involve expanding different parts of a single
   template with  different options in force.)

 * Corrolary: there is no way to control beta features.
   Beta features are always enabled in the module-definition-time
   syntax check.  If a module uses beta features then every derive
   that uses that module must enable beta features.  But:

 * The whole feature (both `define_derive_deftly_module!` and `use`)
   is beta, so only available if the `beta` cargo feature is enabled,
   and the `beta_deftly` template feature is on each template with
   a `use`.

### Documentation sharing

We want a way to share documentation text between different derives.

Right now nothing outside the template expansion is `$-expaneded`,
so `///DOCS` in `define_derive_deftly!` can only be literal.
Extend that as follows:

Allow `///DOCS` *in derives* to also contain `${DOC_REFERENCE}` or
`$DOC_REFERENCE` where that's a definition from a module whose body
is precisely zero or more `#[doc = "..."]` (without any nested
expansions or anything).

So this will allow a limited form of documentation-sharing,
that's about as powerful as `#![doc=include_str!(" ... ")]`,
and can be extended later if we want.

(Implicitly this is a new expansion mechanism, which does not
involve a driver struct.)

Documentation for a derive-deftly *module* can't use this feature.

### Semver implications of `export`:

Normally, template code present in different crates can be processed
by different, perhaps semver-incompatible, versions of derive-deftly.

But, a whole derive must be processed by *one* version of
derive-deftly.  Ie, when you `use` a module from another crate, that
other crate's module's template text gets processed with *your*
version of derive-deftly.

So `export`` means you're on your own: considerations are similar to
(but less radical than) those for exporting a driver.

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
