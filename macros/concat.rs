//! Implementation of string concatenation `${concat }`

use super::framework::*;
use string_template::LexicalContext;

/// Accumulator for string concatenation `${concat }`
///
/// Implements [`ExpansionOutput`] and [`SubstParseContext`]:
/// i.e., it corresponds to the lexical context for a `${concat }`,
/// and collects the string to be output.
#[derive(Debug)]
pub struct Accumulator<L: LexicalContext = ()> {
    def_span: Span,
    out_span: Option<Span>,
    text: String,
    errors: ErrorAccumulator,
    lexical_context: PhantomData<L>,
}

impl<L: LexicalContext> Template<Accumulator<L>> {
    pub fn expand_to_lit<'c>(
        &self,
        kw_span: Span,
        gctx: GeneralContext<'c>,
    ) -> syn::Result<syn::LitStr> {
        let mut accum = concat::Accumulator::new_with_span(kw_span);
        self.expand(gctx, &mut accum);
        accum.finish_literal()
    }

    pub fn expand_onto<'c, O: ExpansionOutput>(
        &self,
        kw_span: Span,
        gctx: GeneralContext<'c>,
        np: &O::NotInPaste,
        out: &mut O,
    ) {
        // We *would* support ${paste ${concat ...}} but it would be weird.
        let _: O::NotInPaste = *np;

        match self.expand_to_lit(kw_span, gctx) {
            Ok(y) => out.append_syn_litstr(&y),
            Err(e) => out.record_error(e),
        }
    }
}

impl<L: LexicalContext> Accumulator<L> {
    fn finish_literal(self) -> syn::Result<syn::LitStr> {
        let out_span = self.out_span.unwrap_or(self.def_span);
        let text = self.errors.finish_with(self.text)?;
        let lit = syn::LitStr::new(&text, out_span);
        Ok(lit)
    }

    fn append_display(&mut self, span: Span, plain: impl Display + Spanned) {
        self.out_span.get_or_insert(span);
        write!(&mut self.text, "{}", plain).expect("Display onto String")
    }

    /// Convert a type to a "nice" string
    ///
    /// `syn::Type as ToTokens` produces a lot of `Spacing::Alone`
    /// which results in a lot of excessive whitespace.
    ///
    /// We use an algorithm which prints something reasonably pretty
    /// which we also think is probably parseable
    /// unless there `None`-delimited groups (which we make visible).
    ///
    /// Note that our documentation doesn't promise parseability,
    /// just readability.
    ///
    /// This algorithm *does* obviously produce parseable output
    /// if the input is just a single identifier token.
    ///
    /// ### Rules
    ///
    ///  * We never put spaces next to any of `<>:`
    ///  * We don't put a space after `&` or `'`
    ///  * We don't put a space before `,`
    ///  * We put spaces just inside `{ }` iff nonempty
    ///  * We don't put spaces just inside `( )` or `[ ]` (or `« »`)
    ///  * Otherwise, we put a space between each pair of TTs
    fn append_type_like(&mut self, tokens: TokenStream) {
        recurse_display_type_like(
            &mut self.text,
            &mut Prev::NeverSpace,
            tokens,
        )
        .expect("formatting type-like tokens into String failed");
    }
}

/// State (previous token) for `recurse_display_type_like`
enum Prev {
    /// Previous token wants to suppress any following space
    NeverSpace,
    /// We're at the start of a `{ }` group
    OpenBrace,
    /// Insert a space unless the *next* token wants to suppress it.
    Other,
}

/// Implementation of `append_type_like`
fn recurse_display_type_like(
    out: &mut String,
    prev: &mut Prev,
    tokens: TokenStream,
) -> fmt::Result {
    for tt in tokens {
        let def_spc = match &prev {
            Prev::NeverSpace => "",
            _other => " ",
        };

        let wr; // forces every arm to write something
        let cat; // next value of prev, forces every arm to set it
        match tt {
            TT::Punct(p) => {
                let p = p.as_char();
                match p {
                    '<' | '>' | ':' => {
                        wr = write!(out, "{}", p);
                        cat = Prev::NeverSpace;
                    }
                    '&' | '\'' => {
                        wr = write!(out, "{}{}", def_spc, p);
                        cat = Prev::NeverSpace;
                    }
                    ',' => {
                        wr = write!(out, "{}", p);
                        cat = Prev::Other;
                    }
                    other => {
                        wr = write!(out, "{}{}", def_spc, other);
                        cat = Prev::Other;
                    }
                }
            }
            TT::Group(g) => {
                let delim = g.delimiter();
                let mut normal = |open, close| {
                    write!(out, "{}{}", def_spc, open)?;
                    recurse_display_type_like(
                        out,
                        &mut Prev::NeverSpace,
                        g.stream(),
                    )?;
                    write!(out, "{}", close)
                };
                match delim {
                    Delimiter::Brace => {
                        write!(out, "{}{{", def_spc)?;
                        let mut inner_prev = Prev::OpenBrace;
                        recurse_display_type_like(
                            out,
                            &mut inner_prev,
                            g.stream(),
                        )?;
                        wr = match inner_prev {
                            Prev::OpenBrace => write!(out, "}}"),
                            _other => write!(out, " }}"),
                        };
                        cat = Prev::Other;
                    }
                    Delimiter::Parenthesis => {
                        wr = normal('(', ')');
                        cat = Prev::Other;
                    }
                    Delimiter::Bracket => {
                        wr = normal('[', ']');
                        cat = Prev::Other;
                    }
                    Delimiter::None => {
                        wr = normal('«', '»');
                        cat = Prev::Other;
                    }
                }
            }
            other => {
                wr = write!(out, "{}{}", def_spc, other);
                cat = Prev::Other;
            }
        }

        let () = wr?;
        *prev = cat;
    }
    Ok(())
}

impl<L: LexicalContext> SubstParseContext for Accumulator<L> {
    type NotInPaste = ();
    type NotInConcat = Void;
    type NotInBool = ();
    type BoolOnly = Void;
    type ConcatOnly = ();
    type DbgContent = Template<Self>;
    type SpecialParseContext = ();
    type AllowTemplate = L;

    fn not_in_paste(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn not_in_concat(span: &impl Spanned) -> syn::Result<Void> {
        Err(span.error("not allowed in within ${concat ...}"))
    }
    fn not_in_bool(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn concat_only(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
}

impl<L: LexicalContext> ExpansionOutput for Accumulator<L> {
    fn append_identfrag_toks<I: IdentFrag>(
        &mut self,
        ident: &I,
    ) -> Result<(), I::BadIdent> {
        self.append_display(ident.span(), ident.fragment());
        Ok(())
    }
    fn append_idpath<A, B, I>(
        &mut self,
        _te_span: Span,
        pre: A,
        ident: &I,
        post: B,
        _grouping: Grouping,
    ) -> Result<(), I::BadIdent>
    where
        A: FnOnce(&mut TokenAccumulator),
        B: FnOnce(&mut TokenAccumulator),
        I: IdentFrag,
    {
        match (|| {
            let mut ta = TokenAccumulator::new();
            pre(&mut ta);
            ta.with_tokens(|ts| ident.frag_to_tokens(ts));
            post(&mut ta);
            ta.tokens()
        })() {
            Ok(y) => self.append_type_like(y),
            Err(e) => self.errors.push(e),
        }
        Ok(())
    }
    fn append_syn_litstr(&mut self, lit: &syn::LitStr) {
        if lit.suffix() != "" {
            self.errors
                .push(lit.error("string suffix forbidden in ${concat }"));
            return;
        }
        self.append_display(lit.span(), lit.value());
    }
    fn append_syn_type(
        &mut self,
        te_span: Span,
        v: syn::Type,
        grouping: Grouping,
    ) {
        self.append_syn_type_inner(te_span, v, grouping);
    }
    fn append_syn_type_inner(
        &mut self,
        _te_span: Span,
        ty: syn::Type,
        _grouping: Grouping,
    ) {
        self.append_type_like(ty.to_token_stream());
    }
    fn dbg_expand<'c>(
        &mut self,
        kw_span: Span,
        ctx: GeneralContext<'c>,
        msg: &mut String,
        content: &Template<Accumulator<L>>,
    ) -> fmt::Result {
        match content.expand_to_lit(kw_span, ctx) {
            Ok(y) => {
                write!(msg, "{:?}", y.value())?;
                self.append_syn_litstr(&y);
            }
            Err(e) => {
                write!(msg, "/* ERROR: {} */", e)?;
                self.record_error(e);
            }
        }
        Ok(())
    }

    fn new_with_span(kw_span: Span) -> Self {
        Accumulator {
            def_span: kw_span,
            out_span: None,
            text: String::new(),
            errors: ErrorAccumulator::default(),
            lexical_context: PhantomData,
        }
    }
    fn ignore_impl(self) -> syn::Result<()> {
        self.errors.finish()
    }

    fn append_tokens_with(
        &mut self,
        (_not_in_paste, not_in_concat): &((), Void),
        _: impl FnOnce(&mut TokenAccumulator) -> syn::Result<()>,
    ) -> syn::Result<()> {
        void::unreachable(*not_in_concat)
    }

    fn append_bool_only(&mut self, bool_only: &Self::BoolOnly) -> ! {
        void::unreachable(*bool_only)
    }

    fn record_error(&mut self, err: syn::Error) {
        self.errors.push(err);
    }
}

impl<L: LexicalContext> Expand<Accumulator<L>>
    for TemplateElement<Accumulator<L>>
{
    fn expand<'c>(
        &self,
        ctx: GeneralContext<'c>,
        out: &mut Accumulator<L>,
    ) -> syn::Result<()> {
        match self {
            TE::Ident(_ident, not_in_concat) => {
                // We *would* support ${concat ident}
                // but it would be confusing.
                void::unreachable(*not_in_concat);
            }
            TE::LitStr(lit) => out.append_syn_litstr(&lit),
            TE::Subst(e) => e.expand(ctx, out)?,
            TE::Repeat(e) => e.expand(ctx, out),
            TE::Literal(_, allow_tokens)
            | TE::Punct(_, allow_tokens)
            | TE::Group { allow_tokens, .. } => {
                void::unreachable(allow_tokens.1)
            }
        }
        Ok(())
    }
}
