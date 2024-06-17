//! Check the examples in the reference manual
//!
//! Looks for bullet points and blockquotes
//! in sections with title starting "Examples":
//!
//! ```text
//!  * `INPUT`: `OUTPUT`
//!  * `INPUT` for struct: `OUTPUT`
//!  * `INPUT` for structs: `OUTPUT`
//!  * `INPUT` for enum: `OUTPUT`
//!  * `INPUT` for enums: `OUTPUT`
//!  * `INPUT` for enum variant: `OUTPUT`
//!  * `INPUT` for enum variants: `OUTPUT`
//!  * `INPUT` for `TYPE-OR-VARIANT`: `OUTPUT`
//!  * `INPUT` for `FIELD` in `TYPE-OR-VARIANT`: `OUTPUT`
//!  * `INPUT` for fields in `TYPE-OR-VARIANT`: `OUTPUT`
//!  * `INPUT` for others: `OUTPUT`
//!  * `INPUT` ...: ``OUTPUT``, ...
//!  * `INPUT` ...: nothing
//!  * `INPUT` ...: error, ``MESSAGE``
//!  * `CONDITION`: true for SOMETHING, SOMETHING, and ...`
//! ```
//!
//! ("others" means not any of the preceding contexts.
//! Note that double backquotes are required for "error,",
//! which allows individual backquotes in the messages themselves.
//! The MESSAGE must then be a substring of the actual error.)
//!
//! In an example of a `CONDITION`,
//! `SOMETHING` can be any of the syntaxes accepted in `for ...`
//! (but not "others", obviously).
//! All the contexts for which it returns true must be listed.
//!
//! In "for" clauses you can also write
//! a leading `struct ` or `Enum::`,
//! and a trailing `;`, `(...);`, or `{...}`.
//!
//! Blockquotes ` ```rust ` are tested separately via rustdoc, so ignored here.
//!
//! Otherwise, they should come in pairs, with, in between,
//! ```text
//!   <!--##examples-for-toplevels-concat TYPE TYPE...##-->
//! ```
//! (which shuld be followed by introductory text for the reader).
//! And then the first is expanded for each TYPE;
//! the results (concatenated) must match the 2nd block.
//!
//! Special directives
//!
//!   * `<!--##examples-ignore##-->`:
//!
//!       Ignore until next blank line
//!
//!   * `<!--##examples-for FOO##-->`:
//!
//!       In bullet point(s), use this as if "for FOO" was written
//!       (ignoring any actual "for FOO")
//!       Applies until end of section (or next such directive)
//!
//! Preceding a ` ```...``` ` quote
//!
//!   * `<!--##examples-structs##-->`:
//!
//!       The quote has the example structs
//!

use super::*;

mod conditions;
mod contexts;
mod for_toplevels_concat;
mod possibilities;
mod reference_extract;

use conditions::ConditionExample;
use contexts::{for_every_example_context, ContextExt as _, Limit};
use for_toplevels_concat::ForToplevelsConcatExample;
use possibilities::PossibilitiesExample;

const INPUT_FILE: &str = "doc/reference.md";

pub type DocLoc = usize;

/// Something that can be checked
pub trait Example {
    fn print_checking(&self);
    fn check(&self, out: &mut Errors, drivers: &[syn::DeriveInput]);
}

/// Allows errors to be aggregated
pub struct Errors {
    ok: Result<(), ()>,
}
impl Errors {
    fn new() -> Self {
        Errors { ok: Ok(()) }
    }
    fn wrong(&mut self, loc: DocLoc, msg: impl Display) {
        eprintln!("{INPUT_FILE}:{loc}: {msg}");
        self.ok = Err(());
    }
}
impl Drop for Errors {
    fn drop(&mut self) {
        if !std::thread::panicking() && !self.ok.is_ok() {
            panic!("documentation examples check failed");
        }
    }
}

fn bail(loc: DocLoc, msg: impl Display) -> ! {
    Errors::new().wrong(loc, msg);
    panic!("Errors should have panicked already!");
}

#[test]
fn check_examples() {
    let mut errs = Errors::new();
    let (structs, examples) = reference_extract::extract(&mut errs);
    for example in &examples {
        example.print_checking();
        example.check(&mut errs, &structs);
    }
    eprintln!("checked {} examples", examples.len());
}
