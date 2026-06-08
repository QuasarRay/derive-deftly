//! Handling of generics - `$tgens` etc.

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
