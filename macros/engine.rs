//! `derive_deftly_engine!()`

use super::framework::*;

/// Input to `derive_deftly_engine!`
#[derive(Debug)]
pub struct EngineInput {
    pub driver_brace: token::Brace,
    pub driver: syn::DeriveInput,
    pub template_brace: token::Brace,
    pub template_crate: syn::Path,
    pub template_name: Option<syn::Path>,
    pub options: DaOptions,
    pub template: Template<TokenAccumulator>,
}

impl Parse for EngineInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // This strange structure is to add a note to most of the errors that
        // come out of syn parsing.  The braced! etc. macros insist that the
        // calling scope throws syn::Error.
        // See the match at the bottom for what the return values mean.
        match (|| {
            let mut options = DaOptions::default();

            // If `got` is an error, returns Some of what the outer closure
            // should return.  Otherwise, return None.
            let unadvised_err_of_unit =
                |got: syn::Result<()>| got.err().map(Err).map(Ok);

            let driver;
            let driver_brace = braced!(driver in input);
            let driver = driver.parse()?;

            if input.peek(syn::token::Bracket) {
                let tokens;
                let _ = bracketed!(tokens in input);
                let r = options
                    .parse_update(&tokens, OpContext::DriverApplicationPassed);
                if let Some(r) = unadvised_err_of_unit(r) {
                    return r;
                }
            }

            let driver_passed;
            let _ = braced!(driver_passed in input);
            let _: TokenStream = driver_passed.parse()?;

            let template;
            let template_brace = braced!(template in input);
            let template = Template::parse(&template);

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
                let r = options.parse_update(&tokens, OpContext::Template);
                if let Some(r) = unadvised_err_of_unit(r) {
                    return r;
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

            let template = match template {
                Ok(template) => template,
                Err(err_return_raw) => return Ok(Err(err_return_raw)),
            };

            Ok(Ok(EngineInput {
                driver_brace,
                driver,
                template_brace,
                template,
                template_crate,
                template_name,
                options,
            }))
        })() {
            Ok(Ok(dae_input)) => Ok(dae_input),
            Ok(Err(err_to_return_directly)) => Err(err_to_return_directly),
            Err(err_needing_advice) => {
                Err(advise_incompatibility(err_needing_advice))
            }
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
    // eprintln!("derive_deftly_engine! {}", &input);
    let input: EngineInput = syn::parse2(input)?;
    // eprintln!("derive_deftly_engine! crate = {:?}", &input.template_crate);

    let DaOptions {
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
