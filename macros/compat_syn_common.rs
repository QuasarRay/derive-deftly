//! Definitions for compatibility with syn, common to 1 and 2

#[allow(unused)] // TODO syn 3 ?
use super::prelude::*;

pub trait SynPunctuatedExt {
    type R;
    /// Works around `pop` giving `Pair` in syn 1, but `T` in syn 2
    fn pop_pair_(&mut self) -> Self::R;
}

impl<T: Clone, P: Clone> SynPunctuatedExt for Punctuated<T, P> {
    type R = Option<syn::punctuated::Pair<T, P>>;

    fn pop_pair_(&mut self) -> Self::R {
        // The only method in syn 1 that doesn't clone T is pop.
        //
        // So, call `pop` giving us either `Pair` or `T`.
        // Then we use ad-hoc dispatch specialisation,
        // converting either T or Pair into T.
        // We still need to clone punct but we expect that to be very cheap.
        //
        // We could use build.rs to probe our syn version and use cfg.
        // But that seems overkill just for this.
        //
        use syn::punctuated::Pair;

        // Has `.into_value() -> T`, for `Q = T` and `Q = Pair`
        struct Dispatch<Q>(Q);

        // syn 2, pop returning Pair
        //
        // inherent method, is preferred when available, but only
        // implemented for the syn 2 return type.
        #[allow(dead_code)] // not used with syn 3
        impl<T, P> Dispatch<Pair<T, P>> {
            fn into_value(self) -> T {
                self.0.into_value()
            }
        }

        // syn 3, pop returning T
        //
        // always implemented since we need to cope with any T,
        // but the Pair impl above takes precedence
        #[allow(dead_code)] // not used with syn 2
        trait Value {
            type T;
            fn into_value(self) -> Self::T;
        }
        impl<T> Value for Dispatch<T> {
            type T = T;
            fn into_value(self) -> T {
                self.0
            }
        }

        let punct: Option<P> =
            self.pairs().rev().next()?.punct().cloned().cloned();
        let value_or_pair = self.pop().expect("just inspected");
        let value: T = Dispatch(value_or_pair).into_value();
        Some(Pair::new(value, punct))
    }
}
