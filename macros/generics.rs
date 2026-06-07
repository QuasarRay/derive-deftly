//! Handling of generics - `$tgens` etc.

use super::framework::*;

use syn::GenericParam as GP;

pub type FilterMode = fn(&syn::GenericParam, &mut TokenAccumulator);

/// Filter mode emitting names only - `$tgnames`
pub fn fm_name(gp: &syn::GenericParam, out: &mut TokenAccumulator) {
    match gp {
        GP::Type(t) => out.append(&t.ident),
        GP::Const(c) => out.append(&c.ident),
        GP::Lifetime(l) => out.append(&l.lifetime),
    }
}

/// Filter mode omitting only the defaults - `$tgens`
pub fn fm_nodefs(gp: &syn::GenericParam, out: &mut TokenAccumulator) {
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

/// Filter mode for including the whole generic - `$tdefgens`
pub fn fm_complete(gp: &syn::GenericParam, out: &mut TokenAccumulator) {
    out.append(gp);
}

fn expand_1(
    pair: syn::punctuated::Pair<&GP, &token::Comma>,
    out: &mut TokenAccumulator,
    composable: WantPunctComposable,
    filter_mode: impl Fn(&syn::GenericParam, &mut TokenAccumulator),
) {
    filter_mode(pair.value(), out);
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
