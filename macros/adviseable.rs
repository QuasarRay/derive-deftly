//! errors with compatibility advice
//!
//! Suitable for local glob import.
//!
//! ### Non-local errors from syn
//!
//! `syn` automatically produces "unexpected token" errors,
//! if not all of the input is consumed, somewhere.
//!
//! Empirically:
//! these errors are squirreled away somewhere, and surface
//! on return from one of the top-level syn `parse` functions
//! (the ones that provide a `ParseStream`).
//!
//! If the top-level function would return `Ok`,
//! the unexpected tokens error appears instead.
//! But if there's going to be an error anyway,
//! the unexpected tokens error is discarded.

use super::prelude::*;

/// A `Result` whose error might, or might not, need compat advice
///
///  * `Err(raw)`: unexpected error, probably mismatched
///    derive-deftly versions.  `XXXX` will return the error
///    but with compat advice for the user added.
///  * `Ok(ErrNeedsNoAdvice(cooked))`: "expected" error, fully reported for
///    the user's benefit.  Returned as-is by `parse_advised`.
///  * `Ok(Ok(T))`.  All went well.
///
/// This odd structure is to add a note to most of the errors that
/// come out of syn parsing.  The `braced!` etc. macros insist that the
/// calling scope throws `syn::Error`.
pub type AdviseableResult<T> = syn::Result<AdviseableInnerResult<T>>;

/// Typically found as `syn::Result<AdvisedResult<T>>`
#[derive(Debug)]
pub enum AdviseableInnerResult<T> {
    /// Success
    Ok(T),

    /// Failure, but doesn't need advice
    ///
    /// Typically found as
    /// `sync::Result::Ok(AdvisedInnerResult::NeedsNoAdvice(..))`.
    ErrNeedsNoAdvice(syn::Error),
}

pub use AdviseableInnerResult::ErrNeedsNoAdvice as ErrNNA;
pub use AdviseableInnerResult::Ok as AOk;

/// Add a warning about derive-deftly version incompatibility
///
/// ### Lost advice hazard
///
/// Take care!
/// `syn` generates errors from unprocessed tokens in [`syn::parse2`] etc.
/// Calling this function *within* `syn::parse2`
/// (ie, somewhere you have a `ParseStream`,
/// will result in those errors not receiving advice.
///
/// Ideally, call this from functions that have a `TokenStream`.
/// If you do that, then functions *isnide* that can
/// use this method, avoiding the problem:
/// any errors stored up by `syn` will emerge at that call site,
/// and be properly dealt with.
pub fn advise_incompatibility(err_needing_advice: syn::Error) -> syn::Error {
    let mut advice = Span::call_site().error(
        "bad input to derive_deftly_engine inner template expansion proc macro; might be due to incompatible derive-deftly versions(s)"
    );
    advice.combine(err_needing_advice);
    advice
}

pub fn handle_adviseable_result<T>(ar: AdviseableResult<T>) -> syn::Result<T> {
    match ar {
        Ok(AOk(good)) => Ok(good),
        Ok(ErrNNA(err_to_return_directly)) => Err(err_to_return_directly),
        Err(err_needing_advice) => {
            Err(advise_incompatibility(err_needing_advice))
        }
    }
}

/// Tries to parse `T` using `call`, possibly offering compat advice
pub fn parse_advised<T>(
    call: impl FnOnce() -> AdviseableResult<T>,
) -> syn::Result<T> {
    handle_adviseable_result(call())
}

/// Within `parse_advised`, handle an error *without* giving advice
///
/// `parse_unadvised! { CONTENT_IDENT => || CLOSURE_BODY }`
/// expects `CONTENT_IDENT` to be the contents from
/// [`braced!`], [`bracketed!]` or [`parenthesized!`].
/// Calls the closure.
/// Errors within the closure won't get advice.
///
/// `parse_unadvised! { CONTENT_IDENT }`
/// shorthand for calling `.parse()` on the content.
///
/// # Sketch
///
/// ```rust,ignore
/// let something;
/// let _ = bracketed!(something in input);
/// parse_unadvised! {
///     something => || {
///         // do something with something, eg something.parse()
///         Ok(...)
///     }
/// }
/// ```
macro_rules! parse_unadvised { {
    $content:ident
} => {
    parse_unadvised! { $content => || $content.parse() }
}; {
    $content:ident => || $( $call:tt )*
} => {
    match syn::parse::Parser::parse2(
        // We convert the input to the `TokenStream2` and back,
        // so that we surface "unexpected token errors" here
        // rather than at the toplevel parsing call.
        |$content: ParseStream<'_>| -> syn::Result<_> {
            $($call)*
        },
        $content.parse::<TokenStream>()?
    ) {
        Ok(y) => y,
        Err::<_, syn::Error>(e) => return Ok(ErrNNA(e)),
    }
} }
