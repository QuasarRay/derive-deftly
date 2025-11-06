# Implementation approach - how does this work?

<!-- @dd-navbar toc .. -->
<!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="../index.html">crate top-level</a> | <a href="../index.html#overall-toc"><strong>overall toc</strong>, macros</a> | <a href="../doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>

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

### Expansion chaining

When a `#[derive(Deftly)]` call site invokes multiple templates,
we don't emit separate macro calls for each template.
Instead, we generate a macro call for the first template,
and pass it the names of subsequent templates.
Each template expansion is responsible for invoking the next.

This allows output from the expansion to be
threaded through the chain,
collecting additional information as we go,
and eventually analysed after *all* the expansions are done.
(This is done by putting`derive_deftly_engine`
as the final entry
in the list of macros to chain to.)

Currently, this data collection is used for
detecting unused meta attributes.

This does mean that specifying a completely unknown template
(for example, one that's misspelled or not in scope)
will prevent error reports from subsequent templates,
which is a shame.

### Overall input syntax for templates and `derive_deftly_engine!`

Because of expansion chaining, calls to the engine,
and to templates, overlap.

```rust,ignore
    macro! {
        { DRIVER.. }
        [ 1 0 AOPTIONS.. ] // not present for d_d_dengine expanding ad-hoc
                           // replaced with just "." at end of chain
    // always present, but always immediately ignored:
        ( .... )
    // these parts passed to d_d_engine only, and not in last call:
        { TEMPLATE.. }
        ( CRATE; [TOPTIONS..]
          TEMPLATE_NAME /* omitted if not available */;
          { IMPORTED_DEFINITIONS.. } // from a module (may be omitted)
          .... )
    // always present:
        // after here is simply passed through by templates:
        [
          // usually one or more of these; none at end, or ad-hoc
          CHAIN_TO ( CHAIN_PASS_AFTER_DRIVER .... )
          ..
        ]
        [ACCUM..]
        ....
    }
```

(We use the notation `....` for room we are leaving
for future expansion;
these are currently empty.)

The engine also accepts a nonoverlapping syntax for handling
template definitions which use modules - see below.

### 1. `define_derive_deftly!` macro for defining a reuseable template

Implemented in `define.rs::define_derive_deftly_func_macro`.

When used like this
```rust,ignore
    define_derive_deftly! {
        MyMacro TOPTIONS..:
        TEMPLATE..
    }
```
Expands to
```rust,ignore
    macro_rules! derive_deftly_template_Template { {
        { $($driver:tt)* }
        [ $($aoptions:tt)* ]
        ( $($future:tt)* )
        $($tpassthrough:tt)*
    } => {
        derive_deftly_engine! {
            { $($driver)* }
            [ $(aoptions)* ]
            ( )
            { TEMPLATE.. }
            ( $crate; [TOPTIONS..] Template;
              { IMPORTED_DEFINITIONS.. } // only if via modules (see below)
              )
            $($tpassthrough)*
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

(The extra `( )` parts after the driver and template
include space for future expansion.)

### 2. `#[derive_deftly(Template)]`, implemented in `#[derive(Deftly)]`

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
        ( )
        [
          derive_deftly_template_Template2 ( [1 0 AOPTIONS2] () )
          derive_deftly_engine ( . () )
        ]
        [] // (nothing accumulated yet)
    }
```

### 3. Actual expansion

The first call to `derive_deftly_template_Template!`
is expanded according to the `macro_rules!` definition,
resulting in a call to `derive_deftly_engine`:

```rust,ignore
    derive_deftly_engine! {
        { #[derive_deftly(..)] pub struct StructName { .. } }
        [ 1 0 AOPTIONS ]
        ( )
        { TEMPLATE.. }
        ( $crate; [TOPTIONS..] Template; )
        [
          derive_deftly_template_Template2 ( [1 0 AOPTIONS2] () )
          derive_deftly_engine ( . () )
        ]
        [] // ACCUM
    }
```

Implemented in `engine.rs::derive_deftly_engine_func_macro`,
this performs the actual template expansion.

### 4. Chaining to the next macro

`derive_deftly_engine!`
then invokes the next template.

In the above example, it outputs:

```rust,ignore
    derive_deftly_template_Template2! {
        { #[derive_deftly(..)] pub struct StructName { .. } }
        [ 1 0 AOPTIONS2 ]
        ( ) // threaded through from the call site
        [ derive_deftly_engine ( . () ) ]
        [_name Template _meta_used [..] _meta_recog [..]] // ACCUM
    }
```

`ACCUM` accumulates information from successive expansions,
and is actually parsed only in the next step.

#### Used meta attribute syntax

`_meta_used` introduces the meta attributes from the driver,
which were used by this template,
in the following syntax:
```text
  ::VARIANT // entries until next :: are for this variant
  .field    // entries until next :: or . are for this field
  ( // each (..) corresponds to one #[deftly()], and matches its tree
    somemeta(
      value_used =, // value was used
      value_used +, // value was used but in context with a default
      bool_tested ?, // boolean was tested
      , // skip a node (subtree) not used at all
      .. // trailing skipped notes are omitted
    )
  ) // empty trailing groups are omitted
```

So for example this

```rust,ignore
struct S {
  #[adhoc(foo(bar, baz))]
  #[adhoc(wombat, zoo = "Berlin")]
  f: T,
}
```

might correspond to this

```text
_meta [
    // nothing (don't need field name even)
    .f () (, zoo?)
    .f () (wombat?, zoo=)
    .f (foo=(bar?, baz?)) (wombat?, zoo?)
]
```

This representation helps minimise the amount of text
which needs to be passed through all the macro chains,
when only a few attributes are used by each template,
while
still being unambiguous and detecting desynchronisation.

Instead of a `[..]` list, `_meta_used` can also be `*`,
meaning that no checking should be done.

### Recognised meta attribute syntax

`_meta_recog` introduces the meta attributes which
might conceivably be recognised by this template.
This is determined statically, by scanning the template.

The information is used only if there are unused attributes,
to assist with producing good error messages.

`_meta_recog` takes a `[ ]` list containing items like `fmeta(foo(bar))`,
or ``fmeta(foo(bar))?` when recognised only as a boolean,
or ``fmeta(foo(bar))+` when only used as a value (without any boolean tests).

### 5. Reporting unused meta attributes

At the end of the list of chained templates,
is (the driver's version of) `derive_deftly_engine!`.
So after all the templates have been processed,
instead of invoking the next template,
we return directly to the engine:

```rust,ignore
    derive_deftly_engine! {
        { #[derive_deftly(..)] pub struct StructName { .. } }
        .
        ( )
        [] // end of the chain
        [_name Template _meta_used [..] _meta_recog [..] _name Template2 ..]
    }
```

The accumulated parts are all of the form: `KEYWORD DATA`;
`_KEYWORD DATA` for a part which can be safely ignored,
if unrecognised.
`DATA` is a single tt.
Each template's parts start with a `_name` part.
There can also be `error` parts which
appear when the template couldn't be parsed
(which is used to suppress "unrecognised attribute" errors).

The actually supplied attributes are compared with
the union of the used attributes,
and errors are reported for unused ones.

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
            ( )
            { $($template)* }
            $($dpassthrough)*
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
       ( crate; [TOPTIONS..] /*no template name*/; )
       []
       [_meta_used *]
    }
```

The literal `$` is there to work around a limitation in `macro_rules!`,
see above.

### 3. Function-like proc macro to do the actual expansion

The result of expanding the above is this:

```rust,ignore
    derive_deftly_engine!{
        { pub struct StructName { /* original struct definition */ } }
        ( )
        { TEMPLATE.. }
        ( crate; [TOPTIONS..] /*no template name*/; )
        []
        [_meta_used *]
    }
```

`derive_deftly_engine` parses `pub struct StructName`,
and implements a bespoke template expander,
whose template syntax resembles the expansion syntax from `macro_rules`.

`crate` is just used as the expansion for `${crate}`.
(For an ad-hoc template, the local crate is correct.)

## Implementation approach - deftly definition modules

### Overall input syntax for modules and `derive_deftly_engine!`

```rust,ignore
    macro! {
	    via_modules [
            {....} // used by `macro`
		    NEXT_MACRO {....} // used by `NEXT_MACRO`
	        ..
			define_derive_deftly {....} DEFWHAT ....
	    ]
        { EARLIER_PREFIX.. }
        { MAIN_PASSTHROUGH.. }
        ( $ EXTRA_PASSTHROUGH.... )
        ....
    }
```

Loosely, the semantics of `derive_deftly_module_MODULE` are:
define a thing, which depends on MODULE's definitions.

Each `derive_deftly_module_MODULE` macro
prepends its own definitions to EARLIER\_PREFIX,
leaving MAIN\_PASSTHROUGH unchanged,
and then calls the next macro in turn.

Eventually `derive_deftly_engine` is reached, with NEXT\_MACROS empty,
and looks at `[ DEFWHAT .... ]` to know what to do.

When a template uses multiple modules,
the module macros are listed in `[ ]` in reverse order,
because the last macro in the list runs last, prepending,
so that its definitions end up first in the output.
Engine comes last because it runs on the "outside",
after all the prepending is done.
The reverse ordering is not strictly necessary -
we could define the macros to *append* and list in forwards order;
it's done this way for future compatibility
with possible other uses of the module system.

EARLIER\_PREFIX has already been dollar-escaped, using `$ orig_dollar`.
MAIN\_PASSTHROUGH has not.

### Module macro

```rust,ignore
    define_derive_deftly_module! {
        DOCS
        [export] MODULE:

        M_DEFINES
    }
```

Expands to:

```rust,ignore
    DOCS
    #[macro_export]
    macro_rules! derive_deftly_module_MODULE { {
        via_modules [
            { $($our_opts:tt)* }
		    $next_macro:path
	        { $($next_opts:tt)* }
			$(rest:tt)*
		]
        { $($predefs:tt)* }
        { $($main:tt)* }
        ( $($extra:tt)* )
        $($ignored:tt)*
    } => {
      $next_macro! {
        via_modules [ { $($next_opts)* } $($rest)* ]
        { M_DEFINES $($predefs)* }
        { $($main)* }
        ( $($extra)* )
      }
    } }
```

In M_DEFINES, dollars have to be turned into `$orig_dollar`.

### Definition of a module-using template

```rust,ignore
    define_derive_deftly! {
        use M1;
        use M2;
        DOCS MyMacro TOPTIONS..: TEMPLATE..
    }
```

Expands to:

```rust,ignore
    derive_deftly_module_M2! {
        via_modules [
		    {}
			derive_deftly_module_M1 {}
			derive_deftly_engine {} define
        ]
        { }
        { DOCS MyMacro TOPTIONS..: TEMPLATE.. }
        ( )
    }
```

Hence:

```rust,ignore
    derive_deftly_module_M1! {
        via_modules [
		    {}
			derive_deftly_engine {} define
        ]
        { M2_DEFINES }
        { DOCS MyMacro TOPTIONS..: TEMPLATE.. }
        ( )
    }
```

Hence:

```rust,ignore
    derive_deftly_engine! {
        via_modules [
		    {} define
	    ]
        { M1_DEFINES M2_DEFINES }
        { DOCS MyMacro TOPTIONS..: TEMPLATE.. }
        ( )
    }
```

Which is then handled as if it were an invocation of `define_derive_deftly!`
without any `use` statements.

The resulting template macro will additionally embody
`{ M1_DEFINES M2_DEFINES }` as `{ IMPORTED_DEFINITIONS.. }`.

### Definition of a module-using module

These are resolved early, so that they are syntax-checked
(and the names of the imported modules resolved) at the definition site:

```rust,ignore
    define_derive_deftly_module! {
        MODULE_DOCS
        [export] MODULE:

        use P1;
        use P2;

        M_DEFINES
    }
```

Expands to:

```rust,ignore
    derive_deftly_module_P2! {
        via_modules [
		    {}
			derive_deftly_module_P1 {}
			derive_deftly_engine {} defmod
		]
        { M_DEFINES }
        { MODULE_DOCS [export] MODULE: }
        ( )
    }
```

Hence:

```rust,ignore
    derive_deftly_module_P1! {
        via_modules [
		    {}
			derive_deftly_engine {} defmod
		]
        { P2_DEFINES M_DEFINES }
        { MODULE_DOCS [export] MODULE: }
        ( )
    }
```

Hence:

```rust,ignore
    derive_deftly_engine {
        via_modules [
		    {} defmod
        ]
        { P1_DEFINES P2_DEFINES M_DEFINES }
        { MODULE_DOCS [export] MODULE: }
        ( )
    }
```

Which is treated as:

```rust,ignore
    define_derive_deftly_module! {
        MODULE_DOCS [export] MODULE:
        P1_DEFINES P2_DEFINES M_DEFINES
    }
```
