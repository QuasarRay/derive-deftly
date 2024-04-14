//! `derive_deftly_engine!()`

use super::framework::*;
use adviseable::*;

/// Input to `derive_deftly_engine!`, principal form (template expansion)
///
/// See `implementation.md`,
/// especially
/// "Overall input syntax for `derive_deftly_engine!` and templates".
#[derive(Debug)]
struct EngineExpandInput {
    driver: syn::DeriveInput,
    options: DdOptions,
    template: Template<TokenAccumulator>,
    template_crate: syn::Path,
    template_name: Option<syn::Path>,
}

/// Input to an invocation of `derive_deftly_engine!`
///
/// The trait impl includes knowledge of how to
/// parse it (via of the [`ParseAdviseable`] supertrait bound)
/// and process it.
///
/// Having this as a trait avoids having to have an enum,
/// with a lot of repeated code.
pub trait EngineInput: ParseAdviseable + Debug + 'static {
    /// Process this input
    ///
    /// Commented-out debugging printing of the *input* has been done.
    ///
    /// Debugging printing of the *output* is to be done by this  function,
    /// since for `EngineExpandInput` it is the `dbg` d-d option,
    /// but for others is commented out raw eprintln.
    fn process(self: Box<Self>) -> syn::Result<TokenStream>;
}

impl ParseAdviseable for EngineExpandInput {
    fn parse_adviseable(input: ParseStream) -> AdviseableResult<Self> {
        let mut options = DdOptions::default();

        let driver;
        let _ = braced!(driver in input);
        let driver = driver.parse()?;

        let opcontext_template;
        if input.peek(syn::token::Bracket) {
            // AOPTIONS appears iff we're being invoked for a precanned
            // template, rather than an adhoc one; it's from the
            // `#[derive()` application.
            opcontext_template = OpContext::TemplateDefinition;
            let tokens;
            let _ = bracketed!(tokens in input);
            parse_unadvised! {
                tokens => || {
                    let oc = OpContext::DriverApplicationPassed;
                    options .parse_update(&tokens, oc)
                }
            }
        } else {
            opcontext_template = OpContext::TemplateAdhoc;
        }

        let through_template;
        let _ = braced!(through_template in input);
        let _: TokenStream = through_template.parse()?;

        let template;
        let _ = braced!(template in input);

        let template_crate;
        let template_name;
        {
            let through_driver;
            let _ = braced!(through_driver in input);
            let input = through_driver;

            template_crate = input.parse()?;
            let _: Token![;] = input.parse()?;

            let tokens;
            let _ = bracketed!(tokens in input);
            parse_unadvised! {
                tokens => || {
                    options.parse_update(&tokens, opcontext_template)
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

        Ok(AOk(EngineExpandInput {
            driver,
            options,
            template,
            template_crate,
            template_name,
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

impl EngineInput for EngineExpandInput {
    fn process(self: Box<Self>) -> syn::Result<TokenStream> {
        dprintln!("derive_deftly_engine! crate = {:?}", &self.template_crate);

        let DdOptions {
            dbg,
            driver_kind,
            expect_target,
            //
        } = self.options;

        if let Some(exp) = driver_kind {
            macro_rules! got_kind { { $($kind:ident)* } => {
            match &self.driver.data {
                $(
                    syn::Data::$kind(..) => ExpectedDriverKind::$kind,
                )*
            }
        } }

            let got_kind = got_kind!(Struct Enum Union);
            if got_kind != exp.value {
                return Err([
                    (exp.span, "expected kind"),
                    (self.driver.span(), "actual kind"),
                ]
                .error(format_args!(
                    "template defined for {}, but applied to {}",
                    exp.value, got_kind,
                )));
            }
        }

        Context::call(
            &self.driver,
            &self.template_crate,
            self.template_name.as_ref(),
            |ctx| {
                let mut output = TokenAccumulator::new();
                self.template.expand(&ctx, &mut output);
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
                    check::check_expected_target_syntax(
                        &ctx,
                        &mut output,
                        target,
                    );
                }

                Ok(output)
            },
        )
    }
}

impl ParseAdviseable for Box<dyn EngineInput> {
    fn parse_adviseable(input: ParseStream) -> AdviseableResult<Self> {
        fn parse_as<T: EngineInput>(
            input: ParseStream,
        ) -> AdviseableResult<Box<dyn EngineInput>> {
            let parsed = T::parse_adviseable(input)?;
            Ok(parsed.map(|parsed| Box::new(parsed) as _))
        }

        let la = input.lookahead1();
        if la.peek(token::Brace) {
            parse_as::<EngineExpandInput>(input)
        } else if la.peek(token::Bracket) {
            parse_as::<accum::EngineAccumInput>(input)
        } else {
            Err(la.error())
        }
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
    let input: Box<dyn EngineInput> = adviseable_parse2(input)?;
    input.process()
}
