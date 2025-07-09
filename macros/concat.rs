//! Implementation of string concatenation `${concat }`

use super::framework::*;

/// Accumulator for string concatenation `${concat }`
///
/// Implements [`ExpansionOutput`] and [`SubstParseContext`]:
/// i.e., it corresponds to the lexical context for a `${concat }`,
/// and collects the string to be output.
#[derive(Debug)]
pub struct Accumulator {
    kw_span: Span,
    text: String,
    errors: ErrorAccumulator,
}

impl Accumulator {
    pub fn finish_onto<O>(self, np: &O::NotInPaste, out: &mut O)
    where
        O: ExpansionOutput,
    {
        // We *would* support ${paste ${concat ...}} but it would be weird.
        let _: O::NotInPaste = *np;

        (|| {
            let text = self.errors.finish_with(self.text)?;
            let lit = syn::LitStr::new(&text, self.kw_span);
            out.append_syn_litstr(&lit);
            Ok(())
        })()
        .unwrap_or_else(|err| out.record_error(err))
    }

    fn append_display(&mut self, plain: impl Display) {
        write!(&mut self.text, "{}", plain).expect("Display onto String")
    }
}

impl SubstParseContext for Accumulator {
    type NotInPaste = ();
    type NotInConcat = Void;
    type NotInBool = ();
    type BoolOnly = Void;
    type DbgContent = Template<Self>;
    type SpecialParseContext = ();

    fn not_in_paste(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn not_in_concat(span: &impl Spanned) -> syn::Result<Void> {
        Err(span.error("not allowed in within ${concat ...}"))
    }
    fn not_in_bool(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn meta_recog_usage() -> meta::Usage {
        meta::Usage::Value
    }
}

impl ExpansionOutput for Accumulator {
    fn append_identfrag_toks<I: IdentFrag>(
        &mut self,
        ident: &I,
    ) -> Result<(), I::BadIdent> {
        self.append_display(ident.fragment());
        Ok(())
    }
    fn append_idpath<A, B, I>(
        &mut self,
        _te_span: Span,
        _pre: A,
        _ident: &I,
        _post: B,
        grouping: Grouping,
    ) -> Result<(), I::BadIdent>
    where
        A: FnOnce(&mut TokenAccumulator),
        B: FnOnce(&mut TokenAccumulator),
        I: IdentFrag,
    {
        todo!() // XXXX
    }
    fn append_syn_litstr(&mut self, lit: &syn::LitStr) {
        self.append_display(lit.value());
    }
    fn append_syn_type_inner(
        &mut self,
        _te_span: Span,
        ty: syn::Type,
        grouping: Grouping,
    ) {
        todo!() // XXXX
    }
    fn dbg_expand(
        &mut self,
        kw_span: Span,
        ctx: &Context,
        msg: &mut String,
        content: &Template<Accumulator>,
    ) -> fmt::Result {
        let mut child = Accumulator::new_with_span(kw_span);
        content.expand(ctx, &mut child);
        match child.errors.examine() {
            Some(e) => write!(msg, "/* ERROR: {} */", e)?,
            None => write!(msg, "{}", child.text)?,
        }
        child.finish_onto(&(), self);
        Ok(())
    }

    fn new_with_span(kw_span: Span) -> Self {
        Accumulator {
            kw_span,
            text: String::new(),
            errors: ErrorAccumulator::default(),
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

impl Expand<Accumulator> for TemplateElement<Accumulator> {
    fn expand(&self, ctx: &Context, out: &mut Accumulator) -> syn::Result<()> {
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
