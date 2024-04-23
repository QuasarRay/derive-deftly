//! `derive_deftly_engine!()`, parsing accumulations

use super::framework::*;
use adviseable::*;

/// `derive_deftly_engine! accumulated form, accumulated information
///
/// We don't reify the whole input;
/// instead, we accumulate directly in the `Parse` impl.
#[derive(Debug)]
pub struct EngineAccumInput {
    accum: Accumulated,
    driver: syn::DeriveInput,
}

#[derive(Debug)]
struct Accumulated {
    /// `Err` if we encountered an `error` part
    recog_metas: Option<meta::Recognised>,
}

impl ParseAdviseable for EngineAccumInput {
    fn parse_adviseable(input: ParseStream) -> AdviseableResult<Self> {
        let _empty_next_brackets_contents;
        let _ = bracketed!(_empty_next_brackets_contents in input);

        let accum;
        let _ = bracketed!(accum in input);
        let accum = accum.parse()?;

        let driver;
        let _ = braced!(driver in input);
        let driver = driver.parse()?;

        let _: TokenStream = input.parse()?;

        Ok(AOk(EngineAccumInput { accum, driver }))
    }
}

impl Parse for Accumulated {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut recog_metas = Some(meta::Recognised::default());

        struct Ignore;

        while !input.is_empty() {
            let kind: syn::Ident = input.parse()?;
            match if kind == "_meta" {
                if let Some(rm) = &mut recog_metas {
                    let content;
                    let _brackets = bracketed!(content in input);
                    let input = content;
                    while !input.is_empty() {
                        use meta::RecogAllow as RA;
                        let allow = match input.parse()? {
                            Some::<Token![?]>(_) => RA::BoolOnly,
                            None => RA::Value,
                        };
                        let desig = input.parse()?;
                        rm.update(desig, allow);
                    }
                    continue;
                } else {
                    Ignore
                }
            } else if kind == "error" {
                recog_metas = None;
                Ignore
            } else if kind.to_string().starts_with('_') {
                Ignore
            } else {
                return Err(
                    kind.error("unrecognised mandatory accumulation kind")
                );
            } {
                Ignore => {
                    let _: TokenTree = input.parse()?;
                }
            }
        }
        Ok(Accumulated { recog_metas })
    }
}

fn check_meta_recog(
    driver: &syn::DeriveInput,
    recog: meta::Recognised,
    errors: &mut ErrorAccumulator,
) -> syn::Result<()> {
    use meta::*;
    use SuppliedScope as SS;

    let c = RefCell::new(RecogChecker::new(&recog, errors));

    let chk_attrs = |attrs: &_, levels| {
        c.borrow_mut().check(&preprocess_attrs(attrs)?, levels);
        Ok::<_, syn::Error>(())
    };

    let chk_punct = |p: &Punctuated<syn::Field, _>| {
        for f in p {
            chk_attrs(&f.attrs, SS::Field)?;
        }
        Ok::<_, syn::Error>(())
    };

    let chk_fields = |f: &_| match f {
        syn::Fields::Named(n) => chk_punct(&n.named),
        syn::Fields::Unnamed(n) => chk_punct(&n.unnamed),
        syn::Fields::Unit => Ok(()),
    };

    match &driver.data {
        syn::Data::Struct(ds) => {
            chk_attrs(&driver.attrs, SS::Struct)?;
            chk_fields(&ds.fields)?;
        }
        syn::Data::Union(du) => {
            chk_attrs(&driver.attrs, SS::Struct)?;
            chk_punct(&du.fields.named)?;
        }
        syn::Data::Enum(de) => {
            chk_attrs(&driver.attrs, SS::Enum)?;
            for v in &de.variants {
                chk_attrs(&v.attrs, SS::Variant)?;
                chk_fields(&v.fields)?;
            }
        }
    }

    Ok(())
}

impl engine::EngineInput for EngineAccumInput {
    fn process(self: Box<Self>) -> syn::Result<TokenStream> {
        let mut errors = ErrorAccumulator::default();
        let mut out = TokenStream::new();

        match self.accum.recog_metas {
            Some(recog) => {
                let r = check_meta_recog(&self.driver, recog, &mut errors);
                errors.handle(r);
            }
            None => {}
        }

        match errors.finish() {
            Ok(()) => {}
            Err(e) => e.into_compile_error().to_tokens(&mut out),
        }

        Ok(out)
    }
}
