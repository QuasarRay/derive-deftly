//! Handling of generics and things that use them
//!
//! $tgens` and `$gens_collate` etc. but also
//! `$ttype` and its variants, and `$impl`

use super::framework::*;

//---------- filtering modes ----------

use syn::GenericParam as GP;
use FilterMode as FM;

#[derive(Debug, Copy, Clone)]
pub enum FilterMode {
    /// Filter mode emitting names only - `$tgnames`
    Names,
    /// Filter mode omitting only the defaults - `$tgens`
    NoDefs,
    /// Filter mode for including the whole generic - `$tdefgens`
    Complete,
}

fn expand_1(
    pair: syn::punctuated::Pair<&GP, &token::Comma>,
    out: &mut TokenAccumulator,
    composable: WantPunctComposable,
    filter_mode: FilterMode,
) {
    let gp = pair.value();

    match filter_mode {
        FM::Names => {
            match gp {
                GP::Type(t) => out.append(&t.ident),
                GP::Const(c) => out.append(&c.ident),
                GP::Lifetime(l) => out.append(&l.lifetime),
            }
            //
        }
        FM::NoDefs => {
            let out_attrs = |out: &mut TokenAccumulator, attrs: &[_]| {
                attrs.iter().for_each(|attr| out.append(attr));
            };
            match gp {
                GP::Type(t) => {
                    out_attrs(out, &t.attrs);
                    out.append(&t.ident);
                    out.append(&t.colon_token);
                    out.append(&t.bounds);
                }
                GP::Const(c) => {
                    out_attrs(out, &c.attrs);
                    out.append(&c.const_token);
                    out.append(&c.ident);
                    out.append(&c.colon_token);
                    out.append(&c.ty);
                }
                GP::Lifetime(l) => out.append(&l),
            }
        }
        FM::Complete => {
            out.append(gp);
            //
        }
    }

    out.append_maybe_punct_composable(&pair.punct(), composable);
}

pub fn expand_tgens(
    ctx: &Context,
    out: &mut TokenAccumulator,
    composable: WantPunctComposable,
    filter_mode: FilterMode,
) {
    for pair in ctx.top.generics.params.pairs() {
        expand_1(pair, out, composable, filter_mode);
    }
}

/// Expand the top-level type
///
/// There are three contexts where the top-level type
/// name might occur with generics, and two syntaxes:
///
/// ```text
///   referring to the type    $ttype     Type::<G>
///   impl'ing for the type    $ttype     Type::<G>
///   defining a new type      $tdeftype  Type<G: bounds>
/// ```
///
/// Handles `$ttype` and `$tdeftype`, and, indirectly, `$vtype`
pub fn expand_ttype<'c, O: ExpansionOutput>(
    ctx: &Context<'c>,
    out: &mut O,
    kw_span: Span,
    colons: Option<Token![::]>,
    filter_mode: generics::FilterMode,
) {
    // specify type to help inference:
    let _: generics::FilterMode = filter_mode;
    let gens = &ctx.top.generics;
    let colons = gens.lt_token.and_then(|_| colons);
    out.append_idpath(
        kw_span,
        |_| {},
        &ctx.top.ident,
        |out| {
            out.append(colons);
            out.append(gens.lt_token);
            expand_tgens(
                ctx,
                out,
                WantPunctComposable::AvoidIntroducingTrailingComma,
                filter_mode,
            );
            out.append(gens.gt_token);
        },
        Grouping::Ungrouped,
    )
    .unwrap_or_else(|e| e.unreachable());
}

//---------- $gens_collate etc. ----------

#[derive(Debug)]
pub struct CollateInputs {
    pub inputs: Template<TokenAccumulator>,
}

/// `SubstDetails` payload for `gens_collate` etc.
#[derive(Debug)]
pub struct Collate<O: SubstParseContext> {
    pub inputs: CollateInputs,
    pub filter_mode: FilterMode,
    pub not_in_bool: O::NotInBool,
    allow_tokens: AllowTokens<O>,
    #[allow(dead_code)]
    beta: beta::Enabled,
}

impl<O: SubstParseContext> Collate<O> {
    pub fn parse(
        input: ParseStream,
        filter_mode: FilterMode,
        kw_span: Span,
    ) -> syn::Result<Self> {
        let allow_tokens = O::allow_tokens(&kw_span)?;
        let not_in_bool = O::not_in_bool(&kw_span)?;

        let inputs = input.parse()?;

        let beta = beta::Enabled::new_for_syntax(kw_span)?;

        let inputs = CollateInputs { inputs };
        Ok(Collate {
            inputs,
            filter_mode,
            not_in_bool,
            allow_tokens,
            beta,
        })
    }
}

impl<O: ExpansionOutput> Collate<O> {
    pub fn expand<'c>(
        &self,
        gctx: GeneralContext<'c>,
        out: &mut O,
        kw_span: Span,
    ) -> syn::Result<()> {
        out.append_tokens_with(&self.allow_tokens, |out| {
            self.inputs.expand(gctx, self.filter_mode, out, kw_span);
            Ok(())
        })
    }
}

impl CollateInputs {
    fn expand<'c>(
        &self,
        gctx: GeneralContext<'c>,
        filter_mode: FilterMode,
        out: &mut TokenAccumulator,
        kw_span: Span,
    ) {
        let mut elements: Vec<(GP, Option<token::Comma>)> = vec![];

        let mut ts = TokenAccumulator::new();
        self.inputs.expand(gctx, &mut ts);
        let ts = match ts.tokens() {
            Ok(y) => y,
            Err(e) => {
                out.record_error(e);
                return;
            }
        };

        struct ParamOrEmpty(Option<syn::GenericParam>);
        impl Parse for ParamOrEmpty {
            fn parse(input: ParseStream) -> syn::Result<Self> {
                Ok(ParamOrEmpty(if input.peek(Token![,]) {
                    None
                } else {
                    Some(input.parse()?)
                }))
            }
        }

        let gps = Punctuated::parse_terminated.parse2(ts.clone());
        let gps = match gps {
            Ok(y) => y,
            Err(e) => {
                out.record_error(e);
                out.record_error(kw_span.error(format_args!(
 "failed to parse expanded argument `{}` as generic parameters (for this generics manipulation)",
                        ts,
                    )));
                // That's enough of this, let's not make more errors.
                return;
            }
        };

        elements.extend(gps.into_pairs().filter_map(|pair| {
            let (ParamOrEmpty(value), punct) = pair.into_tuple();
            let value = value?;
            Some((value, punct))
        }));

        elements.sort_by_key(|(value, _)| match value {
            // syn doesn't promise the order is correct, so we have to
            // open-code it like this, which is rather unsatisfactory
            GP::Lifetime { .. } => 0,
            GP::Type { .. } => 1,
            GP::Const { .. } => 2,
        });

        for (value, punct) in &elements {
            use syn::punctuated::Pair;

            expand_1(
                Pair::new(value, punct.as_ref()),
                out,
                WantPunctComposable::Composable,
                filter_mode,
            );
        }
    }
}

impl UniformlyAnalysable for CollateInputs {
    fn impl_analyse<V: Visitor>(&self, v: &mut V) -> Result<(), V::E> {
        self.inputs.analyse(v)
    }
}

//---------- $impl ----------

/// The parsed contents of a `$impl`
///
/// A bit like some of `syn::ItemImpl` but it has
///  - slightly different syntax (`${impl TRAIT [for TYPE]}`
///  - no body
///  - less support for exciting Rust features eg `unsafe`
#[derive(Debug)]
pub struct ImplShorthand {
    /// `${impl < GENERICS > ...}`
    ///
    /// Default is `$tgens`
    pub generics: Option<ImplGenerics>,

    /// `${impl TRAIT ...}`
    ///
    /// Default is inherent impls.
    /// Turns into `TRAIT for` in the expansion.
    pub trait_: Option<TemplateUntilKeyword>,

    /// `${impl ... for TYPE ...}`
    ///
    /// Default is `$ttype`.
    pub self_: Option<TemplateUntilKeyword>,

    /// `${impl ... where WHERES}`
    ///
    /// Default is `$twheres`
    pub where_: Option<(Token![where], TemplateUntilKeyword)>,
}

#[derive(Debug)]
pub struct ImplGenerics {
    pub lt: Token![<],
    pub params: CollateInputs,
    pub gt: Token![>],
}

impl Parse for ImplShorthand {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let generics = if input.peek(Token![<]) {
            let (lt, inputs, gt) = AngleBrackets::parse_bracketed_template(
                input,
                "unmatched < (start of generics) in ${impl ..}",
            )?;
            let params = CollateInputs { inputs };
            Some(ImplGenerics { lt, params, gt })
        } else {
            None
        };

        macro_rules! parse_if_keyword { { $kw:ident } => {
            if input.peek(Token![$kw]) {
                let kw: Token![$kw] = input.parse()?;
                let value = input.parse::<TemplateUntilKeyword>()?;
                Some((kw, value))
            } else {
                None
            }
        } }

        let trait_ = input.parse::<TemplateUntilKeyword>()?;
        let trait_ = if trait_.is_empty() {
            None
        } else {
            Some(trait_)
        };

        let self_ = if let Some((for_, self_)) = parse_if_keyword!(for) {
            if self_.is_empty() {
                Err(for_.error("expected type after `for`"))?;
            }
            Some(self_)
        } else {
            None
        };

        let where_ = parse_if_keyword!(where);

        if !input.is_empty() {
            let kw: IdentAny = input
                .parse()
                .expect("parse_until_keyword should have had this");
            return Err(kw.error(format_args!(
                "bad syntax for $impl: unknown/unrecognised keyword {}",
                *kw,
            )));
        }

        Ok(ImplShorthand {
            generics,
            trait_,
            self_,
            where_,
        })
    }
}

impl ImplShorthand {
    pub fn expand<'c>(
        &self,
        ctx: &'c Context<'c>,
        out: &mut TokenAccumulator,
        kw_span: Span,
    ) {
        let ImplShorthand { generics, trait_, self_, where_ } //_
            = self;
        let gctx = ctx.as_general();

        out.append(Token![impl](kw_span));

        {
            let filter_mode = FilterMode::NoDefs;
            if let Some(ImplGenerics { lt, params, gt }) = generics {
                out.append(lt);
                params.expand(gctx, filter_mode, out, lt.span());
                out.append(gt);
            } else {
                out.append(Token![<](kw_span));
                expand_tgens(
                    ctx,
                    out,
                    WantPunctComposable::AvoidIntroducingTrailingComma,
                    filter_mode,
                );
                out.append(Token![>](kw_span));
            }
        }

        if let Some(trait_) = trait_ {
            trait_.0.expand(gctx, out);
            out.append(Token![for](kw_span));
        }

        if let Some(self_) = self_ {
            self_.0.expand(gctx, out);
        } else {
            expand_ttype(ctx, out, kw_span, None, FilterMode::Names);
        }

        if let Some((where_kw, where_)) = where_ {
            out.append(where_kw);
            where_.0.expand(gctx, out);
        } else if let Some(clause) = &ctx.top.generics.where_clause {
            // note similar code in `$twheres` in expand.rs
            out.append(clause.where_token);
            out.with_tokens(|out| {
                clause.predicates.to_tokens_punct_composable(out);
            });
        } else {
            out.append(Token![where](kw_span));
        }
    }
}

impl UniformlyAnalysable for ImplShorthand {
    fn impl_analyse<V: Visitor>(&self, v: &mut V) -> Result<(), V::E> {
        let ImplShorthand { generics, trait_, self_, where_ } //_
            = self;

        if let Some(ImplGenerics { lt: _, params, gt: _ }) //_
            = generics
        {
            params.analyse(v)?;
        }

        trait_.analyse(v)?;
        self_.analyse(v)?;

        if let Some((_kw, where_)) = where_ {
            where_.analyse(v)?;
        }

        Ok(())
    }
}

//----- TemplateUntilKeyword -----

#[derive(Debug)]
pub struct TemplateUntilKeyword(Template<TokenAccumulator>);

struct StopAtKeyword;

impl SpecialParseContext for StopAtKeyword {
    fn before_element_hook(
        &mut self,
        input: ParseStream,
    ) -> syn::Result<Option<SpecialInstructions>> {
        let la = input.lookahead1();
        if la.peek(syn::Ident) {
            // ordinary identifier, so not a keyword
            return Ok(None);
        }

        if !la.peek(syn::Ident::peek_any) {
            // not in identifier syntax, so not a keyword
            return Ok(None);
        }

        macro_rules! allow_keywords { { $($kw:ident)* } => { $(
            if la.peek(Token![$kw]) {
                return Ok(None)
            }
        )* } }
        allow_keywords! {
            // used in paths
            self super crate
            // used in types we might be implementing some trait of ours for
            dyn fn mut Self
        }

        // is a keyword, stop parsing
        Ok(Some(SpecialInstructions::EndOfTemplate))
    }
}

impl Parse for TemplateUntilKeyword {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut t = Template::parse_special(input, &mut StopAtKeyword)?;

        // Respan the delimiters of (...) groups, since the user might
        // have needed them for *our* parser, even though Rust's parser
        // is OK without them.
        for te in &mut t.elements {
            match te {
                TE::Group {
                    delimiter: Delimiter::Parenthesis,
                    delim_span,
                    ..
                } => {
                    *delim_span = Span::call_site();
                }
                _other => {}
            }
        }
        Ok(TemplateUntilKeyword(t))
    }
}

impl TemplateUntilKeyword {
    fn is_empty(&self) -> bool {
        self.0.elements.is_empty()
    }
}

impl UniformlyAnalysable for TemplateUntilKeyword {
    fn impl_analyse<V: Visitor>(&self, v: &mut V) -> Result<(), V::E> {
        self.0.analyse(v)
    }
}
