//! errors with compatibility advice
//!
//! Suitable for local glob import.

use super::prelude::*;

/// A `Result` whose error might, or might not, need compat advice
///
///  * `Err(raw)`: unexpected error, probably mismatched
///    derive-deftly versions.  `XXXX` will return the error
///    but with compat advice for the user added.
///  * `Ok(ErrNeedsNoAdvice(cooked))`: "expected" error, fully reported for
///    the user's benefit.  Returned as-is by `parse_advised`.
///  * `Ok(Ok(T))`.  All went well.
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
