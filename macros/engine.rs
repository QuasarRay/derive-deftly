//! `derive_deftly_engine!()`

use super::framework::*;
use adviseable::*;

/// Input to `derive_deftly_engine!`
#[derive(Debug)]
struct EngineInput {
    #[allow(dead_code)]
    driver_brace: token::Brace,
    driver: syn::DeriveInput,
    #[allow(dead_code)]
    template_brace: token::Brace,
    template_crate: syn::Path,
    template_name: Option<syn::Path>,
    options: DdOptions,
    template: Template<TokenAccumulator>,
}

impl ParseAdviseable for EngineInput {
    fn parse_adviseable(input: ParseStream) -> AdviseableResult<Self> {
            let mut options = DdOptions::default();

            let driver;
            let driver_brace = braced!(driver in input);
            let driver = driver.parse()?;

            if input.peek(syn::token::Bracket) {
                // AOPTIONS appears iff we're being invoked for a precanned
                // template, rather than an adhoc one; it's from the
                // `#[derive()` application.
                let tokens;
                let _ = bracketed!(tokens in input);
                parse_unadvised! {
                    tokens => || {
                        let oc = OpContext::DriverApplicationPassed;
                        options .parse_update(&tokens, oc)
                    }
                }
            }

            let driver_passed;
            let _ = braced!(driver_passed in input);
            let _: TokenStream = driver_passed.parse()?;

            let template;
            let template_brace = braced!(template in input);

            let template_crate;
            let template_name;
            {
                let template_passed;
                let _ = braced!(template_passed in input);
                let input = template_passed;

                template_crate = input.parse()?;
                let _: Token![;] = input.parse()?;

                let tokens;
                let _ = bracketed!(tokens in input);
                parse_unadvised! {
                    tokens => || {
                        options.parse_update(&tokens, OpContext::Template)
                    }
                }

                template_name = if input.peek(Token![;]) {
                    None
                } else {
                    Some(input.parse()?)
                };
                let _: Token![;] = input.parse()?;

                let _: TokenStream = input.parse()?;
            }

            let _: TokenStream = input.parse()?;

            let template = parse_unadvised! { template };

            Ok(AOk(EngineInput {
                driver_brace,
                driver,
                template_brace,
                template,
                template_crate,
                template_name,
                options,
            }))
    }
}

impl<'c> Context<'c> {
    /// Calls `f` with a top-level [`Context`] for a [`syn::DeriveInput`]
    ///
    /// `Context` has multiple levels of references to values created
    /// here, so we can't easily provide `Context::new()`.
    pub fn call<T>(
        driver: &syn::DeriveInput,
        template_crate: &syn::Path,
        template_name: Option<&syn::Path>,
        f: impl FnOnce(Context) -> syn::Result<T>,
    ) -> Result<T, syn::Error> {
        let tmetas = preprocess_attrs(&driver.attrs)?;

        let pvariants_one = |fields| {
            let pmetas = tmetas.clone(); // TODO Cow maybe?
            let pfields = preprocess_fields(fields)?;
            let pvariant = PreprocessedVariant {
                fields,
                pmetas,
                pfields,
            };
            syn::Result::Ok((Some(()), vec![pvariant]))
        };

        let union_fields;

        let (variant, pvariants) = match &driver.data {
            syn::Data::Struct(ds) => pvariants_one(&ds.fields)?,
            syn::Data::Union(du) => {
                union_fields = syn::Fields::Named(du.fields.clone());
                pvariants_one(&union_fields)?
            }
            syn::Data::Enum(de) => (
                None,
                de.variants
                    .iter()
                    .map(|variant| {
                        let fields = &variant.fields;
                        let pmetas = preprocess_attrs(&variant.attrs)?;
                        let pfields = preprocess_fields(&variant.fields)?;
                        Ok(PreprocessedVariant {
                            fields,
                            pmetas,
                            pfields,
                        })
                    })
                    .collect::<Result<Vec<_>, syn::Error>>()?,
            ),
        };

        // `variant` is None in enums; otherwise it's Some(())
        // and here we convert it to the real WithinVariant for the fields.
        let variant = variant.map(|()| WithinVariant {
            variant: None, // not actually a variant
            fields: pvariants[0].fields,
            pmetas: &pvariants[0].pmetas,
            pfields: &pvariants[0].pfields,
        });

        let ctx = Context {
            top: &driver,
            template_crate,
            template_name,
            tmetas: &tmetas,
            field: None,
            variant: variant.as_ref(),
            pvariants: &pvariants,
            definitions: Default::default(),
            nesting_depth: 0,
            nesting_parent: None,
        };

        f(ctx)
    }
}

/// `derive_deftly_engine!` -- implements the actual template engine
///
/// In my design, the input contains, firstly, literally the definition
/// that #[derive(Deftly)] was applied to (see NOTES.txt).
/// Using the literal input, rather than some pre-parsed version, is
/// slower, but means that we aren't inventing a nontrivial data format which
/// potentially crosses crate boundaries with semver implications.
pub fn derive_deftly_engine_func_macro(
    input: TokenStream,
) -> syn::Result<TokenStream> {
    dprint_block!(&input, "derive_deftly_engine! input");
    let input: EngineInput = adviseable_parse2(input)?;
    dprintln!("derive_deftly_engine! crate = {:?}", &input.template_crate);

    let DdOptions {
        dbg,
        driver_kind,
        expect_target,
        //
    } = input.options;

    if let Some(exp) = driver_kind {
        macro_rules! got_kind { { $($kind:ident)* } => {
            match &input.driver.data {
                $(
                    syn::Data::$kind(..) => ExpectedDriverKind::$kind,
                )*
            }
        } }

        let got_kind = got_kind!(Struct Enum Union);
        if got_kind != exp.value {
            return Err([
                (exp.span, "expected kind"),
                (input.driver.span(), "actual kind"),
            ]
            .error(format_args!(
                "template defined for {}, but applied to {}",
                exp.value, got_kind,
            )));
        }
    }

    Context::call(
        &input.driver,
        &input.template_crate,
        input.template_name.as_ref(),
        |ctx| {
            let mut output = TokenAccumulator::new();
            input.template.expand(&ctx, &mut output);
            let output = output.tokens()?;

            //    dbg!(&&output);
            if dbg {
                let description = ctx.expansion_description();
                let dump = format!(
                    concat!(
                        "---------- {} (start) ----------\n",
                        "{}\n",
                        "---------- {} (end) ----------\n",
                    ),
                    &description, &output, &description,
                );
                eprint!("{}", dump);
            }

            let mut output = output;
            if let Some(target) = expect_target {
                check::check_expected_target_syntax(&ctx, &mut output, target);
            }

            Ok(output)
        },
    )
}
