//! Core types and traits for parsing and expansion
//!
//! Also re-exports the names that the implementation wants.
//!
//! Should be included with `use super::framework::*`, not `crate::`,
//! so that it works with `tests/directly.rs` too.

pub use super::prelude::*;

pub use super::boolean::*;
pub use super::repeat::*;
pub use super::syntax::*;

pub(super) use super::paste;
pub(super) use super::paste::{IdentFrag, IdentFragInfallible};

pub(super) use general_context::{
    DriverlessContext, GeneralContext, GeneralContextBuf, MissingContextError,
};

/// Context during expansion
///
/// References the driver, and digested information about it.
/// Also represents where in the driver we are,
/// including repetition context.
//
// Has to be Copy because otherwise the borrow checker doesn't understand
// what's going on with ctx_buf in Template::expand.
#[derive(Debug, Clone, Copy)]
pub struct Context<'c> {
    pub top: &'c syn::DeriveInput,
    pub template_crate: &'c syn::Path,
    pub template_name: Option<&'c syn::Path>,
    pub pmetas: &'c meta::PreprocessedMetas,
    pub any_error_occurred: &'c Cell<Result<(), ()>>,
    pub meta_quoted: Option<meta::QuotedMode>,
    pub variant: Option<&'c WithinVariant<'c>>,
    pub field: Option<&'c WithinField<'c>>,
    pub within_loop: WithinLoop<'c>,
    pub pvariants: &'c [PreprocessedVariant<'c>],
    pub defs: DefinitionsContext<'c>,
}

/// Part of the context relating to user-defined expansions and conditions
#[derive(Debug, Clone, Copy, Default)]
pub struct DefinitionsContext<'c> {
    pub defs: Definitions<'c>,
    pub nesting_depth: u16,
    pub nesting_parent: Option<(
        &'c DefinitionsContext<'c>, //
        &'c DefinitionName,
    )>,
}

#[derive(Debug)]
pub struct PreprocessedVariant<'c> {
    pub fields: &'c syn::Fields,
    pub pmetas: &'c meta::PreprocessedMetas,
    pub pfields: Vec<PreprocessedField>,
}

#[derive(Debug)]
pub struct PreprocessedField {
    pub pmetas: meta::PreprocessedMetas,
}

#[derive(Debug, Clone)]
pub struct WithinVariant<'c> {
    pub variant: Option<&'c syn::Variant>,
    pub fields: &'c syn::Fields,
    pub pmetas: &'c meta::PreprocessedMetas,
    pub pfields: &'c [PreprocessedField],
    pub index: u32,
}

#[derive(Debug, Clone)]
pub struct WithinField<'c> {
    pub field: &'c syn::Field,
    pub pfield: &'c PreprocessedField,
    pub index: u32,
}

/// Whether we're in a loop, and if so, its details
///
/// Set only for expansions of a `RepeatedTemplate`,
/// not any kind of implicit looping eg `dbg_all_keywords`, `vpat`, etc.
///
/// At some future point this may have enough information
/// to provide `$loop_index`, etc.
/// Right now it's only used for `dbg_all_keywords`.
#[derive(Debug, Clone, Copy)]
pub enum WithinLoop<'c> {
    None,
    /// Evaluating `${when }` clauses
    When,
    /// Evaluating the body
    Body(&'c WithinLoopBody),
}

#[derive(Debug, Clone, Copy)]
pub struct WithinLoopBody {
    pub yes_i: u32,
    pub yes_count: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Definitions<'c> {
    pub here: &'c [&'c Definition<DefinitionBody>],
    pub conds: &'c [&'c Definition<DefCondBody>],
    pub earlier: Option<&'c Definitions<'c>>,
}

/// Special processing instructions returned by
/// [`before_element_hook`](SpecialParseContext::before_element_hook)
pub enum SpecialInstructions {
    /// This template is finished
    ///
    /// Stop parsing this `Template` though perhaps
    /// the surrounding `Group` is not finished.
    ///
    /// The parser for whatever called `Template::parse`
    /// will continue.
    EndOfTemplate,
}

//==================== SubstParseContext ====================

/// Surrounding lexical context during parsing
///
/// This is the kind of lexical context a piece of a template appears in.
/// It is implemented for
///  * Types that represent an expansion output `ExpansionOutput`;
///    in this case, the lexical context is one where
///    the expansion is accumulated in this type.
///  * Places where template substitution syntax `${keyword }`
///    appears but where no output will be generated (eg, within
///    the condition of `${if }`.
///
/// The associated types are either `Void` or `()`.
/// They appears within the variants of `SubstDetails`,
/// causing inapplicable variants to be eliminated.
///
/// Because a variant is only inhabited if all of its fields are,
/// the conditions are effectively ANDed.
/// So the "default" value (for context that don't have an opnion)
/// is inhabitedness `()`.
///
/// Each type has an associated constructur,
/// used during parsing.
/// So this generates a parse error at parse time,
/// if a construct appears in the wrong place.
pub trait SubstParseContext: Sized {
    /// Uninhabited iff this lexical context is within `${paste }`
    type NotInPaste: Debug + Copy + Sized;
    /// Uninhabited iff this lexical context is within `${concat }`
    type NotInConcat: Debug + Copy + Sized;
    /// Uninhabited iff this lexical context is within a condition.
    type NotInBool: Debug + Copy + Sized;
    /// Uninhabited unless this lexical context is within `${concat }`
    type ConcatOnly: Debug + Copy + Sized;
    /// Uninhabited unless this lexical context is within a condition.
    type BoolOnly: Debug + Copy + Sized;

    /// Uninhabited unless we are allowed to parse a (nested) template
    ///
    /// Used (only) to prevent recursive entry to the parser within `$"..."`.
    type AllowTemplate: Debug + Copy + Sized + string_template::LexicalContext;

    /// Whether this is a boolean context
    //
    // Useful for ad-hoc handling of the way that boolean
    // context has a different notion of syntax.
    const IS_BOOL: bool = false;

    /// Content of the `dbg` keyword
    ///
    /// This has to be in this trait because
    /// `${dbg }` contains a `Template` but `dbg(...)` contains a `Subst`.
    ///
    /// For all `ExpansionContext` impls, should be `Template<Self>`.
    // We'd use an associated type default, but they're unstable.
    ///
    /// We make bespoke output for each context; for boolean this is sui
    /// generis, and for expansions it's in [`ExpansionOutput::dbg_expand`].
    type DbgContent: Parse + Debug + AnalyseRepeat + meta::FindRecogMetas;

    fn not_in_paste(span: &impl Spanned) -> syn::Result<Self::NotInPaste>;
    fn not_in_concat(span: &impl Spanned) -> syn::Result<Self::NotInConcat>;
    fn not_in_bool(span: &impl Spanned) -> syn::Result<Self::NotInBool>;

    fn bool_only(span: &impl Spanned) -> syn::Result<Self::BoolOnly> {
        Err(span.error(
            "derive-deftly keyword is a condition - not valid as an expansion",
        ))
    }
    fn concat_only(span: &impl Spanned) -> syn::Result<Self::ConcatOnly> {
        Err(span
            .error("derive-deftly keyword is only allowed within ${concat }"))
    }

    /// When we find a `fmeta` etc. in this context, does it allow a value?
    ///
    /// Used by the template-scanning code, to report whether an `Xmeta`
    /// in the template justifies a value-bearing `Xmeta` attribute
    /// on/in the driver, or just a boolean.
    fn meta_recog_usage(m: &meta::SubstMeta<Self>) -> meta::Usage {
        if m.default.is_some() {
            meta::Usage::VALUE_ONLY
        } else {
            meta::Usage::VALUE
        }
    }

    /// Parse using `f`, within parens in boolean context, not otherwise
    ///
    /// Useful for parsing the arguments to an argument-taking keyword
    /// which takes an "equivalent" syntax in both contexts.
    fn parse_maybe_within_parens<T>(
        input: ParseStream,
        f: impl FnOnce(ParseStream) -> syn::Result<T>,
    ) -> syn::Result<T> {
        if Self::IS_BOOL {
            let inner;
            let _ = parenthesized!(inner in input);
            f(&inner)
        } else {
            f(input)
        }
    }

    /// Parse maybe a comma (comma in boolean contegxt, not otherwise)
    ///
    /// Useful for parsing the arguments to an argument-taking keyword
    /// which takes an "equivalent" syntax in both contexts.
    fn parse_maybe_comma(input: ParseStream) -> syn::Result<()> {
        if Self::IS_BOOL {
            let _: Token![,] = input.parse()?;
        }
        Ok(())
    }

    /// Return an error suitable for reporting missing arguments
    ///
    /// Helper for handling missing arguments to an argument-taking keyword
    /// which takes an "equivalent" syntax in both contexts.
    fn missing_keyword_arguments(kw_span: Span) -> syn::Result<Void> {
        Err(kw_span.error(format_args!(
 "missing parameters to expansion keyword (NB: argument must be within {{ }})",
        )))
    }

    /// Convenience method to return `AllowTokens`
    ///
    /// (Don't reimplement.)
    fn allow_tokens(span: &impl Spanned) -> syn::Result<AllowTokens<Self>> {
        let span = span.span();
        Ok((Self::not_in_paste(&span)?, (Self::not_in_concat(&span)?)))
    }
}

pub trait SpecialParseContext {
    /// Handle any special syntax for a special kind of template context.
    ///
    /// This method is called only when parsing multi-element [`Template`]s,
    /// It's a hook, called before parsing each `TemplateElement`.
    ///
    /// It should consume any special syntax as appropriate,
    ///
    /// The default implementation is a no-op.
    fn before_element_hook(
        &mut self,
        _input: ParseStream,
    ) -> syn::Result<Option<SpecialInstructions>> {
        Ok(None)
    }

    /// Handle any special syntax, after parsing an element.
    fn after_element_hook<O: SubstParseContext>(
        &mut self,
        _input: ParseStream,
        _elem: &mut TemplateElement<O>,
    ) -> syn::Result<()> {
        Ok(())
    }
}

impl SpecialParseContext for () {}

/// Inhabited iff this lexical content may expand to arbitrary tokens
///
/// Calculated automatically
#[allow(type_alias_bounds)]
pub type AllowTokens<O: SubstParseContext> = (
    <O as SubstParseContext>::NotInPaste,
    <O as SubstParseContext>::NotInConcat,
);

//==================== ExpansionOutput ====================

/// Expansion output accumulator, for a template lexical context
///
/// Each template lexical context has a distinct type which
///  * Represents the lexical context
///  * If that lexical context generates expansions,
///    accumulates the expansion.  That's what this trait is.
///
/// The methods are for accumulating various kinds of things
/// that can be found in templates, or result from template expansion.
///
/// The accumulating type (`Self` might be accumulating
/// tokens ([`TokenStream`]) or strings ([`paste::Items`]).
pub trait ExpansionOutput: SubstParseContext<BoolOnly = Void> {
    /// An identifier (or fragment of one)
    ///
    /// Uses the `IdentFragment` for identifier pasting,
    /// and the `ToTokens` for general expansion.
    fn append_identfrag_toks<I: IdentFrag>(
        &mut self,
        ident: &I,
    ) -> Result<(), I::BadIdent>;

    /// Append a Rust path (scoped identifier, perhaps with generics)
    ///
    /// To facilitate `${pawte }`, the path is provided as:
    ///  * some prefix tokens (e.g., a scoping path),
    ///  * the actual identifer,
    ///  * some suffix tokens (e.g. generics).
    ///
    /// `tspan` is the span of the part of the template
    /// which expanded into this path.
    ///
    /// This is a "more complex" expansion,
    /// in the terminology of the template reference:
    /// If a paste contains more than one, it is an error.
    fn append_idpath<A, B, I>(
        &mut self,
        template_entry_span: Span,
        pre: A,
        ident: &I,
        post: B,
        grouping: Grouping,
    ) -> Result<(), I::BadIdent>
    where
        A: FnOnce(&mut TokenAccumulator),
        B: FnOnce(&mut TokenAccumulator),
        I: IdentFrag;

    /// Append a [`syn::LitStr`](struct@syn::LitStr)
    ///
    /// This is its own method because `syn::LitStr` is not `Display`,
    /// and we don't want to unconditionally turn it into a string
    /// before retokenising it.
    ///
    /// `te_span` is the span of a literal in the template that
    /// determined the string (or was included in it).
    /// This is used by `$[concat }` to arrange that the span of its output
    /// is always the template, not anything that was substituted into it.
    /// This is particularly important for `${error }`.
    fn append_syn_litstr(&mut self, v: &syn::LitStr, te_span: Option<Span>);

    /// Append a [`syn::Type`]
    ///
    /// This is a "more complex" expansion,
    /// in the terminology of the template reference:
    /// If a paste contains more than one, it is an error.
    fn append_syn_type(
        &mut self,
        te_span: Span,
        mut v: syn::Type,
        mut grouping: Grouping,
    ) {
        loop {
            let (inner, add_grouping) = match v {
                syn::Type::Paren(inner) => (inner.elem, Grouping::Parens),
                syn::Type::Group(inner) => (inner.elem, Grouping::Invisible),
                _ => break,
            };
            v = *inner;
            grouping = cmp::max(grouping, add_grouping);
        }
        if let syn::Type::Path(tp) = &mut v {
            typepath_add_missing_argument_colons(tp, te_span);
        }
        self.append_syn_type_inner(te_span, v, grouping)
    }

    /// Append a [`syn::Type`], which has been grouping-normalised
    fn append_syn_type_inner(
        &mut self,
        te_span: Span,
        v: syn::Type,
        grouping: Grouping,
    );

    /// Append using a function which generates tokens
    ///
    /// If you have an `impl `[`ToTokens`],
    /// use [`append_tokens`](ExpansionOutput::append_tokens) instead.
    ///
    /// Not supported within `${paste }`.
    /// The `NotInPaste` parameter makes this method unreachable
    /// when expanding within `${paste }`;
    /// or to put it another way,
    /// it ensures that such an attempt would have been rejected
    /// during template parsing.
    fn append_tokens_with(
        &mut self,
        allow_tokens: &AllowTokens<Self>,
        f: impl FnOnce(&mut TokenAccumulator) -> syn::Result<()>,
    ) -> syn::Result<()>;

    /// "Append" a substitution which can only be used within a boolean
    ///
    /// Such a thing cannot be expanded, so it cannot be appended,
    /// so this function must be unreachable.
    /// `expand_bool_only` is called (in expansion contexts)
    /// to handle uninhabited `SubstDetails` variants etc.
    ///
    /// Implementing it involves demonstrating that
    /// either `self`, or `Self::BoolOnly`, is uninhabited,
    /// with a call to [`void::unreachable`].
    fn append_bool_only(&mut self, bool_only: &Self::BoolOnly) -> !;

    /// Note that an error occurred
    ///
    /// This must arrange to
    /// (eventually) convert it using `into_compile_error`
    /// and emit it somewhere appropriate.
    fn record_error(&mut self, err: syn::Error);

    /// Convenience method for noting an error with span and message
    fn write_error<S: Spanned, M: Display>(&mut self, span: &S, message: M) {
        self.record_error(span.error(message));
    }

    /// Convenience method for writing a `ToTokens`
    ///
    /// Dispatches to
    /// [`append_tokens_with`](ExpansionOutput::append_tokens_with)
    /// Not supported within `${paste }`.
    //
    // I experimented with unifying this with `append_tokens_with`
    // using a `ToTokensFallible` trait, but it broke type inference
    // rather badly and had other warts.
    fn append_tokens(
        &mut self,
        allow_tokens: &AllowTokens<Self>,
        tokens: impl ToTokens,
    ) -> syn::Result<()> {
        self.append_tokens_with(allow_tokens, |out| {
            out.append(tokens);
            Ok(())
        })
    }

    /// Make a new empty expansion output, introduced at `kw_span`
    ///
    /// Normally, call sites use an inherent constructor method.
    /// This one is used for special cases, eg `${ignore ...}`
    fn new_with_span(kw_span: Span) -> Self;

    fn default_subst_meta_as(kw: Span) -> syn::Result<meta::SubstAs<Self>> {
        Err(kw.error("missing `as ...` in meta expansion"))
    }

    /// Implement the core of the `ignore` keyword
    ///
    /// If there was an error, returns it.
    /// Otherwise, discards everything.
    fn ignore_impl(self) -> syn::Result<()>;

    /// Implement the `dbg` keyword
    ///
    /// Specifically:
    ///   * Write the expansion of `child` to `msg` in human-readable form
    ///   * Without a trailing newline
    ///   * Subsume it into `self`
    ///
    /// Expansion errors are to be reported, and subsumed into `self`.
    /// Failures to write may be thrown.
    fn dbg_expand<'c>(
        &mut self,
        kw_span: Span,
        ctx: GeneralContext<'c>,
        msg: &mut String,
        content: &Self::DbgContent,
    ) -> fmt::Result;
}

/// Convenience trait providing `item.expand()`
///
/// Implementations of this are often specific to the [`ExpansionOutput`].
///
/// Having this as a separate trait,
/// rather than hanging it off `ExpansionOutput`,
/// makes the expansion method more convenient to call.
///
/// It also avoids having to make all of these expansion methods
/// members of the `ExpansionOutput` trait.
pub trait Expand<O> {
    fn expand<'c>(
        &self,
        ctx: GeneralContext<'c>,
        out: &mut O,
    ) -> syn::Result<()>;
}
/// Convenience trait providing `fn expand(self)`, infallible version
///
/// Some of our `expand` functions always record errors
/// within the output accumulator
/// and therefore do not need to return them.
pub trait ExpandInfallible<O> {
    fn expand<'c>(&self, ctx: GeneralContext<'c>, out: &mut O);
}

//==================== TokenAccumulator ===================

/// Accumulates tokens, or errors
///
/// We collect all the errors, and if we get an error, don't write
/// anything out.
/// This is because `compile_error!` (from `into_compile_error`)
/// only works in certain places in Rust syntax (!)
#[derive(Debug)]
pub struct TokenAccumulator(Result<TokenStream, syn::Error>);

impl<'c> Context<'c> {
    pub fn is_enum(&self) -> bool {
        matches!(self.top.data, syn::Data::Enum(_))
    }

    /// Description of the whole expansion, suitable for `dbg` option, etc.
    pub fn expansion_description(&self) -> impl Display {
        let ident = &self.top.ident;
        if let Some(templ) = &self.template_name {
            format!(
                "derive-deftly expansion of {} for {}",
                templ.to_token_stream(),
                ident,
            )
        } else {
            format!("derive-deftly expansion, for {}", ident,)
        }
    }
}

impl Default for TokenAccumulator {
    fn default() -> Self {
        TokenAccumulator(Ok(TokenStream::new()))
    }
}

impl TokenAccumulator {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_tokens<R>(
        &mut self,
        f: impl FnOnce(&mut TokenStream) -> R,
    ) -> Option<R> {
        self.0.as_mut().ok().map(f)
    }
    pub fn append(&mut self, t: impl ToTokens) {
        self.with_tokens(|out| t.to_tokens(out));
    }
    pub fn tokens(self) -> syn::Result<TokenStream> {
        self.0
    }
    /// Appends `val`, via [`ToTokensPunctComposable`] or [`ToTokens`]
    pub fn append_maybe_punct_composable(
        &mut self,
        val: &(impl ToTokens + ToTokensPunctComposable),
        composable: WantPunctComposable,
    ) {
        use WantPunctComposable as WPC;

        self.with_tokens(|out| match composable {
            WPC::Composable => val.to_tokens_punct_composable(out),
            WPC::AvoidIntroducingTrailingComma => val.to_tokens(out),
        });
    }
}

impl SubstParseContext for TokenAccumulator {
    type NotInPaste = ();
    type NotInConcat = ();
    type NotInBool = ();
    type DbgContent = Template<Self>;
    type AllowTemplate = ();
    fn not_in_paste(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn not_in_concat(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }
    fn not_in_bool(_: &impl Spanned) -> syn::Result<()> {
        Ok(())
    }

    type BoolOnly = Void;
    type ConcatOnly = Void;
}

impl ExpansionOutput for TokenAccumulator {
    fn append_identfrag_toks<I: IdentFrag>(
        &mut self,
        ident: &I,
    ) -> Result<(), I::BadIdent> {
        self.with_tokens(
            |out| ident.frag_to_tokens(out), //
        )
        .unwrap_or(Ok(()))
    }
    fn append_idpath<A, B, I>(
        &mut self,
        _te_span: Span,
        pre: A,
        ident: &I,
        post: B,
        grouping: Grouping,
    ) -> Result<(), I::BadIdent>
    where
        A: FnOnce(&mut TokenAccumulator),
        B: FnOnce(&mut TokenAccumulator),
        I: IdentFrag,
    {
        let inner = match self.with_tokens(|_outer| {
            let mut inner = TokenAccumulator::new();
            pre(&mut inner);
            inner.append_identfrag_toks(ident)?;
            post(&mut inner);
            Ok(inner)
        }) {
            None => return Ok(()), // earlier errors, didn't process
            Some(Err(e)) => return Err(e),
            Some(Ok(ta)) => ta,
        };

        match inner.tokens() {
            Ok(ts) => self.append(grouping.surround(ident.span(), ts)),
            Err(e) => self.record_error(e),
        }
        Ok(())
    }
    fn append_syn_litstr(&mut self, lit: &syn::LitStr, _: Option<Span>) {
        self.append(lit);
    }
    fn append_syn_type_inner(
        &mut self,
        _te_span: Span,
        ty: syn::Type,
        grouping: Grouping,
    ) {
        self.append(grouping.surround(ty.span(), ty));
    }
    fn append_tokens_with(
        &mut self,
        (_not_in_paste, _not_in_concat): &((), ()),
        f: impl FnOnce(&mut TokenAccumulator) -> syn::Result<()>,
    ) -> syn::Result<()> {
        f(self)
    }

    fn append_bool_only(&mut self, bool_only: &Self::BoolOnly) -> ! {
        void::unreachable(*bool_only)
    }

    fn record_error(&mut self, err: syn::Error) {
        if let Err(before) = &mut self.0 {
            before.combine(err);
        } else {
            self.0 = Err(err)
        }
    }

    fn new_with_span(_kw_span: Span) -> Self {
        Self::new()
    }

    fn ignore_impl(self) -> syn::Result<()> {
        self.0.map(|_: TokenStream| ())
    }

    fn dbg_expand<'c>(
        &mut self,
        _kw_span: Span,
        ctx: GeneralContext<'c>,
        msg: &mut String,
        content: &Template<TokenAccumulator>,
    ) -> fmt::Result {
        let mut child = TokenAccumulator::new();
        content.expand(ctx, &mut child);
        let child = child.tokens();
        match &child {
            Err(e) => write!(msg, "/* ERROR: {} */", e)?,
            Ok(y) => write!(msg, "{}", y)?,
        }
        match child {
            Ok(y) => self.append(y),
            Err(e) => self.record_error(e),
        }
        Ok(())
    }
}

//==================== recording errors in the Context ====================

impl<'c> Context<'c> {
    /// Convert a [`syn::Error`] to `compile_error!` invocation
    ///
    /// Also notes that an error occurred, in the context.
    /// This is used for suppressing unused meta errors if expansion failed,
    /// even if the expansion error was turned into a
    /// `compile_error!` `TokenStream` deep in the expansion code,
    /// so that it's not ordinarily visible to the top-level plumbing
    /// code which needs to generate `_meta_used` etc. in the ACCUM.
    #[allow(dead_code)] // not used with some feature combinations
    pub fn error_into_compile_error(&'c self, e: syn::Error) -> TokenStream {
        self.any_error_occurred.set(Err(()));
        e.error_into_compile_error_unrecorded()
    }
}

pub trait SynErrorExt {
    /// Convert a `syn:Error` to `compile_error!` without noting anywhere
    ///
    /// Implemented for `syn:Error` and also, for convenience,
    /// on `syn::Result<TokenStream>`.
    ///
    /// # Correctness
    ///
    /// No error from template expansion should be passed to this function,
    /// unless we have made arrangements so that any ACCUM passed
    /// on to future expansions will have appropriate
    /// `_error` and `_meta_used *`.
    ///
    /// Typically this means this is appropriate to call only in top-level
    /// plumbing code that is already handling errors explicitly.
    fn error_into_compile_error_unrecorded(self) -> TokenStream;
}

impl SynErrorExt for syn::Error {
    #[allow(clippy::disallowed_methods)] // this is the wrapper
    fn error_into_compile_error_unrecorded(self) -> TokenStream {
        self.into_compile_error()
    }
}

impl SynErrorExt for syn::Result<TokenStream> {
    fn error_into_compile_error_unrecorded(self) -> TokenStream {
        self.unwrap_or_else(|e| e.error_into_compile_error_unrecorded())
    }
}
