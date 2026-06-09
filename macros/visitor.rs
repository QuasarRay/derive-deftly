//! "visitors": `meta::FindRecogMetas`, `AnalyseRepeat` etc.
//!
//! Implementation flow:
//!
//!  * Manual implementations of [`UniformlyAnalysable`],
//!     - gives blanket impls of
//!       [`meta::FindRecogMetas`] [`AnalyseRepeat`]
//!     - uses [`Analysable::analyse`]
//!
//!  * Manual, or blanket impls, of `FindRecogMetas`/`AnalyseRepeat`
//!     - called by analysis-specific code (eg meta code)
//!     - can be called by manual impls of same trait(s)
//!
//!  * Blanket impl of `Analysable`
//!     - called by manual impls of [`UniformlyAnalysable`]

use super::framework::*;

/// For "ordinary" types that don't need to do anything special:
pub trait UniformlyAnalysable {
    /// Manually implement this
    ///
    /// Make calls to [`Analysable::analyse`]
    /// for each piece of the contents.
    fn impl_analyse<V: Visitor>(&self, v: &mut V) -> Result<(), V::E>;
}

pub trait Visitor {
    type E;
    fn visit1<T: Analysable>(t: &T, v: &mut Self) -> Result<(), Self::E>;
}

impl<T: UniformlyAnalysable + ?Sized> meta::FindRecogMetas for T {
    fn find_recog_metas(&self, acc: &mut meta::Recognised) {
        self.impl_analyse(acc).void_unwrap();
    }
}

impl<T: UniformlyAnalysable + ?Sized> AnalyseRepeat for T {
    fn analyse_repeat(
        &self,
        v: &mut RepeatAnalysisVisitor,
    ) -> syn::Result<()> {
        self.impl_analyse(v)
    }
}

impl Visitor for RepeatAnalysisVisitor {
    type E = syn::Error;
    fn visit1<T: Analysable>(t: &T, v: &mut Self) -> Result<(), Self::E> {
        t.analyse_repeat(v)
    }
}

impl Visitor for meta::Recognised {
    type E = Void;
    fn visit1<T: Analysable>(t: &T, v: &mut Self) -> Result<(), Self::E> {
        Ok(t.find_recog_metas(v))
    }
}

pub trait Analysable: AnalyseRepeat + meta::FindRecogMetas + Sized {
    fn analyse<V: Visitor>(&self, v: &mut V) -> Result<(), V::E> {
        V::visit1(self, v)
    }
}

impl<T: AnalyseRepeat + meta::FindRecogMetas + Sized> Analysable for T {}

//---------- macrology ----------

macro_rules! impL_analysable_via_iter { {
    ( $($gens:tt)* ), $i:ty
} => {
    impl<T: Analysable, $($gens)*> UniformlyAnalysable for $i {
        fn impl_analyse<V: Visitor>(&self, v: &mut V) -> Result<(), V::E> {
            #[allow(for_loops_over_fallibles)]
            for item in self {
                item.analyse(v)?;
            }
            Ok(())
        }
    }
} }

impL_analysable_via_iter!((), [T]);
impL_analysable_via_iter!((), Option<T>);
impL_analysable_via_iter!((U), Punctuated<T, U>);

macro_rules! impL_analysable_via_deref { {
    ( $($gens:tt)* ), $i:ty
} => {
    impl<$($gens)*> UniformlyAnalysable for $i {
        fn impl_analyse<V: Visitor>(&self, v: &mut V) -> Result<(), V::E> {
            (**self).analyse(v)
        }
    }
} }

impL_analysable_via_deref!((O: SubstParseContext), Argument<O>);
impL_analysable_via_deref!((T: Analysable), Box<T>);

//---------- impls of UniformlyAnalysable ----------

impl UniformlyAnalysable for ExplicitError {
    fn impl_analyse<V: Visitor>(&self, v: &mut V) -> Result<(), V::E> {
        let ExplicitError { message, span } = self;

        message.analyse(v)?;
        if let Some((span, _)) = span {
            span.analyse(v)?;
        }
        Ok(())
    }
}
