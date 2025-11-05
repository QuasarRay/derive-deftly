//! Macro modules, `define_derive_deftly_module`

use super::framework::*;
use define::TemplateDefinition;

/// A single `use Module;`
///
/// Could come from a template definition preamble,
/// or the start of the body of a module itself.
#[derive(Debug, Clone)]
pub struct UseModule {
    name: ModulePath,
    #[allow(unused)]
    beta_mods: beta::Enabled, // Remove when modules are no longer beta
}

/// Nonempty list of `use Module;` statements.
#[derive(Debug, Clone)]
pub struct SomeUseModules {
    outer: UseModule,
    uses: Vec<UseModule>,
}

/// A `${define}` or `${defcond}`, in `TokenStream`-like form
///
/// Is always dollar-escaped (`$orig_dollar`), when converted to tokens.
///
/// Invariant: it *is* one of those two, in correct syntax.
#[derive(Debug)]
pub struct SharedDef {
    stream_d_escaped: TokenStream,
}

pub type SharedDefs = Concatenated<SharedDef>;

/// The input to `define_derive_deftly_module!`
#[derive(Debug)]
pub struct ModuleDefinition {
    doc_attrs: DocAttributes,
    export: Option<MacroExport>,
    name: ModuleName,
    uses: Vec<UseModule>,
    defs: SharedDefs,
    #[allow(unused)]
    beta_mods: beta::Enabled, // Remove when modules are no longer beta
}

/// The PREFIX inside a `derive_deftly_engine!` call to define a template
#[derive(Debug, Default)]
pub struct ImportedDefinitions {
    parsed: Template<TokenAccumulator>,
}
impl Deref for ImportedDefinitions {
    type Target = Template<TokenAccumulator>;
    fn deref(&self) -> &Template<TokenAccumulator> {
        &self.parsed
    }
}

impl ModuleDefinition {
    fn parse(
        input: ParseStream,
        beta_mods: beta::Enabled,
    ) -> syn::Result<Self> {
        let doc_attrs = input.parse()?;
        let export = MacroExport::parse_option(input)?;
        let name = input.parse()?;
        let mut options =  DdOptions::default();
        options.parse_update(input, OpContext::ModuleDefinition)?;
        let beta_content = options.beta_enabled;

        let _colon: Token![:] = input.parse()?;
        let uses = UseModule::parse_several(input)?;

        let defs = SharedDefs::parse(
            input,
            beta_content,
            OrigDollarDeescapedDiscriminants::NotFound,
        )?;

        Ok(ModuleDefinition {
            doc_attrs,
            export,
            name,
            uses,
            defs,
            beta_mods,
        })
    }
}

impl SharedDefs {
    fn parse(
        input: ParseStream,
        beta_content: Option<beta::Enabled>,
        exp_d_escaped: OrigDollarDeescapedDiscriminants,
    ) -> syn::Result<Self> {
        // We want to:
        // 1. parse a `${define...}` or `${defcond...}` to check it ssyntax.
        // 2. somehow capture what we parsed as a TokenStream
        // 3. Make sure that we dollar-escape it if it wasn't already.

        // Our approach is:
        //
        //  * Use `ParseBuffer::fork` to obtain a separate cursor.
        //  * Parse from there into a `Template`.
        //  * Check that `Template` is what we expect.
        //  * With the original ParseStream, manually fish out
        //   `$`, the `orig_dollar`, and the `Group`.
        //    Escape the `Group` if we need to.
        //  * Check that the cursor is the3 same as we just had.
        //  * Reassemble a ts from those manually-disassembled pieces.

        let mut defs = vec![];
        while input.peek(Token![$]) {
            let def_end_cursor = {
                let input = input.fork();
                let subst = beta::with_maybe_enabled(beta_content, || {
                    Subst::<TokenAccumulator>::parse_entire(&input)
                })?;
                match subst.sd {
                    SD::define(..) | SD::defcond(..) => {}
                    _other => return Err(subst.kw_span.error(
                        "only ${define..} and ${defcond..} are allowed here",
                    )),
                }
                input.cursor()
            };

            let stream_d_escaped = (|| {
                let mut out = TokenStream::new();

                let dollar: Token![$] = input.parse()?;
                dollar.to_tokens(&mut out);

                let got_d_escaped = deescape_orig_dollar(input)?;

                if got_d_escaped.discriminant() != exp_d_escaped {
                    return Err(dollar.error(format_args!(
                   "escaping not as expected: got={:?}, exp={exp_d_escaped:?}",
                        got_d_escaped.discriminant(),
                    )));
                }

                let g_in: proc_macro2::Group = input.parse()?;

                let o_d_span;
                let g_out;

                match got_d_escaped {
                    OrigDollarDeescaped::NotFound => {
                        o_d_span = dollar.span();
                        let stream_d_escaped = escape_dollars(g_in.stream());
                        let mut g = proc_macro2::Group::new(
                            g_in.delimiter(),
                            stream_d_escaped,
                        );
                        g.set_span(g_in.span());
                        g_out = g;
                    }
                    OrigDollarDeescaped::Found(span) => {
                        // The outer $ has been escaped as expected.
                        // Trust the inner input is (preperly) escaped too.
                        // This isn't an adversarial context.
                        o_d_span = span;
                        g_out = g_in;
                    }
                }

                out.extend(quote_spanned! {o_d_span=> orig_dollar });
                g_out.to_tokens(&mut out);

                if input.cursor() != def_end_cursor {
                    return Err(input.error(
 "shared definitions precheck desynchronised with ad-hoc TS extraction"
                    ));
                }

                Ok(out)
            })()
            .map_err(adviseable::advise_incompatibility)?;

            defs.push(SharedDef { stream_d_escaped })
        }
        Ok(Concatenated(defs))
    }
}

impl UseModule {
    pub fn parse_several(input: ParseStream) -> syn::Result<Vec<Self>> {
        let mut uses = vec![];
        while input.peek(Token![use]) {
            let use_kw: Token![use] = input.parse()?;
            let kw_span = use_kw.span();
            let name = input.parse()?;
            let _: Token![;] = input.parse()?;
            uses.push(UseModule {
                name,
                beta_mods: beta::Enabled::new_for_modules_feature(kw_span)?,
            });
        }
        Ok(uses)
    }
}

impl SomeUseModules {
    /// If there are some `use`s, return a `SomeUseModules` to handle them
    pub fn divert(mut uses: Vec<UseModule>) -> Option<Self> {
        let outer = uses.pop()?;
        Some(SomeUseModules { outer, uses })
    }

    /// Output a suitable `via_modules` macro call
    pub fn output(
        self,
        mode: syn::Ident,
        new_prefix_d_escaped: TokenStream,
        main_passthrough: TokenStream,
    ) -> syn::Result<TokenStream> {
        let use_outer = self.outer.name.macro_path();
        let uses = self
            .uses
            .into_iter()
            .map(|u| {
                let path = u.name.macro_path();
                quote! { #path {} }
            })
            .collect_vec();

        let engine = engine_macro_name()?;

        Ok(quote! {
            #use_outer! {
                via_modules [ {} #(#uses)* #engine {} #mode () ]
                { #new_prefix_d_escaped }
                { #main_passthrough }
                ( $ )
            }
        })
    }
}

impl ToTokens for SharedDef {
    fn to_tokens(&self, out: &mut TokenStream) {
        self.stream_d_escaped.to_tokens(out);
    }
}

impl Parse for ImportedDefinitions {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let impossible = || {
            adviseable::advise_incompatibility(
                input.error("unsupported content in imported definitions"),
            )
        };

        let parsed: Template<TokenAccumulator> = beta::with_maybe_enabled(
            beta::Enabled::new_for_imported_definitions(),
            || input.parse(),
        )?;

        for te in &parsed.elements {
            let TE::Subst(subst) = te else {
                return Err(impossible());
            };
            match &subst.sd {
                SD::define(..) | SD::defcond(..) => {}
                _other => return Err(impossible()),
            }
        }

        Ok(ImportedDefinitions { parsed })
    }
}

struct ExpansionsAndAttrsOnly;
impl SpecialParseContext for ExpansionsAndAttrsOnly {
    fn before_element_hook(
        &mut self,
        input: ParseStream,
    ) -> syn::Result<Option<SpecialInstructions>> {
        Ok(
            if input.peek(Token![$])
                || input.peek(Token![#])
                || input.peek(token::Bracket)
            // any other groups besides #[] will be spotted later
            {
                None
            } else {
                Some(SpecialInstructions::EndOfTemplate)
            },
        )
    }
}

impl TemplateDefinition {
    /// Expands the docs, and parses the rest of the template
    pub fn parse_from_via_modules(
        prefix_d_escaped: ParseStream,
        main: ParseStream,
    ) -> syn::Result<Self> {
        let prefix_d_escaped: TokenStream = prefix_d_escaped.parse()?;
        let imported_definitions: ImportedDefinitions =
            syn::parse2(prefix_d_escaped.clone())?;

        let doc_attrs = beta::with_maybe_enabled(
            None, // TODO MODULES improve error messages
            || {
                Template::<TokenAccumulator>::parse_special(
                    main,
                    &mut ExpansionsAndAttrsOnly,
                )
            },
        )?;

        let mut definition: TemplateDefinition =
            // This will harmlessly look again for some doc attributes,
            // bu we just consumed all of those from `main`.
            main.parse()?;

        definition.doc_attrs = {
            let mut expanded = TokenAccumulator::new();
            Template::expand_iter(
                chain!(
                    &imported_definitions.parsed.elements,
                    &doc_attrs.elements,
                ),
                GeneralContext::NoDriver(&DriverlessContext {
                    defs: DefinitionsContext::default(),
                    driver_needed: &|span| {
                        span.error(
 "not allowed outside the main template body; no driver data structure here"
                    )
                    },
                }),
                &mut expanded,
            );
            let expanded = expanded.tokens()?;
            syn::parse2(expanded)?
        };
        definition.imported_definitions_d_escaped = prefix_d_escaped;

        Ok(definition)
    }
}

impl ModuleDefinition {
    pub fn parse_from_via_modules(
        prefix_d_escaped: ParseStream,
        main: ParseStream,
        beta_mods: beta::Enabled,
    ) -> syn::Result<Self> {
        let mut mod_def = ModuleDefinition::parse(main, beta_mods)?;
        let imported = SharedDefs::parse(
            prefix_d_escaped,
            beta::Enabled::new_for_imported_definitions(),
            OrigDollarDeescapedDiscriminants::Found,
        )?;
        mod_def.defs =
            Concatenated(chain!(imported.0, mod_def.defs.0,).collect_vec());
        Ok(mod_def)
    }
}

/// This is `define_derive_deftly_module!`
pub fn define_derive_deftly_module(
    input: TokenStream,
) -> Result<TokenStream, syn::Error> {
    dprint_block!(&input, "define_derive_deftly_module! input");

    let beta_mods = beta::Enabled::new_for_modules_feature(input.span())?;

    let md = Parser::parse2(
        |input: ParseStream<'_>| ModuleDefinition::parse(input, beta_mods),
        input,
    )?;

    define_module(md, "define_derive_deftly_module! output")
}

pub fn define_module(
    md: ModuleDefinition,
    #[allow(unused)] dprint_prefix: &str,
) -> syn::Result<TokenStream> {
    let ModuleDefinition {
        doc_attrs,
        export,
        name,
        uses,
        defs,
        beta_mods: _,
    } = md;

    let export_attr = export.map(|pub_token| {
        let span = pub_token.span();
        quote_spanned!(span=> #[macro_expor])
    });

    let mac_name = name.macro_name();
    let doc_attrs = doc_attrs.to_tokens_with_addendum(format_args!(
        r#"

This is a `derive_deftly` reuseable template code module.
Do not invoke it directly."#,
    ));

    let mk_return = |output| {
        dprint_block!(&output, "{} {}", dprint_prefix, mac_name);
        Ok(output)
    };

    if let Some(divert) = SomeUseModules::divert(uses) {
        return mk_return(divert.output(
            format_ident!("defmod"),
            defs.to_token_stream(),
            quote! { #doc_attrs #export_attr #name: },
        )?);
    }

    mk_return(quote! {
        #doc_attrs
        #export_attr
        macro_rules! #mac_name {
            {
                via_modules [
                    { $($our_opts:tt)* }
                    $next_macro:path
                { $($next_opts:tt)* }
                    $($rest:tt)*
                ]
                { $($predefs:tt)* }
                { $($main:tt)* }
                ( $($extra:tt)* )
                $($ignored:tt)*
            } => {
                $next_macro! {
                    via_modules [ { $($next_opts)* } $($rest)* ]
                    { #defs $($predefs)* }
                    { $($main)* }
                    ( $($extra)* )
                }
            };
            { $($wrong:tt)* } => {
                compile_error!{concat!(
                    "wrong input to derive-deftly module macro ",
                    stringify!(#mac_name: $($wrong)*),
                    "; might be due to incompatible derive-deftly versions(s)",
                )}
            };
        }
    })
}
