//! `#[deftly(...)]` meta attributes

use super::framework::*;

//========== common definitions ==========

/// Indicates one of `fmeta`, `vmeta` or `tmeta`
#[derive(Debug, Clone, Copy)]
pub enum Scope {
    T,
    V,
    F,
}

/// `(foo(bar))` in eg `fmeta(foo(bar))`
///
/// includes the parens
#[derive(Debug, Clone)]
pub struct Label {
    // Nonempty list, each with nonempty segments.
    // Outermost first.
    pub lpaths: Vec<syn::Path>,
}

//========== substitutions in a template ==========

#[derive(Debug, Clone)]
pub struct Subst<O: SubstParseContext> {
    pub scope: Scope,
    pub label: Label,
    pub as_: Option<SubstAs<O>>,
}

#[derive(Debug, Clone, AsRefStr, Display)]
#[allow(non_camel_case_types)] // clearer to use the exact ident
pub enum SubstAs<O: SubstParseContext> {
    tokens(O::NotInBool, O::NotInPaste),
    ty(O::NotInBool),
    str(O::NotInBool),
}

//========== meta attrs in a driver ==========

/// A part like `(foo,bar(baz),zonk="value")`
#[derive(Debug, Clone)]
pub struct PreprocessedValueList {
    pub content: Punctuated<PreprocessedTree, token::Comma>,
}

/// `#[deftly(...)]` helper attributes
pub type PreprocessedMetas = Vec<PreprocessedTree>;

/// An `#[deftly()]` attribute, or a sub-tree within one
#[derive(Debug, Clone)]
pub struct PreprocessedTree {
    pub path: syn::Path,
    pub value: PreprocessedValue,
}

/// Content of a meta attribute
///
/// Examples in doc comments are for
/// `PreprocessedMeta.path` of `foo`,
/// ie the examples are for `#[deftly(foo ..)]`.
#[derive(Debug, Clone)]
pub enum PreprocessedValue {
    /// `#[deftly(foo)]`
    Unit,
    /// `#[deftly(foo = "lit")]`
    Value { equals: token::Eq, value: syn::Lit },
    /// `#[deftly(foo(...))]`
    List(PreprocessedValueList),
}

//========== search and match results ==========

/// Node in tree structure found in driver `#[deftly(some(thing))]`
#[derive(Debug)]
pub enum FoundNode<'l> {
    Unit(Span),
    Lit { path_span: Span, lit: &'l syn::Lit },
}

/// Information about a nearby meta node we found
///
/// "Nearby" means that the node we found is a prefix (in tree descent)
/// of the one we were looking for, or vice versa.
#[derive(Debug)]
pub struct FoundNearbyNode {
    pub kind: FoundNearbyNodeKind,
    /// Span of the identifier in the actual `#[deftly]` driver attribute
    pub path_span: Span,
}

/// How the nearby node relates to the one we were looking for
#[derive(Debug)]
pub enum FoundNearbyNodeKind {
    /// We were looking to go deeper, but found a unit in `#[deftly]`
    Unit,
    /// We were looking to go deeper, but found a `name = value` in `#[deftly]`
    Lit,
    /// We were looking for a leaf, but we found nested list in `#[deftly]`
    List,
}

pub use FoundNearbyNodeKind as FNNK;
pub use FoundNode as FN;

//========== template parsing ==========

impl<O: SubstParseContext> Spanned for Subst<O> {
    fn span(&self) -> Span {
        self.label.span()
    }
}

impl Spanned for Label {
    fn span(&self) -> Span {
        self.lpaths
            .first()
            .expect("empty label")
            .segments
            .first()
            .expect("empty path!")
            .span()
    }
}

impl<O: SubstParseContext> SubstAs<O> {
    fn parse(input: ParseStream, nb: O::NotInBool) -> syn::Result<Self> {
        let kw: IdentAny = input.parse()?;
        let from_sma = |sma: SubstAs<_>| Ok(sma);

        // See keyword_general! in utils.rs
        macro_rules! keyword { { $($args:tt)* } => {
            keyword_general! { kw from_sma SubstAs; $($args)* }
        } }

        let not_in_paste = || O::not_in_paste(&kw);

        keyword! { str(nb) }
        keyword! { tokens(nb, not_in_paste()?) }
        keyword! { ty(nb) }

        Err(kw.error("unknown derive-deftly 'as' syntax type keyword"))
    }
}

impl<O: SubstParseContext> Subst<O> {
    pub fn parse(input: ParseStream, scope: Scope) -> syn::Result<Self> {
        let label: Label = input.parse()?;

        let as_;

        if input.peek(Token![as]) {
            let as_token: Token![as] = input.parse()?;
            let nb = O::not_in_bool(&as_token).map_err(|_| {
                as_token.error("`Xmeta as ...` not allowed in conditions")
            })?;
            as_ = Some(SubstAs::parse(input, nb)?);
        } else {
            as_ = None;
        }

        Ok(Subst { label, as_, scope })
    }
}

//========== driver parsing ==========

impl PreprocessedValueList {
    fn parse_outer(input: ParseStream) -> syn::Result<Self> {
        let meta;
        let _paren = parenthesized!(meta in input);
        Self::parse_inner(&meta)
    }
}

impl PreprocessedValueList {
    pub fn parse_inner(input: ParseStream) -> syn::Result<Self> {
        let content = Punctuated::parse_terminated(input)?;
        Ok(PreprocessedValueList { content })
    }
}

impl Parse for PreprocessedTree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use PreprocessedValue as PV;

        let path = input.call(syn::Path::parse_mod_style)?;
        let la = input.lookahead1();
        let value = if la.peek(Token![=]) {
            let equals = input.parse()?;
            let value = input.parse()?;
            PV::Value { equals, value }
        } else if la.peek(token::Paren) {
            let list = input.call(PreprocessedValueList::parse_outer)?;
            PV::List(list)
        } else if la.peek(Token![,]) || input.is_empty() {
            PV::Unit
        } else {
            return Err(la.error());
        };
        Ok(PreprocessedTree { path, value })
    }
}

impl Parse for Label {
    fn parse(outer: ParseStream) -> syn::Result<Self> {
        fn recurse(
            lpaths: &mut Vec<syn::Path>,
            outer: ParseStream,
        ) -> syn::Result<()> {
            let input;
            let paren = parenthesized!(input in outer);
            let path = input.call(syn::Path::parse_mod_style)?;
            if path.segments.is_empty() {
                return Err(paren
                    .span
                    .error("`deftly` attribute must have nonempty path"));
            }

            lpaths.push(path);
            if !input.is_empty() {
                recurse(lpaths, &input)?;
            }
            Ok(())
        }

        let mut lpaths = vec![];
        recurse(&mut lpaths, outer)?;

        Ok(Label { lpaths })
    }
}

//========== searching and matching ==========

impl Label {
    pub fn search<'a, A, F, G, E>(
        &self,
        pmetas: A,
        f: &mut F,
        g: &mut G,
    ) -> Result<(), E>
    where
        F: FnMut(FoundNode<'a>) -> Result<(), E>,
        G: FnMut(FoundNearbyNode) -> Result<(), E>,
        A: IntoIterator<Item = &'a PreprocessedTree>,
    {
        for pmeta in pmetas {
            Self::search_1(&self.lpaths, pmeta, &mut *f, &mut *g)?;
        }
        Ok(())
    }

    fn search_1<'a, E, F, G>(
        // Nonempty
        lpaths: &[syn::Path],
        ptree: &'a PreprocessedTree,
        f: &mut F,
        g: &mut G,
    ) -> Result<(), E>
    where
        F: FnMut(FoundNode<'a>) -> Result<(), E>,
        G: FnMut(FoundNearbyNode) -> Result<(), E>,
    {
        use PreprocessedValue as PV;

        if ptree.path != lpaths[0] {
            return Ok(());
        }

        let path_span = ptree.path.span();

        let mut nearby = |kind| g(FoundNearbyNode { kind, path_span });

        let deeper = if lpaths.len() <= 1 {
            None
        } else {
            Some(&lpaths[1..])
        };

        match (deeper, &ptree.value) {
            (None, PV::Unit) => f(FN::Unit(path_span))?,
            (None, PV::List(_)) => nearby(FNNK::List)?,
            (None, PV::Value { value, .. }) => f(FN::Lit {
                path_span,
                lit: value,
            })?,
            (Some(_), PV::Value { .. }) => nearby(FNNK::Lit)?,
            (Some(_), PV::Unit) => nearby(FNNK::Unit)?,
            (Some(d), PV::List(l)) => {
                for m in l.content.iter() {
                    Self::search_1(d, m, &mut *f, &mut *g)?;
                }
            }
        }

        Ok(())
    }
}

impl Label {
    pub fn search_eval_bool(
        &self,
        pmetas: &PreprocessedMetas,
    ) -> Result<(), Found> {
        self.search(
            pmetas,
            &mut |_av| /* got it! */ Err(Found),
            &mut |nearby| match nearby.kind {
                FNNK::List => Err(Found), // there's more, that'll do
                FNNK::Unit => Ok(()),
                FNNK::Lit => Ok(()),
            },
        )
    }
}

//========== scope handling ==========

impl<O> Subst<O>
where
    O: SubstParseContext,
{
    pub fn repeat_over(&self) -> Option<RepeatOver> {
        match self.scope {
            Scope::T => None,
            Scope::V => Some(RO::Variants),
            Scope::F => Some(RO::Fields),
        }
    }
}

impl<O> Subst<O>
where
    O: SubstParseContext,
{
    pub fn pmetas<'c>(
        &self,
        ctx: &'c Context<'c>,
    ) -> syn::Result<&'c PreprocessedMetas> {
        Ok(match self.scope {
            Scope::T => &ctx.tmetas,
            Scope::V => &ctx.variant(self)?.pmetas,
            Scope::F => &ctx.field(self)?.pfield.pmetas,
        })
    }
}

//========== template expansion ==========

impl<O> Subst<O>
where
    O: ExpansionOutput,
{
    pub fn expand(
        &self,
        ctx: &Context,
        out: &mut O,
        pmetas: &PreprocessedMetas,
    ) -> syn::Result<()> {
        let mut found = None::<FoundNode>;
        let mut hint = None::<FoundNearbyNode>;
        let self_loc = || (self.span(), "expansion");
        let error_loc = || [ctx.error_loc(), self_loc()];

        self.label.search(
            pmetas,
            &mut |av: FoundNode| {
                if let Some(first) = &found {
                    return Err([(first.path_span(), "first occurrence"),
                            (av.path_span(), "second occurrence"),
                            self_loc()].error(
 "tried to expand just attribute value, but it was specified multiple times"
                ));
                }
                found = Some(av);
                Ok(())
            },
            &mut |nearby| {
                hint.get_or_insert(nearby);
                Ok(())
            },
        )?;

        let found = found.ok_or_else(|| {
            if let Some(hint) = hint {
                let hint_msg = match hint.kind {
                    FNNK::Unit =>
 "expected a list with sub-attributes, found a unit",
                    FNNK::Lit =>
 "expected a list with sub-attributes, found a simple value",
                    FNNK::List =>
 "expected a leaf node, found a list with sub-attributes",
                };
                let mut err = hint.path_span.error(hint_msg);
                err.combine(error_loc().error(
 "attribute value expanded, but no suitable value in data structure definition"
                ));
                err
            } else {
                error_loc().error(
 "attribute value expanded, but no value in data structure definition"
                )
            }
        })?;

        found.expand(self.span(), &self.as_, out)?;

        Ok(())
    }
}

fn metavalue_spans(tspan: Span, vspan: Span) -> [ErrorLoc; 2] {
    [(vspan, "attribute value"), (tspan, "template")]
}

/// Obtain the `LiStr` from a meta node value (ie, a `Lit`)
///
/// This is the thing we actually use.
/// Non-string-literal values are not allowed.
fn metavalue_litstr<'l>(
    lit: &'l syn::Lit,
    tspan: Span,
    msg: fmt::Arguments<'_>,
) -> syn::Result<&'l syn::LitStr> {
    match lit {
        syn::Lit::Str(s) => Ok(s),
        // having checked derive_builder, it doesn't handle
        // Lit::Verbatim so I guess we don't need to either.
        _ => Err(metavalue_spans(tspan, lit.span()).error(msg)),
    }
}

/// Convert a literal found in a meta item into `T`
///
/// `into_what` is used only for error reporting
pub fn metavalue_lit_as<T>(
    lit: &syn::Lit,
    tspan: Span,
    into_what: &dyn Display,
) -> syn::Result<T>
where
    T: Parse + ToTokens,
{
    let s = metavalue_litstr(
        lit,
        tspan,
        format_args!(
            "expected string literal, for conversion to {}",
            into_what,
        ),
    )?;

    let t: TokenStream = s.parse().map_err(|e| {
        // Empirically, parsing a LitStr in actual proc macro context, with
        // proc_macro2, into tokens, can generate a lexical error with a
        // "fallback" Span.  Then, attempting to render the results,
        // including the eventual compiler_error! invocation, back to
        // a compiler proc_ma cor::TokenStream can panic with
        // "compiler/fallback mismatch".
        //
        // https://github.com/dtolnay/syn/issues/1504
        //
        // Attempt to detect this situation.
        match (|| {
            let _: String = (&e).into_iter().next()?.span().source_text()?;
            Some(())
        })() {
            Some(()) => e,
            None => lit.span().error(e.to_string()),
        }
    })?;

    let thing: T = syn::parse2(t)?;
    Ok(thing)
}

impl FoundNode<'_> {
    fn path_span(&self) -> Span {
        match self {
            FN::Unit(span) => *span,
            FN::Lit { path_span, .. } => *path_span,
        }
    }
}

impl<'l> FoundNode<'l> {
    fn expand<O>(
        &self,
        tspan: Span,
        as_: &Option<SubstAs<O>>,
        out: &mut O,
    ) -> syn::Result<()>
    where
        O: ExpansionOutput,
    {
        let spans = |vspan| metavalue_spans(tspan, vspan);

        let lit = match self {
            FN::Unit(vspan) => return Err(spans(*vspan).error(
 "tried to expand attribute which is just a unit, not a literal"
            )),
            FN::Lit { lit, .. } => lit,
        };

        use meta::SubstAs as MSA;

        let default_buf;
        let as_ = match as_ {
            Some(as_) => as_,
            None => {
                default_buf = O::default_subst_meta_as();
                &default_buf
            }
        };

        match as_ {
            MSA::tokens(_, np) => {
                let tokens: TokenStream =
                    metavalue_lit_as(lit, tspan, &"tokens")?;
                out.append_tokens(np, tokens)?;
            }
            as_ @ MSA::ty(..) => {
                out.append_syn_type(tspan, &metavalue_lit_as(lit, tspan, as_)?)
            }
            MSA::str(..) => {
                let s = metavalue_litstr(
                    lit,
                    tspan,
                    format_args!("expected string literal, for meta value",),
                )?;
                out.append_syn_litstr(s);
            }
        }
        Ok(())
    }
}
