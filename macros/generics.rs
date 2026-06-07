//! Handling of generics - `$tgens` etc.
//!
//! Methods for handling generics.  Most take `composable: Boole`,
//! which lets us control the trailing comma.  This is desirable
//! because we should include it for expansions like `$tgens` that the
//! user can append things to, but ideally *not* for expansions like
//! `$ttype` that the user can't.

use super::framework::*;

pub fn expand_tgnames(
    ctx: &Context,
    out: &mut TokenAccumulator,
    composable: bool,
) {
    for pair in ctx.top.generics.params.pairs() {
        use syn::GenericParam as GP;
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
        use syn::GenericParam as GP;
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

pub fn expand_tgens(
    ctx: &Context,
    out: &mut TokenAccumulator,
    composable: bool,
) {
        out.append_maybe_punct_composable(
            &ctx.top.generics.params,
            composable,
        );
}
