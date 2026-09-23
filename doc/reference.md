# **Template syntax (and expansion options) reference**

<!-- @dd-navbar reference .. -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="../index.html">crate top-level</a> | <a href="../index.html#overall-toc">overall toc, macros</a> | <strong>template etc. reference</strong> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>

**Table of contents** <br>
(see also the [Index](#t:index))

<!--##toc##-->
   * [Template syntax overview](#template-syntax-overview)
      * [Named and positional template arguments to expansions and conditions](#named-and-positional-template-arguments-to-expansions-and-conditions)
   * [Repetition and nesting](#repetition-and-nesting)
   * [Expansions](#expansions)
      * [`$fname`, `$vname`, `$tname` -- names](#fname-vname-tname--names)
      * [`$fvis`, `$tvis`, `$fdefvis` -- visibility](#fvis-tvis-fdefvis--visibility)
      * [`$vpat`, `$fpatname` -- pattern matching and value deconstruction](#vpat-fpatname--pattern-matching-and-value-deconstruction)
      * [`$ftype`, `$vtype`, `$ttype`, `$tdeftype` -- types](#ftype-vtype-ttype-tdeftype--types)
      * [`$impl` -- shorthand for impl blocks (**beta**)](#impl--shorthand-for-impl-blocks-beta)
      * [`$tgens`, `$tgnames`, `$twheres`, `$tdefgens` -- generics](#tgens-tgnames-twheres-tdefgens--generics)
      * [`${gens_collate ..}`, `${gens_nodefs ...}`, `${gens_names ..}` -- collate and redact arbitrary generics (**beta**)](#gens_collate--gens_nodefs--gens_names---collate-and-redact-arbitrary-generics-beta)
      * [`${tmeta(...)}` `${vmeta(...)}` `${fmeta(...)}` -- `#[deftly]` attributes](#tmeta-vmeta-fmeta--deftly-attributes)
      * [`${fattrs ...}` `${vattrs ...}` `${tattrs ...}` -- other attributes](#fattrs--vattrs--tattrs---other-attributes)
      * [`$findex` `$vindex` -- field/variant numerical index](#findex-vindex--fieldvariant-numerical-index)
      * [`${paste ...}`, `$<...>`, `${paste_spanned}` -- identifier pasting](#paste---paste_spanned--identifier-pasting)
      * [`${concat ...}` - string literal concatenation (**beta**)](#concat----string-literal-concatenation-beta)
      * [`$"..."`, `$///...` - string and documentation templating (**beta**)](#----string-and-documentation-templating-beta)
      * [`${CASE_CHANGE ...}` -- case changing](#case_change---case-changing)
      * [`${when CONDITION}` -- filtering out repetitions by a predicate](#when-condition--filtering-out-repetitions-by-a-predicate)
      * [`${if COND1 { ... } else if COND2 { ... } else { ... }}` -- conditional](#if-cond1----else-if-cond2----else-----conditional)
      * [`${select1 COND1 { ... } else if COND2 { ... } else { ... }}` -- expect precisely one predicate](#select1-cond1----else-if-cond2----else-----expect-precisely-one-predicate)
      * [`${for fields { ... }}`, `${for variants { ... }}`, `$( )` -- repetition](#for-fields----for-variants-------repetition)
      * [`$crate` -- root of template crate](#crate--root-of-template-crate)
      * [`$tdefkwd` -- keyword introducing the new data structure](#tdefkwd--keyword-introducing-the-new-data-structure)
      * [`$tdefvariants`, `$vdefbody`, `$fdefine` -- tools for defining types](#tdefvariants-vdefbody-fdefine--tools-for-defining-types)
      * [`$loop_index`, `$for_loop_count`, `${loop_exactly_1 ...}` -- repetition information (**beta**)](#loop_index-for_loop_count-loop_exactly_1---repetition-information-beta)
      * [`${v_explicit_discriminant ..}` -- Explicitly specified enum discriminant](#v_explicit_discriminant---explicitly-specified-enum-discriminant)
      * [`${ignore ..}` -- Expand but then discard](#ignore---expand-but-then-discard)
      * [`${dbg ..}`, `$dbg_all_keywords` -- Debugging output](#dbg--dbg_all_keywords--debugging-output)
      * [`${define ...}`, `${defcond ...}` -- user-defined expansions and conditions](#define--defcond---user-defined-expansions-and-conditions)
      * [`${error ...}` -- explicitly throw a compile error](#error---explicitly-throw-a-compile-error)
   * [Conditions](#conditions)
      * [`fvis`, `tvis`, `fdefvis` -- test for public visibility](#fvis-tvis-fdefvis--test-for-public-visibility)
      * [`fmeta(NAME)`, `vmeta(NAME)`, `tmeta(NAME)` -- `#[deftly]` attributes](#fmetaname-vmetaname-tmetaname--deftly-attributes)
      * [`v_is_first`, `f_is_first` -- is this the first variant/field? (**beta**)](#v_is_first-f_is_first--is-this-the-first-variantfield-beta)
      * [`is_struct`, `is_enum`, `is_union`](#is_struct-is_enum-is_union)
      * [`v_is_unit`, `v_is_tuple`, `v_is_named`](#v_is_unit-v_is_tuple-v_is_named)
      * [`tgens`](#tgens)
      * [`loop_initial`, `loop_final` -- repetition information (**beta**)](#loop_initial-loop_final--repetition-information-beta)
      * [`v_explicit_discriminant` -- enum discriminant explicitly specified?](#v_explicit_discriminant--enum-discriminant-explicitly-specified)
      * [`is_empty(..)`, `approx_equal(ARG1, ARG2)` -- equality comparison (token comparison)](#is_empty-approx_equalarg1-arg2--equality-comparison-token-comparison)
      * [`false`, `true`, `not(CONDITION)`, `any(COND1,COND2,...)`, `all(COND1,COND2,...)` -- boolean logic](#false-true-notcondition-anycond1cond2-allcond1cond2--boolean-logic)
      * [`dbg(...)` -- Debug dump of condition value](#dbg--debug-dump-of-condition-value)
   * [Case changing](#case-changing)
   * [Expansion options](#expansion-options)
      * [`expect items`, `expect expr` -- syntax check the expansion](#expect-items-expect-expr--syntax-check-the-expansion)
      * [`for struct`, `for enum`, `for union` -- Insist on a particular driver kind](#for-struct-for-enum-for-union--insist-on-a-particular-driver-kind)
      * [`dbg` -- Print the expansion to stderr, for debugging](#dbg--print-the-expansion-to-stderr-for-debugging)
      * [`meta_quoted` -- control (de)quoting of attribute values (**beta**)](#meta_quoted--control-dequoting-of-attribute-values-beta)
      * [`beta_deftly` -- Enable unstable template features](#beta_deftly--enable-unstable-template-features)
      * [Expansion options example](#expansion-options-example)
   * [Precedence considerations](#precedence-considerations)
      * [`None`-delimited groups](#none-delimited-groups)
   * [Structs used in examples](#structs-used-in-examples)
   * [Keyword index](#keyword-index)
      * [Expansions index](#expansions-index)
      * [Conditions index](#conditions-index)

**Reference documentation for the actual proc macros** is in
the [crate-level docs for derive-deftly](../index.html#macros).

<!-- Link conventions in this document.

    We use x:foo for the expansion "foo".

    We use c:foo for the condition "foo".

    We use t:foo for the topic "foo".

    We use eo:foo for a top-level expansion option "foo".

-->

[**beta**]: ../doc_changelog/index.html#t:beta
[positional argument]: #t:arguments

<div id="t:syntax">

## Template syntax overview

</div>

Within the macro template,
expansions (and control structures) are introduced with `$`.
They generally refer to properties of the data structure that
we're deriving from.
We call that data structure the **driver**.

In general the syntax is:

 * `$KEYWORD`: Invoke the expansion of the keyword `KEYWORD`.
 * `${KEYWORD ARGS...}`: Invoke with parameters.
 * `$( .... )`: Repetition (abbreviated, automatic, form).
   (Note: there is no `+` or `*` after the `)`)
 * `$< .... >`: Identifier pasting (shorthand for
   [`${paste ...}`](#x:paste)).

In all cases, `$KEYWORD` is equivalent to `${KEYWORD}`.

<span id="dollar-dollar">You can pass a `$` through
by writing `$$`.</span>

<span id="keyword-initial-letters">Many
of the expansion keywords start with `f`, `v`, or `s` to indicate
the depth of the thing being expanded:</span>

 * `f...`: Expand something belonging to a particular Field.

 * `v...`: Expand something belonging to a particular Variant.

 * `t...`: Expand something applying to the whole Top-level type.

In the keyword descriptions below,
`X` is used to stand in for one of `f`, `v` or `t`.

Defining a new type based on the driver
requires more complex and subtle syntax,
generated by special-purpose expansions `$Xdef...`.

(Here, within this documentation,
we often write in `CAPITALS` to indicate meta-meta-syntactic elements,
since all of the punctuation is already taken.)

Inner attributes (`#![...]` and `//!...`)
are not allowed in templates.

<div id="t:arguments">

### Named and positional template arguments to expansions and conditions

</div>

Some expansions and conditions take
(possibly optional)
named arguments,
or multiple positional arguments,
whose values are templates:

 * `${KEYWORD NAME=ARG NAME=ARG ...}`
 * `${KEYWORD ARG1 ARG2 ...}`
 * `CONDITION(NAME=ARG, NAME=ARG, ...)`
 * `CONDITION(ARG1, ARG2, ...)`

The acceptable contents vary,
but the syntax is always the same.
Each `ARG` must be one of:

 * `IDENTIFIER`
 * `LITERAL` (eg, `NUMBER` or `"STRING"`)
 * `$EXPANSION` (including `${KEYWORD...}`, `$<...>`, etc.)
 * `{ STUFF }`, where `STUFF` is expanded.
   (The `{ }` are just for delimiting the value, and are discarded).

<div id="t:repetition">

## Repetition and nesting

</div>

The driving data structure can contain multiple variants,
which can in turn contain multiple fields;
there are also attributes.

Correspondingly,
sections of the template, indicated by `${for ...}` and `$(...)`,
are expanded multiple times.

With `${for ...}`, what is iterated over is specified explicitly.

When `$( ... )` is used, what is iterated over is automatically
inferred from the content:
most expansions and conditions imply a "level":
what possibly-repeated part of the driver they correspond to.
All the expansions directly within `$(...)`
must have the same repetition level.

With both `${for }` and `$(...)`,
if the repetition level is "deeper" than the level
of the surrounding template,
the surrounding levels are also repeated over,
effectively "flattening".
For example, expanding `$( $fname )` at the very toplevel,
will iterate over all of the field names;
if the driver is an enum;
it will iterate over all of the fields in each of the variants
in turn.

structs and unions do not have variants, but
derive-deftly treats them as having a single (unnamed) variant.

#### Examples

<!--##examples-for `Enum`##-->
<!--##examples-ignore##-->
For [example enum `Enum`](#structs-used-in-examples):

 * `$($vname,)`: `UnitVariant, TupleVariant, NamedVariant,`
 * `$($fname)`: `0 field field_b field_e field_o`
 * `${for fields { hello }}`: `hello hello hello hello`

<div id="t:repetition-info">

#### Using information about repetitions (**beta**)

</div>

**[`loop_initial`](#c:loop_initial)**,
**[`loop_final`](#c:loop_final)**,
**[`$loop_exactly_1`](#x:loop_exactly_1)**
and
**[`$loop_index`](#x:loop_index)**
provide information about which repetition this is.

All of these count only iterations that are
selected by relevant [`${when }`](#x:when) clauses.
(Consequently, they are not allowed inside `${when }`.)

These always refer to the innermost repetition.
(Dynamic scoping means that it is not possible
to refer to information about outer repetitions,
even by use of `${define }`.)

The actual variant or field position, disregarding `${when }`,
can be obtained via
[`v_is_first`](#c:v_is_first),
[`f_is_first`](#c:f_is_first),
[`$vindex`](#x:vindex),
and
[`$findex`](#x:findex),

**[`for_loop_count`](#x:for_loop_count)**
counts how many repetitions a `for` would contain.

When a repetition over fields implicitly repeats over variants too,
that is a single repetition over all fields,
for the purposes of `loop_...` and `for_loop_count`.

These features are available only in [**beta**].

## Expansions

Each expansion keyword is described in this section.
The examples each show the expansions for (elements of)
[the same example `Unit`, `Tuple`, `Struct` and `Enum`](#t:example-structs),
shown below.

<!-- ## maint/check-keywords-documented expansions ## -->

<div id="x:fname">
<div id="x:tname">
<div id="x:vname">

### `$fname`, `$vname`, `$tname` -- names

</div>
</div>
</div>

The name of the field, variant, or toplevel type.
This is an the identifier (without any path or generics).
For tuple fields, `$fname` is the field number.

`$fname` is not suitable for direct use as a local variable name.
It might clash with other local variables;
and, unlike most other expansions,
`$fname` has the hygiene span of the driver field name.
Instead, use `$vpat`, `$fpatname`,
or `${paste ... $fname ...}` (`$<... $fname ...>`).

#### Examples

 * `$fname`: `0`, `field`, `field_b`
 * `$vname`: `UnitVariant`
 * `$tname`: `Tuple`, `Struct`, `Enum`

<div id="x:fdefvis">
<div id="x:fvis">
<div id="x:tvis">
<div id="t:visibility">

### `$fvis`, `$tvis`, `$fdefvis` -- visibility

</div>
</div>
</div>
</div>

The visibility of the field, or toplevel type.

Expands to `pub`, `pub(crate)`, etc.
Expands to nothing for private types or fields.

This looks only at the syntax in the driver definition;
an item which is `pub` might still not be reachable,
for example if it is in a private inner module.

<div id="t:enum-visibility">

#### Enums and visibility

</div>

In Rust,
enum variants and fields don't have separate visibility;
they inherit visibility from the enum itself.
So there is no `$vvis`.

For enum fields, `$fvis` expands to the same as `$tvis`.
Use `$fvis` for the effective visibility of a field,
eg when defining a derived method.

`$fdefvis` is precisely what was written in the driver field definition,
so always expands to nothing for enum fields -
even though those might be public.
Use `$fdefvis` when defining a new enum.

#### Examples

 * `$tvis` for `Unit`: `pub`
 * `$tvis` for `Enum`: `pub`
 * `$tvis` for others: nothing
 * `$fvis` for `field` in `Struct`: `pub`
 * `$fvis` for `field_b` in `Struct`: `pub(crate)`
 * `$fvis` for fields in `Enum`: `pub`
 * `$fvis` for others: nothing
 * `$fdefvis` for `field` in `Struct`: `pub`
 * `$fdefvis` for `field_b` in `Struct`: `pub(crate)`
 * `$fdefvis` for fields in `Enum`: nothing
 * `$fdefvis` for others: nothing

<div id="x:fpatname">
<div id="x:vpat">

### `$vpat`, `$fpatname` -- pattern matching and value deconstruction

</div>
</div>

`$vpat` expands to a pattern
suitable for matching a value of the top-level type.
It expands to `TYPE { FIELD: f_FNAME, ... }`,
where `TYPE` names the top-level type or enum variant.
(`TYPE` doesn't have generics,
since those are not allowed in patterns.)

Each field is bound to a local variant `f_FNAME`,
where `FNAME` is the actual field name (or tuple field number).

`$fpatname` expands to `f_FNAME` for the current field.

#### `$vpat` named arguments

 * `self`: top level type path.  Default is `$tname`.
   Must expand to a syntactically valid type path,
   without generics.
 * `vname`: variant name.  Default is `$vname`.
   Not expanded for structs.
 * `fprefix`: prefix to use for the local bindings.
   Useful if you need to bind multiple values at once.
   (Then, reference the bindings with `$<FPREFIX $fname>`;
   `$fpatname` doesn't take a `fprefix` argument.)
   Default is `f_`.

These use derive-deftly's usual
[syntax for named arguments](#named-and-positional-template-arguments-to-expansions-and-conditions).

#### Examples

 * `$vpat` for structs: `Unit { }`, `Tuple { 0: f_0, }`
 * `$vpat` for enum variant: `Enum::NamedVariant { field: f_field, ... }`
 * `$fpatname`: `f_0`, `f_field`
 * `${vpat self=$<$tname Reference> vname=$<Ref $vname> fprefix=other_}`: `EnumReference::RefNamedVariant { field: other_field, ... }`

<div id="x:ftype">
<div id="x:tdeftype">
<div id="x:ttype">
<div id="x:vtype">

### `$ftype`, `$vtype`, `$ttype`, `$tdeftype` -- types

</div>
</div>
</div>
</div>

The type of the field, variant, or the toplevel type.

`$ftype`, `$vtype` and `$ttype`
are suitable for referencing the type in any context
(for example, when defining the type of a binding,
or as a type parameter for a generic type).
These contains all necessary generics
(as names, without any bounds etc., but within `::<...>`).

`$vtype` includes both the top-level enum type, and the variant.
To construct a value, prefer `$vtype` rather than `$ttype`,
since `$vtype` works with enums too.

`$tdeftype` is
the driver type in a form suitable for defining
a new type with a derived name (eg, using pasting).
Contains all the necessary generics, with bounds,
within `<...>` but without an introducing `::`.

The toplevel type expansions, `$ttype` and `$tdeftype`,
don't contain a path prefix, even when
a driver type argument to
`derive_deftly_adhoc!`
has a path prefix.

`$vtype` (and `$ttype` and `$tdeftype`) are not suitable for matching.
Use `$vpat` for that.

#### `$vtype` named arguments

 * `self`: top level type.  Default is `$ttype`.
   Must expand to a syntactically valid type.
 * `vname`: variant name.  Default is `$vname`.
   Not expanded for structs.

These can be specified using pasting `$<...>`
to name related (derived) types and variants.

They use derive-deftly's usual
[syntax for named arguments](#named-and-positional-template-arguments-to-expansions-and-conditions).

#### Examples

 * `$ftype`: `« std::iter::Once::<T> »`, `« Option::<i32> »`
 * `$vtype` for struct: `Tuple::<'a, 'l, T, C>`
 * `$vtype` for enum variant: `Enum::TupleVariant::<'a, 'l, T, C>`
 * `$ttype`: `Enum::<'a, 'l, T, C>`
 * `$tdeftype`: `Enum<'a, 'l: 'a, T: Display = usize, const C: usize = 1>`
 * `${vtype self=$<$ttype Reference> vname=$<Ref $vname>}`
   for enum variant:
   `EnumReference::RefTupleVariant::<'a, 'l, T, C>`

<div id="x:impl">

### `$impl` -- shorthand for impl blocks (**beta**)

</div>

  * **`$impl`**
   is short for `impl<$tgens> $ttype where $wheres`.
  * **`${impl TRAIT}`**
   is short for `impl<$tgens> TRAIT for $ttype where $wheres`.
 * **`${impl [<GENERICS>] TRAIT [for TYPE] [where WHERES]}`**
   allows each the output pieces to be separately overridden.

An `$impl` expands to one of the following:

 * `impl<GENERICS> TYPE where WHERES`
 * `impl<GENERICS> TRAIT for TYPE where WHERES`

The usual usage is `${impl ...} { ITEMS }`.

The overrideable elements are:

 * **`<GENERICS>`**:
   Generic bindings.
   The default is `$tgens`.
   `GENERICS` is processed as if by [`$gens_nodefs`](#x:gens_nodefs).

 * **`TRAIT`**:
   Trait to implement.
   The default is to make inherent impls (no output `for` clause).

 * **`for TYPE`**:
   Type to implement for.
   The default is [`$ttype`](#x:ttype).

 * **`where WHERES`**:
   Any `where` clauses.  The default is [`$twheres`](#x:twheres).
   (The expansion of `$impl` always contains the actual `where` keyword.)

Note that unlike normal `impl` syntax,
`for` inside `${impl }` *introduces the self type to implement for*,
rather than retrospectively indicating
that the preceding path was a trait.
So `${impl Trait} {..}`, unlike (inherent) `impl Type {..}`.

Each of the input pieces `GENERICS`, `TRAIT`, `TYPE` and `WHERES`
is a template which is expanded,
and then placed in the appropriate part of the resulting `$impl`.
`GENERICS` is terminated by the closing `>`.
`TRAIT`, `TYPE` and `WHERES` are terminated by the `for` and `where` keywords,
and may not contain other unbracketed keywords except
`self` `super` `crate` `dyn` `fn` `mut` `Self`.

Available only in [**beta**].

#### Examples

##### For `SimpleUnit`

<!--##examples-for `SimpleUnit`##-->

 * `$impl`: `impl<> SimpleUnit where`
 * `${impl Trait}`: `impl<> Trait for SimpleUnit where`
 * `${impl for Type}`: `impl<> Type where`

##### For `Struct`

<!--##examples-for `Struct`##-->

 * `$impl`: `impl<'a, 'l: 'a, T: Display, const C: usize> Struct<'a, 'l, T, C> where T: 'l, T: TryInto<u8>,`
 * `${impl AssertUnwindSafe} {}`: `impl<'a, 'l: 'a, T: Display, const C: usize> AssertUnwindSafe for Struct<'a, 'l, T, C> where T:  'l, T: TryInto<u8>, {}`
 * `${impl for Type}`: `impl<'a, 'l: 'a, T: Display, const C: usize> Type where T: 'l, T: TryInto<u8>,`
 * `${impl where}`: `impl<'a, 'l: 'a, T: Display, const C: usize> Struct<'a, 'l, T, C> where`
 * `${impl<> Trait for Type where}`: `impl<> Trait for Type where`

<div id="x:tdefgens">
<div id="x:tgens">
<div id="x:tgnames">
<div id="x:twheres">

### `$tgens`, `$tgnames`, `$twheres`, `$tdefgens` -- generics

</div>
</div>
</div>
</div>

Generic parameters and bounds, from the toplevel type,
in various forms.

   * **`$tgens`**:
     The generic arguments, with bounds
     (and the types of const generics)
     but without defaults.
     Suitable for use when starting an `impl`.

   * **`$tgnames`**:
     The generic argument names, without bounds.
     Suitable for use in a field type or in the body of an impl.

   * **`$twheres`**:
     The where clauses, as written in the toplevel type definition.

   * **`$tdefgens`**:
     The generic arguments, with bounds, *with* defaults,
     as written in the toplevel type definition,
     suitable for defining a derived type.

If not empty, each of these will always have a trailing comma.

Bounds appear in `$tgens`/`$tdefgens` or `$twheres`,
according to where they appear in the toplevel type,
so for full support of generic types the template must expand both.

#### Examples

 * `$tgens`: `'a, 'l: 'a, T: Display, const C: usize,`
 * `$tgnames`: `'a, 'l, T, C,`
 * `$twheres`: `T: 'l, T: TryInto<u8>,`
 * `$tdefgens`: `'a, 'l: 'a, T: Display = usize, const C: usize = 1,`

<div id="x:gens_collate">
<div id="x:gens_names">
<div id="x:gens_nodefs">

### `${gens_collate ..}`, `${gens_nodefs ...}`, `${gens_names ..}` -- collate and redact arbitrary generics (**beta**)

</div>
</div>
</div>

 * **`${gens_FILTER GENERICS...]}`**

Each of these expansion keywords:
 - expands the content (into tokens);
 - parses the result as a series of generic parameters
   (tolerating empty items, ie consecutive commas);
 - possibly redacts each of those parameters,
   selecting only part of each one;
 - collates all of the parameters into the proper order:
   lifetimes, types, then constants;

Collation is needed to add generic parameters to existing generic lists,
since simply appending or prepending doesn't always work.
Redaction is needed for proper use of generics in every syntactic context.

 * **`${gens_collate ..}`**: Just collates:
   each generic parameter is reproduced in its entirety.
   Like `$tdefgens`.
   Suitable for defining a new derived type.

 * **`${gens_nodefs ..}`**:
   Filters out defaults.
   Like `$tgens`.
   Suitable for binding parameters, e.g. when starting an `impl`.

 * **`${gens_names ..}`**:
   Only the names.
   Like `$tgnames`.
   Suitable for referring to already-bound parameters.

The expansion always ends in a comma, if it's not empty.

Available only in [**beta**].

#### Examples

 * `${gens_collate 'a: 'b, A: B = (), const C: i32 = 0}`: `'a: 'b, A: B = (), const C: i32 = 0,`
 * `${gens_nodefs 'a: 'b, A: B = (), const C: i32 = 0}`: `'a: 'b, A: B, const C: i32,`
 * `${gens_names 'a: 'b, A: B = (), const C: i32 = 0}`: `'a, A, C,`
 * `${gens_collate 'a, A, const C: i32, 'b, B, const D: i32}`: `'a, 'b, A, B, const C: i32, const D: i32,`
 * `${gens_collate $tgens}` for `SimpleUnit`: `/*empty*/`

##### More complex examples, involving example `Struct`

 * `${gens_nodefs $tdefgens, CorrectlyInserted}`: `'a, 'l: 'a, T: Display, CorrectlyInserted, const C: usize,`
 * `${gens_collate 'pre: 'bound, Pre: Bound = (), const PRE: i32 = 2, $tdefgens 'post: 'bound, Post: Bound = (), const POST: i32 = 3}`: `'pre: 'bound, 'a, 'l: 'a, 'post: 'bound, Pre: Bound = (), T: Display = usize, Post: Bound = (), const PRE: i32 = 2, const C: usize = 1, const POST: i32 = 3,`
 * `${gens_nodefs 'pre: 'bound, Pre: Bound = (), const PRE: i32 = 2, $tdefgens 'post: 'bound, Post: Bound = (), const POST: i32 = 3}`: `'pre: 'bound, 'a, 'l: 'a, 'post: 'bound, Pre: Bound, T: Display, Post: Bound, const PRE: i32, const C: usize, const POST: i32,`
 * `${gens_names 'pre: 'bound, Pre: Bound = (), const PRE: i32 = 2, $tdefgens 'post: 'bound, Post: Bound = (), const POST: i32 = 3}`: `'pre, 'a, 'l, 'post, Pre, T, Post, PRE, C, POST,`

<div id="x:fmeta">
<div id="x:tmeta">
<div id="x:vmeta">

### `${tmeta(...)}` `${vmeta(...)}` `${fmeta(...)}` -- `#[deftly]` attributes

</div>
</div>
</div>

Accesses macro parameters passed via `#[deftly(...)]` attributes.

 * **`${Xmeta(NAME)}`**:
   Looks for `#[deftly(NAME = ...VALUE...)]`,
   and expands to the driver-specified `VALUE`.
   `NAME = ...VALUE...` can be

   * **`NAME = { VALUE }`**: Rust code, enclosed in in curly brackets.
     (**beta**)
   * **`NAME = VALUE`**: Rust code (nonempty),
     terminated by the next comma, or closing `)`.
     Angle brackets `< >` and equals `=` are only allowed in `VALUE`
     inside `( )` or `{ }`, to avoid ambiguity.
     (**beta**)
   * **`NAME = "VALUE"`**: a string literal, which is re-parsed as Rust code
     (unless otherwise stated, below).

   The choice of value syntax is up to the driver,
   and is not made available to the template,
   except as described below.

   Normally,
   `as ..` must be given, to specify how `VALUE` should be treated;
   within `$(paste ..}`, `as str` is the default.

 * **`${Xmeta(SUB(NAME))}`**:
   Looks for `#[deftly(SUB(NAME = ...))]`.
   The `#[deftly()]` is parsed as
   a set of nested, comma-separated, lists.
   So this would find `NAME` 
   in `#[deftly(SUB1, SUB(N1, NAME = "VALUE", N2), SUB2)]`.
   The label can be arbitrarily deep, e.g.: `${Xmeta(L1(L2(L3(ATTR))))}`.

 * **`${Xmeta(...) as SYNTYPE}`**:
   Treats the value as a `SYNTYPE`.
   `SYNTYPE`s available are:

    * **`str`**: Expands to a string literal
      with the same contents as
      the string provided for `"VALUE"`.

      Ie, the attribute's string value is *not* parsed.
      This is the default within pasting and case changing,
      if no `as` was specified.
      Within pasting and case changing,
      the provided string becomes part of the pasted identifier
      (and so must consist of legal identifier characters).

      `NAME = { VALUE }` and bare `NAME = VALUE` are not allowed.

    * **`ty`**:
      `VALUE` is parsed as a type,
      possibly with generics etc. (`syn::Type`).
      When expanded, generic arguments have any missing `::` inserted,
      so that the expansion is suitable for use in any context,
      (such as invoking an inherent or trait method).

    * **`path`**:
      `VALUE` is parsed as a path,
      possibly with generics etc. (`syn::Path`).
      Like `as ty` but non-path types are forbidden.
      Rust uses the same path syntax for types and modules,
      so this is suitable for accepting a module path, too.

    * **`expr`**:
      `VALUE` is parsed as an expression.

      When expanded, it is surrounded with `( )`
      to ensure correct precedence.
      With `{ VALUE }`, the surrounding `{ }` are not stripped,
      and appear within the usual `( )`;
      this permits a block expression surrounded by only one pair of `{ }`.

    * **`ident`**:
      `VALUE` is parsed as an identifier (or keyword).
      (Within pasting, prefer `as str`, the default;
      `as ident` rejects initial digits, and the empty string.)

    * **`vis`**:
      `VALUE` is parsed as a visibility,
      possibly qualified (eg. `pub(in super)`..

    * **`attrs`**:
      `VALUE` is treated as a series of meta attributes,
      in one of the two following forms:

      * `attrname ... , attrname ...`
        where `attrname`s are paths,
        and `...` is anything not containing commas.
        Each `attrname ...` becomes `#[attrname ...]` in the expansion.

        (With the bare `NAME = VALUE` syntax, the value can only
        contain one attribute.  To specify more than one, write
        `NAME = "... "`,
        `NAME = { ... }`,
        or
        `NAME = attrname ..., NAME = attrname ...`.)

      * `#[attrname ...] #[attrname ...]`
        which are simply reproduced in the expansion.

      Unlike the other `${Xmeta as ...}` expansions,
      it is not an error for the the driver to supply zero, or several,
      matching `#[deftly]` meta values.
      `default` is not supported.

      Available only in [**beta**].

    * **`items`**:
      `VALUE` is parsed as zero or more Rust Items (`syn::Item`).

      The bare form `NAME = VALUE` is not allowed,
      because Rust items can contain commas.

    * **`token_stream`**:
      `VALUE` is parsed as an arbtitrary sequence of tokens
      (`TokenStream`).

      When using this option, be careful about operator precedence:
      see [Precedence considerations](#precedence-considerations).

      The bare form `NAME = VALUE` is not allowed.

 * **`${Xmeta(...) .. , default DEFAULT}`**:
   If there is no `VALUE` expands
   the [positional argument] `DEFAULT` instead.

   NB: in this case the expansions of `DEFAULT` is used as is:
   *not* affected by any `as ..` clause;
   *not* surrounded with additional `( )` (for `as expr`),
   nor any additional [`None`-delimited group](#t:none-delimiters).
   See [Precedence considerations](#precedence-considerations).

 * **`${Xmeta(...) .. , quoted strip | quoted rigorous | quoted retain}`** ([**beta**]):

   Specifies how to handle `NAME = "..."` in a `#[deftly]`:

   * **No `quoted`** specified:

     Only `NAME = "VALUE"` is supported.
     The quotes are always stripped and the content re-parsed as Rust code.

     This is currently the default.

   * **`quoted strip`**:

     If `NAME = "VALUE"` is specified, the quotes are stripped,
     and the content is reparsed as Rust code.

     This will probably become the default in future.

   * **`quoted rigorous`**:

     For `as expr`,
     `NAME = "VALUE"` is forbidden,
     because it's ambiguous.

     This can be helpful for transition to unquoted attribute values,
     if expression(s) of string type are (possibly) expected, since
     existing attributes would be silently misinterpreted by `quoted strip`.

     With `SYNTYPE`s other than `as expr`
     the quotes are always stripped and the contents reparsed.

     This may become the recommended mode for new templates,
     and should typically be set via the
     [`meta_quoted`](#eo:meta_quoted) template option.

   * **`quoted retain`**:

     For `as expr`,
     `NAME = "VALUE"` is treated as specifying a string literal,
     i.e., as just another instance of `NAME = VALUE`.
     The quotes are not stripped.

     With `SYNTYPE`s other than `as expr`
     the quotes are always stripped and the contents reparsed.

   Available only in [**beta**].

   The default can be overridden by the
   [`meta_quoted` template option](#eo:meta_quoted).

   <!--

   TODO When this feature is stabilised, it will affect only
   `as expr`, and we can change its summary to:

   > Specifies how to handle `NAME = "..."` with `as expr` in a `#[deftly]`

   -->

When expanding `${Xmeta}`,
it is an error if the value was not specified in the driver,
and also an error if multiple values were specified.

For a struct, both `$tmeta` and `$vmeta`
look in the top-level attributes.
This allows a template to have uniform handling of attributes
which should affect how a set of fields should be processed.

Within `${Xmeta ..}`,
options (`as` and `default`)
are each introduced with a keyword,
and separated by commas.

#### Attribute namespacing

`derive-deftly` does not impose any namespacing within `#[deftly]`:
all templates see the same `deftly` meta attributes.

To avoid clashes,
macros intended for general use should look for attributes
within a namespace for that template.
The usual convention is to accept attributes
scoped within the snake-cased name of the template,
as demonstrated
[in the introduction](https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/constructor-attrs.html#meta-attr-scope).

#### Unrecognised/unused `#[deftly(...)]` attributes

Every `#[deftly(...)]` attribute on the input data structure
must correspond to a `${Xmeta...}` expansion
(or `fmeta(...)` boolean test, as applicable)
in the template(s) applied to that driver.

The `Xmeta` reference must have been actually expanded (or tested),
so parts of the template that weren't expanded don't count.

These checks are disabled by `#[derive_deftly_adhoc]`.

#### Examples

 * `${tmeta(simple) as ty}`: `« String »`
 * `${tmeta(missing) as ty, default String}`: `String`
 * `${tmeta(simple) as path}`: `« String »`
 * `${tmeta(simple) as str}`: `"String"`
 * `${tmeta(simple) as token_stream}`: `String`
 * `${tmeta(gentype) as ty}`: `« Vec::<i32> »`
 * `${tmeta(gentype) as str}`: `"Vec<i32>"`
 * `${tmeta(gentype) as token_stream}`: `Vec<i32>`
 * `${vmeta(value) as ident}`: `unit_toplevel`, `enum_variant`
 * `${tmeta(some_vis) as vis}`: `pub(crate)`
 * `${fmeta(nested(inner)) as expr}` for `field` in `Struct`: `(42)`
 * `${fmeta(nested(inner)) as expr, quoted rigorous}` for `field` in `Struct`: rejected, ``string literal attribute value is ambiguous``
 * `${fmeta(nested(inner)) as expr, quoted retain}` for `field` in `Struct`: `("42")`
 * `${vmeta(items) as items}` for `TupleVariant`: `type T = i32; const K: T = 7;`
 * `${vmeta(items) as token_stream}` for `TupleVariant`: `type T = i32; const K: T = 7;`
 * `${tmeta(items_quoted) as items}` for `Struct`: `type T = i8;`
 * `${tmeta(items_quoted) as token_stream}` for `Struct`: `type T = i8;`
 * `${fmeta(nested)}`: rejected, ``expected a leaf node, found a list with sub-attributes``

#### Examples of `as attrs` value handing

<!--##examples-ignore##-->
In `Enum::NamedVariant`:

 * `${fmeta(same_attrs) as attrs}` for `field`: `#[some] #[other(y)]`
 * `${fmeta(same_attrs) as attrs}` for `field_b`: `#[some] #[other(y)]`
 * `${fmeta(same_attrs) as attrs}` for `field_e`: `#[some] #[other(y)]`
 * `${fmeta(exciting_attrs) as attrs}`: `#[with = funny::<A,B>::syntax]`

#### Examples involving pasting

 * `$<Small ${tmeta(simple)}>`: `SmallString`
 * `$<Small ${tmeta(simple) as str}>`: `SmallString`
 * `$<Small ${tmeta(simple) as ty}>`: `« SmallString »`
 * `$<Small ${tmeta(gentype) as ty}>`: `« SmallVec::<i32> »`
 * `$<$ttype ${tmeta(simple) as str}>`: `UnitString::<C>`
 * `$<$ttype ${tmeta(simple) as ty}>`: error, ``multiple nontrivial entries``

<div id="x:fattrs">
<div id="x:tattrs">
<div id="x:vattrs">

### `${fattrs ...}` `${vattrs ...}` `${tattrs ...}` -- other attributes

</div>
</div>
</div>

Expands to attributes, including non-`#[deftly()]` ones.
The attributes can be filtered:

  * **`$Xattrs`**: All the attributes
    except `#[deftly]` and `#[derive_deftly]`
  * **`${Xattrs A1, A2, ...}`**, or
    **`${Xattrs = A, A2, ...}`**:
    Attributes `#[A1...]` and `#[A2...]` only.
  * **`${Xattrs ! A1, A2, ...}`**:
    All attributes *except* those.

With `${Xattrs}`, unlike `${Xmeta}`,

   * The expansion is the whole of each attribute, including the `#[...]`;
   * All attributes are included.
   * But `#[deftly(...)]` `#[derive_deftly(...)]`
     and `#[derive_deftly_adhoc(...)]`
     are *excluded* by default,
     because typically they would be rejected by the compiler:
     the expanded output is (perhaps) no longer within `#[derive(Deftly)]`,
     so those attributes might be unrecognised there.
   * The attributes can be filtered by toplevel attribute name,
     but not deeply manipulated.
   * `$vattrs` does not, for a non-enum, include the top-level attributes .

Note that derive macros,
only see attributes
that come *after* the `#[derive(...)]` that invoked them.
So derive-deftly templates only see attributes
that come *after* the `#[derive(..., Deftly, ...)]`.

#### Examples

##### For `Unit`

<!--##examples-for `Unit`##-->

 * `${tattrs}`: ``#[derive(Clone)]``
 * `${tattrs ! deftly}`: ``#[derive(Clone)]``
 * `${tattrs missing}`: nothing
 * `${tattrs derive}`: ``#[derive(Clone)]``
 * `${vattrs deftly}`: nothing

##### For `Tuple`

<!--##examples-for `Tuple`##-->

 * `${tattrs}`: ``#[doc=" Title for `Tuple`"] #[repr(C)]``
 * `${tattrs repr}`: ``#[repr(C)]``
 * `${tattrs repr, deftly}`: ``#[deftly(unused)] #[repr(C)]``
 * `${tattrs ! derive, doc}`: ``#[deftly(unused)] #[repr(C)] #[derive_deftly(SomeOtherTemplate)]``

##### For `Enum`

 * `${vattrs deftly}` for `UnitVariant`: `#[deftly(value="enum_variant")]`

<div id="x:findex">
<div id="x:vindex">

### `$findex` `$vindex` -- field/variant numerical index

</div>
</div>

The numerical index of the field or variant, starting at 0.

Can be used as a number, or a tuple field name.

Counts all fields/variants, including ones filtered out by `${when }`,
unlike [`$loop_index`](#x:loop_index).

#### Examples

 * `$findex`: `0`, `1`, ..
 * `$vindex` for `Enum`: `0`, `1`, ..
 * `$vindex` for `Struct`: `0`

<div id="x:paste">
<div id="x:paste_spanned">

### `${paste ...}`, `$<...>`, `${paste_spanned}` -- identifier pasting

</div>
</div>

Expands the contents and pastes it together into a single identifier.
The contents may only contain identifer fragments, strings (`"..."`),
and (certain) expansions.
Supported expansions are `$ftype`, `$ttype`, `$tdeftype`, `$Xname`,
`${Xmeta as str / ty / path / ident}`,
`$<...>`,
`${paste ...}`,
`${CASE_CHANGE ...}`,
`$tdefkwd`,
as well as conditionals and repetitions.

The contents can contain at most one occurrence of
a more complex type expansion `${Xtype}`
(or `${Xmeta as ty)`),
which must refer to a path (perhaps with generics, and/or surrounding `( )`).
Then the pasting will be applied to the final path element identifier,
and the surroundings reproduced unaltered.

Iff necessary, the result will be a raw identifier.

The span (for hygiene and error reporting) is that of the `${paste }`
invocation in the template.

`${paste_spanned SPAN CONTENT}`
allows control of the identifier "span",
which is used by Rust to control hygiene and error reporting.
`CONTENT` is pasted together, and then the span from `SPAN` is applied.
Both are positional arguments.
`$fname`, `$ftype` and `$vname` are good options for `SPAN`.
Explicitly setting the span can have surprising results;
some testing (even, experimentation) may be needed.
Meta attributes used in `SPAN` do not count as having been used,
for the purposes of unused attribute checking;
if necessary, use `${ignore }`.
Available only in [**beta**].

#### Examples

 * `$<Zingy $ftype Builder>` for `TupleVariant`:
    `« std::iter::ZingyOnceBuilder::<T> »`
 * `${paste x_ $fname}` for tuple: `x_0`
 * `${paste_spanned $vname { x_ $fname }}` for tuple: `x_0`,
    (with the span of `$vname`)
 * `${paste $fname _x}` for tuple: error, ``constructed identifier "0_x" is invalid``

<div id="x:concat">

### `${concat ...}` - string literal concatenation (**beta**)

</div>

Concatenates the content and expands to a string literal.
Only certain contents are allowed:

 * Strings literals (`"..."`):
   the contents of the string is used.

 * Expansions that expand to identifiers:
   the text of the identifier is used.
   For raw identifiers, only the identifier name is used.

 * Expansions that expand to types:
   the type's source code text is used (as if with `stringify!`).

   **The precise representation is neither defined nor stable!**
   Whitespace, `::` and even `« »` may be added or removed!
   The result can be used in documentation or messages
   but should not reinterpreted as Rust code,
   nor compared for equality.

 * Expansions that expand to literal strings:
   `${Xmeta as str}`,
   and the non-identifier case conversions: `${kebab_case ...}` etc.

So, supported expansions are
those allowed in [`${paste ...}`](#x:paste),
all case conversions,
and `${concat }` itself.

`${concat }` does the jobs of `std::concat!` and `std::stringify!`
but can be used in more places and is more convenient.

This feature is available only in [**beta**].

#### Examples

 * `${concat "first" "second"}`: `"firstsecond"`
 * `${concat $tname "Suffix"}`: `"TupleSuffix"`
 * `${concat $ttype "Suffix"}`: `"Tuple::<'a, 'l, T, C>Suffix"`
 * `${concat $<$ttype Suffix>}`: `"TupleSuffix::<'a, 'l, T, C>"`
 * `${concat "Prefix" $ftype}`: `"Prefix<T as TryInto<u8>>::Error"`
 * `${concat $<Prefix $ftype>}`: `"<T as TryInto::<u8>>::PrefixError"`
 * `${concat ${snake_case $vname}}`: `"named_variant"`
 * `${concat $<r#raw_ident>}`: `"raw_ident"`

<div id="string-template">

### `$"..."`, `$///...` - string and documentation templating (**beta**)

</div>

`$"..."` expands to a new string literal, processing
`$`-expansions within the input string.

`$///...` is treated by Rust as `$#[doc = "..."]`
which derive-deftly expands to a `#[doc = "..."]` attribute,
processing `$`-expansions within the input doc string.

This is a convenience syntax for `${concat }`.
The leading `$` (or `doc =`) is followed by a (possbily raw) string literal.
The input string literal may contain:

 * Literal text.
 * Expansions in the forms
   `$keyword`, `${keyword ...}`, and `$<...>`.
   The inner `...` are always lexed as Rust tokens,
   and handled the same way as inside `${concat }`.
 * Doubled dollars `$$` to specify a single dollar `$`.

However, `"`, `'` and ``\`` are completely forbidden within any `$` expansion
even within brackets, and there is no quoting facility.
Control structures like `$( )`, `${for }` and `${if }` are also forbidden
(i.e. re-entering the `$"..."` lexical/parsing context
from within an expansion in the `$"..."` is not supported).

Instead:
put the `$"..."` within a `${concat }`,
end the `$"..."` before the forbidden part,
and restart it afterwards;
or
put the forbidden template elements
into a user-defined expansion whose body is precisely `${concat }`,
and reference that from within the `$"..."` or `$///`.

Available only in [**beta**].

#### Examples

 * `$"type is $ttype."`: `"type is Unit::<C>."`
 * `$"got $<Alt $ttype> from pasting."`: `"got AltUnit::<C> from pasting."`
 * `$"meta is ${tmeta(simple) as str}."`: `"meta is String."`
 * ``$/// doc for `$ttype`.``: ``#[doc = " doc for `Unit::<C>`."]``

#### Examples, working with forbidden content

 * `$"for $tname, ${if tmeta(simple) { y }}?"`: error, ``control structures are not allowed in string templates``
 * `$r#"for $tname, ${concat ${if tmeta(simple) { "y" }}}?"#`: error, ``nested strings, characters/lifetimes, and backslashes are not allowed``
 * `${concat $"for $tname, " ${if tmeta(simple) { "y" }} $"."}`: `"for Unit, y."`
 * `${define Y ${concat ${if tmeta(simple) { "y" }}}} $"for $tname, $Y."`: `"for Unit, y."`

### `${CASE_CHANGE ...}` -- case changing

Expands the content, and changes its case
(eg. uppercase to lowercase, etc.
See [Case changing](#case-changing).
`CASE_CHANGE` is one of the values listed there.

<div id="x:when">

### `${when CONDITION}` -- filtering out repetitions by a predicate

</div>

Allowed only within repetitions, and only at the toplevel
of the repetition,
before other content.
Skips this repetition if the `CONDITION` is not true.

#### Example

 * `$( ${when vmeta(value)} ${vmeta(value) as str} )` for `Enum`: `"enum_variant"`

<div id="x:if">

### `${if COND1 { ... } else if COND2 { ... } else { ... }}` -- conditional

</div>

Conditionals.  The else clause is, of course, optional.
The `else if` between arms is also optional,
but `else` in the fallback clause is mandatory.
So you can write `${if COND1 { ... } COND2 { ... } else { ... }`.

#### Examples

 * `${if is_enum { E } is_struct { S }}` for `Enum`: `E`
 * `${if is_enum { E } is_struct { S }}` for others: `S`
 * `$( ${if v_is_named { N } v_is_tuple { T }} )` for `Enum`: `T N`
 * `$( ${if v_is_named { N } v_is_tuple { T } else { X }} )` for `Enum`: `X T N`
 * `${if v_is_unit { U } tmeta(gentype) { GT }}` for `Unit`: `U`

<div id="x:select1">

### `${select1 COND1 { ... } else if COND2 { ... } else { ... }}` -- expect precisely one predicate

</div>

Conditionals which insist on exactly one of the tests being true.
Syntax is identical to that of `${if }`.
*All* of the `COND` are always evaluated.
Exactly one of them must be true;
or, none of them, but only if an `else` is supplied -
otherwise it is an error.

#### Examples

 * `${select1 is_enum { E } is_struct { S }}`: `E`, `S`
 * `${select1 v_is_named { N } v_is_tuple { T }}` for `Enum`: rejected, ``no conditions matched, and no else clause``
 * `$( ${select1 v_is_named { N } v_is_tuple { T } else { X }} )` for `Enum`: `X T N`
 * `${select1 v_is_unit { U } tmeta(gentype) { GT }}` for `Unit`: rejected, ``multiple conditions matched``

<div id="x:for">

### `${for fields { ... }}`, `${for variants { ... }}`, `$( )` -- repetition

</div>

`${for ...}` expands the contents once per field, or once per variant.

`$( ... )` expands the input with an appropriate number of iterations -
see [Repetition and nesting](#repetition-and-nesting).

<div id="x:crate">

### `$crate` -- root of template crate

</div>

`$crate` always refers to the root of the crate 
defining the template.
Within an `export`ed template,
being expanded in another crate,
it refers to the crate containing the template definition.
In templates being used locally,
it refers to the current crate, ie simply `crate`.

This is similar to the `$crate` builtin expansion
in `macro_rules!`.

<div id="x:tdefkwd">

### `$tdefkwd` -- keyword introducing the new data structure

</div>

Expands to `struct`, `enum`, or `union`.

<div id="x:fdefine">
<div id="x:tdefvariants">
<div id="x:vdefbody">

### `$tdefvariants`, `$vdefbody`, `$fdefine` -- tools for defining types

</div>
</div>
</div>

These, used together, allow the template to expand to a
new definition, mirroring the driver type in form.

**`${tdefvariants VARIANTS..}`** expands to `{ VARIANTS.. }` for an enum,
or just `VARIANTS..` otherwise.
Usually, it would contain a `$( )` repeating over the variants,
expanding `$vdefbody` for each one.

**`${vdefbody VNAME FIELDS..}`** expands to the definition of a variant,
with a appropriate delimiters.
`VNAME` is in the standard syntax for a positional argument,
and `FIELDS..` is the rest of the content.
Usually, `FIELDS..` would contain a `$( )` repeating over the fields,
using `$fdefine` to introduce each one.
Specifically:

<!--##examples-possibilities-blockquote##-->
<!--iiiiiiiiiiiiiiiiiiiiiii fffffffffffffffffffff oooooooooooooooooo ###############-->

```rust,dd-directly
# let _ = r##"
 ${vdefbody VNAME FIELDS}   for unit                FIELDS;          [*] ie  ;
 ${vdefbody VNAME FIELDS}   for tuple             ( FIELDS );
 ${vdefbody VNAME FIELDS}   for braced struct     { FIELDS }
 ${vdefbody VNAME FIELDS}   for unit variant      VNAME   FIELDS,    [*] ie  VNAME,
 ${vdefbody VNAME FIELDS}   for tuple variant     VNAME ( FIELDS ),
 ${vdefbody VNAME FIELDS}   for braced variant    VNAME { FIELDS },
# "##;
```

**`${fdefine FNAME}`** expands to `FNAME:` in the context of
named fields (a "struct" or "struct variant"),
or nothing otherwise.
`FNAME` is in the standard syntax for a positional argument,

`[*]`: In the unit and unit variant cases,
`FIELDS` ought to expand to nothing;
otherwise, the expansion of `$vdefbody`
will probably be syntactically invalid in context.

#### Example

```rust,dd-directly
# let _ = r##"
$tvis $tdefkwd $<$tname Copy><$tdefgens>
${tdefvariants $(
    ${vdefbody $<$vname Copy> $(
        $fdefvis ${fdefine $<$fname _copy>} $ftype,
    ) }
) }
# "##;
```

<!--##examples-for-toplevels-concat Tuple Enum##-->
Expands to (when applied to `Tuple` and `Enum`):

```rust,dd-directly
# let _ = r##"
struct TupleCopy<'a, 'l: 'a, T: Display = usize, const C: usize = 1,>(
    &'a &'l T,
);
pub enum EnumCopy<'a, 'l: 'a, T: Display = usize, const C: usize = 1,> {
    UnitVariantCopy,
    TupleVariantCopy(std::iter::Once::<T>,),
    NamedVariantCopy { field_copy: &'l &'a T, ... },
}
# "##;
```

<div id="x:for_loop_count">
<div id="x:loop_exactly_1">
<div id="x:loop_index">

### `$loop_index`, `$for_loop_count`, `${loop_exactly_1 ...}` -- repetition information (**beta**)

</div>
</div>
</div>

 * **`$loop_index`** is the index of this repetition, starting at zero.
 * **`${for_loop_count fields COND}`** and
   **`${for_loop_count variants COND}`**
   count how many fields, or variants, would match condition `COND`.

`loop_index` counts only repetitions
selected by relevant [`${when }`](#x:when) clauses --
unlike [`$vindex`](#x:vindex) and [`$findex`](#x:findex).
See above for
[full information](#t:repetition).

Can be used as a number, or a tuple field name.

 * **`${loop_exactly_1 ..}`** throws a compile error
   unless this loop has exactly one repetition.

   `${loop_exactly_1 }` is checked if (and only if)
   the surrounding loop is expanded;
   so it is a supplement to the loop itself,
   not content that is expanded for each iteration.

   `${loop_exactly_1 }` must appear with `${when }`, before other content.
   The relative ordering of `${loop_exactly_1 }` and `${when }` has no effect.

   The syntax is exactly as for [`${error }`](#x:error).

All these features are available only in [**beta**].

#### Examples

 * `$( ${when not(fmeta(same_attrs))} $loop_index = $fname; )` for `Enum`: `0 = 0; 1 = field_o;`
 * `${for_loop_count fields not(fmeta(same_attrs))}` for `Enum`: `2`
 * `$( ${loop_exactly_1 "too few"} $fname )` for `Unit`: error, ``too few (in this data structure)``
 * `$( ${loop_exactly_1 "too many"} $fname )` for `Struct`: error, ``too many (first: in this field)``
 * `$( ${loop_exactly_1 "just right"} $fname )` for `Tuple`: `0`

<div id="x:v_explicit_discriminant">

### `${v_explicit_discriminant ..}` -- Explicitly specified enum discriminant

</div>

**`$v_explicit_discriminant`**
expands to the enum discriminant explicitly explicit with
the Rust `Variant = DISCRIMINANT` syntax.

**Minority feature**: avoid using this feature unless you have to.
If not every variant has an explicit discriminant,
it can't reliably give you the actual discriminant value.
Usually it is better to use `Enum::Variant as _`, or perhaps
`strum::EnumDiscriminants`.

If no discriminant was explicitly specified, it is an error.

The expression is enclosed in `( )` to avoid precedence problems.

#### Examples

 * `$v_explicit_discriminant` for `UnitVariant`: `(2)`
 * `$v_explicit_discriminant` for `TupleVariant`: error, ``enum variant does not have explicit discriminant``
 * `$v_explicit_discriminant` for `Struct`: error, ``not an enum, so no explicit discriminant``

<div id="x:ignore">

### `${ignore ..}` -- Expand but then discard

</div>

`${ignore CONTENT}` expands `CONTENT`, and then discards it.
The `${ignore ..}` therefore expands to nothing.

All side-effects of `CONTENT` *do* occur.  So:
if expanding `CONTENT` causes an error,
`${ignore }` *does* report that error;
the content of `${ignore }` *can* affect
the repetition scope of its surroundings.
`${ignore }` is permitted in `${paste }` and case changing.

<div id="x:dbg">
<div id="x:dbg_all_keywords">

### `${dbg ..}`, `$dbg_all_keywords` -- Debugging output

</div>
</div>

`${dbg { CONTENT }}` expands to the expansion of `CONTENT`,
but it also prints the expansion to the compiler stderr.

`${dbg "NOTE" { CONTENT }}` adds the note `"NOTE"`
to the heading of the expansion dump,
for identification purposes.

`$dbg_all_keywords` dumps expansions of all keywords:

It prints a listing of all the available expansion keywords,
and conditions,
along with their expansions and values.
When invoked at the toplevel,
it prints a report for each variant and field.
(The output goes to the compiler's stderr;
the actual expansion is empty.)

This can be helpful to see which expansion keywords
might be useful for a particular task.
(Before making a final selection of keyword
you probably want to refer to this reference manual.)

You will not want to leave these options in production code,
as they make builds noisy.

See also
the [`dbg` expansion option](#dbg--print-the-expansion-to-stderr-for-debugging),
and
the [`dbg` condition](#dbg--debug-dump-of-condition-value).

#### Example

```rust
# use derive_deftly::{Deftly, derive_deftly_adhoc};
#[derive(Deftly)]
#[derive_deftly_adhoc]
enum Enum {
    Unit,
    Tuple(usize),
    Struct { field: String },
}
derive_deftly_adhoc! {
    Enum:
    $dbg_all_keywords
    // ... rest of the template you're developing ...
}
```

<div id="x:defcond">
<div id="x:define">

### `${define ...}`, `${defcond ...}` -- user-defined expansions and conditions

</div>
</div>

`${define NAME BODY}` defines a reuseable piece of template.
Afterwards, `$NAME` (and `${NAME}`) expand `BODY`.

`${defcond NAME CONDITION}` defines a reuseable condition.
Afterwards, the name `NAME` can be used as a condition -
evaluating `CONDITION`.

`NAME` is an identifier.
It may not start with a lowercase letter or underscore:
those expansion names are reserved for
derive-deftly's built-in functionality.

`BODY` is in the
[standard syntax for positional arguments](#named-and-positional-template-arguments-to-expansions-and-conditions).
When generating Rust code, be careful about operator precedence:
see [Precedence considerations](#precedence-considerations).

`CONDITION` is in the standard syntax for a condition.

`NAME` is visible after its definition in the same template or group,
including in inner templates and groups.
Definitions may be re-defined, in the same scope, or inner scopes.
Scope is dynamic,
both for derive-deftly built-ins and user definitions:
`BODY` and `CONDITION` are captured
without expansion/evaluation at the site of `$define`/`$defcond`,
and the contents expanded/evaluated
each time according
to the values and definitions prevailing
in the dynamic context where `NAME` is used.

(Therefore, you can `$define`/`$defcond` an identifier
at a point where its contents are not in scope,
and expand it later when they are.)

`${NAME}` may only be used
inside pasting and case changing
if `BODY` was precisely an invocation of `${paste }`/`$<...>`.

`${NAME}` may only be used
inside `${concat ...}`/`$"..."`
if `BODY` was precisely an invocation of `${concat }`/`$"..."`,
`${paste }`/`$<...>`,
or a string literal.

You can define an expansion and a condition with the same name;
they won't interfere.

#### Examples

 * `${define VN $vname} ${for variants { $VN }}`:
   `UnitVariant TupleVariant NamedVariant`
 * `${define FN $<$fname _>} $<${for fields { "F" $FN }}>`:
   `F0_`, `Ffield_Ffield_b_`

##### Example including a condition

```rust,dd-directly
# let _ = r##"
${define T_FIELDS ${paste $tname Fields}}
// Note that fvis is not in scope here; that's okay,
// but we can only _use_ F_ENABLE when fvis _is_ in scope.
${defcond F_ENABLE all(fvis, v_is_named)}
$tvis struct $T_FIELDS { $(
    ${when F_ENABLE} $fvis $fname: bool,
) }
$tvis const ${shouty_snake_case ALL_ $T_FIELDS}: $T_FIELDS = { $(
    ${when F_ENABLE} $fname: true,
) };
# "##;
```

<!--##examples-for-toplevels-concat Unit Tuple Struct##-->
Expands to (for `Unit`, `Tuple` and `Struct`):

```rust,dd-directly
# let _ = r##"
pub struct UnitFields {}
pub const ALL_UNIT_FIELDS: UnitFields = {};
struct TupleFields {}
const ALL_TUPLE_FIELDS: TupleFields = {};
struct StructFields {
    pub field: bool,
}
const ALL_STRUCT_FIELDS: StructFields = {
    field: true,
};
# "##;
```

<div id="x:error">

### `${error ...}` -- explicitly throw a compile error

</div>

 * `${error "message"}`: equivalent to `${error message="message"}`.
 * `${error message=MESSAGE [span=SPAN]}`,
   ([**beta**]),
   using the usual
[syntax for named arguments](#named-and-positional-template-arguments-to-expansions-and-conditions).

Generates a compilation error, if expanded.

This can be used anywhere a derive-deftly expansion is allowed;
(unlike std's `compile_error!`,
which, like any Rust macro, is permitted only in certain syntactic contexts).

`MESSAGE` must be valid within [`${concat }`](#x:concat).

#### Spans (locations at which the error is reported)

derive-deftly will also always produce a copy of the error
pointing at the specific driver, variant, and field,
in the context of which this `${error }` was expanded,
so use of `span=SPAN` is not normally necessary.

It will also produce a copy of the error pointing at the error string,
in the actual template, if it can.
(When the template is imported from another crate, this isn't possible.)

If `SPAN` is specified, it indicates the principal location of the error,
which will be reported first.

To report an error which was caused by the interaction of multiple inputs,
write `${error }` several times with appropriate `SPAN`s.
Good practice is to use the same message for each location,
and distinguish them by appending notes in the form `" (LOCATION)"`.

#### Examples

 * `${error "simple error"}`: error, ``simple error``
 * `${error message="as arg"}`: error, ``as arg``
 * `${error message="with span" span=$ftype}`: error, ``with span``
 * `${error message=${concat "bad " ${tmeta(simple) as str}}}` for `Unit`: error, ``bad String``

## Conditions

Conditions all start with a `KEYWORD`.
They are found within `${if }`, `${when }`, and `${select1 }`.

<!-- ## maint/check-keywords-documented conditions ## -->

<div id="c:fdefvis">
<div id="c:fvis">
<div id="c:tvis">

### `fvis`, `tvis`, `fdefvis` -- test for public visibility

</div>
</div>
</div>

True iff the field, or the whole toplevel type, is `pub`.

See
[`$fvis`, `$tvis` and `$fdefvis`](#fvis-tvis-fdefvis--visibility)
for details of the semantics (especially for enums),
and the difference between `$fvis` and `$fdefvis`.

Within-crate visibility, e.g. `pub(crate)`, is treated as "not visible"
for the purposes of `fvis` and `tvis`
(although the `$fvis` and `$tvis` expansions will handle those faithfully).

#### Examples

 * `tvis`: true for `Unit`, and `Enum`
 * `fvis`: true for `field` in `Struct`, and fields in `Enum`
 * `fdefvis`: true for `field` in `Struct`

<!--##examples-ignore##-->
And in each case, false for all others.
(Refer to the [example structs](#structs-used-in-examples), below.)

<div id="c:fmeta">
<div id="c:tmeta">
<div id="c:vmeta">

### `fmeta(NAME)`, `vmeta(NAME)`, `tmeta(NAME)` -- `#[deftly]` attributes

</div>
</div>
</div>

Looks for `#[deftly(NAME)]`.

True iff there was such an attribute.

The condition is true if there is at least one matching entry,
and (unlike `${Xmeta}`)
the corresponding driver attribute does not need to be a `LIT`.

So `Xmeta(SUB(NAME))` is true if the driver has
`#[deftly(SUB(NAME(INNER=...)))]` or `#[deftly(SUB(NAME))]` or
`#[deftly(SUB(NAME=...))]` or even `#[deftly(SUB(NAME()))]`.

`Xmeta(SUB(NAME))` works, just as with the `${Xmeta ...}` expansion.
See [`${Xmeta ...}`](#tmeta-vmeta-fmeta--deftly-attributes)
for information about namespacing and handling of unused attributes.

#### Examples

 * `tmeta(unused)`: true for `Tuple`
 * `tmeta(gentype)`: true for `Unit`
 * `vmeta(value)`: true for `Unit`, and `Enum::UnitVariant`
 * `fmeta(nested)`: true for `field` in `Struct`

<div id="c:f_is_first">
<div id="c:v_is_first">

### `v_is_first`, `f_is_first` -- is this the first variant/field? (**beta**)

</div>
</div>

 * `v_is_frist` is true for the first variant.
 * `f_is_frist` is true for the first field.

Unlike [`loop_initial`](#c:loop_initial),
these are not influenced by, and can be used within,
[`${when }`](#x:when).

Available only in [**beta**].

#### Examples

 * `$( ${when f_is_first} $fname )`: `0`, `field`
 * `$( ${when v_is_first} $vtype )`: `Unit::<C>`, `Tuple::<'a, 'l, T, C>`, `Struct::<'a, 'l, T, C>`, `Enum::UnitVariant::<'a, 'l, T, C>`

<div id="c:is_enum">
<div id="c:is_struct">
<div id="c:is_union">

### `is_struct`, `is_enum`, `is_union`

</div>
</div>
</div>

The driver data structure is a struct, enum, or union, respectively.

Prefer to avoid these explicit tests,
when writing a template to work with either structs or enums.
Instead,
use `match` and `$vpat` for deconstructing values,
and `$vtype` for constructing them.
Use `$tdefvariants` when defining a derived type.

<div id="c:v_is_named">
<div id="c:v_is_tuple">
<div id="c:v_is_unit">

### `v_is_unit`, `v_is_tuple`, `v_is_named`

</div>
</div>
</div>

Whether and what kind of fields there are.

Prefer to avoid these explicit tests,
when writing a template to work with any shape of structure.
Instead,
match using Rust's universal `Typename { }` syntax,
possibly via `$vpat` and `$fpatname`,
or via `$vtype`.
The `Typename { }` syntax can be used for matching and constructing
all kinds of structures, including units and tuples.
Use `$vdefbody` and `$fdefine` when defining a derived type.

#### Examples

 * `v_is_unit`: true for `struct Unit;`, `SimpleUnit`, and `Enum::UnitVariant;`
 * `v_is_tuple`: true for `struct Tuple(...);`, and `Enum::TupleVariant(...);`
 * `v_is_named`: true for `struct Struct {...}`, and `Enum::NamedVariant {...}`

<div id="c:tgens">

### `tgens`

</div>

Whether the top-level type has generics.

#### Examples

 * `tgens`: true for `Unit`, `Tuple`, `Struct`, `Enum`

<div id="c:loop_final">
<div id="c:loop_initial">

### `loop_initial`, `loop_final` -- repetition information (**beta**)

</div>
</div>

 * **`loop_initial`** is true for first iteration of this repetition.
 * **`loop_final`** is true for the final iteration of this repetition.

These count only repetitions
selected by relevant [`${when }`](#x:when) clauses --
unlike
[`v_is_first`](#c:v_is_first)
and
[`f_is_first`](#c:f_is_first).
See above for
[full information about the semantics](#t:repetition-info).

Available only in [**beta**].

#### Examples

 * `$( $fname ${if not(loop_final) {, }} )` for `Struct`: `field, field_b`
 * `${concat $( ${when not(v_is_unit)} $vname ${if not(loop_final) {", "}} )}`: `"TupleVariant, NamedVariant"`
 * `$( ${when fmeta(same_attrs)} ${if loop_initial {let }} $fpatname ${if loop_final {;} else {,}} )` for `Unit`: `/*empty*/`
 * `$( ${when fmeta(same_attrs)} ${if loop_initial {let }} $fpatname ${if loop_final {;} else {,}} )` for `Enum`: `let f_field, f_field_b, f_field_e;`

<div id="c:v_explicit_discriminant">

### `v_explicit_discriminant` -- enum discriminant explicitly specified?

</div>

Does this variant have an explicitly specified discriminant, in
the Rust `Variant = DISCRIMINANT` syntax?

False for structs and unions.

#### Examples

 * `v_explicit_discriminant`: true for `UnitVariant`

<div id="c:approx_equal">
<div id="c:is_empty">

### `is_empty(..)`, `approx_equal(ARG1, ARG2)` -- equality comparison (token comparison)

</div>
</div>

`is_empty` expands the content, and is true if
the expansion produced no tokens.

`approx_equal` expands the two `ARGS`s (as series of tokens)
and compares them for (a kind of) equality.

Span is disregarded, so
two identifiers that would refer to different types or values,
but which have the same name,
would count as equal.

Spacing is disregarded, even between punctuation characters.
For example, `approx_equal` regards `<<` as equal to `< <`.
This means expansions might count as equal
even if the Rust compiler would accept one and reject the other;
and, expansions might count as equal
even if macros could tell the difference.

Also,
`None`-delimited groups,
which are used by macros (including derive-deftly and `macro_rules!`)
for preventing precedence surprises,
are flattened - the wrapping by an invisible group is ignored.
This means that two expressions with different values,
due to different evaluation orders,
can compare equal!

If both inputs are valid Rust types,
they will only compare equal if they are syntactically the same.
(Note that different ways of writing the same type
are treated as different:
for example, `Vec<u8>` is not equal to `Vec<u8, Global>`
and `std::os::raw::c_char` is not equal to `std::ffi::c_char`.)

Literals are generally compared by value:
 * Integer literals are compared by value, ignoring any type suffixes.
   (Both values `>u64` is unsupported.)
 * String, byte and character literals are compared by value.
   `c"..."` literals are unsupported.
   (Suffixes are unsupported.).
 * Floating point literals are compared *textually*, not by value;
   so are considered equal only if written identically.
 * Negative literals are compared as two tokens,
   `-` and a nonnegative literal.
 * Comparison of other literals is unsupported.

Raw identifiers are considered unequal to non-raw identifiers,
even if the designated identifier is the same.

<!--
  The alternative is either to say that `r#for` == `for`,
  or to somehow make equality depend on whether *this* version of Rust
  considers the thing a keyword.
  "*This* version of Rust" ought to be the edition, but whose edition?
  Also I don't think we can calculate that.
-->

The `ARG`s are in derive-deftly's usual
[syntax for positional arguments](#named-and-positional-template-arguments-to-expansions-and-conditions).

<div id="c:all">
<div id="c:any">
<div id="c:false">
<div id="c:not">
<div id="c:true">

### `false`, `true`, `not(CONDITION)`, `any(COND1,COND2,...)`, `all(COND1,COND2,...)` -- boolean logic

</div>
</div>
</div>
</div>
</div>

`any()` and `all()` short circuit:
as soon as they have established they answer,
they don't test the remaining conditions.
(This affects error handling,
and meta attribute use checking.)

<div id="c:dbg">

### `dbg(...)` -- Debug dump of condition value

</div>

`dbg(CONDITION)` evaluates `CONDITION`,
but it also prints the boolean value to the compiler stderr.

`dbg("NOTE", CONDITION}` adds the note `"NOTE"`
to the debug message
for identification purposes.

You will not want to leave this option in production code,
as it makes builds noisy.

See also
the [`${dbg ..}` expansion](#dbg--dbg_all_keywords--debugging-output)
and
the [`dbg` expansion option](#dbg--print-the-expansion-to-stderr-for-debugging).

<div id="x:kebab_case">
<div id="x:lower_camel_case">
<div id="x:pascal_case">
<div id="x:shouty_kebab_case">
<div id="x:shouty_snake_case">
<div id="x:snake_case">
<div id="x:title_case">
<div id="x:train_case">
<div id="x:upper_camel_case">
<div id="kebab_case">
<div id="shouty_kebab_case">
<div id="title_case">
<div id="train_case">

## Case changing

</div>
</div>
</div>
</div>
</div>
</div>
</div>
</div>
</div>
</div>
</div>
</div>
</div>

`${CASE_CHANGE ...}`
(where `CASE_CHANGE` is one of the keywords in the table, below)
makes an identifier
with a different case to the input which produces it.
This is useful to make identifiers with the natural spelling
for their kind,
out of identifiers originally for something else.

If the content's expansion is a path, only the final segment is changed.

The content must be valid within `${paste }`,
and is treated the same way.
`${CASE_CHANGE }` may appear within pasting and vice versa.

`${kebab_case ..}`, `${shouty_kebab_case`},
`${title_case }`, and `${train_case }`
don't generate valid Rust identifiers and
are only allowed within [`${concat }`](#x:concat).
(Therefore they are [**beta**] features.)

This table shows the supported case styles.
Note that changing the case can add and remove underscores.
The precise details are as for [`heck`],
which is used to implement the actual case changing.

<!-- ## maint/check-keywords-documented cases ## -->

| `CASE_CHANGE`        | `CASE_CHANGE` aliases            | Name in [`heck`]  | Example of results    | Allowed in  |
|----------------------|----------------------------------|-----------------------------------|-----------------------|---------------|
| `pascal_case`        | `upper_camel_case`               | `UpperCamelCase`                  | `PascalCase`          | anywhere      |
| `snake_case`         |                                  | `SnakeCase`                       | `snake_case`          | anywhere      |
| `shouty_snake_case`  |                                  | `ShoutySnakeCase`                 | `SHOUTY_SNAKE_CASE`   | anywhere      |
| `lower_camel_case`   |                                  | `LowerCamelCase`                  | `lowerCamelCase`      | anywhere      |
| `kebab_case`         |                                  | `KebabCase`                       | `kebab-case`          | `${concat }`  |
| `shouty_kebab_case`  |                                  | `ShoutyKebabCase`                 | `shouty-kebab-case`   | `${concat }`  |
| `title_case`         |                                  | `TitleCase`                       | `Title Case`          | `${concat }`  |
| `train_case`         |                                  | `TrainCase`                       | `Train-Case`          | `${concat }`  |


#### Examples

 * `${shouty_snake_case $ttype}`: `ENUM::<'a, 'l, T, C>`
 * `${pascal_case $fname}`: `Field`, `FieldB`
 * `${pascal_case x_ $fname _y}`: `XFieldBY`
 * `$<x_ ${lower_camel_case $fname} _y>`: `x_fieldB_y`
 * `${lower_camel_case $fname}` for tuple: error, ``constructed identifier "0" is invalid``
 * `${concat ${kebab_case $fname}}`: `"field-b"`
 * `${concat ${shouty_kebab_case $fname}}`: `"FIELD-B"`
 * `${concat ${title_case $fname}}`: `"Field B"`
 * `${concat ${train_case $fname}}`: `"Field-B"`

## Expansion options

You can pass options,
which will be applied to each relevant template expansion:

```rust,ignore
// Expand TEMPLATE for DataStructureType, with OPTIONS
derive_deftly_adhoc! { DataStructureType OPTIONS,... : TEMPLATE }

// Define a template Template which always expands with OPTIONS
define_derive_deftly! { Template OPTIONS,...: TEMPLATE }

// Expand Template for DataStructureType, with OPTIONS
#[derive(Deftly)]
#[derive_deftly(Template[OPTIONS,...])]
struct DataStructureType {
# }
```

Multiple options, perhaps specified in different places,
may apply to a single expansion.
Even multiple occurrences of the same option are fine,
so long as they don't contradict each other.

The following expansion options are recognised:

<div id="eo:expect">

### `expect items`, `expect expr` -- syntax check the expansion

</div>

Syntax checks the expansion,
checking that it can be parsed as items, or as an expression.

If not, it is an error.
Also, then, an attempt is made to produce
compiler error message(s) pointing to the syntax error
*in a copy of the template expansion*,
as well as reporting the error at
the part of the template or driver which generated
that part of the expansiuon.

This is useful for debugging.

Due to limitations in Rust, the copy of the template expansion
cannot properly represent the results of the
[`$crate` expansion](#x:crate).
When an `expect` fails for an template imported from another crate,
you may also see spurious name resolution errors.
These should be ignored.

Note that a template defined with `define_derive_adhoc!`
must always expand to items, anyway,
because Rust insists that a `#[derive]` expands to items.

<div id="eo:for">

### `for struct`, `for enum`, `for union` -- Insist on a particular driver kind

</div>

Checks the driver data structure kind
against the `for` option value.
If it doesn't match, it is an error.

This is useful to produce good error messages:
Normally, derive-deftly does not explicitly check the driver kind,
and simply makes it available to the template via expansion variables.
But, often,
a template is written only with a particular driver kind in mind,
and otherwise produces syntactically invalid output
leading to confusing compiler errors.

This option is only allowed in a template,
not in a driver's `#[derive_deftly]` attribute.

<div id="eo:dbg">

### `dbg` -- Print the expansion to stderr, for debugging

</div>

Prints the template's expansion to stderr, during compilation,
for debugging purposes.

You will not want to leave this option in production code,
as it makes builds noisy.

See also
the [`${dbg ..}` expansion](#dbg--dbg_all_keywords--debugging-output),
the [`dbg` condition](#dbg--debug-dump-of-condition-value),
the [`$dbg_all_keywords` expansion](#dbg--dbg_all_keywords--debugging-output).

<div id="eo:meta_quoted">

### `meta_quoted` -- control (de)quoting of attribute values (**beta**)

</div>

Changes the default for [`${Xmeta(...) as ...}`](#x:fmeta),
in template, to:

  * **`quoted strip`** (allow unquoted values, strip any quotes), or
  * **`quoted rigorous`** (only allow quoted values if unambiguous), or
  * **`quoted retain`** (allow unquoted values,
    `as expr` does not strip `"..."` and yields the provided literal).

This option is only allowed in a template,
not in a driver's `#[derive_deftly]` attribute.
If affects the whole template including `$Xmeta` expansions
in any imported modules.

Available only in [**beta**].
In the future, `quoted strip` will probably become the default.

<!--
TODO When this feature is stabilised, it will affect only
`as expr`, and we can change its summary to:

> Changes the default for [`${Xmeta(...) as expr}`](#x:fmeta),
> in this module or template, to:

-->

<div id="eo:beta_deftly">

### `beta_deftly` -- Enable unstable template features

</div>

Enables
[beta template features](../doc_changelog/index.html#t:beta).

This option is only allowed in a template or module,
not in a driver's `#[derive_deftly]` attribute.

### Expansion options example

```
# use derive_deftly::{define_derive_deftly, Deftly};
define_derive_deftly! { Nothing for struct, expect items: }

#[derive(Deftly)]
#[derive_deftly(Nothing[expect items, dbg])]
struct Unit;
```

This defines a reuseable template `Nothing`
which can be applied only to structs,
and whose output is syntax checked as item(s).
(The template's actual expansion is empty,
so it does indeed expand to zero items.)

Then it applies that to template to `struct Unit`,
restating the requirement that the expansion should be item(s).
and dumping the expansion to stderr during compilation.

## Precedence considerations

When using
`${Xmeta as token_stream}`,
`${Xmeta ... default ...}`,
and user-defined expansions (`${define ...}`)
it can be necessary to add `{ }` or `( )`
to avoid surprising expansions due to operator precedence.

```
# use derive_deftly::{Deftly, derive_deftly_adhoc};
#[derive(Deftly)]
#[derive_deftly_adhoc]
struct S(u32, u32);

let product = derive_deftly_adhoc!(
    S:
    ${define F_PLUS_TWO {$fname + 2}}
    ${for fields { $F_PLUS_TWO * }} 1
    // (0 + 2) * (1 + 2) * 1 = 2 * 3 * 1 = 6
    // but this is 
    // 0 + 2 * 1 + 2 * 1 = 0 + (2 * 1) + (2 * 1) = 4
);

assert_eq!(product, 4);
```

Rust demands that *types* are expressed unambiguously,
so precedence problems, and lack of `( )` (or `< >`),
are detected by the compiler, and rejected.

<div id="t:none-delimiters">

### `None`-delimited groups

</div>

In theory Rust has a feature that would help with this:
syntactic groups
[can be surrounded by invisible delimiters](https://doc.rust-lang.org/proc_macro/enum.Delimiter.html#variant.None).

However, as of May 2024 this feature does not work (and has never worked).
See [rust-lang/rust#67062](https://github.com/rust-lang/rust/issues/67062).

Nevertheless, derive-deftly surrounds certain expansions with
None-delimited groups.
These are shown in the example outputs, in this reference,
surrounded by guillemets `« »`.

This is done for
 * `$ftype`
 * `$Xmeta as ty`

<div id="t:example-structs">

## Structs used in examples

</div>

The example expansions in the syntax reference 
are those generated for the following driver types:

<!--##examples-structs##-->

```rust,dd-directly
# let _ = r##"
#
# use std::fmt::Display;
# use std::convert::TryInto;
#
#[derive(Deftly)]
#[derive(Clone)]
struct SimpleUnit;

#[derive(Deftly)]
#[derive(Clone)]
#[deftly(simple = "String", gentype = "Vec<i32>")]
#[deftly(value = "unit_toplevel", some_vis = "pub(crate)")]
pub struct Unit<const C: usize = 1>;

#[derive(Deftly, Clone)]
/// Title for `Tuple`
#[deftly(unused)]
#[repr(C)]
#[derive_deftly(SomeOtherTemplate)]
struct Tuple<'a, 'l: 'a, T: Display = usize, const C: usize = 1>(
    &'a &'l T,
);

#[derive(Deftly)]
#[deftly(items_quoted = "type T = i8;")]
struct Struct<'a, 'l: 'a, T: Display = usize, const C: usize = 1>
where T: 'l, T: TryInto<u8>
{
    #[deftly(nested(inner = "42"))]
    pub field: &'l &'a T,
    pub(crate) field_b: String,
}

#[derive(Deftly)]
pub enum Enum<'a, 'l: 'a, T: Display = usize, const C: usize = 1>
where T: 'l, T: TryInto<u8>
{
    #[deftly(value = "enum_variant")]
    UnitVariant = 2,
    #[deftly(items = "type T = i32; const K: T = 7;")]
    TupleVariant(std::iter::Once::<T>),
    NamedVariant { 
        #[deftly(same_attrs = "some", same_attrs = "other(y)")]
        #[deftly(exciting_attrs = "#[with = funny::<A,B>::syntax]")]
        field: &'l &'a T,
        #[deftly(same_attrs = "some, other(y)")]
        field_b: String,
        #[deftly(same_attrs = "#[some] #[other(y)]")]
        field_e: <T as TryInto<u8>>::Error,
        field_o: Option<i32>,
     },
}
# "##;
```

<div id="t:index">

## Keyword index

</div>

<!-- @dd-navbar in-reference .. -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="../index.html">crate top-level</a> | <a href="../index.html#overall-toc">overall toc, macros</a> | <a href="../doc_reference/index.html"><strong>template etc. reference</strong></a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>

### Expansions index

<!--## index x ##-->

 * **$c…**: [`concat`](#x:concat), [`crate`](#x:crate)
 * **$d…**: [`dbg`](#x:dbg), [`dbg_all_keywords`](#x:dbg_all_keywords), [`defcond`](#x:defcond), [`define`](#x:define)
 * **$e…**: [`error`](#x:error)
 * **$f…**: [`fattrs`](#x:fattrs), [`fdefine`](#x:fdefine), [`fdefvis`](#x:fdefvis), [`findex`](#x:findex), [`fmeta`](#x:fmeta), [`fname`](#x:fname), [`for`](#x:for), [`for_loop_count`](#x:for_loop_count), [`fpatname`](#x:fpatname), [`ftype`](#x:ftype), [`fvis`](#x:fvis)
 * **$g…**: [`gens_collate`](#x:gens_collate), [`gens_names`](#x:gens_names), [`gens_nodefs`](#x:gens_nodefs)
 * **$i…**: [`if`](#x:if), [`ignore`](#x:ignore), [`impl`](#x:impl)
 * **$k…**: [`kebab_case`](#x:kebab_case)
 * **$l…**: [`loop_exactly_1`](#x:loop_exactly_1), [`loop_index`](#x:loop_index), [`lower_camel_case`](#x:lower_camel_case)
 * **$p…**: [`pascal_case`](#x:pascal_case), [`paste`](#x:paste), [`paste_spanned`](#x:paste_spanned)
 * **$s…**: [`select1`](#x:select1), [`shouty_kebab_case`](#x:shouty_kebab_case), [`shouty_snake_case`](#x:shouty_snake_case), [`snake_case`](#x:snake_case)
 * **$t…**: [`tattrs`](#x:tattrs), [`tdefgens`](#x:tdefgens), [`tdefkwd`](#x:tdefkwd), [`tdeftype`](#x:tdeftype), [`tdefvariants`](#x:tdefvariants), [`tgens`](#x:tgens), [`tgnames`](#x:tgnames), [`title_case`](#x:title_case), [`tmeta`](#x:tmeta), [`tname`](#x:tname), [`train_case`](#x:train_case), [`ttype`](#x:ttype), [`tvis`](#x:tvis), [`twheres`](#x:twheres)
 * **$u…**: [`upper_camel_case`](#x:upper_camel_case)
 * **$v…**: [`v_explicit_discriminant`](#x:v_explicit_discriminant), [`vattrs`](#x:vattrs), [`vdefbody`](#x:vdefbody), [`vindex`](#x:vindex), [`vmeta`](#x:vmeta), [`vname`](#x:vname), [`vpat`](#x:vpat), [`vtype`](#x:vtype)
 * **$w…**: [`when`](#x:when)

### Conditions index

<!--## index c ##-->

 * **a…**: [`all`](#c:all), [`any`](#c:any), [`approx_equal`](#c:approx_equal)
 * **d…**: [`dbg`](#c:dbg)
 * **f…**: [`f_is_first`](#c:f_is_first), [`false`](#c:false), [`fdefvis`](#c:fdefvis), [`fmeta`](#c:fmeta), [`fvis`](#c:fvis)
 * **i…**: [`is_empty`](#c:is_empty), [`is_enum`](#c:is_enum), [`is_struct`](#c:is_struct), [`is_union`](#c:is_union)
 * **l…**: [`loop_final`](#c:loop_final), [`loop_initial`](#c:loop_initial)
 * **n…**: [`not`](#c:not)
 * **t…**: [`tgens`](#c:tgens), [`tmeta`](#c:tmeta), [`true`](#c:true), [`tvis`](#c:tvis)
 * **v…**: [`v_explicit_discriminant`](#c:v_explicit_discriminant), [`v_is_first`](#c:v_is_first), [`v_is_named`](#c:v_is_named), [`v_is_tuple`](#c:v_is_tuple), [`v_is_unit`](#c:v_is_unit), [`vmeta`](#c:vmeta)

