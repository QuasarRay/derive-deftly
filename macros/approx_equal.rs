//! Implementation of `${approx_equal ..}`, and support functions

use super::prelude::*;

use Equality::*;

/// Return value of a (perhaps approximate) equality comparison
///
/// (Avoids use of `bool`)
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Equality {
    Equal,
    Different,
}

impl Equality {
    /// Compare `a` and `b`
    ///
    /// (Name is short but avoids clash with `Ord::cmp`)
    pub fn cmpeq<T: Eq>(a: &T, b: &T) -> Self {
        if a == b {
            Equal
        } else {
            Different
        }
    }
}

/// Compare, and return early if different
///
///  * **`cmpeq!(d: Equality)`**:
///    If `d` is `Different`, returns `Ok(d)`.
///    (The containing scope should return `Result<Equality,>`.)
///
///  * **`cmpeq!<T: Eq>(a: T, b: T);`**:
///    compares `a` and `b` using `Equality::cmpeq`,
///    and returns immediately if `a != b`,
///    or the comparison failed.
macro_rules! cmpeq {
    { $a:expr, $b:expr } => {
        cmpeq!(Equality::cmpeq(&$a, &$b));
    };
    { $r:expr } => {
        if let d @ Different = $r {
            return Ok(d);
        }
    };
}

/// Return the input, but with `None`-delimited `Group`s flattened away
///
/// Loses some span information.
pub fn flatten_none_groups(ts: TokenStream) -> TokenStream {
    fn recurse(out: &mut TokenStream, input: TokenStream) {
        for tt in input {
            match tt {
                TT::Group(g) if g.delimiter() == Delimiter::None => {
                    recurse(out, g.stream());
                }
                TT::Group(g) => {
                    let span = g.span();
                    let mut g = proc_macro2::Group::new(
                        g.delimiter(),
                        flatten_none_groups(g.stream()),
                    );
                    // We lose some span information here.
                    g.set_span(span);
                    out.extend([TT::Group(g)]);
                }
                _ => out.extend([tt]),
            }
        }
    }

    let mut out = TokenStream::new();
    recurse(&mut out, ts);
    out
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
///
/// ### `None`-delimited `Group`s
///
/// We flatten these
///
/// This is necessary, because otherwise
/// apparently-identical pieces of code count as different.
///
/// This does mean that two things which are `approx_equal`
/// can be expressions with different values!
///
/// But, the Rust grammar for types doesn't permit ambiguity,
/// so the type equality guarantee of `approx_equal` is preserved.
//
// Comparing for equality has to be done by steam.
// And a lot of stringification.
pub fn tokens_cmpeq(a: TokenStream, b: TokenStream) -> syn::Result<Equality> {
    use proc_macro2::Group;

    fn tt_cmpeq(a: TokenTree, b: TokenTree) -> syn::Result<Equality> {
        let discrim = |tt: &_| match tt {
            TT::Punct(_) => 0,
            TT::Literal(_) => 1,
            TT::Ident(_) => 2,
            TT::Group(_) => 3,
        };

        cmpeq!(discrim(&a), discrim(&b));
        match (a, b) {
            (TT::Group(a), TT::Group(b)) => group_cmpeq(a, b),
            (a, b) => Ok(Equality::cmpeq(&a.to_string(), &b.to_string())),
        }
    }

    fn group_cmpeq(a: Group, b: Group) -> syn::Result<Equality> {
        let delim = |g: &Group| {
            proc_macro2::Group::new(g.delimiter(), TokenStream::new())
                .to_string()
        };
        cmpeq!(delim(&a), delim(&b));
        tokens_cmpeq(a.stream(), b.stream())
    }

    let a = flatten_none_groups(a);
    let b = flatten_none_groups(b);

    for ab in a.into_iter().zip_longest(b) {
        let (a, b) = match ab {
            EitherOrBoth::Both(a, b) => (a, b),
            EitherOrBoth::Left(_) => return Ok(Different),
            EitherOrBoth::Right(_) => return Ok(Different),
        };
        match tt_cmpeq(a, b)? {
            Equal => {}
            neq => return Ok(neq),
        }
    }
    return Ok(Equal);
}
