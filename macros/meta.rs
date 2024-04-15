//! `#[deftly(...)]` meta attributes

use super::framework::*;

use indexmap::IndexMap;

//---------- common definitions ----------

/// Indicates one of `fmeta`, `vmeta` or `tmeta`
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[derive(AsRefStr, EnumString, EnumIter)]
#[rustfmt::skip]
pub enum Scope {
    // NB these keywords are duplicated in SubstDetails
    #[strum(serialize = "tmeta")] T,
    #[strum(serialize = "vmeta")] V,
    #[strum(serialize = "fmeta")] F,
}

/// Scope of a *supplied* meta (`#[deftly(...)]`) attribute
///
/// Also encodes, for metas at the toplevel,
/// whether it's a struct or an enum.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)] //
#[derive(strum::Display, EnumIter)]
#[strum(serialize_all = "snake_case")]
pub enum SuppliedScope {
    Struct,
    Enum,
    Variant,
    Field,
}

impl SuppliedScope {
    fn recog_search(self) -> impl Iterator<Item = Scope> {
        use Scope as S;
        use SuppliedScope as SS;
        match self {
            SS::Struct => &[S::T, S::V] as &[_],
            SS::Enum => &[S::T],
            SS::Variant => &[S::V],
            SS::Field => &[S::F],
        }
        .iter()
        .copied()
    }

    fn widest(self) -> Scope {
        self.recog_search().next().unwrap()
    }
}

/// `(foo(bar))` in eg `fmeta(foo(bar))`
///
/// includes the parens
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Label {
    // Nonempty list, each with nonempty segments.
    // Outermost first.
    pub lpaths: Vec<syn::Path>,
}

/// Meta designator eg `fmeta(foo(bar))`
// Field order must be the same as BorrowedDesig
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Desig {
    pub scope: Scope,
    pub label: Label,
}

#[derive(Hash)]
// Field order must be the same as meta::Desig
struct BorrowedDesig<'p> {
    pub scope: Scope,
    pub lpaths: &'p [&'p syn::Path],
}

//---------- substitutions in a template ----------

#[derive(Debug, Clone)]
pub struct SubstMeta<O: SubstParseContext> {
    pub desig: Desig,
    pub as_: Option<SubstAs<O>>,
}

#[derive(Debug, Clone, AsRefStr, Display)]
#[allow(non_camel_case_types)] // clearer to use the exact ident
pub enum SubstAs<O: SubstParseContext> {
    tokens(O::NotInBool, O::NotInPaste),
    ty(O::NotInBool),
    str(O::NotInBool),
}

//---------- meta attrs in a driver ----------

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

//---------- search and match results ----------

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

//---------- meta attr enumeration and checking ----------

#[derive(Debug, Copy, Clone, Hash, PartialOrd, Ord, Eq, PartialEq)] //
#[derive(EnumIter)]
pub enum RecogAllow {
    BoolOnly,
    Value,
}

#[derive(Default, Debug, Clone)]
pub struct Recognised {
    map: IndexMap<Desig, RecogAllow>,
}

pub trait FindRecogMetas {
    /// Search for `fmeta(..)` etc. expansions
    ///
    /// Add to `acc` any that are
    /// (recusively) within `self`, syntactically,
    fn find_recog_metas(&self, acc: &mut Recognised);
}

//==================== implementations ====================

//---------- template parsing ----------

impl<O: SubstParseContext> SubstMeta<O> {
    fn span_whole(&self, scope_span: Span) -> Span {
        spans_join(chain!(
            [scope_span], //
            self.desig.label.spans(),
        ))
        .unwrap()
    }
}

impl Label {
    /// Nonempty
    pub fn spans(&self) -> impl Iterator<Item = Span> + '_ {
        self.lpaths.iter().map(|path| path.span())
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

impl<O: SubstParseContext> SubstMeta<O> {
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

        Ok(SubstMeta {
            desig: Desig { label, scope },
            as_,
        })
    }
}

//---------- driver parsing ----------

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

//---------- searching and matching ----------

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

//---------- scope and designator handling ----------

impl<O> SubstMeta<O>
where
    O: SubstParseContext,
{
    pub fn repeat_over(&self) -> Option<RepeatOver> {
        match self.desig.scope {
            Scope::T => None,
            Scope::V => Some(RO::Variants),
            Scope::F => Some(RO::Fields),
        }
    }
}

impl<O> SubstMeta<O>
where
    O: SubstParseContext,
{
    pub fn pmetas<'c>(
        &self,
        ctx: &'c Context<'c>,
        kw_span: Span,
    ) -> syn::Result<&'c PreprocessedMetas> {
        Ok(match self.desig.scope {
            Scope::T => &ctx.tmetas,
            Scope::V => &ctx.variant(&kw_span)?.pmetas,
            Scope::F => &ctx.field(&kw_span)?.pfield.pmetas,
        })
    }
}

impl ToTokens for Label {
    fn to_tokens(&self, out: &mut TokenStream) {
        let mut lpaths = self.lpaths.iter().rev();
        let mut current =
            lpaths.next().expect("empty path!").to_token_stream();
        let r = loop {
            let span = current.span();
            let mut group =
                proc_macro2::Group::new(Delimiter::Parenthesis, current);
            group.set_span(span);
            let wrap = if let Some(y) = lpaths.next() {
                y
            } else {
                break group;
            };
            current = quote! { #wrap #group };
        };
        r.to_tokens(out);
    }
}

impl Desig {
    fn to_tokens(&self, scope_span: Span, out: &mut TokenStream) {
        let scope: &str = self.scope.as_ref();
        Ident::new(scope, scope_span).to_tokens(out);
        self.label.to_tokens(out);
    }
}

impl Parse for Desig {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let scope: syn::Ident = input.parse()?;
        let scope = scope
            .to_string()
            .parse()
            .map_err(|_| scope.error("invalid meta keyword/level"))?;
        let label = input.parse()?;
        Ok(Self { scope, label })
    }
}

impl indexmap::Equivalent<Desig> for BorrowedDesig<'_> {
    fn equivalent(&self, desig: &Desig) -> bool {
        let BorrowedDesig { scope, lpaths } = self;
        *scope == desig.scope
            && itertools::equal(lpaths.iter().copied(), &desig.label.lpaths)
    }
}

/// `Display`s as a `#[deftly(...)]` as the user might write it
struct DisplayAsIfSpecified<'r> {
    lpaths: &'r [&'r syn::Path],
    /// Included after the innermost lpath, inside the parens
    inside_after: &'r str,
}
impl Display for DisplayAsIfSpecified<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#[deftly")?;
        for p in self.lpaths {
            write!(f, "({}", p.to_token_stream())?;
        }
        write!(f, "{}", self.inside_after)?;
        for _ in self.lpaths {
            write!(f, ")")?;
        }
        Ok(())
    }
}

// Tests that our `BorrowedDesig` `equivalent` impl is justified.
#[test]
fn check_borrowed_desig() {
    use super::*;
    use indexmap::Equivalent;
    use itertools::iproduct;
    use std::hash::{Hash, Hasher};

    #[derive(PartialEq, Eq, Debug, Default)]
    struct TrackingHasher(Vec<Vec<u8>>);
    impl Hasher for TrackingHasher {
        fn write(&mut self, bytes: &[u8]) {
            self.0.push(bytes.to_owned());
        }
        fn finish(&self) -> u64 {
            unreachable!()
        }
    }
    impl TrackingHasher {
        fn hash(t: impl Hash) -> Self {
            let mut self_ = Self::default();
            t.hash(&mut self_);
            self_
        }
    }

    type Case = (Scope, &'static [&'static str]);
    const TEST_CASES: &[Case] = &[
        (Scope::T, &["path"]),
        (Scope::T, &["r#path"]),
        (Scope::V, &["path", "some::path"]),
        (Scope::V, &["r#struct", "with_generics::<()>"]),
        (Scope::F, &[]), // illegal Desig, but test anyway
    ];

    struct Desigs<'b> {
        owned: Desig,
        borrowed: BorrowedDesig<'b>,
    }
    impl Desigs<'_> {
        fn with((scope, lpaths): &Case, f: impl FnOnce(Desigs<'_>)) {
            let scope = *scope;
            let lpaths = lpaths
                .iter()
                .map(|l| syn::parse_str(l).expect(l))
                .collect_vec();
            let owned = {
                let label = Label {
                    lpaths: lpaths.clone(),
                };
                Desig { scope, label }
            };
            let lpaths_borrowed;
            let borrowed = {
                lpaths_borrowed = lpaths.iter().collect_vec();
                BorrowedDesig {
                    scope,
                    lpaths: &*lpaths_borrowed,
                }
            };
            f(Desigs { owned, borrowed })
        }
    }

    // Test that for each entry in TEST_CASES, when parsed into Paths, etc.,
    // BorrowedDesig is `equivalent` to, and hashes the same as, Desig.

    for case in TEST_CASES {
        Desigs::with(case, |d| {
            assert!(d.borrowed.equivalent(&d.owned));
            assert_eq!(
                TrackingHasher::hash(&d.owned),
                TrackingHasher::hash(&d.borrowed),
            );
        });
    }

    // Compare every TEST_CASES entry with every other entry.
    // See if the owned forms are equal (according to `PartialEq`).
    // Insist that the Borrowed vs owned `equivalent` relation agrees,
    // in both directions.
    // And, if they are equal, insist that the hashes all agree.

    for (case0, case1) in iproduct!(TEST_CASES, TEST_CASES) {
        Desigs::with(case0, |d0| {
            Desigs::with(case1, |d1| {
                let equal = d0.owned == d1.owned;
                assert_eq!(equal, d0.borrowed.equivalent(&d1.owned));
                assert_eq!(equal, d1.borrowed.equivalent(&d0.owned));
                if equal {
                    let hash = TrackingHasher::hash(&d0.owned);
                    assert_eq!(hash, TrackingHasher::hash(&d1.owned));
                    assert_eq!(hash, TrackingHasher::hash(&d0.borrowed));
                    assert_eq!(hash, TrackingHasher::hash(&d1.borrowed));
                }
            });
        });
    }
}

//---------- template expansion ----------

impl<O> SubstMeta<O>
where
    O: ExpansionOutput,
{
    pub fn expand(
        &self,
        ctx: &Context,
        kw_span: Span,
        out: &mut O,
        pmetas: &PreprocessedMetas,
    ) -> syn::Result<()> {
        let mut found = None::<FoundNode>;
        let mut hint = None::<FoundNearbyNode>;
        let span_whole = self.span_whole(kw_span);
        let self_loc = || (span_whole, "expansion");
        let error_loc = || [ctx.error_loc(), self_loc()];

        self.desig.label.search(
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

        found.expand(span_whole, &self.as_, out)?;

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

        use SubstAs as SA;

        let default_buf;
        let as_ = match as_ {
            Some(as_) => as_,
            None => {
                default_buf = O::default_subst_meta_as();
                &default_buf
            }
        };

        match as_ {
            SA::tokens(_, np) => {
                let tokens: TokenStream =
                    metavalue_lit_as(lit, tspan, &"tokens")?;
                out.append_tokens(np, tokens)?;
            }
            as_ @ SA::ty(..) => {
                out.append_syn_type(tspan, &metavalue_lit_as(lit, tspan, as_)?)
            }
            SA::str(..) => {
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

//---------- impl Recognised etc. ----------

impl Recognised {
    /// Ensures that `self[k] >= v`
    pub fn update(&mut self, k: Desig, v: RecogAllow) {
        let ent = self.map.entry(k).or_insert(v);
        *ent = cmp::max(*ent, v);
    }
}

impl ToTokens for Recognised {
    fn to_tokens(&self, out: &mut TokenStream) {
        for (desig, allow) in &self.map {
            match allow {
                RecogAllow::BoolOnly => {
                    out.extend(quote! { ? });
                }
                RecogAllow::Value => {}
            }
            desig.to_tokens(Span::call_site(), out);
        }
    }
}

impl RecogAllow {
    fn max() -> Self {
        Self::iter().rev().next().unwrap()
    }
}

//---------- support for checking ----------

pub struct RecogChecker<'r, 'e> {
    /// The actual set of recognised meta attributes
    #[allow(dead_code)] // XXXX
    recog_real: &'r Recognised,
    /// "Working" set of recognised meta attributes
    ///
    /// We add things to this to suppress duplicate error messages,
    /// but that means it has synthetic entries.
    recog_work: Cow<'r, Recognised>,
    errors: &'e mut ErrorAccumulator,
}

impl<'r, 'e> RecogChecker<'r, 'e> {
    pub fn new(
        recog: &'r Recognised,
        errors: &'e mut ErrorAccumulator,
    ) -> Self {
        RecogChecker {
            errors,
            recog_real: recog,
            recog_work: Cow::Borrowed(recog),
        }
    }

    /// Check one meta attribute found in a driver
    pub fn check(
        &mut self,
        pmetas: &[PreprocessedTree],
        supplied: SuppliedScope,
    ) {
        let mut paths = vec![];
        for pmeta in pmetas {
            self.check_1(supplied, &mut paths, pmeta);
        }
    }

    fn check_1<'p>(
        &mut self,
        supplied: SuppliedScope,
        lpaths: &mut Vec<&'p syn::Path>,
        ptree: &'p PreprocessedTree,
    ) {
        use PreprocessedValue as PV;
        use RecogAllow as RA;

        lpaths.push(&ptree.path);
        // closure stops us forgetting to pop again via an early exit
        (|| {
            let want = match &ptree.value {
                PV::Unit => RA::BoolOnly,
                PV::Value { .. } => RA::Value,
                PV::List(l) if l.content.is_empty() => RA::BoolOnly,
                PV::List(l) => {
                    for ptree in &l.content {
                        self.check_1(supplied, lpaths, ptree);
                    }
                    // don't care about *this* path, care about deeper ones
                    return;
                }
            };

            struct Nearest {
                allow: RecogAllow,
                scope: Scope,
                span: Span,
            }

            let mut nearest: Option<Nearest> = None;
            for scope in supplied.recog_search() {
                let test = BorrowedDesig { scope, lpaths };

                if let Some((got_desig, &got)) =
                    self.recog_work.map.get_key_value(&test)
                {
                    if got >= want {
                        // found a justification - a recognised meta
                        // that matches this provided one
                        return;
                    }

                    let span = got_desig
                        .label
                        .lpaths
                        .last()
                        .expect("empty lpaths")
                        .span();

                    if Some(got) > nearest.as_ref().map(|n| n.allow) {
                        let allow = got;
                        nearest = Some(Nearest { allow, scope, span });
                    }
                }
            }

            let scope;
            let error;

            if let Some(found) = nearest {
                scope = found.scope;
                error =
                    [(ptree.path.span(), "provided, in driver"),
                     (found.span, "recognised, in template")].error(
                         match (want, found.allow) {
                             (RA::Value, RA::BoolOnly) =>
 "meta attribute value provided, but is used only as a boolean",
                             (RA::Value, RA::Value) |
                             (RA::BoolOnly, RA::BoolOnly) |
                             (RA::BoolOnly, RA::Value) =>
                                 panic!("want < found"),
                         }
                    );
            } else {
                error = unrecognised_error(
                    &self.recog_real,
                    supplied,
                    ptree.path.span(),
                    lpaths,
                )
                .void_unwrap_err();
                scope = supplied.widest();
            };

            self.errors.push(error);

            // avoid repeated errors, by adding this to recog
            let lpaths = lpaths.iter().copied().cloned().collect();
            let label = Label { lpaths };
            let desig = Desig { scope, label };
            self.recog_work.to_mut().update(desig, RA::max());
        })();
        lpaths.pop();
    }
}

fn unrecognised_error(
    recog_real: &Recognised,
    supplied: SuppliedScope,
    supplied_span: Span,
    lpaths: &[&syn::Path],
) -> Result<Void, syn::Error> /* return type allows (ab)use of ? */ {
    use RecogAllow as RA;

    let try_case = |e: Option<_>| e.map(Err).unwrap_or(Ok(()));
    let some_err = |m: &dyn Display| Some(supplied_span.error(m));

    // Look to see if it would have been recognised in
    // another scope.  That would mean the attr is
    // merely misplaced, rather than totally wrong.
    try_case((|| {
        let y_scopes = Scope::iter()
            .filter(|&scope| {
                recog_real
                    .map
                    .contains_key(&BorrowedDesig { scope, lpaths })
            })
            .collect_vec();

        if y_scopes.is_empty() {
            return None;
        }
        let y_ss = SuppliedScope::iter()
            .filter(|ss| ss.recog_search().any(|s| y_scopes.contains(&s)))
            .map(|ss| ss.to_string())
            .join("/");
        let y_scopes = y_scopes.iter().map(|s| s.as_ref()).join("/");
        some_err(&format_args!(
 "meta attribute provided for {}, but recognised by template only for {} ({})",
            supplied,
            y_ss,
            y_scopes,
        ))
    })())?;

    // Look to see if user supplied `#[deftly(foo(bar))]`
    // when the template wanted `#[deftly(foo = "bar")]`.
    try_case((|| {
        let (upper, recog) = supplied.recog_search().find_map(|scope| {
            let upper = BorrowedDesig {
                scope,
                lpaths: lpaths.split_last()?.1,
            };
            let recog = recog_real.map.get(&upper)?;
            Some((upper, recog))
        })?;
        if *recog != RA::Value {
            return None;
        }
        some_err(&format_args!(
 "nested meta provided and not recognised; but, template would recognise {}",
            DisplayAsIfSpecified {
                lpaths: upper.lpaths,
                inside_after: " = ..",
            },
        ))
    })())?;

    // Look to see if the specified attribute is a prefix of
    // a recognised one
    try_case((|| {
        recog_real.map.keys().find_map(|desig| {
            // deliberately ignore scope
            if !itertools::equal(
                lpaths.iter().copied(),
                desig.label.lpaths.iter().take(lpaths.len()),
            ) {
                return None;
            }
            Some(())
        })?;
        some_err(&format_args!(
 "meta attribute provided, but not recognised; template only uses it as a container"
        ))
    })())?;

    Err(supplied_span
        .error("meta attribute provided, but not recognised by template"))
}

//---------- impl FindRecogMetas ----------

impl<O: SubstParseContext> FindRecogMetas for Template<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        for e in &self.elements {
            e.find_recog_metas(acc)
        }
    }
}

impl<O: SubstParseContext> FindRecogMetas for TemplateElement<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        match self {
            TE::Ident(_) | TE::LitStr(_) | TE::Literal(..) | TE::Punct(..) => {
            }
            TE::Group { template, .. } => template.find_recog_metas(acc),
            TE::Subst(n) => n.find_recog_metas(acc),
            TE::Repeat(n) => n.find_recog_metas(acc),
        }
    }
}

impl<O: SubstParseContext> FindRecogMetas for RepeatedTemplate<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        self.template.find_recog_metas(acc);
        self.whens.find_recog_metas(acc);
    }
}

impl<O: SubstParseContext> FindRecogMetas for Subst<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        self.sd.find_recog_metas(acc);
    }
}

macro_rules! impL_find_recog_metas_via_iter { {
    ( $($gens:tt)* ), $i:ty
} => {
    impl<T: FindRecogMetas, $($gens)*> FindRecogMetas for $i {
        fn find_recog_metas(&self, acc: &mut Recognised) {
            for item in self {
                item.find_recog_metas(acc)
            }
        }
    }
} }
impL_find_recog_metas_via_iter!((), [T]);
impL_find_recog_metas_via_iter!((), Option<T>);
impL_find_recog_metas_via_iter!((U), Punctuated<T, U>);

impl<T: FindRecogMetas> FindRecogMetas for Box<T> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        self.deref().find_recog_metas(acc)
    }
}

impl<O: SubstParseContext> FindRecogMetas for SubstDetails<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        use SD::*;

        match self {
            Xmeta(v) => v.find_recog_metas(acc),
            vpat(v, _, _) => v.find_recog_metas(acc),
            vtype(v, _, _) => v.find_recog_metas(acc),
            tdefvariants(v, _, _) => v.find_recog_metas(acc),
            fdefine(v, _, _) => v.find_recog_metas(acc),
            vdefbody(a, b, _, _) => {
                a.find_recog_metas(acc);
                b.find_recog_metas(acc);
            }
            paste(v, _) => v.find_recog_metas(acc),
            ChangeCase(v, _, _) => v.find_recog_metas(acc),

            when(v, _) => v.find_recog_metas(acc),
            define(v, _) => v.find_recog_metas(acc),
            defcond(v, _) => v.find_recog_metas(acc),

            not(v, _) => v.find_recog_metas(acc),
            any(v, _) | all(v, _) => v.find_recog_metas(acc),
            is_empty(_, v) => v.find_recog_metas(acc),
            approx_equal(_, v) => v.find_recog_metas(acc),

            For(v, _) => v.find_recog_metas(acc),
            If(v, _) | select1(v, _) => v.find_recog_metas(acc),
            ignore(v, _) => v.find_recog_metas(acc),
            dbg(v) => v.content_parsed.find_recog_metas(acc),

            tname(_)
            | ttype(_)
            | tdeftype(_)
            | vname(_)
            | fname(_)
            | ftype(_)
            | fpatname(_)
            | tdefkwd(_)
            | Vis(_, _)
            | tattrs(_, _, _)
            | vattrs(_, _, _)
            | fattrs(_, _, _)
            | tgens(_)
            | tdefgens(_, _)
            | tgnames(_, _)
            | twheres(_, _)
            | UserDefined(_)
            | False(_)
            | True(_)
            | is_struct(_)
            | is_enum(_)
            | is_union(_)
            | v_is_unit(_)
            | v_is_tuple(_)
            | v_is_named(_)
            | error(..)
            | dbg_all_keywords(_)
            | Crate(_, _) => {}
        }
    }
}

impl<O: SubstParseContext> FindRecogMetas for SubstMeta<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let allow = O::meta_recog_allow();
        acc.update(self.desig.clone(), allow);
    }
}

impl FindRecogMetas for SubstVType {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let Self { self_, vname } = self;
        self_.find_recog_metas(acc);
        vname.find_recog_metas(acc);
    }
}

impl FindRecogMetas for SubstVPat {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let Self { vtype, fprefix } = self;
        vtype.find_recog_metas(acc);
        fprefix.find_recog_metas(acc);
    }
}

impl<B: FindRecogMetas> FindRecogMetas for Definition<B> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let Self {
            name: _,
            body_span: _,
            body,
        } = self;
        body.find_recog_metas(acc);
    }
}

impl FindRecogMetas for DefinitionBody {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        match self {
            DefinitionBody::Normal(v) => v.find_recog_metas(acc),
            DefinitionBody::Paste(v) => v.find_recog_metas(acc),
        }
    }
}

impl<O: SubstParseContext> FindRecogMetas for SubstIf<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let Self {
            tests,
            otherwise,
            kw_span: _,
        } = self;
        for (if_, then) in tests {
            if_.find_recog_metas(acc);
            then.find_recog_metas(acc);
        }
        otherwise.find_recog_metas(acc);
    }
}
