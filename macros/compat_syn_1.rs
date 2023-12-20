//! Definitions for compatibility with syn 1

use super::prelude::*;

//---------- Spanned ----------

pub use syn::spanned::Spanned;

//---------- Attribute methods ----------

pub trait AttributeExt1 {
    /// syn 2 has this as an inherent method
    fn path(&self) -> &syn::Path;
}
impl AttributeExt1 for syn::Attribute {
    fn path(&self) -> &syn::Path {
        &self.path
    }
}

impl AttributeExt12 for syn::Attribute {
    fn call_in_parens<T, F>(&self, f: F) -> syn::Result<T>
    where
        F: FnOnce(ParseStream<'_>) -> syn::Result<T>,
    {
        (|outer: ParseStream<'_>| {
            let inner;
            let _paren = parenthesized!(inner in outer);
            f(&inner)
        })
        .parse2(self.tokens.clone())
    }
}
