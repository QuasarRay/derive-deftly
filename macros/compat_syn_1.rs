//! Definitions for compatibility with syn 1

#[allow(unused_imports)]
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
