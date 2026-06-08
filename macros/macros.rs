#![allow(clippy::style, clippy::complexity)]
#![deny(clippy::disallowed_methods)]
#![doc=include_str!("README.md")]
//
// This is the actual proc-macro crate.
//
// All it exports (or can export) are the proc macros themselves.
// Everything else that is `pub` could be written `pub(crate)`.

mod prelude;

pub(crate) use prelude::*;

// Implementation - common parts
#[macro_use]
pub(crate) mod utils;
#[macro_use]
pub(crate) mod adviseable;
pub(crate) mod framework;
pub(crate) mod general_context;

// Implementation - specific areas
#[macro_use]
pub(crate) mod repeat;
pub(crate) mod accum;
pub(crate) mod approx_equal;
pub(crate) mod boolean;
pub(crate) mod concat;
pub(crate) mod dbg_allkw;
pub(crate) mod expand;
pub(crate) mod generics;
pub(crate) mod meta;
pub(crate) mod modules;
pub(crate) mod options;
pub(crate) mod paste;
pub(crate) mod string_template;
pub(crate) mod syntax;

#[cfg_attr(not(feature = "beta"), path = "beta_disabled.rs")]
pub(crate) mod beta;

// Implementations of each proc-macros
pub(crate) mod adhoc;
pub(crate) mod define;
pub(crate) mod derive;
pub(crate) mod engine;
pub(crate) mod semver;

pub(crate) mod compat_syn_2;
pub(crate) mod compat_syn_common;

#[doc=include_str!("HACKING.md")]
mod _doc_hacking {}

#[doc=include_str!("NOTES.md")]
mod _doc_notes {}

/// Dummy of proc_macro for use when compiling outside of proc macro context
#[cfg(not(proc_macro))]
pub(crate) mod proc_macro {
    pub(crate) use proc_macro2::TokenStream;
}

//========== `expect`, the `check` module (or dummy version) ==========

// "expect" feature; module named check.rs for tab completion reasons
#[cfg(feature = "expect")]
mod check;
#[cfg(not(feature = "expect"))]
mod check {
    use super::prelude::*;
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Target(Void);

    impl FromStr for Target {
        type Err = Void;
        fn from_str(_: &str) -> Result<Self, Void> {
            panic!("output syntax checking not supported, enable `expect` feature of `derive-deftly`")
        }
    }

    pub fn check_expected_target_syntax(
        _ctx: &framework::Context,
        _output: &mut TokenStream,
        target: DdOptVal<Target>,
    ) {
        void::unreachable(target.value.0)
    }

    pub fn check_expect_opcontext(
        op: &DdOptVal<Target>,
        _context: OpContext,
    ) -> syn::Result<()> {
        void::unreachable(op.value.0)
    }
}
impl DdOptValDescribable for check::Target {
    const DESCRIPTION: &'static str =
        "expected output syntax (`expect` option)";
}

//========== actual macro entrypoints ==========

/// Wraps an actual macro implementation function that uses a proc_macro2
/// implementation to expose a proc_macro implementation instead.
//
// Clippy gives false positives for converting between proc_macro[2]::TokenStream.
#[allow(clippy::useless_conversion)]
fn wrap_macro_func<F>(
    func: F,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream
where
    F: FnOnce(
        proc_macro2::TokenStream,
    ) -> Result<proc_macro2::TokenStream, syn::Error>,
{
    let input = proc_macro2::TokenStream::from(input);
    let output = func(input).error_into_compile_error_unrecorded();
    proc_macro::TokenStream::from(output)
}

/// Template expansion engine, internal
///
/// <!-- @dd-navbar macros . -->
/// <!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="index.html">crate top-level</a> | <a href="index.html#overall-toc">overall toc, <strong>macros</strong></a> | <a href="doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>
///
/// Normally you do not need to mention this macro.
///
/// derive-deftly does its work by
/// (defining and then) invoking various interrelated macros
/// including `macro_rules` macros and proc macros.
/// These ultimately end up calling this macro,
/// which takes a template and a data structure,
/// and expands the template for that data structure.
///
/// This macro's behvaiour is not currently stable or documented.
/// If you invoke it yourself, you get to keep all the pieces.
#[cfg_attr(proc_macro, proc_macro)]
pub fn derive_deftly_engine(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_macro_func(engine::derive_deftly_engine_func_macro, input)
}

/// Expand an ad-hoc template, on a data structure decorated `#[derive_deftly_adhoc]`
///
/// <!-- @dd-navbar macros . -->
/// <!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="index.html">crate top-level</a> | <a href="index.html#overall-toc">overall toc, <strong>macros</strong></a> | <a href="doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>
///
/// ```
// We're in the macro crate, where the facade crate is not available.
// So we must do some namespace-swizzling.
/// # use derive_deftly_macros as derive_deftly;
// `proc-macro-crate` says `Itself` so generates ::derive_deftly_engine,
// which is wrong for a doctest.  Fudge that.  We must also make sure
// we're not inside main here, so we must define a main.
/// # use derive_deftly::derive_deftly_engine;
/// # fn main(){}
/// use derive_deftly::{Deftly, derive_deftly_adhoc};
/// #[derive(Deftly)]
/// #[derive_deftly_adhoc]
/// struct DdtaStructureType { }
///
// Smoke and mirrors so we can use metasyntactic OPTIONS and TEMPLATE.
/// # macro_rules! derive_deftly_adhoc { {
/// #     $x:ident OPTIONS,..: TEMPLATE
/// # } => { derive_deftly_macros::derive_deftly_adhoc! {
/// #     $x expect items: fn x(){}
/// # } } }
/// derive_deftly_adhoc! {
///     DdtaStructureType OPTIONS,..:
///     TEMPLATE
/// }
/// ```
///
/// Expands the template `TEMPLATE` for the type `DdtaStructureType`,
///
/// `OPTIONS,..` is an optional comma-separated list of
/// [expansion options](doc_reference/index.html#expansion-options).
///
/// The definition of `DdtaStructureType` must have been decorated
/// with [`#[derive(Deftly)]`](crate::Deftly),
/// and `#[derive_deftly_adhoc]`,
/// and the resulting `derive_deftly_driver_TYPE` macro must be
/// available in scope.
///
/// `derive_deftly_adhoc!` can be used in any context
/// where the Rust language permits macro calls.
/// For example, it can expand to expressions, statements,
/// types, or patterns.
#[cfg_attr(proc_macro, proc_macro)]
pub fn derive_deftly_adhoc(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_macro_func(adhoc::derive_deftly_adhoc, input)
}

/// Define a reuseable template
///
/// <!-- @dd-navbar macros . -->
/// <!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="index.html">crate top-level</a> | <a href="index.html#overall-toc">overall toc, <strong>macros</strong></a> | <a href="doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>
///
/// ```text
/// define_derive_deftly! {
///     [use SomeModule; ..]
///     [/// DOCS]
///     [export] MyMacro OPTIONS,..:
///     TEMPLATE
/// }
/// ```
///
/// Then, `MyMacro` can be used with
/// [`#[derive(Deftly)]`](crate::Deftly)
/// `#[derive_deftly(MyMacro)]`.
///
/// <span id="options-in-define">`OPTIONS,..`</span>
/// is an optional comma-separated list of
/// [expansion options](doc_reference/index.html#expansion-options),
/// which will be applied whenever this template is expanded.
///
/// <span id="docs-in-define">`DOCS`</span>,
/// if supplied, are used as the rustdocs
/// for the captured template macro `derive_deftly_template_MyMacro`.
/// derive-deftly will then also append a note about
/// how to invoke the template.
/// (The `#[doc]` attribute syntax can also be used here.)
///
/// `use` in the preamble refers not to a Rust language module (`mod`)
/// but to a module defined with [`define_derive_deftly_module!`].
///
/// ## Template definition macro `derive_deftly_template_MyMacro`
///
/// The template is made into a `macro_rules` macro
/// named `derive_deftly_template_MyMacro`,
/// which is referenced when the template is applied.
///
/// The template definition macro
/// from `define_derive_deftly!`
/// must be in scope at the point where you try to use it
/// (with `#[derive(Deftly)] #[derive_deftly(MyMacro)]`).
/// If the template definition is in another module,
/// you may need to annotate that module with `#[macro_use]`,
/// or add `use` statements(s).
/// See the
/// [documentation for `#[derive(Deftly)]`](derive.Deftly.html#scoping-and-ordering-within-the-same-crate).
///
/// ## Exporting a template for use by other crates
///
/// With `export MyMacro`, `define_derive_deftly!` exports the template
/// for use by other crates.
/// Then, it is referred to in other crates
/// with `#[derive_ahdoc(this_crate::MyMacro)]`.
///
/// I.e., `export MyMacro` causes the `derive_deftly_template_MyMacro`
/// pattern macro to be exported with `#[macro_export]`.
///
/// Note that a template is always exported at the crate top level,
/// not in a sub-module,
/// even if it is *defined* in a sub-module.
/// Also, note that `export` does not have any effect on
/// visibility of the template *within the same crate*.
/// You may still need `#[macro_use]`.
///
/// ### You must re-export `derive_deftly`; semver implications
///
/// When exporting a template to other crates, you must also
/// re-export `derive_deftly`,
/// at the top level of your crate:
///
/// ```ignore
/// #[doc(hidden)]
/// pub use derive_deftly;
/// ```
/// This is used to find the template expansion engine,
/// and will arrange that your template is expanded
/// by the right version of derive-deftly.
/// The template syntax is that for *your* version of `derive-deftly`,
/// even if the depending crate uses a different version of derive-deftly.
///
/// You should *not* treat a breaking change
/// to derive-deftly's template syntax
/// (which is a major change to derive-deftly),
/// nor a requirement to use a newer template feature,
/// as a breaking changes in the API of your crate.
/// (You *should* use `#[doc(hidden)]`, or other approaches,
/// to discourage downstream crates from using
/// the derive-deftly version you re-export.
/// Such use would be outside the semver guarantees.)
///
/// You *should* call
/// [`derive_deftly::template_export_semver_check!`](macro@template_export_semver_check)
/// once in each crate that exports macros.
/// This will notify you, by breaking your build,
/// if you update to a derive-deftly version
/// that has semver implications for other crates that use your macros.
///
/// Changes that would require a semver bump
/// for all libraries that export templates,
/// will be rare, and specially marked in the derive-deftly changelog.
/// Search for sections with titles containing "template export semver".
///
/// ## Namespacing within a template
///
/// Within the template,
/// items within your crate can be referred to with
/// [`$crate`](doc_reference/index.html#x:crate).
///
/// For other items,
/// including from the standard library e.g., `std::option::Option`,
/// you may rely on the context which uses the template
/// to have a reasonable namespace,
/// or use a explicit paths starting with `std` or `::std` or `::core`
/// or `$crate` (perhaps naming a re-export).
///
/// Overall, the situation is similar to defining
/// an exported `macro_rules` macro.
#[cfg_attr(proc_macro, proc_macro)]
pub fn define_derive_deftly(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_macro_func(define::define_derive_deftly_func_macro, input)
}

/// Perform ad-hoc templating driven by a data structure
///
/// <!-- @dd-navbar macros . -->
/// <!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="index.html">crate top-level</a> | <a href="index.html#overall-toc">overall toc, <strong>macros</strong></a> | <a href="doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>
///
/// This macro does two things:
///
///  1. If `#[derive_deftly(MyMacro)]` attributes are also specified,
///     they are taken to refer to reuseable templates
///     defined with
///     [`define_derive_deftly!`](macro@crate::define_derive_deftly).
///     Each such `MyMacro` is applied to the data structure.
///
///     <span id="expansion-options">You can specify
///     [expansion options](doc_reference/index.html#expansion-options)
///     for each such template application, by writing
///     `#[derive_deftly(MyMacro[OPTIONS,..])]`, where
///     `[OPTIONS,..]` is a comma-separated list of expansion options
///     contained within `[ ]`.</span>
///
///  2. If `#[derive_deftly_adhoc]` is specified,
///     captures the data structure definition,
///     so that it can be used with calls to
///     [`derive_deftly_adhoc!`](macro@crate::derive_deftly_adhoc).
///
/// ## `#[deftly]` attribute
///
/// The contents of `#[deftly]` attributes are made available
/// to templates via the
/// [`${Xmeta}`](doc_reference/index.html#tmeta-vmeta-fmeta--deftly-attributes)
/// expansions.
///
/// If none of the template(s) recognise them,
/// [it is an error](doc_reference/index.html#unrecognisedunused-deftly-attributes),
/// (unless `#[derive_deftly_adhoc]` is specified).
///
/// `derive-deftly`
/// [does not impose any namespacing](doc_reference/index.html#attribute-namespacing)
/// within `#[deftly]`:
///
/// ## Scoping and ordering within the same crate
///
/// **Summary of required ordering**
///
///  1. `define_derive_deftly! { MyMacro = ... }`
///  2. `#[derive(Deftly)] #[derive_deftly(MyMacro)] struct MyStruct { ... }`
///  3. `derive_deftly_adhoc! { MyStruct: ... }`
///
/// Any reusable templates defined with
/// `define_derive_deftly!` must lexically their precede
/// uses with `#[derive(Deftly) #[derive_deftly(...)]`.
///
/// And, for one-off templates (`derive_deftly_adhoc!`),
/// the data structure with its `#[derive(Deftly)]`
/// must lexically precede
/// the references in `derive_deftly_adhoc!`,
/// so that the data structure definition macro
/// is in scope.
///
/// In each case,
/// if the definition is in another module
/// in the same crate,
/// the defining module's `mod` statement must come before
/// the reference,
/// and
/// the `mod` statement will need `#[macro_use]`.
/// So the placement and order of `mod` statements can matter.
/// Alternatively, it is possible to use path-based scoping;
/// there is
/// [an example in the Guide](https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/templates-in-modules.html#path-scope).
///
/// ## Applying a template (derive-deftly macro) from another crate
///
/// `#[derive_deftly(some_crate::MyMacro)]`
/// applies an exported template
/// defined and exported by `some_crate`.
///
/// You can import a template from another crate,
/// so you can apply it with an unqualified name,
/// with `use`,
/// but the `use` must refer to
/// the actual pattern macro name `derive_deftly_template_MyMacro`:
/// ```
// See the doc comment for `derive_deftly_adhoc`.
/// # use derive_deftly_macros as derive_deftly;
/// # use derive_deftly::derive_deftly_engine;
/// # fn main(){}
// We can't make another crate.  Fake up the macro definition
/// # derive_deftly::define_derive_deftly! { TheirMacro: }
/// use derive_deftly::Deftly;
// and don't really try to import it, then
/// # #[cfg(any())]
/// use other_crate::derive_deftly_template_TheirMacro;
/// #[derive(Deftly)]
/// #[derive_deftly(TheirMacro)]
/// struct MyStruct { // ...
/// # }
/// ```
///
/// ## Captured data structure definition `derive_deftly_driver_TYPE`
///
/// With `#[derive_deftly_adhoc]`,
/// the data structure is captured
/// for use by
/// [`derive_deftly_adhoc!`](macro@crate::derive_deftly_adhoc).
///
/// Specifically, by defining
/// a `macro_rules` macro called `derive_deftly_driver_TYPE`,
/// where `TYPE` is the name of the type
/// that `#[derive(Deftly)]` is applied to.
///
/// ### Exporting the driver for downstream crates' templates
///
// Really, the documentation about this in `pub-a.rs` and `pub-b.rs`,
// should be somewhere in our rustdoc output.
// But I don't want to put it *here* because it would completely
// dominate this macro documentation.
// So for now just reference the source tree docs.
// (We can't really easily provide even a link.)
// I think this is such a minority feature,
// that hiding the docs like this is OK.
//
/// To cause the macro embodying the driver struct to be exported,
/// write:
/// `#[derive_deftly_adhoc(export)]`.
/// The driver can then be derived from in other crates,
/// with `derive_deftly_adhoc! { exporting_crate::DriverStruct: ... }`.
///
/// #### Semver hazards
///
/// This is a tricky feature,
/// which should only be used by experts
/// who fully understand the implications.
/// It effectively turns the body of the struct into a macro,
/// with a brittle API
/// and very limited support for namespacing or hygiene.
///
/// See `pub mod a_driver` in the example file `pub-a.rs`,
/// in the source tree,
/// for a fuller discussion of the implications,
/// and some advice.
///
/// If you do this, you must **pin your derive-deftly** to a minor version,
/// as you may need to treat *minor* version updates in derive-deftly
/// as semver breaks for your crate.
/// And every time you update, you must read the `CHANGELOG.md`,
/// since there is nothing that will warn you automatically
/// about breaking changes.
//
// This is the implementation of #[derive(Deftly)]
#[cfg_attr(
    proc_macro,
    proc_macro_derive(
        Deftly,
        attributes(deftly, derive_deftly, derive_deftly_adhoc)
    )
)]
pub fn derive_deftly(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_macro_func(derive::derive_deftly, input)
}

/// Define a module with reuseable template definitions (**beta**)
///
/// <!-- @dd-navbar macros . -->
/// <!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="index.html">crate top-level</a> | <a href="index.html#overall-toc">overall toc, <strong>macros</strong></a> | <a href="doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>
///
/// ```text
/// define_derive_deftly_module! {
///     [/// DOCS]
///     [export] MyModule OPTIONS,..:
///     [use SubModule; ..]
///     TEMPLATE_DEFINITIONS
/// }
/// ```
///
/// Then, `use MyModule` can be used in [`define_derive_deftly!`]
/// (and, in another `define_derive_deftly_module!`).
///
/// TEMPLATE_DEFINITIONS may contain *only* `${define ..}` and `${defcond }`.
/// (So it cannot define derives as-such, but it could contain
/// the whole *body* of a derive as a `${define}`.)
///
/// This feature is [**beta**](doc_changelog/index.html#t:beta).
/// It requires the `beta` cargo feature
/// (but **not** any
/// [`beta_deftly` template option](doc_reference/index.html#eo:beta_deftly)).
///
/// ## Scope and (lack of) namespacing
///
/// All names defined in a module are accessible within
/// any tamplate or module that uses the module (directly or indirectly).
///
/// So there is no namespacing: names are "global".
/// Like all `${define }` and `${defcond }`, scope is dynamic, not lexical.
///
/// Definitions in imported modules can be shadowed by subsequent definitions,
/// including subsequent modules, and the importing template or module.
///
/// So definitions in general-purpose modules should usually
/// have a namespace component within their name.
/// This applies especially to "internal" definitions,
/// which the module user is not intended to use or redefine.
///
/// ## Documentation expansions in `define_derive_deftly!`
///
/// In `define_derive_deftly!`, `DOCS` can contain both literal
/// `#[doc]` attributes, and template expansions
/// that expand to `#[doc]` attributes.
/// The template expansions can refer to `${define}`s from imported modules.
/// This allows re-use of documentation fragments.
///
/// The documentation for a *module* cannot contain expansions.
///
/// ## Placement of `use`
///
/// In `define_derive_deftly!` `use Module` appears in the preamble,
/// before the name of the template being defined.
/// `use` statements in the body become part of the expansion,
/// so will refer to Rust modules.
///
/// In `define_derive_deftly_module!`,
/// `use Module` appears within the body, before the definitions.
/// (This reflects the fact that the imported module becomes part
/// of the module being defined, and that imported definitions
/// cannot be used in the module's documentation.)
///
/// ## Options
///
/// The only `OPTION` supported is `beta_deftly`.
///
/// Beta features can be used in a module, if `beta_deftly` is specified
/// in the *module definition*.  A module which uses beta features
/// can be imported by a template or module which does not itself
/// specify `beta_deftly.`
///
/// `use` statements do not themselves require specifying `beta_deftly`;
/// the cargo feature is sufficient.
///
/// ## Module definition macro `derive_deftly_module_MyModule`
///
/// The module's definitions are made into a `macro_rules` macro
/// named `derive_deftly_module_MyModule`,
/// which is referenced where the module is imported.
///
/// The module needs to be in Rust macro scope where it's `use`d.
/// It does not need to be in scope for `derive` whose template uses it.
/// (The module's definitions are bodily incorporated into the
/// importing template (or module) macro_rules macro.)e
///
/// ### Semver implications of `export`:
///
/// Normally, template code present in different crates can be processed
/// by different, perhaps semver-incompatible, versions of derive-deftly.
///
/// But, a whole derive must be processed by *one* version of
/// derive-deftly.  Ie, when you `use` a module from another crate, that
/// other crate's module's template text gets processed with *your*
/// version of derive-deftly.
///
/// Additionally, the lack of namespacing provides ample opportunity
/// for unintended interactions and uncontrolled dependencies on internals.
///
/// There are no features in derive-deftly for helping make
/// an exported module with a stable API, whatever that means.
#[cfg_attr(proc_macro, proc_macro)]
pub fn define_derive_deftly_module(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_macro_func(modules::define_derive_deftly_module, input)
}

/// Check semver compatibility, for a crate which exports macros
///
/// <!-- @dd-navbar macros . -->
/// <!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="index.html">crate top-level</a> | <a href="index.html#overall-toc">overall toc, <strong>macros</strong></a> | <a href="doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>
///
/// Causes a compilation error
/// if and only if the specified version of `derive-deftly`
/// is prior to the last *relevant change*,
/// compared to the currently-running one.
///
/// A *relevant change* is one which has semver implications
/// for the API of a crate which exports derive-deftly templates.
///
/// ## When and how to call this
///
/// If you export templates, with `define_derive_deftly! { export ... }`,
/// call this macro too, once in your crate.
///
/// Pass it the version of `derive-deftly` that was current,
/// when you last read the `derive-deftly` changelog
/// and considered breaking changes.
///
/// (The argument must be a string literal, containing a
/// 2- or 3-element version number.
/// If the 3rd element is omitted, 0 is used.)
///
/// ## Guarantee
///
/// You can upgrade your derive-deftly version,
/// even across a semver-breaking change to derive-deftly,
/// without making any consequential update to your crate's own semver.
///
/// If a new version of derive-adhoc means *your* crate's
/// API has semver-relevant changes, this macro will throw an error.
/// (Of course that will only happen across semver-breaking
/// updates of derive-deftly.)
///
/// (Exporting a *driver* struct for derivation in downstream crates,
/// `#[derive_deftly_adhoc(export)]`, is not covered by this promise.)
///
/// ## Example
///
/// ```
/// # use derive_deftly_macros as derive_deftly;
/// derive_deftly::template_export_semver_check!("0.13.0");
/// ```
#[cfg_attr(proc_macro, proc_macro)]
pub fn template_export_semver_check(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    wrap_macro_func(semver::template_export_semver_check_func_macro, input)
}
