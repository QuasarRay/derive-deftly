//! Handling of generics - `$tgens` etc.

use super::framework::*;

use syn::GenericParam as GP;

pub fn expand_tgens_names(
    ctx: &Context,
    out: &mut TokenAccumulator,
    composable: WantPunctComposable,
) {
    for pair in ctx.top.generics.params.pairs() {
        match pair.value() {
            GP::Type(t) => out.append(&t.ident),
            GP::Const(c) => out.append(&c.ident),
            GP::Lifetime(l) => out.append(&l.lifetime),
        }
        out.append_maybe_punct_composable(&pair.punct(), composable);
    }
}

pub fn expand_tgens_nodefs(
    ctx: &Context,
    out: &mut TokenAccumulator,
) {
    for pair in ctx.top.generics.params.pairs() {
        let out_attrs = |out: &mut TokenAccumulator, attrs: &[_]| {
            attrs.iter().for_each(|attr| out.append(attr));
        };
        match pair.value() {
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
        out.with_tokens(|out| {
            pair.punct().to_tokens_punct_composable(out);
        });
    }
}

pub fn expand_tgens_complete(
    ctx: &Context,
    out: &mut TokenAccumulator,
    composable: WantPunctComposable,
) {
    for pair in ctx.top.generics.params.pairs() {
        out.append(pair.value());
        out.append_maybe_punct_composable(&pair.punct(), composable);
    }
}
