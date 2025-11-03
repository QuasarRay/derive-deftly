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
    type NotInConcat = ();
    type NotInBool = Void;
    type ConcatOnly = Void;
    type BoolOnly = ();
    const IS_BOOL: bool = true;
    type DbgContent = Subst<BooleanContext>;

    fn not_in_paste(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn not_in_concat(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn bool_only(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn meta_recog_usage() -> meta::Usage {
        meta::Usage::BoolOnly
    }

    fn not_in_bool(span: &impl Spanned) -> syn::Result<Void> {
        Err(span.error(
            "derive-deftly construct is an expansion - not valid in a condition",
        ))
    }

    type SpecialParseContext = ();

    fn missing_keyword_arguments(kw_span: Span) -> syn::Result<Void> {
        Err(kw_span.error("missing parameters to condition"))
    }
}

impl Subst<BooleanContext> {
    pub fn eval_bool<'c>(
        &self,
        gctx: GeneralContext!('c),
    ) -> syn::Result<bool> {
        // eprintln!("@@@@@@@@@@@@@@@@@@@@ EVAL {:?}", self);
        let kw_span = self.kw_span;

        let ctx = gctx.full_ctx(kw_span);

        let v_fields = || ctx?.variant(&kw_span).map(|v| &v.fields);
        use syn::Fields as SF;

        let expand = |x: &Template<_>| {
            let mut out = TokenAccumulator::new();
            x.expand(gctx, &mut out);
            let out = out.tokens()?;
            Ok::<TokenStream, syn::Error>(out)
        };

        let r = match &self.sd {
            SD::is_enum(..) => ctx?.is_enum(),
            SD::is_struct(..) => matches!(ctx?.top.data, syn::Data::Struct(_)),
            SD::is_union(..) => matches!(ctx?.top.data, syn::Data::Union(_)),
            SD::v_is_unit(..) => matches!(v_fields()?, SF::Unit),
            SD::v_is_tuple(..) => matches!(v_fields()?, SF::Unnamed(..)),
            SD::v_is_named(..) => matches!(v_fields()?, SF::Named(..)),
            SD::tgens(_) => !ctx?.top.generics.params.is_empty(),

            SD::Xmeta(sm) => {
                let meta::SubstMeta {
                    desig: meta::Desig { label, scope: _ },
                    as_,
                    default,
                } = sm;
                match default {
                    None => {}
                    Some((_, nb, _)) => void::unreachable(*nb),
                };
                use meta::SubstAs as mSA;
                if let Some(as_) = as_ {
                    match as_ {
                        mSA::expr(nb, ..)
                        | mSA::ident(nb, ..)
                        | mSA::items(nb, ..)
                        | mSA::path(nb)
                        | mSA::str(nb)
                        | mSA::token_stream(nb, ..)
                        | mSA::ty(nb) => void::unreachable(*nb),
                    }
                };
                is_found(label.search_eval_bool(sm.pmetas(ctx?, kw_span)?))
            }

            SD::is_empty(_, content) => expand(content)?.is_empty(),
            SD::approx_equal(_, [a, b]) => {
                tokens_cmpeq(expand(a)?, expand(b)?, kw_span)?
                    == Equality::Equal
            }

            SD::UserDefined(name) => name.lookup_eval_bool(gctx)?,

            SD::False(..) => false,
            SD::True(..) => true,

            SD::not(v, _) => !v.eval_bool(gctx)?,
            SD::any(vs, _) => vs
                .iter()
                .find_map(|v| match v.eval_bool(gctx) {
                    Ok(true) => Some(Ok(true)),
                    Err(e) => Some(Err(e)),
                    Ok(false) => None,
                })
                .unwrap_or(Ok(false))?,
            SD::all(vs, _) => vs
                .iter()
                .find_map(|v| match v.eval_bool(gctx) {
                    Ok(true) => None,
                    Err(e) => Some(Err(e)),
                    Ok(false) => Some(Ok(false)),
                })
                .unwrap_or(Ok(true))?,

            SD::Vis(vis, _) => match vis.syn_vis(ctx?, kw_span)? {
                syn::Visibility::Public(_) => true,
                _ => false,
            },

            SD::dbg(ddr) => {
                let r = ddr.content_parsed.eval_bool(gctx);
                let () = &ddr.content_string;
                let w = |s: fmt::Arguments| {
                    eprintln!(
                        "derive-deftly dbg condition {} evaluated to {}",
                        ddr.display_heading(gctx),
                        s,
                    )
                };
                match &r {
                    Ok(y) => w(format_args!("{:?}", y)),
                    Err(e) => w(format_args!("error: {}", e)),
                };
                r?
            }

            // ## maint/check-keywords-documented NotInBool ##
            SD::tname(not_in_bool)
            | SD::ttype(not_in_bool)
            | SD::tdeftype(not_in_bool)
            | SD::vname(not_in_bool)
            | SD::fname(not_in_bool)
            | SD::ftype(not_in_bool)
            | SD::vtype(_, _, not_in_bool)
            | SD::tdefkwd(not_in_bool)
            | SD::vindex(not_in_bool, ..)
            | SD::findex(not_in_bool, ..)
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
            | SD::paste_spanned(_, _, not_in_bool, ..)
            | SD::ChangeCase(.., paste::ChangeCaseOkIn { not_in_bool, .. })
            | SD::concat(_, not_in_bool, ..)
            | SD::when(_, not_in_bool)
            | SD::define(_, not_in_bool)
            | SD::defcond(_, not_in_bool)
            | SD::For(_, not_in_bool)
            | SD::If(_, not_in_bool)
            | SD::select1(_, not_in_bool)
            | SD::ignore(_, not_in_bool)
            | SD::error(_, not_in_bool)
            | SD::dbg_all_keywords(not_in_bool)
            | SD::Crate(_, not_in_bool) => void::unreachable(*not_in_bool),
        };
        Ok(r)
    }
}

impl DefinitionName {
    fn lookup_eval_bool<'c>(
        &self,
        ctx: GeneralContext!('c),
    ) -> syn::Result<bool> {
        let ctx = ctx.reduce_lifetime();

        let (def, ctx) = ctx.find_definition::<DefCondBody>(self)?
            .ok_or_else(|| {
                let mut error = self.error(format!(
                    "user-defined condition `{}` not found",
                    self,
                ));
                if let Some(def) = ctx.defs().defs
                    .find_raw::<DefinitionBody>(self)
                {
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
