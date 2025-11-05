//! Handle `beta_deftly` template option, when `beta` cargo feature enabled
//!
//! For instructions on adding a beta feature,
//! see [`beta::Enabled`].
//!
//! This is a bit annoying.  It has to be an ambient property,
//! so that the syn `Parse` trait can be implemented.

use super::prelude::*;

use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};

/// Token indicating that beta feature(s) are or can be enabled
///
/// When adding a new beta feature:
///
///  * Put an instance of [`beta::Enabled`]
///    in the appropriate piece of parsed template syntax,
///    For example, in the [`SubstDetails`](super::syntax::SubstDetails)
///    along with the `O::` markers.
///
///  * When parsing, obtain the value from [`Enabled::new_for_syntax`].
///
///  * Add a test case to `tests/minimal-ui/disabled.rs`
///    which *omits* the `beta_deftly` option, and therefore fails,
///    thus demonstrating that the feature gate works as intended.
///
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub struct Enabled {}

pub type MaybeEnabled = Result<Enabled, ErrorGenerator<'static>>;

#[derive(Copy, Clone)]
enum ThreadState {
    Unset,
    Set(MaybeEnabled),
}

use ThreadState as TS;

thread_local! {
    static ENABLED: Cell<ThreadState> = Cell::new(TS::Unset);
}

/// Workaround for `LazyStatic<Cell>::get` MSRV of 1.73.0.
fn threadlocal_get() -> ThreadState {
    ENABLED.with(|c| c.get())
}
fn threadlocal_set(s: ThreadState) {
    ENABLED.with(|c| c.set(s))
}

/// Call `f` with beta features enabled or not
///
/// Used by the parser for `TopTemplate`
pub fn with_maybe_enabled<R>(
    enabled: MaybeEnabled,
    f: impl FnOnce() -> R,
) -> R {
    assert!(matches!(threadlocal_get(), TS::Unset));
    threadlocal_set(TS::Set(enabled));
    // Unwind safety: we re-throw the panic,
    // so even if f or R wasn't, no-one observes any broken invariant.
    let r = catch_unwind(AssertUnwindSafe(f));
    threadlocal_set(TS::Unset);
    match r {
        Ok(r) => r,
        Err(e) => resume_unwind(e),
    }
}

impl Enabled {
    /// If the cargo feature is enabled, return `Ok(Enabled)`
    ///
    /// Used when parsing the `beta_deftly` template option.
    //
    // (In this version of the source code it *always* returns Ok.
    // Returning Err is done by beta_disabled.rs.)
    pub fn new_for_dd_option(_: Span) -> syn::Result<Self> {
        Ok(Enabled {})
    }

    /// If the cargo feature is enabled, return `Ok(Enabled)`
    ///
    /// Used for things that are inherently part of the modules feature,
    /// such as `use`s and `define_derive_deftly_module!`.
    //
    // Remove when modules are no longer beta
    //
    // (In this version of the source code it *always* returns Ok.
    // Returning Err is done by beta_disabled.rs.)
    pub fn new_for_modules_feature(_: Span) -> syn::Result<Self> {
        Ok(Enabled {})
    }

    /// If the cargo feature is enabled, return `Some`.
    ///
    /// Used when parsing template text imported from modules.
    ///
    /// The rule is that modules have their own `beta_deftly` option,
    /// and beta features are allowed in module text iff the module
    /// enabled them.  The importing template doesn't need to.
    //
    // (In this version of the source code it *always* returns Ok.
    // Returning Err is done by beta_disabled.rs.)
    pub fn new_for_imported_definitions() -> MaybeEnabled {
        Ok(Enabled {})
    }

    /// If the `beta_deftly` template feature is enabled, return `Ok(Enabled)`
    ///
    /// Used when parsing beta syntax, in templates.
    #[allow(dead_code)] // sometimes we might not have any beta features
    pub fn new_for_syntax(span: Span) -> syn::Result<Self> {
        match threadlocal_get() {
            TS::Unset => {
                Err(span.error("internal error! beta::ENABLED Unset"))
            }
            TS::Set(ue) => ue.map_err(|eh| eh(span)),
        }
    }

    /// Makes `new_for_syntax` work properly within `f`, in test cases
    #[cfg(test)]
    #[allow(dead_code)]
    pub fn test_with_parsing<R>(f: impl FnOnce() -> R) -> R {
        with_maybe_enabled(Ok(Enabled {}), f)
    }
}
