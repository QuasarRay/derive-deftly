//! `#[deftly(...)]` meta attributes
//!
//! # Used meta checking
//!
//! Most of this file is concerned with generating
//! accurate and useful messages
//! when a driver is decorated with `#[deftly(...)]` attributes
//! which are not used by any template.
//!
//! We distinguish "used" metas from "recognised" ones.
//!
//! "Used" ones are those actually tested, and used,
//! during the dynamic expansion of the template.
//! They are recorded in the [`PreprocessedMetas`],
//! which contains a `Cell` for each supplied node.
//!
//! "Recognised" ones are those which appear anywhere in the template.
//! These are represented in a data structure [``Recognised`].
//! This is calculated by scanning the template,
//! using the `FindRecogMetas` trait.
//!
//! Both of these sets are threaded through
//! the ACCUM data in successive template expansions;
//! in the final call (`EngineFinalInput`),
//! they are combined together,
//! and the driver's metas are checked against them.

use super::framework::*;

use indexmap::IndexMap;

use PreprocessedContent as PC;
use PreprocessedValue as PV;
use Usage as U;

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

#[derive(Debug)]
pub struct SubstMeta<O: SubstParseContext> {
    pub desig: Desig,
    pub as_: Option<SubstAsSpecified<O>>,
    pub default: Option<(Argument<O>, O::NotInBool, beta::Enabled)>,
    pub quoted: Option<(QuotedMode, O::NotInBool)>,
}

#[derive(Debug)]
pub struct SubstAsSpecified<O: SubstParseContext> {
    pub kw_span: Span,
    pub as_: SubstAs<O>,
    pub nb: O::NotInBool,

    /// This is checked at parse time vs the rest of the struct.
    /// Technically we don't need to store it, but this makes that clearer.
    pub multiplicity: MultiplicityCollector<Void>,
}

#[derive(Debug, Clone, AsRefStr, Display)]
#[allow(non_camel_case_types)] // clearer to use the exact ident
pub enum SubstAs<O: SubstParseContext> {
    expr(AllowTokens<O>, SubstAsSupported<ValueExpr>),
    ident,
    items(AllowTokens<O>, SubstAsSupported<ValueItems>),
    path,
    str,
    token_stream(AllowTokens<O>),
    ty,
    vis(AllowTokens<O>, beta::Enabled),
    attrs(AllowTokens<O>, beta::Enabled),
}

//---------- meta attrs in a driver ----------

/// A part like `(foo,bar(baz),zonk="value")`
#[derive(Debug)]
pub struct PreprocessedContentList {
    pub content: Punctuated<PreprocessedTree, token::Comma>,
}

/// `#[deftly(...)]` helper attributes
pub type PreprocessedMetas = Vec<PreprocessedContentList>;

/// An `#[deftly()]` attribute, or a sub-tree within one
///
/// Has interior mutability, for tracking whether the value is used.
/// (So should ideally not be Clone, to help avoid aliasing bugs.)
#[derive(Debug)]
pub struct PreprocessedTree {
    pub path: syn::Path,
    pub content: PreprocessedContent,
    pub used: Cell<Usage>,
}

/// Content of a meta attribute
///
/// Examples in doc comments are for
/// `PreprocessedMeta.path` of `foo`,
/// ie the examples are for `#[deftly(foo ..)]`.
#[derive(Debug)]
pub enum PreprocessedContent {
    /// `#[deftly(foo)]`
    Unit,
    /// `#[deftly(foo = "lit")]`
    Value { value: PreprocessedValue },
    /// `#[deftly(foo(...))]`
    List(PreprocessedContentList),
}

/// Value of a meta attribute with a value
#[derive(Debug)]
pub enum PreprocessedValue {
    /// `#[deftly(foo = "lit")]`
    Quoted { value: syn::LitStr },

    /// `#[deftly(foo = { ... })]`
    Braced {
        brace_token: syn::token::Brace,
        value: TokenStream,
    },

    /// `#[deftly(foo = ... )]` terminating at a comma
    ///
    /// Parsing is very relaxed at this stage.
    /// Actually *using* this attribute value should insist on
    /// only values that make sense to have bare.
    /// So definitely nothing that can have a `,` in it.
    ///
    /// Things that could start with `{ }` are fun but tolerated
    /// at this stage.    Eg, expressions: if we find `{ }`
    /// we parse it as `Braced` and therefore an expression of the form
    /// `{ BLOCK }.method` must be inside another pair of `{ }`.
    Bare { value: TokenStream },
}

//---------- search and match results ----------

/// Node in tree structure found in driver `#[deftly(some(thing))]`
#[derive(Debug)]
pub struct FoundNode<'l> {
    kind: FoundNodeKind<'l>,
    path_span: Span,
    ptree: &'l PreprocessedTree,
}

/// Node in tree structure found in driver `#[deftly(some(thing))]`
#[derive(Debug)]
pub enum FoundNodeKind<'l> {
    Unit,
    Value(&'l PreprocessedValue),
}

/// Information about a nearby meta node we found
///
/// "Nearby" means that the node we found is a prefix (in tree descent)
/// of the one we were looking for, or vice versa.
#[derive(Debug)]
pub struct FoundNearbyNode<'l> {
    pub kind: FoundNearbyNodeKind,
    /// Span of the identifier in the actual `#[deftly]` driver attribute
    pub path_span: Span,
    pub ptree: &'l PreprocessedTree,
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
pub use FoundNodeKind as FNK;

//---------- multiplicity ----------

/// `as`-specific accumulator for found attribute values of type `T`
///
/// Used as `MultiplicityCollector<FoundNode<'_>>` in [`SubstMeta::expand`].
///
/// The variant selected is determined solely by the `as`
/// (or `Singleton` if not present).
///
/// Also present in `SubstAsSpecified`, as `MultiplicityCollector<Void>`,
/// mostly to prompt us to do the errors cross-checks
/// of the various elements of the [`SubstMeta`].
/// Specifically, that we don't use `default` with any `as` that
/// has `MultiplicityCollector::Several`.
///
/// Always absent in boolean context, because it lives in `SubstAsSpecified`.
#[derive(Debug)]
pub enum MultiplicityCollector<T> {
    /// Value can only be specified once.  Defaults are supported.
    Singleton(Option<T>),

    /// Zero or more are allowed.  Defaults are not supported.
    ///
    /// When expanding, the expansions for each value
    /// can just be concatenated in the output.
    Several(Vec<T>),
}

impl<T> Default for MultiplicityCollector<T> {
    fn default() -> Self {
        MultiplicityCollector::Singleton(Default::default())
    }
}

impl<T> MultiplicityCollector<T> {
    pub fn as_singleton_for_default(
        &self,
    ) -> Result<&Option<T>, impl Display + Debug> {
        use MultiplicityCollector as MC;
        match self {
            MC::Singleton(s) => Ok(s),
            _ => Err("`default` is not allowed with this `as`"),
        }
    }
}

impl MultiplicityCollector<Void> {
    fn start_collecting<'n>(&self) -> MultiplicityCollector<FoundNode<'n>> {
        use MultiplicityCollector as MC;
        macro_rules! variants { { $( $v:ident ),* } => {
            match self { $(
                MC::$v(_) => MC::$v(Default::default()),
            )* }
        } }
        variants! { Singleton, Several }
    }
}

impl<'f> MultiplicityCollector<FoundNode<'f>> {
    pub fn found(
        &mut self,
        self_loc: ErrorLoc<'_>,
        this: FoundNode<'f>,
    ) -> syn::Result<()> {
        use MultiplicityCollector as MC;
        match self {
            MC::Singleton(found) => {
                if let Some(first) = found {
                    return Err([(first.path_span, "first occurrence"),
                     (this.path_span, "second occurrence"),
                     self_loc].error(
 "tried to expand just attribute value, but it was specified multiple times"
                    ));
                }
                *found = Some(this);
                Ok(())
            }
            MC::Several(found) => {
                found.push(this);
                Ok(())
            }
        }
    }

    pub fn iter_for_expand_or_else(
        self,
        on_none: impl FnOnce() -> syn::Error,
    ) -> syn::Result<impl Iterator<Item = FoundNode<'f>>> {
        use MultiplicityCollector as MC;
        Ok(match self {
            MC::Singleton(None) => return Err(on_none()),
            MC::Singleton(Some(n)) => Left(iter::once(n)),
            MC::Several(l) => Right(l.into_iter()),
        })
    }
}

//---------- meta attr enumeration and checking ----------

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq)]
pub struct UsageInfo<M> {
    unit: Option<M>,
    value: Option<M>,
}

pub type Usage = UsageInfo<IsUsed>;

impl Usage {
    pub const BOOL_ONLY: Usage = UsageInfo {
        unit: Some(IsUsed),
        value: None,
    };
    pub const VALUE_ONLY: Usage = UsageInfo {
        unit: None,
        value: Some(IsUsed),
    };
    pub const VALUE: Usage = UsageInfo {
        unit: Some(IsUsed),
        value: Some(IsUsed),
    };
    pub const NONE: Usage = UsageInfo {
        unit: None,
        value: None,
    };
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, strum::EnumIter)]
pub enum UsageMode {
    Unit,
    Value,
}

impl<M> UsageInfo<M> {
    pub fn get(&self, mode: UsageMode) -> Option<&M> {
        match mode {
            UsageMode::Unit => self.unit.as_ref(),
            UsageMode::Value => self.value.as_ref(),
        }
    }
    pub fn get_mut(&mut self, mode: UsageMode) -> &mut Option<M> {
        match mode {
            UsageMode::Unit => &mut self.unit,
            UsageMode::Value => &mut self.value,
        }
    }
}

impl std::ops::BitOr for Usage {
    type Output = Usage;
    fn bitor(self, other: Usage) -> Usage {
        let mut out = self;
        for mode in UsageMode::iter() {
            let ent = out.get_mut(mode);
            *ent = cmp::max(*ent, other.get(mode).copied());
        }
        out
    }
}

#[derive(Default, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct IsUsed;

/// One lot of used metas in accumulation - argument to a `_meta_used` accum
#[derive(Debug)]
pub struct UsedGroup {
    pub content: TokenStream,
}

/// Something representing possibly checking that meta attributes are used
#[derive(Debug, Clone)]
pub enum CheckUsed<T> {
    /// Yes, check them, by/with/from/to `T`
    Check(T),
    /// No, don't check them.
    Unchecked,
}

/// Information for meta checking, found in accumulation
#[derive(Debug, Default)]
pub struct Accum {
    pub recog: Recognised,
    pub used: Vec<UsedGroup>,
}

#[derive(Default, Debug, Clone)]
pub struct Recognised {
    map: IndexMap<Desig, Usage>,
}

pub trait FindRecogMetas {
    /// Search for `fmeta(..)` etc. expansions
    ///
    /// Add to `acc` any that are
    /// (recusively) within `self`, syntactically,
    fn find_recog_metas(&self, acc: &mut Recognised);
}

//==================== implementations) ====================

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

impl<O: SubstParseContext> SubstAsSpecified<O> {
    fn parse(input: ParseStream, nb: O::NotInBool) -> syn::Result<Self> {
        let kw: IdentAny = input.parse()?;
        let kw_span = kw.span();
        let beta = || beta::Enabled::new_for_syntax(kw_span);
        let from_sma = |as_: SubstAs<_>| {
            let multiplicity = as_.multiplicity();
            Ok(SubstAsSpecified {
                //
                as_,
                nb,
                kw_span,
                multiplicity,
            })
        };

        // See keyword_general! in utils.rs
        macro_rules! keyword { { $($args:tt)* } => {
            keyword_general! { kw from_sma SubstAs; $($args)* }
        } }

        let allow_tokens = || O::allow_tokens(&kw);
        fn supported<P>(kw: &IdentAny) -> syn::Result<SubstAsSupported<P>>
        where
            P: SubstAsSupportStatus,
        {
            SubstAsSupportStatus::new(&kw)
        }

        keyword! { expr(allow_tokens()?, supported(&kw)?) }
        keyword! { ident }
        keyword! { items(allow_tokens()?, supported(&kw)?) }
        keyword! { path }
        keyword! { str }
        keyword! { token_stream(allow_tokens()?) }
        keyword! { ty }
        keyword! { vis(allow_tokens()?, beta()?) }
        keyword! { attrs(allow_tokens()?, beta()?) }

        Err(kw.error("unknown derive-deftly 'as' syntax type keyword"))
    }
}

impl<O: SubstParseContext> SubstAs<O> {
    fn multiplicity(&self) -> MultiplicityCollector<Void> {
        use MultiplicityCollector as MC;
        use SubstAs as SA;
        match self {
            SA::attrs(..) => MC::Several(vec![]),
            _ => MC::Singleton(None),
        }
    }
}

impl<O: SubstParseContext> SubstMeta<O> {
    pub fn parse(
        input: ParseStream,
        kw_span: Span,
        scope: Scope,
    ) -> syn::Result<Self> {
        if input.is_empty() {
            O::missing_keyword_arguments(kw_span)?;
        }

        let label: Label = input.parse()?;

        fn store<V>(
            kw: Span,
            already: &mut Option<(Span, V)>,
            call: impl FnOnce() -> syn::Result<V>,
        ) -> syn::Result<()> {
            if let Some((already, _)) = already {
                return Err([(*already, "first"), (kw, "second")]
                    .error("`${Xmeta ..}` option repeated"));
            }
            let v = call()?;
            *already = Some((kw, v));
            Ok(())
        }

        let mut as_ = None::<(Span, SubstAsSpecified<O>)>;
        let mut default = None;
        let mut quoted = None::<(Span, (QuotedMode, O::NotInBool))>;

        while !O::IS_BOOL && !input.is_empty() {
            let keyword = Ident::parse_any(input)?;
            let kw_span = keyword.span();
            let nb = O::not_in_bool(&kw_span).expect("checked already");
            let ue = || beta::Enabled::new_for_syntax(kw_span);
            if keyword == "as" {
                store(kw_span, &mut as_, || {
                    SubstAsSpecified::parse(input, nb)
                })?;
            } else if keyword == "default" {
                store(kw_span, &mut default, || {
                    Ok((input.parse()?, nb, ue()?))
                })?;
            } else if keyword == "quoted" {
                store(kw_span, &mut quoted, || Ok((input.parse()?, nb)))?;
            } else {
                return Err(keyword.error("unknown option in `${Xmeta }`"));
            }
            if input.is_empty() {
                break;
            }
            let _: Token![,] = input.parse()?;
        }

        if let Some((_, as_)) = &as_ {
            if let Some((def_span, _)) = default {
                let _: &Option<Void> = as_
                    .multiplicity
                    .as_singleton_for_default()
                    .map_err(|m| {
                        [(as_.kw_span, "`as` value"), (def_span, "`default`")]
                            .error(m)
                    })?;
            }
        }

        macro_rules! ret { { $( $f:ident )* } => {
            SubstMeta {
                desig: Desig { label, scope },
                $( $f: $f.map(|(_span, v)| v), )*
            }
        } }

        Ok(ret! {
            as_
            default
            quoted
        })
    }
}

//---------- driver parsing ----------

impl PreprocessedContentList {
    fn parse_outer(input: ParseStream) -> syn::Result<Self> {
        let meta;
        let _paren = parenthesized!(meta in input);
        Self::parse_inner(&meta)
    }
}

impl PreprocessedContentList {
    pub fn parse_inner(input: ParseStream) -> syn::Result<Self> {
        let content = Punctuated::parse_terminated(input)?;
        Ok(PreprocessedContentList { content })
    }
}

impl Parse for PreprocessedTree {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path = input.call(syn::Path::parse_mod_style)?;
        let la = input.lookahead1();
        let content = if la.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            PC::Value { value }
        } else if la.peek(token::Paren) {
            let list = input.call(PreprocessedContentList::parse_outer)?;
            PC::List(list)
        } else if la.peek(Token![,]) || input.is_empty() {
            PC::Unit
        } else {
            return Err(la.error());
        };
        let used = Usage::default().into(); // will be filled in later
        Ok(PreprocessedTree {
            path,
            content,
            used,
        })
    }
}

impl Parse for PreprocessedValue {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let la = input.lookahead1();
        let value = if la.peek(syn::LitStr) {
            let value = input.parse()?;
            PV::Quoted { value }
        } else if la.peek(syn::token::Brace) {
            let value;
            let brace_token = braced!(value in input);
            let value = value.parse()?;
            PV::Braced { brace_token, value }
        } else {
            if input.is_empty() {
                QuotedMode::change_here_when_default_changes(
                    // remove the `= ""` from the suggestion
                );
                return Err(input.error(
 r#"un-bracked, un-quoted meta values may not be empty; write `= ""` or `= {}` instead"#,
                ));
            }
            let mut value = TokenStream::new();
            while !(input.is_empty() || input.peek(Token![,])) {
                let tt: TT = input.parse()?;
                const FORBID: &[char] = &['<', '>', '='];
                if let TT::Punct(p) = &tt {
                    if FORBID.contains(&p.as_char()) {
                        QuotedMode::change_here_when_default_changes(
                            // remove the `= "..."` from the suggestion
                        );
                        return Err(tt.error(
 r#"un-braced, un-quoted meta values may not contain any of `<` `>` `=`; write `= "..." or `= { ... }` instead"#
                        ));
                    }
                }
                value.extend([tt]);
            }
            PV::Bare { value }
        };
        Ok(value)
    }
}

impl PreprocessedValue {
    pub fn span(&self) -> Span {
        match self {
            PV::Quoted { value } => value.span(),
            PV::Braced { brace_token, .. } => brace_token.span.span(),
            PV::Bare { value } => value.span(),
        }
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

//---------- extension methods on syn::Attribute ----------

/// Helper trait to deal with syn's idea of attribute contents
///
/// We expect all our attributes to be parenthesised.
pub trait AttributeExt12 {
    /// Parse the content within an `#[attribute(content)]`
    ///
    /// Parses the `content` as `T`.
    fn parse_in_parens<T: Parse>(&self) -> syn::Result<T> {
        self.call_in_parens(Parse::parse)
    }

    /// Like `parse_in_parens`, but takes a parser callback.
    fn call_in_parens<T, F>(&self, f: F) -> syn::Result<T>
    where
        F: FnOnce(ParseStream<'_>) -> syn::Result<T>;
}

impl AttributeExt12 for syn::Attribute {
    fn call_in_parens<T, F>(&self, f: F) -> syn::Result<T>
    where
        F: FnOnce(ParseStream<'_>) -> syn::Result<T>,
    {
        let list = self.meta.require_list()?;
        let _paren: syn::token::Paren = match list.delimiter {
            syn::MacroDelimiter::Paren(p) => p,
            _ => return Err(list.error("expected parenthesised attributes")),
        };
        f.parse2(list.tokens.clone())
    }
}

//---------- searching and matching ----------

impl Label {
    /// Caller must note meta attrs that end up being used!
    pub fn search<'a, F, G, E>(
        &self,
        pmetas: &'a [PreprocessedContentList],
        f: &mut F,
        g: &mut G,
    ) -> Result<(), E>
    where
        F: FnMut(FoundNode<'a>) -> Result<(), E>,
        G: FnMut(FoundNearbyNode<'a>) -> Result<(), E>,
    {
        for m in pmetas {
            for l in &m.content {
                Self::search_1(&self.lpaths, l, &mut *f, &mut *g)?;
            }
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
        G: FnMut(FoundNearbyNode<'a>) -> Result<(), E>,
    {
        if ptree.path != lpaths[0] {
            return Ok(());
        }

        let path_span = ptree.path.span();

        let mut nearby = |kind| {
            g(FoundNearbyNode {
                kind,
                path_span,
                ptree,
            })
        };

        let deeper = if lpaths.len() <= 1 {
            None
        } else {
            Some(&lpaths[1..])
        };

        match (deeper, &ptree.content) {
            (None, PC::Unit) => f(FoundNode {
                path_span,
                kind: FNK::Unit,
                ptree,
            })?,
            (None, PC::List(_)) => nearby(FNNK::List)?,
            (None, PC::Value { value, .. }) => f(FoundNode {
                path_span,
                kind: FNK::Value(value),
                ptree,
            })?,
            (Some(_), PC::Value { .. }) => nearby(FNNK::Lit)?,
            (Some(_), PC::Unit) => nearby(FNNK::Unit)?,
            (Some(d), PC::List(l)) => {
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
        let found = |ptree: &PreprocessedTree| {
            ptree.update_used(Usage::BOOL_ONLY);
            Err(Found)
        };
        self.search(
            pmetas,
            &mut |av| /* got it! */ found(av.ptree),
            &mut |nearby| match nearby.kind {
                FNNK::List => found(nearby.ptree),
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
            Scope::T => &ctx.pmetas,
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
            let group = group_new_with_span(
                Delimiter::Parenthesis,
                current.span(),
                current,
            );
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

//---------- conditional support for `Xmeta as items` ----------

#[cfg(feature = "meta-as-expr")]
pub type ValueExpr = syn::Expr;
#[cfg(not(feature = "meta-as-expr"))]
pub type ValueExpr = MetaUnsupported;

#[cfg(feature = "meta-as-items")]
pub type ValueItems = Concatenated<syn::Item>;
#[cfg(not(feature = "meta-as-items"))]
pub type ValueItems = MetaUnsupported;

/// newtype to avoid coherence - it doesn't impl `Parse + ToTokens`
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)] // rust-lang/rust/issues/145936
pub struct MetaUnsupported(Void);

#[derive(Debug, Copy, Clone)]
pub struct SubstAsSupported<P: SubstAsSupportStatus>(P::Marker);

/// Implemented for syn types supported in this build, and `MetaUnsupported`
pub trait SubstAsSupportStatus: Sized {
    type Marker;
    type Parsed: Parse + ToTokens;
    fn new(kw: &IdentAny) -> syn::Result<SubstAsSupported<Self>>;
}

impl<P: SubstAsSupportStatus> SubstAsSupported<P> {
    fn infer_type(&self, _parsed: &P::Parsed) {}
}

impl<T: Parse + ToTokens> SubstAsSupportStatus for T {
    type Marker = ();
    type Parsed = T;
    fn new(_kw: &IdentAny) -> syn::Result<SubstAsSupported<Self>> {
        Ok(SubstAsSupported(()))
    }
}

impl SubstAsSupportStatus for MetaUnsupported {
    type Marker = MetaUnsupported;
    type Parsed = TokenStream;
    fn new(kw: &IdentAny) -> syn::Result<SubstAsSupported<Self>> {
        Err(kw.error(format_args!(
            // We're a bit fast and loose here: if kw contained `_`,
            // or there were aliases, this message would be a bit wrong.
            "${{Xmeta as {}}} used but cargo feature meta-as-{} disabled",
            **kw, **kw,
        )))
    }
}

impl ToTokens for MetaUnsupported {
    fn to_tokens(&self, _out: &mut TokenStream) {
        void::unreachable(self.0)
    }
}

//---------- quoted modes and allows ----------

/// Handling mode derived from the `as`
///
/// This is the distillation of the `as`-specific
/// quoted/braced/bare handling,
/// from the `as` arms in [`FoundNode::expand`].
#[derive(Debug)]
enum AsValueHandling {
    /// "Normal" handling
    ///
    /// * double quotes are always stripped and content reparsed
    /// * bare values *are* allowed
    Normal,

    /// Handling for `as expr`
    ///
    /// * double quotes are *sometimes* stripped depending on `quoted` options
    /// * bare values *are* allowed
    Expr,

    /// Syntax is not suitable for including bare
    ///
    /// * double quotes are always stripped and content reparsed
    /// * bare values are *not* allowed
    NoBare,
}

/// Quote handling mode specified by template
///
/// `quoted strip`, `meta_quoted retain` etc.
#[derive(Debug, Clone, Copy)]
pub struct QuotedMode {
    pub mode: QuotedModeMode,
    #[allow(unused)]
    pub beta: beta::Enabled,
}

/// Just the quote handling mode specified by template
///
/// Beta is unchecked here and there is no span.
#[derive(Debug, Clone, Copy, Eq, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum QuotedModeMode {
    /// Future default
    Strip,
    // TODO should we call this `strict`?  But that is very like `strip`!
    Rigorous,
    Retain,
}

impl Parse for QuotedMode {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mode: DdOptVal<QuotedModeMode> = input.parse()?;
        let beta = beta::Enabled::new_for_syntax(mode.span)?;
        let mode = mode.value;
        Ok(QuotedMode { mode, beta })
    }
}

impl QuotedMode {
    /// Called everywhere we need to make a change
    /// when the default `quoted` changes
    ///
    /// We have tried to make this fail to compile when that happens.
    /// Resolve by deleting this function and following the instructions
    /// at call cites.
    ///
    /// # Example
    ///
    /// ```ignore
    /// QuotedMode::change_here_when_default_changes(
    ///     // remove the spurious wombat
    /// );
    /// ```
    pub fn change_here_when_default_changes() {
        matches!(
            None, //
            Some(QuotedMode { beta: _, .. }),
        );
        matches!(
            None,
            Some(SubstMeta::<TokenAccumulator> { quoted: None, .. })
        );
        matches!(
            None,
            Some(DdOptions {
                meta_quoted: None,
                ..
            })
        );
        matches!(
            None.map(|eo| DdOptions::meta_quoted_default(eo)),
            Some(Ok(Some::<QuotedMode>(_))),
        );
    }
}

//---------- template expansion ----------

impl<O> SubstMeta<O>
where
    O: ExpansionOutput,
    TemplateElement<O>: Expand<O>,
{
    pub fn expand(
        &self,
        ctx: &Context,
        kw_span: Span,
        out: &mut O,
        pmetas: &PreprocessedMetas,
    ) -> syn::Result<()> {
        let SubstMeta {
            desig,
            as_,
            default,
            quoted,
        } = self;
        let mut hint = None::<FoundNearbyNode>;
        let span_whole = self.span_whole(kw_span);
        let self_loc = (span_whole, "expansion");
        let error_loc = [ctx.error_loc(), self_loc];

        let mut found = self
            .as_
            .as_ref()
            .map(|as_| as_.multiplicity.start_collecting())
            .unwrap_or_default();

        desig.label.search(
            pmetas,
            &mut |av: FoundNode| found.found(self_loc, av),
            &mut |nearby| {
                hint.get_or_insert(nearby);
                Ok(())
            },
        )?;

        if let Some((def, ..)) = default {
            if match found
                .as_singleton_for_default()
                .expect("erroneous multiplicity not detected at parse")
            {
                None => true,
                Some(FoundNode {
                    kind: FNK::Unit,
                    ptree,
                    ..
                }) => {
                    // Specified as unit, but we have a default.
                    // So ignore the unit for these purposes.
                    ptree.update_used(Usage::VALUE_ONLY);
                    true
                }
                _ => false,
            } {
                return Ok(def.expand(ctx.as_general(), out));
            }
        }

        let found = found.iter_for_expand_or_else(|| {
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
                err.combine(error_loc.error(
 "attribute value expanded, but no suitable value in data structure definition"
                ));
                err
            } else {
                error_loc.error(
 "attribute value expanded, but no value in data structure definition"
                )
            }
        })?;

        let quoted = quoted.map(|(q, _nb)| q).or(ctx.meta_quoted);

        for found in found {
            found.ptree.update_used(Usage::VALUE);
            found.expand(span_whole, as_, quoted, out)?;
        }

        Ok(())
    }
}

fn metavalue_spans(tspan: Span, vspan: Span) -> [ErrorLoc<'static>; 2] {
    [(vspan, "attribute value"), (tspan, "template")]
}

/// Obtain the `LiStr` from a meta node value (ie, a `Lit`)
///
/// This is the thing we actually use.
/// Non-string-literal values are not allowed.
fn metavalue_litstr<'l>(
    val: &'l PreprocessedValue,
    tspan: Span,
    msg: fmt::Arguments<'_>,
) -> syn::Result<&'l syn::LitStr> {
    match val {
        PV::Quoted { value } => Ok(value),
        // having checked derive_builder, it doesn't handle
        // Lit::Verbatim so I guess we don't need to either.
        _ => Err(metavalue_spans(tspan, val.span()).error(msg)),
    }
}

/// Common arguments to `metavalue_as`
///
/// Logically speaking, these are arguments to `metavalue_as`.
/// They're the ones that are the same for (nearly) all the arms in
/// [`FoundNode::expand`].
/// That it much easier to see the wood for the trees there.
struct MetaValueAsInput<'i> {
    val: &'i PreprocessedValue,
    quoted: Option<QuotedModeMode>,
    tspan: Span,
    into_what: &'i dyn Display,
}

/// Convert a literal found in a meta item into `T`
///
/// `into_what` is used only for error reporting
fn metavalue_as<T>(
    as_value_handling: AsValueHandling,
    MetaValueAsInput {
        val,
        quoted,
        tspan,
        into_what,
    }: MetaValueAsInput<'_>,
) -> syn::Result<T>
where
    T: Parse + ToTokens,
{
    use AsValueHandling as AVH;
    use QuotedModeMode as QMM;

    let err = |m| {
        Err([(val.span(), "attribute value"), (tspan, "template")].error(m))
    };

    let quoted = match (quoted, val) {
        (None, PV::Braced { .. }) | (None, PV::Bare { .. }) => {
            QuotedMode::change_here_when_default_changes(
                // get rid of this this whole arm - and the Option?
                // and get rid of `change_here_when_default_changes` too
            );
            return err(format_args!(
                "expected string literal, for conversion to {}",
                into_what,
            ));
        }
        (None, PV::Quoted { .. }) => QMM::Strip, // future default
        (Some(q), _) => q,
    };

    let t = match val {
        PV::Braced { value, brace_token } => match as_value_handling {
            AVH::Normal | AVH::NoBare => value.clone(),
            // If the value was braced, reconstruct the Group, so that
            //     #[deftly(e = { stmt; value })]
            // works as expected.  `( )` are added later by FoundNode::expand.
            AVH::Expr => {
                let span = brace_token.span.span();
                let mut group = proc_macro2::Group::new(
                    Delimiter::Brace,
                    value.clone(),
                );
                group.set_span(span);
                group.to_token_stream()
            }
        },
        PV::Quoted { value } => match as_value_handling {
            AVH::Normal | AVH::NoBare => metavalue_lit_reparse(value)?,
            AVH::Expr => match quoted {
                QMM::Strip => metavalue_lit_reparse(value)?,
                QMM::Retain => value.to_token_stream(),
                QMM::Rigorous => {
                    return err(format_args!(
                        r#"string literal attribute value is ambiguous; use `( )` or `{{ }}`"#
                    ))
                }
            },
        },
        PV::Bare { value } => match as_value_handling {
            AVH::Normal | AVH::Expr => value.clone(),
            AVH::NoBare => {
                return err(format_args!(
                    r#"attribute value for conversion to {} must be enclosed in braces {{..}} or quotes "..""#,
                    into_what,
                ))
            }
        },
    };
    let thing: T = syn::parse2(t.clone()).map_err(|mut e| {
        e.combine(tspan.error(format_args!(
            "failed to parse attribute value `{}` as {} (here in template)",
            t, into_what,
        )));
        e
    })?;
    Ok(thing)
}

fn metavalue_lit_reparse(s: &syn::LitStr) -> syn::Result<TokenStream> {
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
            None => s.span().error(e.to_string()),
        }
    })?;

    Ok(t)
}

impl<'l> FoundNode<'l> {
    fn expand<O>(
        &self,
        tspan: Span,
        as_: &Option<SubstAsSpecified<O>>,
        quoted: Option<QuotedMode>,
        out: &mut O,
    ) -> syn::Result<()>
    where
        O: ExpansionOutput,
    {
        let spans = |vspan| metavalue_spans(tspan, vspan);

        let val = match self.kind {
            FNK::Unit => return Err(spans(self.path_span).error(
 "tried to expand attribute which is just a unit, not a literal"
            )),
            FNK::Value(value) => value,
        };

        use AsValueHandling as AVH;
        use SubstAs as SA;

        let default_buf;
        let as_ = match as_ {
            Some(as_) => &as_.as_,
            None => {
                default_buf = O::default_subst_meta_as(tspan)?;
                &default_buf
            }
        };
        let quoted = quoted.map(|q| q.mode);

        let input = MetaValueAsInput {
            val,
            tspan,
            quoted,
            into_what: as_,
        };

        match as_ {
            SA::expr(.., at, supported) => {
                // Any surrounding `{ }` are re-added by metavalue_as
                let expr = metavalue_as(AVH::Expr, input)?;
                supported.infer_type(&expr);
                // Now surround with `( )` for precedence reasons.
                // (And, if we surrounded with `{ }` above, to turn the
                // block into an expression that's useable everywhere, for eg
                //     #[deftly(fn = { |_| .. })]
                // and then
                //     ${fmeta(fn as expr)}(arg)
                // which we need to expand to
                //     ({ |_| .. })(arg)
                // not
                //     { |_| .. }(arg)   // <- doesn't compile
                let content = Grouping::Parens.surround(expr.span(), expr);
                out.append_tokens(at, content)?;
            }
            SA::ident => {
                let ident: IdentAny = metavalue_as(AVH::Normal, input)?;
                out.append_identfrag_toks(&*ident)?;
            }
            SA::items(at, supported) => {
                let items = metavalue_as(AVH::NoBare, input)?;
                supported.infer_type(&items);
                out.append_tokens(at, items)?;
            }
            SA::path => out.append_syn_type(
                tspan,
                syn::Type::Path(metavalue_as(AVH::Normal, input)?),
                Grouping::Invisible,
            ),
            SA::str => {
                let s = metavalue_litstr(
                    val,
                    tspan,
                    format_args!("expected string literal, for meta value",),
                )?;
                out.append_syn_litstr(s, None);
            }
            SA::ty => out.append_syn_type(
                tspan,
                metavalue_as(AVH::Normal, input)?,
                Grouping::Invisible,
            ),
            SA::vis(at, ..) => {
                let vis: syn::Visibility = metavalue_as(AVH::Normal, input)?;
                out.append_tokens(at, vis)?;
            }
            SA::attrs(at, _) => {
                let attrs: UserAttrs = metavalue_as(AVH::Normal, input)?;
                out.append_tokens(at, attrs)?;
            }
            SA::token_stream(at) => {
                let tokens: TokenStream = metavalue_as(
                    AVH::NoBare,
                    MetaValueAsInput {
                        into_what: &"tokens",
                        ..input
                    },
                )?;
                out.append_tokens(at, tokens)?;
            }
        }
        Ok(())
    }
}

//---------- UserAttrs - values for `as attrs` ----------

struct UserAttrs {
    attrs: Vec<UserAttr>,
}

struct UserAttr {
    hash: Token![#],
    content: proc_macro2::Group,
}

impl Parse for UserAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let la = input.lookahead1();
        let mut attrs = vec![];
        if la.peek(Ident::peek_any) {
            while !input.is_empty() {
                let mut content = TokenStream::new();
                let kw: IdentAny = input.parse()?;
                let span = kw.span();
                content.extend([TT::Ident(kw.0)]);
                loop {
                    if input.peek(Token![,]) {
                        let _: Token![,] = input.parse()?;
                        break;
                    }
                    if input.is_empty() {
                        break;
                    }
                    let tt: TokenTree = input.parse()?;
                    content.extend([tt]);
                }
                let mut content =
                    proc_macro2::Group::new(Delimiter::Bracket, content);
                content.set_span(span);
                let hash = Token![#](span);
                attrs.push(UserAttr { hash, content });
                // We have eaten the comma, or found EOF
            }
        } else if la.peek(Token![#]) {
            while !input.is_empty() {
                let hash = input.parse()?;
                let content: proc_macro2::Group = input.parse()?;
                if content.delimiter() != Delimiter::Bracket {
                    return Err(content
                        .span_open()
                        .error("expected #[ ], not some other delimiter"));
                }
                attrs.push(UserAttr { hash, content });
            }
        } else if input.is_empty() {
        } else {
            return Err(la.error());
        }
        Ok(UserAttrs { attrs })
    }
}

impl ToTokens for UserAttrs {
    fn to_tokens(&self, out: &mut TokenStream) {
        for UserAttr { hash, content } in &self.attrs {
            hash.to_tokens(out);
            content.to_tokens(out);
        }
    }
}

//==================== implementations - usage checking ====================

impl Parse for CheckUsed<UsedGroup> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let la = input.lookahead1();
        Ok(if la.peek(Token![*]) {
            let _star: Token![*] = input.parse()?;
            mCU::Unchecked
        } else if la.peek(token::Bracket) {
            let group: proc_macro2::Group = input.parse()?;
            let content = group.stream();
            mCU::Check(UsedGroup { content })
        } else {
            return Err(la.error());
        })
    }
}

impl Recognised {
    /// Ensures that `self[k] >= v`
    pub fn update(&mut self, k: Desig, v: Usage) {
        let ent = self.map.entry(k).or_insert(v);
        *ent = *ent | v
    }
}

impl ToTokens for Recognised {
    fn to_tokens(&self, out: &mut TokenStream) {
        for (desig, allow) in &self.map {
            match *allow {
                Usage::BOOL_ONLY => out.extend(quote! { ? }),
                Usage::VALUE_ONLY => out.extend(quote! { + }),
                Usage::VALUE => {}
                Usage::NONE => panic!("should be impossible!"),
            }
            desig.to_tokens(Span::call_site(), out);
        }
    }
}

impl PreprocessedTree {
    pub fn update_used(&self, ra: Usage) {
        self.used.set(self.used.get() | ra);
    }
}

//---------- decoding used metas ----------

impl PreprocessedContentList {
    fn decode_update_used(&self, input: ParseStream) -> syn::Result<()> {
        for ptree in &self.content {
            if input.is_empty() {
                return Ok(());
            }
            if !input.peek(Token![,]) {
                let path = input.call(syn::Path::parse_mod_style)?;
                if path != ptree.path {
                    return Err([
                        (path.span(), "found"),
                        (ptree.path.span(), "expected"),
                    ].error(
 "mismatch (desynchronised) incorporating previous expansions' used metas"
                    ));
                }
                let used = if input.peek(Token![=]) {
                    let _: Token![=] = input.parse()?;
                    Some(Usage::VALUE)
                } else if input.peek(Token![?]) {
                    let _: Token![?] = input.parse()?;
                    Some(Usage::BOOL_ONLY)
                } else if input.peek(Token![+]) {
                    let _: Token![+] = input.parse()?;
                    Some(Usage::VALUE_ONLY)
                } else {
                    None
                };
                if let Some(used) = used {
                    ptree.update_used(used);
                }
                if input.peek(token::Paren) {
                    let inner;
                    let paren = parenthesized!(inner in input);
                    let sub_list = match &ptree.content {
                        PC::Unit | PC::Value { .. } => return Err([
                            (paren.span.open(), "found"),
                            (ptree.path.span(), "defined"),
                        ].error(
 "mismatch (tree vs terminal) incorporating previous expansions' used metas"
                        )),
                        PC::List(l) => l,
                    };
                    sub_list.decode_update_used(&inner)?;
                }
            }
            if input.is_empty() {
                return Ok(());
            }
            let _: Token![,] = input.parse()?;
        }
        Ok(())
    }
}

impl<'c> Context<'c> {
    pub fn decode_update_metas_used(
        &self,
        input: /* group content */ ParseStream,
    ) -> syn::Result<()> {
        #[derive(Default)]
        struct Intended {
            variant: Option<syn::Ident>,
            field: Option<Either<syn::Ident, u32>>,
            attr_i: usize,
        }
        let mut intended = Intended::default();

        let mut visit =
            |pmetas: &PreprocessedMetas,
             current_variant: Option<&syn::Ident>,
             current_field: Option<Either<&syn::Ident, &u32>>| {
                loop {
                    let la = input.lookahead1();
                    if input.is_empty() {
                        // keep visiting until we exit all the loops
                        return Ok(());
                    } else if la.peek(Token![::]) {
                        let _: Token![::] = input.parse()?;
                        intended = Intended {
                            variant: Some(input.parse()?),
                            field: None,
                            attr_i: 0,
                        };
                    } else if la.peek(Token![.]) {
                        let _: Token![.] = input.parse()?;
                        intended.field = Some(match input.parse()? {
                            syn::Member::Named(n) => Either::Left(n),
                            syn::Member::Unnamed(i) => Either::Right(i.index),
                        });
                        intended.attr_i = 0;
                    } else if {
                        let intended_field_refish = intended
                            .field
                            .as_ref()
                            .map(|some: &Either<_, _>| some.as_ref());

                        !(current_variant == intended.variant.as_ref()
                            && current_field == intended_field_refish)
                    } {
                        // visit subsequent things, hopefully one will match
                        return Ok(());
                    } else if la.peek(token::Paren) {
                        // we're in the right place and have found a #[deftly()]
                        let i = intended.attr_i;
                        intended.attr_i += 1;
                        let m = pmetas.get(i).ok_or_else(|| {
                            input.error("more used metas, out of range!")
                        })?;
                        let r;
                        let _ = parenthesized!(r in input);
                        m.decode_update_used(&r)?;
                    } else {
                        return Err(la.error());
                    }
                }
            };

        visit(&self.pmetas, None, None)?;

        WithinVariant::for_each(self, |ctx, wv| {
            let current_variant = wv.variant.map(|wv| &wv.ident);

            if !wv.is_struct_toplevel_as_variant() {
                visit(&wv.pmetas, current_variant, None)?;
            }

            WithinField::for_each(ctx, |_ctx, wf| {
                let current_field = if let Some(ref ident) = wf.field.ident {
                    Either::Left(ident)
                } else {
                    Either::Right(&wf.index)
                };

                visit(&wf.pfield.pmetas, current_variant, Some(current_field))
            })
        })

        // if we didn't consume all of the input, due to mismatches/
        // misordering, then syn will give an error for us
    }
}

//---------- encoding used metas ---------

impl PreprocessedTree {
    /// Returns `(....)`
    fn encode_useds(
        list: &PreprocessedContentList,
    ) -> Option<proc_macro2::Group> {
        let preamble = syn::parse::Nothing;
        let sep = Token![,](Span::call_site());
        let mut ts = TokenStream::new();
        let mut ot = TokenOutputTrimmer::new(&mut ts, &preamble, &sep);
        for t in &list.content {
            t.encode_used(&mut ot);
            ot.push_sep();
        }
        if ts.is_empty() {
            None
        } else {
            Some(proc_macro2::Group::new(Delimiter::Parenthesis, ts))
        }
    }

    /// Writes `path?=(...)` (or, rather, the parts of it that are needed)
    fn encode_used(&self, out: &mut TokenOutputTrimmer) {
        struct OutputTrimmerWrapper<'or, 'o, 't, 'p> {
            // None if we have written the path already
            path: Option<&'p syn::Path>,
            out: &'or mut TokenOutputTrimmer<'t, 'o>,
        }

        let mut out = OutputTrimmerWrapper {
            path: Some(&self.path),
            out,
        };

        impl OutputTrimmerWrapper<'_, '_, '_, '_> {
            fn push_reified(&mut self, t: &dyn ToTokens) {
                if let Some(path) = self.path.take() {
                    self.out.push_reified(path);
                }
                self.out.push_reified(t);
            }
        }

        let tspan = Span::call_site();

        match self.used.get() {
            Usage::BOOL_ONLY => out.push_reified(&Token![?](tspan)),
            Usage::VALUE => out.push_reified(&Token![=](tspan)),
            Usage::VALUE_ONLY => out.push_reified(&Token![+](tspan)),
            Usage::NONE => {}
        }

        match &self.content {
            PC::Unit | PC::Value { .. } => {}
            PC::List(l) => {
                if let Some(group) = PreprocessedTree::encode_useds(l) {
                    out.push_reified(&group);
                }
            }
        }
    }
}

impl<'c> Context<'c> {
    /// Returns `[::Variant .field () ...]`
    pub fn encode_metas_used(&self) -> Result<proc_macro2::Group, ()> {
        let _: () = self.any_error_occurred.get()?;

        let parenthesize =
            |ts| proc_macro2::Group::new(Delimiter::Parenthesis, ts);
        let an_empty = parenthesize(TokenStream::new());

        let mut ts = TokenStream::new();

        struct Preamble<'p> {
            variant: Option<&'p syn::Variant>,
            field: Option<&'p WithinField<'p>>,
        }
        impl ToTokens for Preamble<'_> {
            fn to_tokens(&self, out: &mut TokenStream) {
                let span = Span::call_site();
                if let Some(v) = self.variant {
                    Token![::](span).to_tokens(out);
                    v.ident.to_tokens(out);
                }
                if let Some(f) = self.field {
                    Token![.](span).to_tokens(out);
                    f.fname(span).to_tokens(out);
                }
            }
        }

        let mut last_variant: *const syn::Variant = ptr::null();
        let mut last_field: *const syn::Field = ptr::null();

        fn ptr_of_ref<'i, InDi>(r: Option<&'i InDi>) -> *const InDi {
            r.map(|r| r as _).unwrap_or_else(ptr::null)
        }

        let mut encode = |pmetas: &PreprocessedMetas,
                          wv: Option<&WithinVariant>,
                          wf: Option<&WithinField>| {
            let now_variant: *const syn::Variant =
                ptr_of_ref(wv.map(|wv| wv.variant).flatten());
            let now_field: *const syn::Field =
                ptr_of_ref(wf.map(|wf| wf.field));

            let preamble = Preamble {
                variant: (!ptr::eq(last_variant, now_variant)).then(|| {
                    last_field = ptr::null();
                    let v = wv.expect("had WithinVariant, now not");
                    v.variant.expect("variant was syn::Variant, now not")
                }),
                field: (!ptr::eq(last_field, now_field)).then(|| {
                    wf.expect("had WithinField (Field), now not") //
                }),
            };
            let mut ot =
                TokenOutputTrimmer::new(&mut ts, &preamble, &an_empty);
            for m in pmetas {
                if let Some(group) = PreprocessedTree::encode_useds(m) {
                    ot.push_reified(group);
                } else {
                    ot.push_sep();
                }
            }
            if ot.did_preamble().is_some() {
                last_variant = now_variant;
                last_field = now_field;
            }

            Ok::<_, Void>(())
        };

        encode(&self.pmetas, None, None).void_unwrap();
        WithinVariant::for_each(self, |ctx, wv| {
            if !wv.is_struct_toplevel_as_variant() {
                encode(&wv.pmetas, Some(wv), None)?;
            }
            WithinField::for_each(ctx, |_ctx, wf| {
                encode(&wf.pfield.pmetas, Some(wv), Some(wf))
            })
        })
        .void_unwrap();

        Ok(proc_macro2::Group::new(Delimiter::Bracket, ts))
    }
}

//---------- checking used metas ----------

struct UsedChecker<'c, 'e> {
    current: Vec<&'c syn::Path>,
    reported: &'e mut HashSet<Label>,
    recog: &'c Recognised,
    supplied_scope: SuppliedScope,
    errors: &'e mut ErrorAccumulator,
}

impl PreprocessedTree {
    fn check_used<'c>(&'c self, checker: &mut UsedChecker<'c, '_>) {
        checker.current.push(&self.path);

        let mut err = |err| {
            let lpaths = checker.current.iter().copied().cloned().collect();
            if checker.reported.insert(Label { lpaths }) {
                checker.errors.push(err);
            }
        };

        let mut need = |supplied_mode| {
            let used = self.used.get();
            match used.get(supplied_mode) {
                Some(IsUsed) => {}
                None => err(unrecognised_error(
                    checker.recog,
                    checker.supplied_scope,
                    supplied_mode,
                    self.path.span(),
                    used,
                    &checker.current,
                )
                .void_unwrap_err()),
            }
        };

        match &self.content {
            PC::Unit => need(UsageMode::Unit),
            PC::Value { .. } => need(UsageMode::Value),
            PC::List(l) => {
                if l.content.is_empty() {
                    err(self.path.error(
 "empty nested list in #[deftly], is not useable by any template"
                    ));
                }
                for subtree in &l.content {
                    subtree.check_used(checker);
                }
            }
        }

        checker
            .current
            .pop()
            .expect("pushed earlier, but can't pop?");
    }
}

impl<'c> Context<'c> {
    pub(crate) fn check_metas_used(
        &self,
        errors: &mut ErrorAccumulator,
        recog: &Recognised,
    ) {
        use SuppliedScope as SS;

        let mut reported = HashSet::new();

        let mut chk_pmetas = |supplied_scope, pmetas: &PreprocessedMetas| {
            let mut checker = UsedChecker {
                reported: &mut reported,
                current: vec![],
                recog,
                errors,
                supplied_scope,
            };

            for a in pmetas {
                for l in &a.content {
                    l.check_used(&mut checker);
                }
            }

            Ok::<_, Void>(())
        };

        chk_pmetas(
            match &self.top.data {
                syn::Data::Struct(_) | syn::Data::Union(_) => SS::Struct,
                syn::Data::Enum(_) => SS::Enum,
            },
            &self.pmetas,
        )
        .void_unwrap();

        WithinVariant::for_each(self, |ctx, wv| {
            // If variant is None, this is the imaginary variant for
            // the toplevel, and it has a copy of the ref to the metas,
            // in which case we don't want to process it again.
            if !wv.is_struct_toplevel_as_variant() {
                chk_pmetas(SS::Variant, &wv.pmetas)?;
            }
            WithinField::for_each(ctx, |_ctx, wf| {
                chk_pmetas(SS::Field, &wf.pfield.pmetas)
            })
        })
        .void_unwrap();
    }
}

fn unrecognised_error(
    recog: &Recognised,
    supplied_scope: SuppliedScope,
    supplied_mode: UsageMode,
    supplied_span: Span,
    used: Usage,
    lpaths: &[&syn::Path],
) -> Result<Void, syn::Error> /* return type allows (ab)use of ? */ {
    // This could have been a method on UsedChecker, but that runs into
    // borrowck trouble.

    let try_case = |e: Option<_>| e.map(Err).unwrap_or(Ok(()));
    let some_err = |m: &dyn Display| Some(supplied_span.error(m));

    // Maybe this would have been recognised in other circumstances.
    // If so, report that.
    try_case((|| {
        let recog_allow = supplied_scope
            .recog_search()
            .map(|scope| {
                recog
                    .map
                    .get(&BorrowedDesig { scope, lpaths })
                    .copied()
                    .unwrap_or_default()
            })
            .reduce(std::ops::BitOr::bitor)
            .unwrap_or_default();

        // Have we supplied more than would ever be recognised?
        let _: &IsUsed = recog_allow.get(supplied_mode)?;

        match used {
            U::NONE => some_err(
 &"meta attribute provided, and (conditionally) recognised; but not used in these particular circumstances"
            ),
            U::BOOL_ONLY => some_err(
 &"meta attribute provided with value, and (conditionally) recognised with value; but in these particular circumstances only used as a boolean"
            ),
            U::VALUE_ONLY => some_err(
 &"meta attribute provided as a unit, and (conditionally) recognised as such; but in these particular circumstances only used in contexts with a default value, so the unit is ignored"
            ),
            U::VALUE => unreachable!(),
        }
    })())?;

    // Now we know that even the static scan doesn't
    // recognise this item.

    // Maybe it's just that a value was supplied by mistake
    try_case((|| {
        (supplied_mode == UsageMode::Value).then(|| ())?;
        let _: IsUsed = used.unit?;
        some_err(
            &"meta attribute value provided, but is used only as a boolean",
        )
    })())?;

    // Maybe it's just that a *unit* was supplied by mistake
    try_case((|| {
        (supplied_mode == UsageMode::Unit).then(|| ())?;
        let _: IsUsed = used.value?;
        some_err(
 &"meta attribute provided as unit (flag), but is used only with default values, so the unit is ignored"
        )
    })())?;

    // Look to see if it would have been recognised in
    // another scope.  That would mean the attr is
    // merely misplaced, rather than totally wrong.
    try_case((|| {
        let y_scopes = Scope::iter()
            .filter(|&scope| {
                recog.map.contains_key(&BorrowedDesig { scope, lpaths })
            })
            .collect_vec();

        if y_scopes.is_empty() {
            return None;
        }
        let y_ss = SuppliedScope::iter()
            .filter(|ss| ss.recog_search().any(|s| y_scopes.contains(&s)));
        if y_ss.clone().contains(&supplied_scope) {
            return None;
        }
        let y_ss = y_ss.format("/");
        let y_scopes = y_scopes.iter().map(|s| s.as_ref()).format("/");
        some_err(&format_args!(
 "meta attribute provided for {}, but recognised by template only for {} ({})",
            supplied_scope,
            y_ss,
            y_scopes,
        ))
    })())?;

    // Look to see if user supplied `#[deftly(foo(bar))]`
    // when the template wanted `#[deftly(foo = "bar")]`.
    try_case((|| {
        let (upper, recog) =
            supplied_scope.recog_search().find_map(|scope| {
                let upper = BorrowedDesig {
                    scope,
                    lpaths: lpaths.split_last()?.1,
                };
                let recog = recog.map.get(&upper)?;
                Some((upper, recog))
            })?;
        let _: IsUsed = recog.value?;
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
        recog.map.keys().find_map(|desig| {
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

//---------- impls of FindRecogMetas ----------

impl<O: SubstParseContext> FindRecogMetas for TemplateElement<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        match self {
            TE::Ident(..)
            | TE::LitStr(_)
            | TE::Literal(..)
            | TE::Punct(..) => {}
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
        if let Some((_, exactly_1)) = &self.exactly_1 {
            exactly_1.find_recog_metas(acc);
        }
    }
}

impl<O: SubstParseContext> FindRecogMetas for Subst<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        self.sd.find_recog_metas(acc);
    }
}

impl<O: SubstParseContext> FindRecogMetas for SubstDetails<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        use SD::*;

        match self {
            Xmeta(v) => v.find_recog_metas(acc),
            vpat(v, _, _) => v.find_recog_metas(acc),
            vtype(v, _, _) => v.find_recog_metas(acc),
            Impl(v, ..) => v.find_recog_metas(acc),
            tdefvariants(v, _, _) => v.find_recog_metas(acc),
            fdefine(v, _, _) => v.find_recog_metas(acc),
            vdefbody(a, b, _, _) => {
                a.find_recog_metas(acc);
                b.find_recog_metas(acc);
            }
            paste(v, _) => v.find_recog_metas(acc),
            paste_spanned(v, ..) => v.find_recog_metas(acc),
            ChangeCase(v, ..) => v.find_recog_metas(acc),
            concat(v, ..) => v.find_recog_metas(acc),
            error(v, ..) => v.find_recog_metas(acc),
            StringTemplate(v, ..) => v.find_recog_metas(acc),
            DocTemplate(v, ..) => v.value.find_recog_metas(acc),

            when(v, _) => v.find_recog_metas(acc), // probably redundant
            loop_exactly_1(v, ..) => v.find_recog_metas(acc),
            define(v, _) => v.find_recog_metas(acc),
            defcond(v, _) => v.find_recog_metas(acc),
            GensCollate(v) => v.inputs.find_recog_metas(acc),

            not(v, _) => v.find_recog_metas(acc),
            any(v, _) | all(v, _) => v.find_recog_metas(acc),
            is_empty(_, v) => v.find_recog_metas(acc),
            approx_equal(_, v) => v.find_recog_metas(acc),

            for_loop_count(flc, ..) => flc.find_recog_metas(acc),

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
            | v_explicit_discriminant(_)
            | vindex(..)
            | findex(..)
            | v_is_first(..)
            | f_is_first(..)
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
            | loop_index(..)
            | loop_initial(..)
            | loop_final(..)
            | require_beta(..)
            | dbg_all_keywords(_)
            | Crate(_, _) => {}
        }
    }
}

impl<O: SubstParseContext> FindRecogMetas for SubstMeta<O> {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let recog = O::meta_recog_usage(self);
        acc.update(self.desig.clone(), recog);
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

impl FindRecogMetas for ForLoopCount {
    fn find_recog_metas(&self, acc: &mut Recognised) {
        let Self { over: _, cond } = self;
        cond.find_recog_metas(acc);
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
            DefinitionBody::Concat(v) => v.find_recog_metas(acc),
            DefinitionBody::LitStr(_) => {}
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
