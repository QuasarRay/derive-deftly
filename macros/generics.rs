//! Handling of generics - `$tgens` etc., `$gens_collate`, etc.

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

impl AnalyseRepeat for CollateInputs {
    fn analyse_repeat(
        &self,
        visitor: &mut RepeatAnalysisVisitor,
    ) -> syn::Result<()> {
        self.inputs.analyse_repeat(visitor)
    }
}

impl meta::FindRecogMetas for CollateInputs {
    fn find_recog_metas(&self, acc: &mut meta::Recognised) {
        self.inputs.find_recog_metas(acc);
    }
}
