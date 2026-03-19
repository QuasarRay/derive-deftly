//! String templating `$"..."`

use super::framework::*;

mod lex;
use lex::*;

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

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub struct LexicalContextToken {}

type Accum = concat::Accumulator<LexicalContextToken>;

impl LexicalContext for LexicalContextToken {
    fn new_for_syntax(span: Span) -> syn::Result<Self> {
        Err(span.error(
 "control structures are not allowed in string templates (within an expansion in a string template, reentering the string template lexing/parsing context is not possible)"
        ))
    }
    fn new_unchecked() -> Self {
        LexicalContextToken {}
    }
}

fn is_delim_get_end(c: char) -> Option<u8> {
    Some(match c {
        '{' => b'}',
        '(' => b')',
        '[' => b']',
        _ => return None,
    })
}

fn parse_tokens_bracketed(
    hd: &mut HadDollar,
    out: &mut Vec<TemplateElement<Accum>>,
    mut bracket_stack: Vec<u8>,
) -> syn::Result<()> {
    while let Some(end) = bracket_stack.last() {
        let end = *end as char;

        let c = hd.lc.next_char().ok_or_else(|| {
            hd.error(format_args!(
                "unexpected end of string, espected {}",
                end
            ))
        })?;

        if c == end {
            let _: u8 = bracket_stack.pop().expect("was nonempty");
        } else if c == '}' || c == ')' || c == ']' {
            return Err(hd.error(format_args!(
                r#"mismatched closing delimiter, expected {}"#,
                end,
            )));
        } else if let Some(end) = is_delim_get_end(c) {
            bracket_stack.push(end);
        } else if c == '"' || c == '\\' || c == '\'' {
            return Err(hd.error(
                r#"nested strings, characters/lifetimes, and backslashes are not allowed in $"...""#
            ));
        } else {
            // OK, just literal text
        }
    }
    assert_eq!(bracket_stack, &[] as &[u8]);

    parse_tokens_since_dollar(hd, out)
}

fn parse_tokens_since_dollar(
    hd: &mut HadDollar,
    out: &mut Vec<TemplateElement<Accum>>,
) -> syn::Result<()> {
    let l = hd.chunk_after_dollar();

    (|| {
        let ts: TokenStream = syn::parse_str(l)?;
        let subst = Parser::parse2(
            |input: ParseStream<'_>| {
                let la = input.lookahead1();
                Subst::<Accum>::parse_after_dollar(
                    la,
                    input,
                    // bypass deescape_orig_dollar; we don't support
                    // $orig_dollar in "..." because that's silly,
                    // and in any case it wouldn't work because we'd
                    // be parsing a TS with just `$ orig_dollar`.
                    OrigDollarHandled::NotFound,
                )
            },
            ts,
        )?;
        out.push(TE::Subst(subst));
        Ok(())
    })()
    .map_err(|e| hd.map_syn_err(e))
}

fn parse_lit_str(input: &syn::LitStr) -> syn::Result<Template<Accum>> {
    let mut out = vec![];

    let whole = input.value();

    let span = input.span();
    let mut lc = LexingContext::new(span, &whole);

    let mk_lit = |s: &str| TE::LitStr(syn::LitStr::new(s, span));

    loop {
        let (lit, hd) = lc.find_dollar();
        out.push(mk_lit(lit));

        let mut hd = if let Some(hd) = hd {
            hd
        } else {
            break;
        };

        let c = hd
            .lc
            .next_char()
            .ok_or_else(|| span.error("unexpected end of string after `$`"))?;

        if c == '$' {
            out.push(mk_lit("$"));
        } else if c == '<' {
            parse_tokens_bracketed(&mut hd, &mut out, vec![b'>'])?;
        } else if let Some(end) = is_delim_get_end(c) {
            // This parses ${ } and $( ) which we want and also $[ ]
            // which we don't want right now but will reject later.
            parse_tokens_bracketed(&mut hd, &mut out, vec![end])?;
        } else if unicode_ident::is_xid_start(c) {
            loop {
                match hd.lc.peek_char() {
                    Some(c) if unicode_ident::is_xid_continue(c) => {
                        let _: char = hd.lc.next_char().expect("peeked");
                    }
                    _ => break,
                }
            }
            parse_tokens_since_dollar(&mut hd, &mut out)?;
        } else {
            return Err(hd.error(
                "invalid/unsupported syntax after `$` in string template",
            ));
        }
    }

    TemplateWithWhens {
        elements: out,
        allow_recursive_parse: LexicalContextToken {},
    }
    .try_into()
}

pub fn parse_as_subst<O: SubstParseContext>(
    input: syn::LitStr,
) -> syn::Result<Subst<O>> {
    let kw_span = input.span();
    let sd = SD::StringTemplate(
        parse_lit_str(&input)?,
        O::not_in_bool(&kw_span)?,
        O::not_in_paste(&kw_span)?,
        beta::Enabled::new_for_syntax(kw_span)?,
    );
    Ok(Subst { kw_span, sd })
}
