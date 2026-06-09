//! Handling of repetition, and also useful methods on context

use super::framework::*;

pub const NESTING_LIMIT: u16 = 100;

// TODO hopefully remove these self::'s rust-lang/rust#153730
pub use self::RepeatOver as RO;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Display)]
#[strum(serialize_all = "snake_case")]
pub enum RepeatOver {
    Variants,
    Fields,
}

#[derive(Debug, Clone)]
struct RepeatOverInference {
    over: RepeatOver,
    span: Span,
}

#[derive(Default, Debug, Clone)]
pub struct RepeatAnalysisVisitor {
    over: Option<RepeatOverInference>,
}

pub trait AnalyseRepeat {
    fn analyse_repeat(
        &self,
        visitor: &mut RepeatAnalysisVisitor,
    ) -> syn::Result<()>;
}

/// What would `${fname}` expand to?  As type from [`syn`].
///
/// Implements [`quote::IdentFragment`] and [`ToTokens`].
#[derive(Debug, Eq, PartialEq)]
pub enum Fname<'r> {
    Name(&'r syn::Ident),
    Index(syn::Index),
}

impl RepeatAnalysisVisitor {
    fn set_over(&mut self, over: RepeatOverInference) -> syn::Result<()> {
        match &self.over {
            None => self.over = Some(over),
            Some(already) => {
                if already.over != over.over {
                    let mut e1 = already.span.error(format_args!(
 "inconsistent repetition depth: firstly, {} inferred here",
                            already.over,
                        ));
                    let e2 = over.span.error(format_args!(
 "inconsistent repetition depth: secondly, {} inferred here",
                            over.over,
                        ));
                    e1.combine(e2);
                    return Err(e1);
                }
            }
        }
        Ok(())
    }

    pub fn finish(self, start: DelimSpan) -> Result<RepeatOver, syn::Error> {
        Ok(self
            .over
            .ok_or_else(|| {
                start.error(
            "no contained expansion field determined what to repeat here"
        )
            })?
            .over)
    }
}

//---------- AnalyseRepeat impls ----------

impl<O: SubstParseContext> AnalyseRepeat for SubstIf<O> {
    fn analyse_repeat(
        &self,
        visitor: &mut RepeatAnalysisVisitor,
    ) -> syn::Result<()> {
        for (cond, _) in &self.tests {
            cond.analyse_repeat(visitor)?;
            // We don't analyse the consequents.  We would have to scan
            // them all unconditionally, which is rather strange.  It might
            // even cause trouble: the template author might have used
            // conditions (including for example `is_enum` to try to select an
            // expansion appropriate for the context.
            //
            // It seems less confusing to avoid this, and require the
            // template author to use `${for ...}`.
        }
        if let Some(consequence) = &self.otherwise {
            consequence.analyse_repeat(visitor)?;
        }
        Ok(())
    }
}

impl<O: SubstParseContext> AnalyseRepeat for TemplateElement<O> {
    fn analyse_repeat(
        &self,
        visitor: &mut RepeatAnalysisVisitor,
    ) -> syn::Result<()> {
        match self {
            TE::Ident(..) => {}
            TE::Literal(..) => {}
            TE::LitStr(_) => {}
            TE::Punct(..) => {}
            TE::Repeat(_) => {}
            TE::Group { template, .. } => template.analyse_repeat(visitor)?,
            TE::Subst(exp) => exp.analyse_repeat(visitor)?,
        }
        Ok(())
    }
}

impl<O: SubstParseContext> AnalyseRepeat for Subst<O> {
    fn analyse_repeat(
        &self,
        visitor: &mut RepeatAnalysisVisitor,
    ) -> syn::Result<()> {
        macro_rules! recurse { { $v:expr } => { {
            $v.analyse_repeat(visitor)?;
            None
        } } }

        let over = match &self.sd {
            SD::tname(..) => None,
            SD::vname(..) => Some(RO::Variants),
            SD::fname(..) => Some(RO::Fields),
            SD::ttype(..) => None,
            SD::tdeftype(..) => None,
            SD::ftype(..) => Some(RO::Fields),
            SD::fpatname(_) => Some(RO::Fields),
            SD::vindex(..) => Some(RO::Variants),
            SD::findex(..) => Some(RO::Fields),
            SD::v_is_first(..) => Some(RO::Variants),
            SD::f_is_first(..) => Some(RO::Fields),
            SD::Vis(SubstVis::T, ..) => None,
            SD::Vis(SubstVis::F, ..) => Some(RO::Fields),
            SD::Vis(SubstVis::FD, ..) => Some(RO::Fields),
            SD::Xmeta(sm) => sm.repeat_over(),
            SD::tattrs(..) => None,
            SD::vattrs(..) => Some(RO::Variants),
            SD::fattrs(..) => Some(RO::Fields),
            SD::tgens(..) => None,
            SD::tdefgens(..) => None,
            SD::tgnames(..) => None,
            SD::GensCollate(b) => recurse!(&b.inputs),
            SD::twheres(..) => None,
            SD::vpat(..) => Some(RO::Variants),
            SD::vtype(..) => Some(RO::Variants),
            SD::v_explicit_discriminant(..) => Some(RO::Variants),
            SD::tdefkwd(..) => None,
            SD::is_struct(..) => None,
            SD::is_enum(..) => None,
            SD::is_union(..) => None,
            SD::v_is_unit(..) => Some(RO::Variants),
            SD::v_is_tuple(..) => Some(RO::Variants),
            SD::v_is_named(..) => Some(RO::Variants),
            SD::loop_index(..) => None,
            SD::loop_initial(..) => None,
            SD::loop_final(..) => None,
            SD::Impl(body, ..) => recurse!(body),
            SD::tdefvariants(..) => None,
            SD::fdefine(..) => Some(RO::Fields),
            SD::vdefbody(..) => Some(RO::Variants),
            SD::paste(body, ..) => recurse!(body),
            SD::paste_spanned(span, content, ..) => {
                span.analyse_repeat(visitor)?;
                content.analyse_repeat(visitor)?;
                None
            }
            SD::ChangeCase(body, ..) => recurse!(body),
            SD::concat(body, ..) => recurse!(body),
            SD::StringTemplate(body, ..) => recurse!(body),
            SD::DocTemplate(body, ..) => recurse!(body.value),
            SD::when(..) => None, // out-of-place when, ignore it
            SD::loop_exactly_1(..) => None, // out-of-place, ignore it
            SD::define(..) => None,
            SD::defcond(..) => None,
            SD::UserDefined(..) => None,
            SD::is_empty(_, content) => recurse!(content),
            SD::approx_equal(_, ab) => recurse!(ab),
            SD::not(cond, _) => recurse!(cond),
            SD::If(conds, ..) | SD::select1(conds, ..) => recurse!(conds),
            SD::any(conds, _) | SD::all(conds, _) => recurse!(conds),
            // Have a RepeatOver, but don't imply anything about its context.
            SD::For(..) | SD::for_loop_count(..) => None,
            SD::False(..) | SD::True(..) => None, // condition: ignore.
            SD::error(ee, _) => recurse!(ee),
            SD::require_beta(..) => None,
            SD::ignore(content, _) => recurse!(content),
            SD::dbg(ddr) => recurse!(ddr.content_parsed),
            SD::dbg_all_keywords(..) => None,
            SD::Crate(..) => None,
        };
        if let Some(over) = over {
            let over = RepeatOverInference {
                over,
                span: self.kw_span,
            };
            visitor.set_over(over)?;
        }
        Ok(())
    }
}

//---------- WithinRepeatLevel ----------

/// Implemented for [`WithinVariant`] and [`WithinField`]
///
/// For combining code that applies similarly for different repeat levels.
pub trait WithinRepeatLevel<'w>: 'w {
    fn level_display_name() -> &'static str;

    fn current(ctx: &'w Context) -> Option<&'w Self>;

    /// Iterate over all the things at this level
    ///
    /// If it needs a current container of the next level up (eg, a
    /// field needing a variant) and there is none current, iterates
    /// over containing things too.
    fn for_each<'c, F, E>(ctx: &'c Context<'c>, call: F) -> Result<(), E>
    where
        'c: 'w,
        F: FnMut(&Context, &Self) -> Result<(), E>;

    fn missing_repeat_level_special_error(
        _ctx: &'w Context,
        _span: Span,
    ) -> Option<syn::Error> {
        None
    }
}

impl<'w> WithinRepeatLevel<'w> for WithinVariant<'w> {
    fn level_display_name() -> &'static str {
        "variant"
    }

    fn current(ctx: &'w Context) -> Option<&'w WithinVariant<'w>> {
        ctx.variant
    }

    fn for_each<'c, F, E>(ctx: &'c Context<'c>, mut call: F) -> Result<(), E>
    where
        'c: 'w,
        F: FnMut(&Context, &WithinVariant<'w>) -> Result<(), E>,
    {
        let mut within_variant =
            |index, variant, ppv: &'c PreprocessedVariant| {
                let fields = &ppv.fields;
                let pmetas = &ppv.pmetas;
                let pfields = &ppv.pfields;
                let wv = WithinVariant {
                    variant,
                    fields,
                    pmetas,
                    pfields,
                    index,
                };
                let wv = &wv;
                let ctx = Context {
                    variant: Some(wv),
                    ..*ctx
                };
                call(&ctx, wv)
            };
        match &ctx.top.data {
            syn::Data::Enum(syn::DataEnum { variants, .. }) => {
                for (index, (variant, pvariant)) in
                    izip!(variants, ctx.pvariants).enumerate()
                {
                    let index = index.try_into().expect(">=2^32 variants!");
                    within_variant(index, Some(variant), pvariant)?;
                }
            }
            syn::Data::Struct(_) | syn::Data::Union(_) => {
                within_variant(0, None, &ctx.pvariants[0])?;
            }
        }
        Ok(())
    }
}

impl<'w> WithinRepeatLevel<'w> for WithinField<'w> {
    fn level_display_name() -> &'static str {
        "field"
    }

    fn current(ctx: &'w Context) -> Option<&'w WithinField<'w>> {
        ctx.field
    }

    fn for_each<'c, F, E>(ctx: &'c Context<'c>, mut call: F) -> Result<(), E>
    where
        'c: 'w,
        F: FnMut(&Context, &WithinField<'w>) -> Result<(), E>,
    {
        ctx.for_with_within(|ctx, variant: &WithinVariant| {
            for (index, (field, pfield)) in
                izip!(variant.fields, variant.pfields,).enumerate()
            {
                // Ideally WithinField would contain a within_variant field
                // but the genercity of the lifetimes is hard to express.
                // I haven't found a version that compiles, unless we
                // *copy* the WithinVariant for each field.
                let index = index.try_into().expect(">=2^32 fields!");
                let wf = WithinField {
                    field,
                    index,
                    pfield,
                };
                let wf = &wf;
                let ctx = Context {
                    field: Some(wf),
                    ..*ctx
                };
                call(&ctx, wf)?;
            }
            Ok(())
        })
    }

    fn missing_repeat_level_special_error(
        ctx: &'w Context,
        span: Span,
    ) -> Option<syn::Error> {
        if let Some(wv) = &ctx.variant {
            return Some(span.error(
                if wv.variant.is_some() {
                    // Definitely within a variant repeat group.
                    "must be within a field, so within a field repeat group; but, is only within a variant repeat group"
                } else {
                    // We have a WithinVariant but not a syn::Variant.
                    // The driver isn't an enum.  We can't tell if we're
                    // lexically within a variant repeat group.
                    "must be within a field, so within a field repeat group"
                }
            ));
        }
        None
    }
}

impl<'c> Context<'c> {
    /// Returns an [`ErrorLoc`] for the current part of the driver
    pub fn error_loc(&self) -> ErrorLoc<'static> {
        if let Some(field) = &self.field {
            (field.field.span(), "in this field")
        } else if let Some(variant) =
            &self.variant.and_then(|variant| variant.variant.as_ref())
        {
            (variant.span(), "in this variant")
        } else {
            (self.top.span(), "in this data structure")
        }
    }

    /// Obtains the relevant `Within`(s), and calls `call` for each one
    ///
    /// If there is a current `W`, simply calls `call`.
    /// Otherwise, iterates over all of them and calls `call` for each one.
    pub fn for_with_within<'w, W, F, E>(&'c self, mut call: F) -> Result<(), E>
    where
        'c: 'w,
        W: WithinRepeatLevel<'w>,
        F: FnMut(&Context, &W) -> Result<(), E>,
    {
        let ctx = self;
        if let Some(w) = W::current(ctx) {
            call(ctx, w)?;
        } else {
            W::for_each(ctx, call)?;
        }
        Ok(())
    }

    /// Obtains the current `Within` of type `W`
    ///
    /// Demands that there actually is a current container `W`.
    /// If we aren't, calls it an error.
    fn within_level<W>(&'c self, why: &dyn Spanned) -> syn::Result<&'c W>
    where
        W: WithinRepeatLevel<'c>,
    {
        W::current(self).ok_or_else(|| {
            let span = why.span();
            W::missing_repeat_level_special_error(self, span).unwrap_or_else(
                || {
                    span.error(format_args!(
                        "must be within a {} (so, in a repeat group)",
                        W::level_display_name(),
                    ))
                },
            )
        })
    }

    /// Obtains the current field (or calls it an error)
    pub fn field(&self, why: &dyn Spanned) -> syn::Result<&WithinField<'_>> {
        self.within_level(why)
    }
    /// Obtains the current variant (or calls it an error)
    pub fn variant(
        &self,
        why: &dyn Spanned,
    ) -> syn::Result<&WithinVariant<'_>> {
        self.within_level(why)
    }
    /// Obtains the current variant as a `syn::Variant`
    pub fn syn_variant(
        &self,
        why: &dyn Spanned,
    ) -> syn::Result<&syn::Variant> {
        let r = self.variant(why)?.variant.as_ref().ok_or_else(|| {
            why.span().error("expansion only valid in enums")
        })?;
        Ok(r)
    }
}

//---------- RepeatedTemplate ----------

impl<O> ExpandInfallible<O> for RepeatedTemplate<O>
where
    Template<O>: ExpandInfallible<O>,
    O: ExpansionOutput,
{
    fn expand<'c>(&self, ctx: GeneralContext<'c>, out: &mut O) {
        let ctx = match ctx.full_ctx(self.span) {
            Ok(y) => y,
            Err(e) => {
                out.record_error(e.into());
                return;
            }
        };

        match self.over {
            RO::Variants => self.expand_repeated::<WithinVariant>(ctx, out),
            RO::Fields => self.expand_repeated::<WithinField>(ctx, out),
        };
    }
}

impl<O> RepeatedTemplate<O>
where
    Template<O>: ExpandInfallible<O>,
    O: ExpansionOutput,
{
    fn expand_repeated<'w, 'c: 'w, W: WithinRepeatLevel<'w>>(
        &self,
        ctx: &'c Context<'c>,
        out: &mut O,
    ) {
        // Determine the loop iterations, for use by loop progress conditions.
        // This depends on our conditions being stable (ie, always
        // yielding the same values or errors).  We check that, below.
        let mut filters = Vec::<Result<bool, ()>>::new();
        ctx.for_with_within(|ctx, _: &W| {
            let f = self.filter_inner(ctx);
            filters.push(f.map_err(|e| out.record_error(e)));
            // for_with_within expects a fallible closure, but we want to do
            // infallible work in our infallible context, so we use `Void`
            // as the error type.
            Ok::<_, Void>(())
        })
        .void_unwrap();

        let is_yes = |f: Result<bool, ()>| f == Ok(true);
        let yes_count: u32 = filters
            .iter()
            .filter(|f| is_yes(**f))
            .count()
            .try_into()
            .expect("repetition iterated more than 2^32 times");

        if let Some((e1_span, e1_error)) = &self.exactly_1 {
            if yes_count == 0 {
                e1_error.throw(ctx.as_general(), *e1_span, [ctx.error_loc()])
            } else if yes_count == 1 {
                Ok(())
            } else {
                // We want to produce two locs, first and second.
                // But (a) we have ctx.for_with_within., not an iterator
                // and (b) we must make Strings and faff with lifetimes
                let mut i = 0;
                let mut locs = [None, None]; // poor bod's ArrayVec
                let _: Result<(), ()> = ctx.for_with_within(|ctx, _: &W| {
                    // just trust that the condition is stable;
                    // we're reporting an error in any case.
                    let f = self.filter_inner(ctx).map_err(|_| ());
                    if is_yes(f) {
                        let (span, ctx_s) = ctx.error_loc();
                        let loc = locs.get_mut(i).ok_or(())?;
                        let fs = ["first", "second"][i];
                        *loc = Some((span, format!("{}: {}", fs, ctx_s)));
                        i += 1;
                    }
                    Ok::<_, ()>(())
                });
                e1_error.throw(
                    ctx.as_general(),
                    *e1_span,
                    locs.iter().map(|b| {
                        let b = b.as_ref().expect("all added");
                        (b.0, &*b.1)
                    }),
                )
            }
            .unwrap_or_else(|e| out.record_error(e));
        }

        // We use `u32` for the `${when }`-filtered iterations ("yes"";
        // those are exposed via `loop_index` which must
        // be able to produce a tuple label, which is u32.
        //
        // Using `usize` for counts including *all* iterations is easy
        // and also means we3 can't accidentally confuse the two counters.
        let mut raw_i: usize = 0;
        let mut yes_i: u32 = 0;
        ctx.for_with_within(|ctx, _: &W| {
            let f_with_err = self.filter_inner(ctx);
            // errors are recorded above, don't record them again;
            // but if condition was unstable, include error in the panic
            let f = f_with_err.as_ref().map_err(|_| ()).map(|y| *y);
            assert_eq!(f, filters[raw_i], "2nd={:?}", f_with_err);
            if is_yes(f) {
                let mut ctx = ctx.clone();
                let info = WithinLoopBody { yes_i, yes_count };
                ctx.within_loop = WithinLoop::Body(&info);
                self.template.expand(ctx.as_general(), out);
                yes_i += 1;
            }
            raw_i += 1;
            Ok::<_, Void>(())
        })
        .void_unwrap()
    }
}

impl<O: ExpansionOutput> RepeatedTemplate<O> {
    /// private, does the condition
    fn filter_inner(&self, ctx: &Context) -> syn::Result<bool> {
        let mut ctx = ctx.clone();
        ctx.within_loop = WithinLoop::When;

        for when in &self.whens {
            match when.eval_bool(ctx.as_general())? {
                true => continue,
                false => return Ok(false),
            }
        }

        Ok(true)
    }
}

//---------- ForLoopCount ----------

#[derive(Debug)]
pub struct ForLoopCount {
    pub over: RepeatOver,
    pub cond: Box<Condition>,
}

impl Parse for ForLoopCount {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let over = input.parse()?;
        let cond = input.parse()?;
        Ok(ForLoopCount { cond, over })
    }
}

impl ForLoopCount {
    pub fn count(&self, ctx: &Context) -> syn::Result<u32> {
        match self.over {
            RO::Variants => self.count_inner::<WithinVariant>(ctx),
            RO::Fields => self.count_inner::<WithinField>(ctx),
        }
        .map(|c| c.try_into().expect("for_loop_count >= 2^32"))
    }
}

impl ForLoopCount {
    fn count_inner<'w, 'c: 'w, W: WithinRepeatLevel<'w>>(
        &self,
        ctx: &'c Context<'c>,
    ) -> syn::Result<usize> {
        let mut yes_count = 0;
        ctx.for_with_within(|ctx, _: &W| {
            if self.cond.eval_bool(ctx.as_general())? {
                yes_count += 1;
            }
            Ok::<_, syn::Error>(())
        })?;
        Ok(yes_count)
    }
}

//---------- RepeatOver ----------

impl Parse for RepeatOver {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let over: Ident = input.parse()?;
        Ok(if over == "fields" {
            RepeatOver::Fields
        } else if over == "variants" {
            RepeatOver::Variants
        } else {
            return Err(over.error("expected 'fields' or 'variants'"));
        })
    }
}

//---------- Fname ----------

impl<'w> WithinField<'w> {
    /// What would `${fname}` expand to?
    pub fn fname(&self, tspan: Span) -> Fname<'_> {
        if let Some(fname) = &self.field.ident {
            // todo is this the right span to emit?
            Fname::Name(fname)
        } else {
            Fname::Index(syn::Index {
                index: self.index,
                span: tspan,
            })
        }
    }
}

impl IdentFrag for Fname<'_> {
    type BadIdent = IdentFragInfallible;
    fn frag_to_tokens(
        &self,
        out: &mut TokenStream,
    ) -> Result<(), IdentFragInfallible> {
        Ok(self.to_tokens(out))
    }
    fn fragment(&self) -> String {
        self.to_string()
    }
}
impl Display for Fname<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fname::Name(v) => quote::IdentFragment::fmt(v, f),
            Fname::Index(v) => quote::IdentFragment::fmt(v, f),
        }
    }
}
impl ToTokens for Fname<'_> {
    fn to_tokens(&self, out: &mut TokenStream) {
        match self {
            Fname::Name(v) => v.to_tokens(out),
            Fname::Index(v) => v.to_tokens(out),
        }
    }
}

//---------- v_explicit_discriminant ----------

#[derive(strum::Display)]
pub enum NoVariantExlicitDiscriminant {
    #[strum(to_string = "not an enum, so no explicit discriminant")]
    NotEnum,
    #[strum(to_string = "enum variant does not have explicit discriminant")]
    None,
}

impl<'c> Context<'c> {
    pub fn v_explicit_discriminant(
        &'c self,
        span: Span,
    ) -> syn::Result<Result<&'c syn::Expr, NoVariantExlicitDiscriminant>> {
        let variant = self.variant(&span)?;
        Ok((|| {
            use NoVariantExlicitDiscriminant as N;
            let (_eq, expr) = variant
                .variant
                .ok_or(N::NotEnum)?
                .discriminant
                .as_ref()
                .ok_or(N::None)?;
            Ok(expr)
        })())
    }
}

//---------- useful impl ----------

impl<'w> WithinVariant<'w> {
    pub fn is_struct_toplevel_as_variant(&self) -> bool {
        self.variant.is_none()
    }
}

//---------- definitions (user-defined keywords) ----------

impl<'c> GeneralContext<'c> {
    pub fn find_definition<B>(
        &'c self,
        call: &'c DefinitionName,
    ) -> syn::Result<Option<(&'c Definition<B>, GeneralContextBuf<'c>)>>
    where
        Definitions<'c>: AsRef<[&'c Definition<B>]>,
        B: 'static,
    {
        let def = match self.defs().defs.find_raw(call) {
            Some(y) => y,
            None => return Ok(None),
        };

        let defs = self.defs().find_definition_deeper(&def.name, call)?;
        let mut self_ = self.clone_buf();
        *self_.as_mut() = defs;
        Ok(Some((def, self_)))
    }

    pub fn within_loop_body(
        &'c self,
        span: Span,
    ) -> syn::Result<&'c WithinLoopBody> {
        match self.full_ctx(span)?.within_loop {
            WithinLoop::None => Err(span.error(
                "loop information requested, but not within a loop",
            )),
            WithinLoop::When => Err(span.error(
                "loop information is not available within ${when } because ${when } controls the repetitions",
            )),
            WithinLoop::Body(info) => Ok(info),
        }
    }
}

impl<'c> DefinitionsContext<'c> {
    /// Internal function for use by `find_definition`
    pub fn find_definition_deeper(
        &'c self,
        def: &'c DefinitionName,
        call: &'c DefinitionName,
    ) -> syn::Result<DefinitionsContext<'c>> {
        let nesting_depth = self.nesting_depth + 1;
        let stack_entry = (self, call);
        if nesting_depth > NESTING_LIMIT {
            // We report the definition site of the innermost reference:
            let mut errs = def.error(format_args!(
 "probably-recursive user-defined expansion/condition (more than {} deep)",
                NESTING_LIMIT
            ));
            // And the unique reference sites from the call stack
            let calls = {
                let mut ascend = Some(stack_entry);
                iter::from_fn(|| {
                    let (ctx, call) = ascend?;
                    ascend = ctx.nesting_parent;
                    Some((call, ctx.nesting_depth))
                })
                .collect_vec()
            };

            // Collect and reverse because we preferentially want
            // to display less deep entries (earlier ones), which are
            // furthest away in the stack chain.

            let calls = calls
                .iter()
                .rev()
                .unique_by(
                    // De-dup by pointer identity on the name as found
                    // at the call site.  These are all references
                    // nodes in our template AST, so this is correct.
                    |(call, _)| *call as *const DefinitionName,
                )
                .collect_vec();

            // We report the deepest errors first, since they're
            // definitely implicated in the cycle and more interesting.
            // (So this involves collecting and reversing again.)
            for (call, depth) in calls.iter().rev() {
                errs.combine(call.error(format_args!(
             "reference involved in too-deep expansion/condition, depth {}",
                    depth,
                )));
            }
            return Err(errs);
        }
        Ok(DefinitionsContext {
            nesting_depth,
            nesting_parent: Some(stack_entry),
            ..*self
        })
    }
}

pub struct DefinitionsIter<'c, B>(
    Option<&'c Definitions<'c>>,
    PhantomData<&'c B>,
);

impl<'c, B> Iterator for DefinitionsIter<'c, B>
where
    Definitions<'c>: AsRef<[&'c Definition<B>]>,
{
    type Item = &'c [&'c Definition<B>];
    fn next(&mut self) -> Option<Self::Item> {
        let here = self.0?;
        let r = here.as_ref();
        self.0 = here.earlier;
        Some(r)
    }
}

impl<'c> Definitions<'c> {
    pub fn iter<B>(&'c self) -> DefinitionsIter<'c, B>
    where
        Definitions<'c>: AsRef<[&'c Definition<B>]>,
    {
        DefinitionsIter(Some(self), PhantomData)
    }

    /// Find the definition of `name` as a `B`, without recursion checking
    ///
    /// The caller is responsible for preventing unbounded recursion.
    pub fn find_raw<B>(
        &'c self,
        name: &DefinitionName,
    ) -> Option<&'c Definition<B>>
    where
        Definitions<'c>: AsRef<[&'c Definition<B>]>,
        B: 'static,
    {
        self.iter()
            .map(|l| l.iter().rev())
            .flatten()
            .find(|def| &def.name == name)
            .cloned()
    }
}

impl<'c> AsRef<[&'c Definition<DefinitionBody>]> for Definitions<'c> {
    fn as_ref(&self) -> &[&'c Definition<DefinitionBody>] {
        self.here
    }
}
impl<'c> AsRef<[&'c Definition<DefCondBody>]> for Definitions<'c> {
    fn as_ref(&self) -> &[&'c Definition<DefCondBody>] {
        self.conds
    }
}
