//! private prelude for proc macro stuff
//!
//! Should be included with `use super::prelude::*`, not `crate::`,
//! so that it works with `tests/directly.rs` too.

pub use std::cell::Cell;
pub use std::cmp;
pub use std::collections::{BTreeSet, HashSet, VecDeque};
pub use std::convert::{TryFrom, TryInto};
pub use std::default::Default;
pub use std::fmt::{self, Debug, Display, Write as _};
pub use std::iter;
pub use std::marker::PhantomData;
pub use std::mem;
pub use std::ops::Deref;
pub use std::ptr;
pub use std::str::FromStr;
pub use std::thread::panicking;

pub use itertools::{chain, izip, Either, Either::*, EitherOrBoth, Itertools};
pub use proc_macro2::extra::DelimSpan;
pub use proc_macro2::{Delimiter, Ident, Punct};
pub use proc_macro2::{Span, TokenStream, TokenTree};
pub use quote::{format_ident, quote, quote_spanned, ToTokens};
pub use strum::IntoEnumIterator as _;
pub use strum::{AsRefStr, Display};
pub use strum::{EnumIter, EnumString};
pub use syn::ext::IdentExt;
pub use syn::parse::{Lookahead1, Parse, ParseBuffer, ParseStream, Parser};
pub use syn::punctuated::Punctuated;
pub use syn::token;
pub use syn::Token;
pub use syn::{braced, bracketed, parenthesized};
pub use void::{ResultVoidErrExt as _, ResultVoidExt as _, Void};

pub use TokenTree as TT;

pub use super::approx_equal::{tokens_cmpeq, Equality};
pub use super::define::escape_dollars;
pub use super::paste::TokenPastesAsIdent;
pub use super::utils::AngleBrackets;

pub use super::utils::{
    braced_group, delimit_token_group, dummy_path, engine_macro_name,
    group_clone_set_stream, group_new_with_span, respan_hygiene, spans_join,
    typepath_add_missing_argument_colons, DocAttributes, ErrorAccumulator,
    ErrorGenerator, ErrorLoc, Grouping, IdentAny, MacroExport, MakeError,
    ModuleName, ModulePath, Spanned, TemplateName, TemplatePath,
    ToTokensPunctComposable, TokenOutputTrimmer, WantPunctComposable,
};

pub(super) use super::{
    accum, adviseable, beta, check, concat, dbg_allkw, define, engine,
    framework, general_context, generics, meta, modules, repeat,
    string_template,
};

#[allow(unused_imports)] // used in tests
pub use super::utils::{Concatenated, Discard};

pub use generics::SubstImpl;
pub use meta::CheckUsed as mCU;

pub use framework::SynErrorExt as _;
pub use meta::AttributeExt12 as _;
pub use repeat::AnalyseRepeat as _;
pub use string_template::LexicalContext as _;

// this is used by the dprintln! macro, which can't just use crate
// but which doesn't use the import if it's not enabled
#[allow(unused_imports)]
pub use super::utils::dprint;

#[allow(unused)] // TODO syn 3 ?
pub use super::compat_syn_common::*;

pub use super::options::*;
pub use super::visitor::*;
