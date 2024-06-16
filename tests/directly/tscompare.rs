//! Comparison of ``TokenStream`s for test cases

use super::*;

#[derive(Debug)]
pub struct DissimilarTokenStreams {
    exp: TokenStream,
    got: TokenStream,
    same: TokenStream,
    diff: itertools::EitherOrBoth<TokenTree, TokenTree>,
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedOutputNoneGrouping {
    Explicit,
    Flatten,
}

/// When displaying, call `eprintln_extended_info` too
pub struct Mismatch {
    pub kind: &'static str,
    pub got: String,
    pub context_desc: String,
    pub info: Option<DissimilarTokenStreams>,
}

/// Parsed expected output from a test case
#[must_use]
pub struct ExpectedOutput {
    /// `Err` means we expected an error containing that string
    pub exp: Result<TokenStream, String>,
    pub grouping: ExpectedOutputNoneGrouping,
}

impl Display for ExpectedOutput {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.exp {
            Ok(y) => Display::fmt(y, f),
            Err(e) => write!(f, "error: {}", e),
        }
    }
}

impl DissimilarTokenStreams {
    pub fn eprintln(&self, in_title: impl Display) {
        use EitherOrBoth as EOB;
        let in_title = in_title.to_string();
        eprintln!(
            "----- difference report {}{}-----",
            in_title,
            if in_title.is_empty() { "" } else { " " }
        );
        eprintln!(" expected:        {}", &self.exp);
        eprintln!(" actual:          {}", &self.got);
        eprintln!(" similar prefix:  {}", &self.same);

        let side = |s, getter: fn(_) -> Option<TokenTree>| {
            eprintln!(
                " {:16} {}",
                format!("{} token:", s),
                match getter(self.diff.clone()) {
                    None => format!("(none)"),
                    Some(TokenTree::Group(g)) => {
                        format!(
                            "{}",
                            proc_macro2::Group::new(
                                g.delimiter(),
                                quote!("..."),
                            )
                        )
                    }
                    Some(other) => format!("{}", other),
                }
            )
        };
        side("expected", EOB::left);
        side("actual", EOB::right);
    }
}

/// Convenience version of `syn::parse_str` with more convenient error
pub fn parse_str<T>(s: &str, what: &str) -> Result<T, String>
where T: syn::parse::Parse
{
    syn::parse_str(s).map_err(|e| {
        format!(r#"failed to parse {}: {:?}: {}"#, what, s, e)
    })
}

impl ExpectedOutput {
    pub fn parse(
        stated: Result<&str, &str>,
        grouping: ExpectedOutputNoneGrouping,
    ) -> Result<ExpectedOutput, String> {
        let exp = match stated {
            Ok(exp) => Ok({
                if let Some(inner) = (|| {
                    let exp = exp.strip_prefix("«")?;
                    let exp = exp.strip_suffix("»")?;
                    Some(exp.trim())
                })() {
                    assert!(matches!(
                        grouping,
                        ExpectedOutputNoneGrouping::Explicit,
                    ));
                    let inner = parse_str(inner, "output (inner group)")?;
                    proc_macro2::Group::new(Delimiter::None, inner)
                        .to_token_stream()
                } else {
                    parse_str(exp, "output")?
                }
            }),
            Err(msg) => Err(msg.to_string()),
        };
        Ok(ExpectedOutput { exp, grouping })
    }

    pub fn compare(
        &self,
        got: syn::Result<TokenStream>,
        context_desc: &str,
    ) -> Result<(), Mismatch> {
        let mk_mismatch = |kind, info, got: &dyn Display| Mismatch {
            kind,
            info,
            got: got.to_string(),
            context_desc: context_desc.to_owned(),
        };

        let handle_syn_error = |e: syn::Error| {
            mk_mismatch("syn::Error", None, &format_args!("error: {}", e))
        };

        match &self.exp {
            Ok(exp) => {
                let got = got.map_err(handle_syn_error)?;
                let got = match self.grouping {
                    ExpectedOutputNoneGrouping::Explicit => got,
                    ExpectedOutputNoneGrouping::Flatten => {
                        flatten_none_groups(got)
                    }
                };
                check_expected_actual_similar_tokens(
                    &exp, &got, //
                )
                    .map_err(|info| {
                        mk_mismatch("mismatch", Some(info), &got)
                    })?;
                //println!("  MATCHED {}", &context_desc);
            }
            Err(exp) => {
                let got = match got {
                    Err(n) => Ok(n),
                    Ok(y) => Err(y),
                };
                let got = got
                    .map_err(|got| {
                        mk_mismatch("unexpected success", None, &got)
                    })?
                    .to_string();
                if !got.contains(exp) {
                    return Err(mk_mismatch("wrong error", None, &got));
                }
            }
        }
        Ok(())
    }
}

impl Mismatch {
    pub fn eprintln_initial(&self) {
        eprintln!(
            "{}: {} [{}]",
            self.kind, self.got, self.context_desc
        )
    }

    pub fn eprintln_extended(&self) {
        if let Some(info) = &self.info {
            info.eprintln(format!("[{}]", self.context_desc));
        }
    }
}

/// Tries to compare but disregarding spacing, which is unpredictable
///
/// What we really want to know is
/// whether the two `TokenStream`s mean the same.
/// This is not so straightforward.
/// Neither `TokenStream` nor `TokenTree` are `PartialEq`,
/// The string representation of a `TokenStream` has unpredictable spacing:
/// it can even inherit *some but not all* of the input spacing,
/// and, empirically, it can depend on whether the tokens went through `syn`
/// (and presumably which `syn` type(s)).
/// For example, output from `derive-deftly`
/// that came via an expansion that used `syn::Type`
/// can have different spacing to
/// a string with the same meaning, converted to `TokenStream` and back.
///
/// The algorithm in this function isn't perfect but I think it will do.
pub fn check_expected_actual_similar_tokens(
    exp: &TokenStream,
    got: &TokenStream,
) -> Result<(), DissimilarTokenStreams> {
    use EitherOrBoth as EOB;

    /// Having `recurse` return this ensures that on error,
    /// we inserted precisely one placeholder message.
    struct ErrorPlaceholderInserted(EitherOrBoth<TokenTree, TokenTree>);

    fn recurse(
        a: &TokenStream,
        b: &TokenStream,
        same_out: &mut TokenStream,
    ) -> Result<(), ErrorPlaceholderInserted> {
        let mut input =
            a.clone().into_iter().zip_longest(b.clone().into_iter());
        loop {
            if input
                .clone()
                .filter_map(|eob| eob.left())
                .collect::<TokenStream>()
                .to_string()
                == "..."
            {
                // disregard rest of this group
                return Ok(());
            }
            let eob = match input.next() {
                Some(y) => y,
                None => break,
            };
            let mut mk_err = |tokens: TokenStream| {
                tokens.to_tokens(same_out);
                Err(ErrorPlaceholderInserted(eob.clone()))
            };
            let (a, b) = match &eob {
                EOB::Both(a, b) => (a, b),
                EOB::Left(_a) => {
                    return mk_err(quote!(MISSING_ACTUAL_TOKEN_HERE));
                }
                EOB::Right(_b) => {
                    return mk_err(quote!(UNEXPECTED_ACTUAL_TOKEN_HERE));
                }
            };
            if !match (a, b) {
                (TT::Group(a), TT::Group(b)) => {
                    if a.delimiter() != b.delimiter() {
                        return mk_err(quote!(
                            FOUND_DIFFERENT_DELIMITERS_HERE
                        ));
                    }
                    let mut sub = TokenStream::new();
                    let r = recurse(&a.stream(), &b.stream(), &mut sub);
                    proc_macro2::Group::new(a.delimiter(), sub)
                        .to_tokens(same_out);
                    let () = r?;
                    continue;
                }
                (TT::Group(_), _) => return mk_err(quote!(LH_GROUPED_HERE)),
                (_, TT::Group(_)) => return mk_err(quote!(RH_GROUPED_HERE)),
                (a, b) => a.to_string() == b.to_string(),
            } {
                return mk_err(quote!(FOUND_DIFFERENCE_HERE));
            }
            a.to_tokens(same_out);
        }
        Ok(())
    }

    let mut same = TokenStream::new();
    recurse(exp, got, &mut same).map_err(|ErrorPlaceholderInserted(diff)| {
        DissimilarTokenStreams {
            same,
            diff,
            exp: exp.clone(),
            got: got.clone(),
        }
    })
}
