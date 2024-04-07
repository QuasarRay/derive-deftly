//! Handling of boolean evaluation (conditions)

use super::framework::*;

/// Implementor of [`SubstParseContext`] for booleans
///
/// No values of this type are ever created -
/// it's just a generic parameter, used to select the associated
/// marker types (and their constructor methods( in SubstParseContext.
///
/// So it can be uninhabited.
#[derive(Debug)]
pub struct BooleanContext(Void);

pub struct Found;

fn is_found(r: Result<(), Found>) -> bool {
    r.is_err()
}

impl SubstParseContext for BooleanContext {
    type NotInPaste = ();
    type NotInBool = Void;
    type BoolOnly = ();
    fn not_in_paste(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn bool_only(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn meta_recog_allow() -> meta::RecogAllow {
        meta::RecogAllow::BoolOnly
    }

    fn not_in_bool(span: &impl Spanned) -> syn::Result<Void> {
        Err(span.error(
            "derive-deftly construct is an expansion - not valid in a condition",
        ))
    }

    type SpecialParseContext = ();
}

impl Subst<BooleanContext> {
    pub fn eval_bool(&self, ctx: &Context) -> syn::Result<bool> {
        // eprintln!("@@@@@@@@@@@@@@@@@@@@ EVAL {:?}", self);
        let kw_span = self.kw_span;

        let v_fields = || ctx.variant(&kw_span).map(|v| &v.fields);
        use syn::Fields as SF;

        let expand = |x: &Template<_>| {
            let mut out = TokenAccumulator::new();
            x.expand(ctx, &mut out);
            let out = out.tokens()?;
            Ok::<TokenStream, syn::Error>(out)
        };

        let r = match &self.sd {
            SD::is_enum(..) => ctx.is_enum(),
            SD::is_struct(..) => matches!(ctx.top.data, syn::Data::Struct(_)),
            SD::is_union(..) => matches!(ctx.top.data, syn::Data::Union(_)),
            SD::v_is_unit(..) => matches!(v_fields()?, SF::Unit),
            SD::v_is_tuple(..) => matches!(v_fields()?, SF::Unnamed(..)),
            SD::v_is_named(..) => matches!(v_fields()?, SF::Named(..)),
            SD::tgens(_) => !ctx.top.generics.params.is_empty(),

            SD::Xmeta(sm) => {
                let meta::SubstMeta {
                    desig: meta::Desig { label, scope: _ },
                    as_,
                } = sm;
                use meta::SubstAs as MSA;
                if let Some(as_) = as_ {
                    match as_ {
                        MSA::str(nb) | MSA::tokens(nb, ..) | MSA::ty(nb) => {
                            void::unreachable(*nb)
                        }
                    }
                };
                is_found(label.search_eval_bool(sm.pmetas(ctx, kw_span)?))
            }

            SD::is_empty(_, content) => expand(content)?.is_empty(),
            SD::approx_equal(_, [a, b]) => {
                tokens_cmp(expand(a)?, expand(b)?) == Ordering::Equal
            }

            SD::UserDefined(name) => name.lookup_eval_bool(ctx)?,

            SD::False(..) => false,
            SD::True(..) => true,

            SD::not(v, _) => !v.eval_bool(ctx)?,
            SD::any(vs, _) => vs
                .iter()
                .find_map(|v| match v.eval_bool(ctx) {
                    Ok(true) => Some(Ok(true)),
                    Err(e) => Some(Err(e)),
                    Ok(false) => None,
                })
                .unwrap_or(Ok(false))?,
            SD::all(vs, _) => vs
                .iter()
                .find_map(|v| match v.eval_bool(ctx) {
                    Ok(true) => None,
                    Err(e) => Some(Err(e)),
                    Ok(false) => Some(Ok(false)),
                })
                .unwrap_or(Ok(true))?,

            SD::Vis(vis, _) => match vis.syn_vis(ctx, kw_span)? {
                syn::Visibility::Public(_) => true,
                _ => false,
            },

            // ## maint/check-keywords-documented NotInBool ##
            SD::tname(not_in_bool)
            | SD::ttype(not_in_bool)
            | SD::tdeftype(not_in_bool)
            | SD::vname(not_in_bool)
            | SD::fname(not_in_bool)
            | SD::ftype(not_in_bool)
            | SD::vtype(_, _, not_in_bool)
            | SD::tdefkwd(not_in_bool)
            | SD::tattrs(_, _, not_in_bool)
            | SD::vattrs(_, _, not_in_bool)
            | SD::fattrs(_, _, not_in_bool)
            | SD::tdefgens(_, not_in_bool)
            | SD::tgnames(_, not_in_bool)
            | SD::twheres(_, not_in_bool)
            | SD::vpat(_, _, not_in_bool)
            | SD::fpatname(not_in_bool)
            | SD::tdefvariants(_, _, not_in_bool)
            | SD::fdefine(_, _, not_in_bool)
            | SD::vdefbody(_, _, _, not_in_bool)
            | SD::paste(_, not_in_bool)
            | SD::ChangeCase(_, _, not_in_bool)
            | SD::when(_, not_in_bool)
            | SD::define(_, not_in_bool)
            | SD::defcond(_, not_in_bool)
            | SD::For(_, not_in_bool)
            | SD::If(_, not_in_bool)
            | SD::select1(_, not_in_bool)
            | SD::error(_, not_in_bool)
            | SD::dbg_all_keywords(not_in_bool)
            | SD::Crate(_, not_in_bool) => void::unreachable(*not_in_bool),
        };
        Ok(r)
    }
}

impl DefinitionName {
    fn lookup_eval_bool(&self, ctx: &Context<'_>) -> syn::Result<bool> {
        let (def, ctx) = ctx.find_definition::<DefCondBody>(self)?.ok_or_else(|| {
            let mut error = self.error("user-defined condition not fund");
            if let Some(def) = ctx.definitions.find_raw::<DefinitionBody>(self) {
                // Condition syntax looks like fine tokens,
                // so the ${define } wouldn't spot this mistake.
                error.combine(
                    def.name.error(
"this user-defined expansion used as a condition (perhaps you meant ${defcond ?}"
                    )
                );
            }
            error
        })?;

        def.body.eval_bool(&ctx)
    }
}

/// Compares two `TokenStream`s for "equivalence"
///
/// We intend that two `TokenStream`s count as "equivalent"
/// if they mean the same thing to the compiler,
/// modulo any differences in spans.
///
/// We also disregard spacing.  This is not 100% justifiable but
/// I think there are no token sequences differing only in spacing
/// which are *both* valid and which differ in meaning.
///
/// ### Why ?!
///
/// `< <` and `<<` demonstrate that it is not possible to provide
/// a fully correct and coherent equality function on Rust tokens,
/// without knowing the parsing context:
///
/// In places where `<<` is a shift operator, `< <` is not legal.
/// But in places where `<<` introduces two lots of generics,
/// `<<` means the same.
///
/// I think a function which treats `< <` and `<<` as equal is more useful
/// than one that doesn't, because it will DTRT for types.
//
// Comparing for equality has to be done by steam.
// And a lot of stringification.
pub fn tokens_cmp(a: TokenStream, b: TokenStream) -> cmp::Ordering {
    use proc_macro2::Group;

    fn tt_cmp(a: TokenTree, b: TokenTree) -> Ordering {
        let discrim = |tt: &_| match tt {
            TT::Punct(_) => 0,
            TT::Literal(_) => 1,
            TT::Ident(_) => 2,
            TT::Group(_) => 3,
        };

        discrim(&a).cmp(&discrim(&b)).then_with(|| match (a, b) {
            (TT::Group(a), TT::Group(b)) => group_cmp(a, b),
            (l, r) => l.to_string().cmp(&r.to_string()),
        })
    }

    fn group_cmp(a: Group, b: Group) -> Ordering {
        let delim = |g: &Group| {
            proc_macro2::Group::new(g.delimiter(), TokenStream::new())
                .to_string()
        };
        delim(&a)
            .cmp(&delim(&b))
            .then_with(|| tokens_cmp(a.stream(), b.stream()))
    }

    for ab in a.into_iter().zip_longest(b) {
        let (a, b) = match ab {
            EitherOrBoth::Both(a, b) => (a, b),
            EitherOrBoth::Left(_) => return Ordering::Greater,
            EitherOrBoth::Right(_) => return Ordering::Less,
        };
        match tt_cmp(a, b) {
            Ordering::Equal => {}
            neq => return neq,
        }
    }
    return Ordering::Equal;
}
