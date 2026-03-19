//! String templating `$"..."`

use super::framework::*;

/// Token that indicates it's OK to parse a template in this context.
///
/// Trait implemented by [`SubstParseContext::AllowTemplate`].
///
/// This implements the restriction that we don't re-enter template parsing
/// from within `$"..."`.
///
/// This restriction exists so that we don't undermine the rule
/// that control structures like `${ignore ...}` or `${if }`
/// parse their body in exactly the same way as their surrounds are parsed.
/// Without this restriction, template text within `$" ${ ... } "`
/// would be *lexed* as Rust tokens
/// (unlike the surroundings, which are lexed as raw literal text)
/// but *parsed* as if within `${concat}`.
/// That would be anomalous (and hard to explain in the docs).
///
/// We can't implement this restriction in the lexer because the lexer
/// doesn't know the parsing context.
///
/// Having this as an associated type on `SubstParseContext`
/// is precisely right:
///
/// When we re-enter template parsing within `$"..."`
/// with a fixed parsing context, that's fine:
/// that is a new parsing context which therefore wouldn't
/// normally inherit its surrounding context, so wouldn't
/// be expected to inherit the within-`$"..."`
/// even when it appears within `$"..."`
/// (This includes even `${concat }`, although the lexical restriction
/// against strings makes that not very useful.)
///
/// But, when we re-enter template parsing within `$"..."`
/// with the *same* parsing context, namely
/// `concat::Accumulator<string_template::LexicalContextToken>`,
/// we are trying to parse an inner template in `$"..."`
/// which *would* want to inherit the `$"..."` context.
/// That's what's forbidden, by having
/// `LexicalContextToken::new_for_syntax` always fail.
pub trait LexicalContext: Debug + Copy {
    fn new_for_syntax(span: Span) -> syn::Result<Self>;

    /// Allow constructing a `Template` even when parsing within `$"..."`
    ///
    /// Should be used only when a
    /// `Template<concat::Accumulator<string_template::LexicalContextToken>>`
    /// must be constructed in circumstances where we know
    /// the forbidden recursion rule isn't being violated.
    ///
    /// For example, here in `string_template`, the top-level lexer/parser
    /// ends up returning a `SubstDetails` containing such a `Template`.
    fn new_unchecked() -> Self;
}

impl LexicalContext for () {
    fn new_for_syntax(_: Span) -> syn::Result<Self> {
        Ok(())
    }
    fn new_unchecked() -> Self {
        ()
    }
}
