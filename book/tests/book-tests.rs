//! This crate exists as a place from which to run the doctests
//! in our mdbook.
//!
//! (mdbook has an `mdbook test` subcommand, but it does badly with
//! dependencies.  See
//! <https://github.com/rust-lang/mdBook/issues/706>.)
//!
//! Use the `update_tests.sh` script to revise the `doctests` module.

// This is not real rustdoc; we don't expect the links to work.
// They will refer to things by their mdbook paths, not by
// rustdoc identities.
#![allow(rustdoc::broken_intra_doc_links)]

#[allow(unused)]
mod prelude {
    pub use derive_deftly::{define_derive_deftly, Deftly};
}

mod chapters {
    include!(concat!(env!("OUT_DIR"), "/book_chapters.rs"));
}
