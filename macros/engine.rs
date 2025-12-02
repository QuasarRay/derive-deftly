//! `derive_deftly_engine!()`

use super::framework::*;
use adviseable::*;
use modules::ImportedDefinitions;

/// Input to `derive_deftly_engine!`, principal form (template expansion)
///
/// See `implementation.md`,
/// especially
/// "Overall input syntax for `derive_deftly_engine!` and templates".
#[derive(Debug)]
struct EngineExpandInput {
    driver: syn::DeriveInput,
    options: DdOptions,
    imported_definitions: ImportedDefinitions,
    template: TopTemplate,
    template_crate: syn::Path,
    template_name: Option<syn::Path>,
    chain_next: Option<ChainNext>,
    chain_after: TokenStream,
    accum: TokenStream,
}

#[derive(Debug)]
pub struct ChainNext {
    pub call: TokenStream,
    pub after_driver: TokenStream,
}

enum EngineContext {
    Expand {
        opcontext_template: OpContext,
        options: DdOptions,
    },
    Final {},
}

#[derive(Debug)]
enum EngineInput {
    Expand(EngineExpandInput),
    Final(accum::EngineFinalInput),
    DefinitionViaModules(define::TemplateDefinition),
    ModuleDefinitionViaModules(modules::ModuleDefinition),
}

impl Parse for ChainNext {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let call = input.parse::<syn::Path>()?.to_token_stream();
        let after_driver;
        let _ = parenthesized!(after_driver in input);
        let after_driver = after_driver.parse()?;
        Ok(ChainNext { call, after_driver })
    }
}

impl ToTokens for ChainNext {
    fn to_tokens(&self, out: &mut TokenStream) {
        let ChainNext { call, after_driver } = self;
        quote!( #call (#after_driver) ).to_tokens(out);
    }
}

fn parse_via_mdoules_invocation(
    input: ParseStream,
) -> AdviseableResult<EngineInput> {
    let chain; // Inside [ ] after via_modules
    let this_args; // Inside { } and [ ], our arguments
    let prefix_d_escaped;
    let main;
    let extra; // inside the final ( $ ... )

    let _ = bracketed!(chain in input);
    let _ = braced!(this_args in chain);
    let mode = syn::Ident::parse_any(&chain)?;
    let _ = braced!(prefix_d_escaped in input);
    let _ = braced!(main in input);
    let _ = parenthesized!(extra in input);
    let _dollar: Token![$] = extra.parse()?;

    let r = if mode == "define" {
        let def = parse_unadvised! { main => || {
            define::TemplateDefinition::parse_from_via_modules(
                &prefix_d_escaped,
                main,
            )
        } };

        EngineInput::DefinitionViaModules(def)
    } else if mode == "defmod" {
        let beta = beta::Enabled::new_for_modules_feature(mode.span())?;
        let def = parse_unadvised! { main => || {
            modules::ModuleDefinition::parse_from_via_modules(
                &prefix_d_escaped,
                main,
                beta,
            )
        } };

        EngineInput::ModuleDefinitionViaModules(def)
    } else {
        return Err(
            mode.error(format!("unknown engine via_modules mode `{}`", mode))
        );
    };

    // Discard at three places where we leave room for expansion
    let _: TokenStream = extra.parse()?;
    let _: TokenStream = this_args.parse()?;
    let _: TokenStream = chain.parse()?;

    Ok(AOk(r))
}

impl ParseAdviseable for EngineInput {
    fn parse_adviseable(input: ParseStream) -> AdviseableResult<Self> {
        if input.peek(Ident::peek_any) {
            let special = syn::Ident::parse_any(input)?;
            return if special == "via_modules" {
                parse_via_mdoules_invocation(input)
            } else {
                Err(special.error(format!(
                    "unknown engine special expansion request `{}`",
                    special,
                )))
            };
        }

        let driver;
        let _ = braced!(driver in input);
        let driver = driver.parse()?;

        let engine_context;
        if input.peek(syn::token::Bracket) {
            // AOPTIONS appears iff we're being invoked for a precanned
            // template, rather than an adhoc one; it's from the
            // `#[derive()` application.
            let tokens;
            let mut options = DdOptions::default();

            let _ = bracketed!(tokens in input);
            parse_unadvised! {
                tokens => || {
                    let oc = OpContext::DriverApplicationPassed;
                    options .parse_update(&tokens, oc)
                }
            }
            engine_context = EngineContext::Expand {
                opcontext_template: OpContext::TemplateDefinition,
                options,
            };
        } else if input.peek(Token![.]) {
            let _indicator: Token![.] = input.parse()?;
            engine_context = EngineContext::Final {};
        } else {
            engine_context = EngineContext::Expand {
                opcontext_template: OpContext::TemplateAdhoc,
                options: DdOptions::default(),
            };
        }

        let future_ignored;
        let _ = parenthesized!(future_ignored in input);
        let _: TokenStream = future_ignored.parse()?;

        let r = match engine_context {
            EngineContext::Expand {
                opcontext_template,
                options,
            } => EngineExpandInput::parse_adviseable_remainder(
                driver,
                options,
                input,
                opcontext_template,
            )?
            .map(EngineInput::Expand),
            EngineContext::Final {} => {
                accum::EngineFinalInput::parse_adviseable_remainder(
                    driver, input,
                )?
                .map(EngineInput::Final)
            }
        };
        Ok(r)
    }
}

impl EngineExpandInput {
    fn parse_adviseable_remainder(
        driver: syn::DeriveInput,
        mut options: DdOptions,
        input: ParseStream,
        opcontext_template: OpContext,
    ) -> AdviseableResult<Self> {
        let template;
        let _ = braced!(template in input);

        let template_crate;
        let template_name;
        let imported_definitions;
        {
            let through_driver;
            let _ = parenthesized!(through_driver in input);
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

            imported_definitions = if !input.is_empty() {
                let imported_definitions;
                let _ = braced!(imported_definitions in input);
                imported_definitions.parse()?
            } else {
                ImportedDefinitions::default()
            };

            let _: TokenStream = input.parse()?;
        }

        let (chain_next, chain_after);
        {
            let chain;
            let _ = bracketed!(chain in input);
            let input = chain;
            chain_next = if !input.is_empty() {
                Some(input.parse()?)
            } else {
                None
            };
            chain_after = input.parse()?;
        }

        let accum;
        let _ = bracketed!(accum in input);
        let accum = accum.parse()?;

        let _: TokenStream = input.parse()?;

        let template = parse_unadvised! {
            template => || TopTemplate::parse(
                template,
                options.beta_enabled,
            )
        };

        Ok(AOk(EngineExpandInput {
            driver,
            options,
            imported_definitions,
            template,
            template_crate,
            template_name,
            chain_next,
            chain_after,
            accum,
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
            let pmetas = &tmetas;
            let pfields = preprocess_fields(fields)?;
            let pvariant = PreprocessedVariant {
                fields,
                pmetas,
                pfields,
            };
            syn::Result::Ok((Some(()), vec![pvariant]))
        };

        let union_fields;
        let variants_pmetas: Vec<_>;

        let (variant, pvariants) = match &driver.data {
            syn::Data::Struct(ds) => pvariants_one(&ds.fields)?,
            syn::Data::Union(du) => {
                union_fields = syn::Fields::Named(du.fields.clone());
                pvariants_one(&union_fields)?
            }
            syn::Data::Enum(de) => (None, {
                variants_pmetas = de
                    .variants
                    .iter()
                    .map(|variant| preprocess_attrs(&variant.attrs))
                    .try_collect()?;
                izip!(&de.variants, &variants_pmetas)
                    .map(|(variant, pmetas)| {
                        let fields = &variant.fields;
                        let pfields = preprocess_fields(&variant.fields)?;
                        Ok(PreprocessedVariant {
                            fields,
                            pmetas,
                            pfields,
                        })
                    })
                    .collect::<Result<Vec<_>, syn::Error>>()?
            }),
        };

        // `variant` is None in enums; otherwise it's Some(())
        // and here we convert it to the real WithinVariant for the fields.
        let variant = variant.map(|()| WithinVariant {
            variant: None, // not actually a variant
            fields: pvariants[0].fields,
            pmetas: &pvariants[0].pmetas,
            pfields: &pvariants[0].pfields,
            index: 0,
        });

        let ctx = Context {
            top: &driver,
            template_crate,
            template_name,
            pmetas: &tmetas,
            field: None,
            variant: variant.as_ref(),
            pvariants: &pvariants,
            defs: DefinitionsContext {
                defs: Default::default(),
                nesting_depth: 0,
                nesting_parent: None,
            },
            within_loop: WithinLoop::None,
        };

        f(ctx)
    }
}

impl EngineExpandInput {
    fn process(self) -> syn::Result<TokenStream> {
        dprintln!("derive_deftly_engine! crate = {:?}", &self.template_crate);

        let DdOptions {
            dbg,
            driver_kind,
            expect_target,
            beta_enabled,
            //
        } = self.options;

        // This was used when parsing EngineExpandInput.template
        let _: Option<_> = beta_enabled;

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

        let outcome = Context::call(
            &self.driver,
            &self.template_crate,
            self.template_name.as_ref(),
            |ctx| {
                let mut output = TokenAccumulator::new();
                Template::expand_iter(
                    chain!(
                        &self.imported_definitions.elements,
                        &self.template.elements,
                    ),
                    ctx.as_general(),
                    &mut output,
                );
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

                let metas_used = ctx.encode_metas_used();

                Ok((output, metas_used))
            },
        );

        let (expanded, metas_used) = match outcome {
            Ok((expanded, metas_used)) => (Ok(expanded), Ok(metas_used)),
            Err(e) => (Err(e), Err(())),
        };

        let chain_call;

        if let Some(ChainNext { call, after_driver }) = &self.chain_next {
            let driver = &self.driver;
            let chain_after = &self.chain_after;

            let mut accum = self.accum.to_token_stream();
            if let Some(name) = &self.template_name {
                accum.extend(quote!( _name [#name] ));
            }
            match &metas_used {
                Ok(metas_used) => {
                    accum.extend(quote!( _meta_used #metas_used ));

                    use meta::FindRecogMetas as _;
                    let mut meta_recog = meta::Recognised::default();
                    self.template.find_recog_metas(&mut meta_recog);
                    accum.extend(quote!( _meta_recog [#meta_recog] ));
                }
                Err(()) => {
                    accum.extend(quote!( _error [] ));
                }
            }

            chain_call = quote! {
                #call! {
                    { #driver }
                    #after_driver
                    [ #chain_after ]
                    [ #accum ]
                }
            }
        } else {
            chain_call = TokenStream::new();
        };

        let mut out = expanded.unwrap_or_else(|e| e.into_compile_error());
        dprint_block!(&out, "derive_deftly_engine! expansion output");
        dprint_block!(&chain_call, "derive_deftly_engine! chain call");
        out.extend(chain_call);

        Ok(out)
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
    match input {
        EngineInput::Expand(i) => i.process(),
        EngineInput::Final(i) => i.process(),
        EngineInput::DefinitionViaModules(def) => define::define_template(
            def,
            "derive_deftly_engine! output, define_derive_deftly via modules",
        ),
        EngineInput::ModuleDefinitionViaModules(def) => {
            modules::define_module(
                def,
                "derive_deftly_engine! output, define module via modules",
            )
        }
    }
}
