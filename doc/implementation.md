# Implementation approach - how does this work?

**You do not need to understand this in order to use derive-deftly.**

Also, you should not rely on the details here.
They don't form part of the public interface.

## Introduction

It is widely understood that proc macro invocations
cannot communicate with each other.
(Some people have tried sneaking round the back with disk files etc.
but this can break due to incremental and concurrent compilation.)

But, a proc macro can *define a `macro_rules` macro*.
Then, later proc macros can invoke that macro.
The expansion can even invoke further macros.
In this way, a proc macro invocation *can* communicate with
subsequent proc macro invocations.
(This trick is also used by
 [`ambassador`](https://crates.io/crates/ambassador).)

There is a further complication.
One macro X cannot peer into and dismantle
the expansion of another macro Y.
(This is necessary for soundness, when macros can generate `unsafe`.)
So we must sometimes define macros that apply other macros
(whose name is supplied as an argument)
to what is conceptually the first macro's output.

## Implementation approach - reusable template macros

## 1. `define_derive_deftly!` macro for defining a reuseable template

Implemented in `define.rs::define_derive_deftly_func_macro`.

When used like this
```rust,ignore
    define_derive_deftly! {
        MyMacro TOPTIONS.. =
        TEMPLATE..
    }
```
Expands to
```rust,ignore
    macro_rules! derive_deftly_template_Template { {
        { $($driver:tt)* }
        [ $($aoptions:tt)* ]
        { $($future:tt)* }
        $($tpassthrough:tt)*
    } => {
        derive_deftly_engine! {
            { $($driver)* }
            [ $(aoptions)* ]
            { $($tpassthrough)* }
            { TEMPLATE.. }
            { $crate; [TOPTIONS..] Template; }
        }
    }; {
        [ $chain0:path, $($chains:path,)* ]
        [ $($accum:tt)* ]
        $($passthrough:tt)*
    } => {
        $chain0 ! {
            [ $($chains,)* ]
            [ $($accum)* _name [Template] _meta [ fmeta(foo) ?tmeta(bar) ] ]
            $($passthrough)*
        }
    } }
```

Except, every `$` in the TEMPLATE is replaced with `$orig_dollar`.
This is because a `macro_rules!` template
is not capable of generating a
literal in the expansion `$`.
(With the still-unstable
[`decl_macro`](https://github.com/rust-lang/rust/issues/83527)
feature, `$$` does this.)
`macro_rules!` simply passes through `$orig_dollar`
(as it does with all unrecognised variables), and
the template expansion engine treats `$orig_dollar` as just a single `$`.

(The extra `{ }` parts after the driver and template
include space for future expansion.)

The second arm of the macro is used for meta attribute checking.
The template is parsed and distilled into the list of
meta attributes that it recognises,
and baked into the macro definition.

## 2. `#[derive_deftly(Template)]`, implemented in `#[derive(Deftly)]`

Template use is implemented in `derive.rs::derive_deftly`.

This

```rust,ignore
    #[derive(Deftly)]
    #[derive_deftly(Template[AOPTIONS..], Template2)]
    pub struct StructName { .. }
```

Generates:

```rust,ignore
    derive_deftly_template_Template! {
        { #[derive_deftly(..)] struct StructName { .. } }
        [ 1 0 AOPTIONS ]
        { }
    }
    derive_deftly_template_Template2! { .. }
    derive_deftly_template_Template! { // first template in derive()
        [ derive_deftly_template_Template2, derive_deftly_expand, ]
        [ ]
        /* no AOPTIONS; we don't try to accumulate them */
        { #[derive_adhoc(Template, Template2)] struct StructName { .. } }
    }
```

## 3. Actual expansion

The first call to `derive_deftly_template_Template!`
is expanded according to the `macro_rules!` definition,
resulting in a call to `derive_deftly_engine`:

```rust,ignore
    derive_deftly_engine! {
        { #[derive_deftly(..)] pub struct StructName { .. } }
        [ 1 0 AOPTIONS ]
        { }
        { TEMPLATE.. }
        { $crate; [TOPTIONS..] Template; }
    }
```

Implemented in `engine.rs::derive_deftly_engine_func_macro`,
this performs the actual template expansion.

### Accumulation expansion

The second call matches the 2nd arm in the template macro.
Each template's arm takes the next macro from `$chain0`,
adds its recognised meta attributes to the list in `$accum`,
and then chains to `$chain0!`
passing the subsequent template(s) in `$chains`.
Eventually, the result is passed to the engine
which also receives, in the `$passthrough`,
the `AOPTIONS` and the driver:

```rust,ignore
derive_deftly_engine! {
	[ ]
	[ _name [Template] _meta [ fmeta(foo) ?tmeta(bar) ] ]
    { #[derive_adhoc(Template, Template2)] struct StructName { .. } }
}

```

The accumulated parts are all of the form: `KEYWORD [DATA]`;
`_KEYWORD [DATA]` for a part which can be safely ignored,
if unrecognised.
Each template's parts start with a `_name` part.
There can also be `error []` parts which
appear when the template couldn't be parsed
(which is used to suppress "unrecognised attribute" errors).

(Doing this with two macro calls means that
if a template is misreferenced, the user gets two errors.
But the alternative is threading *all* the expansions
through this chaining, which is confusing,
and means that the first missing template would stop all processing.)

## Implementation approach - ad-hoc macro applications

### 1. `#[derive(Deftly)]` feature for saving struct definitions

Also implemented in `derive.rs::derive_deftly`.

When applied to (e.g.) `pub struct StructName`,
with `#[derive_deftly_adhoc]` specified
generates this

```rust,ignore
    macro_rules! derive_deftly_driver_StructName { {
        { $($template:tt)* }
        { ($orig_dollar:tt) $(future:tt)* }
        $($dpassthrough:tt)* 
     } => {
        derive_deftly_engine!{
            { pub struct StructName { /* original struct definition */ } }
            /* no AOPTIONS since there's no derive() application */
            { }
            { $($template)* }
            { $($dpassthrough)* }
        }
    } }
```

(Again, the extra `{ }` parts after the driver and template
include space for future expansion.)

In the `pub struct` part every `$` is replaced with `$orig_dollar`,
to use the `$` passed in at the invocation site.
(This is only relevant if the driver contains `$` somehow,
for example in helper attributes.)

### 2. `derive_deftly_adhoc!` function-like proc macro for applying to a template

Implemented in `adhoc.rs::derive_deftly_adhoc`.

When applied like this
```rust,ignore
    derive_deftly_adhoc!{
       StructName TOPTIONS..:
       TEMPLATE..
    }
```

Expands to
```rust,ignore
    derive_deftly_driver_StructName! {
       { TEMPLATE.. }
       { ($) }
       crate; [TOPTIONS..]
    }
```

The literal `$` is there to work around a limitation in `macro_rules!`,
see above.

### 3. Function-like proc macro to do the actual expansion

The result of expanding the above is this:

```rust,ignore
    derive_deftly_engine!{
        { pub struct StructName { /* original struct definition */ } }
        { }
        { TEMPLATE.. }
        { crate; [TOPTIONS..] /*no template name*/; }
    }
```

`derive_deftly_engine` parses `pub struct StructName`,
and implements a bespoke template expander,
whose template syntax resembles the expansion syntax from `macro_rules`.

`crate` is just used as the expansion for `${crate}`.
(For an ad-hoc template, the local crate is correct.)
