//! Handling of generics - `$tgens` etc., `$gens_collate`, etc.

use super::framework::*;

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

#[derive(Debug)]
pub struct CollateDetails {
    filter_mode: FilterMode,
    pub inputs: Vec<Template<TokenAccumulator>>,
}

/// `SubstDetails` payload for `gens_collate` etc.
#[derive(Debug)]
pub struct Collate<O: SubstParseContext> {
    pub details: CollateDetails,
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

        let mut inputs = vec![];
        while !input.is_empty() {
            let Argument(ie) = input.parse()?;
            inputs.push(ie);
        }

        let beta = beta::Enabled::new_for_syntax(kw_span)?;

        let details = CollateDetails {
            filter_mode,
            inputs,
        };
        Ok(Collate {
            details,
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
            self.details.expand(gctx, out, kw_span);
            Ok(())
        })
    }
}

impl CollateDetails {
    fn expand<'c>(
        &self,
        gctx: GeneralContext<'c>,
        out: &mut TokenAccumulator,
        kw_span: Span,
    ) {
        let mut elements = vec![];

        for ie in &self.inputs {
            let mut ts = TokenAccumulator::new();
            ie.expand(gctx, &mut ts);
            let ts = match ts.tokens() {
                Ok(y) => y,
                Err(e) => {
                    out.record_error(e);
                    continue;
                }
            };
            let gps = Punctuated::parse_terminated.parse2(ts.clone());
            let gps = match gps {
                Ok(y) => y,
                Err(e) => {
                    out.record_error(e);
                    out.record_error(kw_span.error(format_args!(
 "failed to parse expanded argument `{ts}` as generic parameters (for this generics manipulation)"
                    )));
                    // That's enough of this, let's not make more errors.
                    return;
                }
            };

            elements.extend(gps.into_pairs());
        }

        elements.sort_by_key(|pair| match pair.value() {
            // syn doesn't promise the order is correct, so we have to
            // open-code it like this, which is rather unsatisfactory
            GP::Lifetime { .. } => 0,
            GP::Type { .. } => 1,
            GP::Const { .. } => 2,
        });

        for pair in &elements {
            use syn::punctuated::Pair;

            let pair = match pair {
                Pair::Punctuated(t, p) => Pair::Punctuated(t, p),
                Pair::End(t) => Pair::End(t),
            };

            expand_1(
                pair,
                out,
                WantPunctComposable::Composable,
                self.filter_mode,
            );
        }
    }
}
