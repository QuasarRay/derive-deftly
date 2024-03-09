//! errors with compatibility advice

use super::prelude::*;

/// Add a warning about derive-deftly version incompatibility
pub fn advise_incompatibility(err_needing_advice: syn::Error) -> syn::Error {
    let mut advice = Span::call_site().error(
        "bad input to derive_deftly_engine inner template expansion proc macro; might be due to incompatible derive-deftly versions(s)"
    );
    advice.combine(err_needing_advice);
    advice
}

/// Tries to parse `T` using `call`, possibly offering compat advice
///
/// `call` can return:
///
///  * `Err(raw)`: unexpected error, probably mismatched
///    derive-deftly versions.  `parse_advised` will return the error
///    but with compat advice for the user added.
///  * `Ok(Err(cooked))`: "expected" error, fully reported for
///    the user's benefit.  Returned as-is by `parse_advised`.
///  * `Ok(Ok(T))`.  All went well.
pub fn parse_advised<T>(
    call: impl FnOnce() -> syn::Result<syn::Result<T>>,
) -> syn::Result<T> {
    match call() {
        Ok(Ok(good)) => Ok(good),
        Ok(Err(err_to_return_directly)) => Err(err_to_return_directly),
        Err(err_needing_advice) => {
            Err(advise_incompatibility(err_needing_advice))
        }
    }
}
