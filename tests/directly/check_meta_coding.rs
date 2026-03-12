//! Test used meta node encoding/decoding

use super::*;
use macros::meta::*;

use PreprocessedContent as PC;
use Usage as U;

//---------- ctx and pmetas and ptrees walker ----------

trait NodeCall: FnMut(&PreprocessedTree) {}
impl<F> NodeCall for F where F: FnMut(&PreprocessedTree) {}

fn process_nodes_ptree(ptree: &PreprocessedTree, nc: &mut impl NodeCall) {
    nc(ptree);
    match &ptree.content {
        PC::List(l) => process_nodes_pvl(l, nc),
        PC::Unit | PC::Value { .. } => {}
    }
}
fn process_nodes_pvl(pvl: &PreprocessedContentList, nc: &mut impl NodeCall) {
    for ptree in &pvl.content {
        process_nodes_ptree(ptree, nc);
    }
}
fn process_nodes_pmetas(
    pmetas: &[PreprocessedContentList],
    nc: &mut impl NodeCall,
) -> Result<(), Void> {
    for pvl in pmetas {
        process_nodes_pvl(pvl, nc);
    }
    Ok(())
}
fn process_nodes_ctx(ctx: &Context, nc: &mut impl NodeCall) {
    process_nodes_pmetas(&ctx.pmetas, nc).void_unwrap();

    WithinVariant::for_each(&ctx, |ctx, wv| {
        process_nodes_pmetas(&wv.pmetas, nc)?;
        WithinField::for_each(ctx, |_ctx, wf| {
            process_nodes_pmetas(&wf.pfield.pmetas, nc)
        })
    })
    .void_unwrap();
}
fn process_nodes_top<T>(
    top: &syn::DeriveInput,
    scenario: &[(&syn::Path, Usage)],
    start: impl FnOnce(&Context),
    mut nc: impl FnMut(&PreprocessedTree, Usage),
    finish: impl FnOnce(Context) -> T,
) -> T {
    Context::call(&top, &dummy_path(), None, None, |ctx| {
        start(&ctx);
        let mut rs = scenario.iter();
        process_nodes_ctx(&ctx, &mut |ptree: &PreprocessedTree| {
            let (node_path, node_allow) = *rs.next().unwrap();
            assert_eq!(ptree.path, *node_path);
            nc(ptree, node_allow);
        });
        assert!(rs.next().is_none());
        Ok(finish(ctx))
    })
    .unwrap()
}

//---------- core algorithm ----------

type NodeInScenario<'n> = (&'n syn::Path, Usage);
struct ScenarioDebug<'a>(&'a [NodeInScenario<'a>]);

fn roundtrip_scenario(
    top: &syn::DeriveInput,
    input_scenario: &[NodeInScenario],
    output_scenario: &[NodeInScenario],
) {
    let encoded = process_nodes_top(
        &top,
        input_scenario,
        |_ctx| {},
        |ptree, allow| {
            if allow != U::NONE {
                ptree.update_used(allow);
            }
        },
        |ctx| ctx.encode_metas_used().stream(),
    );

    process_nodes_top(
        &top,
        output_scenario,
        |ctx| {
            Parser::parse2(
                |input: ParseStream| ctx.decode_update_metas_used(input),
                encoded.clone(),
            )
            .unwrap();
        },
        |ptree, allow| {
            assert_eq!(
                ptree.used.get(), allow,
     "mismatch; at ptree.path={}; encoded {}; scenario I={:?} O={:?}",
                &encoded,
                ptree.path.to_token_stream(),
                ScenarioDebug(input_scenario),
                ScenarioDebug(output_scenario),
            )
        },
        |_ctx| {},
    );
}

impl Debug for ScenarioDebug<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[ ")?;
        for (p, a) in self.0 {
            let a = match *a {
                U::NONE => '-',
                U::BOOL_ONLY => '?',
                U::VALUE_ONLY => '+',
                U::VALUE => '=',
            };
            write!(f, "{}{}", p.to_token_stream(), a)?;
        }
        write!(f, "]")
    }
}

//---------- test matrix generation ----------

fn check_matching_for(topkind: &str, variants: &[&str], fields: &[&str]) {
    let mk_ident = |s: &_| syn::Ident::new(s, Span::call_site());

    let mut vs_toks = TokenStream::new();
    let cell_paths: Vec<syn::Path> = vec![];
    let cell_paths = RefCell::new(cell_paths);

    let add_metas = |forwhat, full| {
        let mut metas = TokenStream::new();
        let mut cell_paths = cell_paths.borrow_mut();
        for bi in [0, 1] {
            let mut bmetas = Punctuated::<_, Token![,]>::new();
            for mi in [0, 1] {
                // trim test matrix:
                // one meta node only if we haven't got n^5 already
                // (ideally, we'd do some kind of two-pass thing where
                // we prioritise things at the end, but that'd be a pain)
                let full = full && cell_paths.len() < 6;
                if !full && [bi, mi] != [0, 0] {
                    continue;
                }
                let name = mk_ident(&format!("{}{}{}", forwhat, bi, mi));
                cell_paths.push(name.clone().into());
                bmetas.push(name);
            }
            if bmetas.len() != 0 {
                metas.extend(quote!(#[deftly(#bmetas)]));
            }
        }
        metas
    };

    // trim test matrix:
    // multiple meta nodes for toplevel only if there's only 1 variant
    let tmetas = add_metas("t", variants.len() <= 1);

    let aliased = if topkind == "struct" {
        // The toplevel attributes on a struct can be seen by a template in
        // two ways: via tmeta, or via vmeta.  We test both.
        // We *dnn't* filter out the synthetic struct toplevel variant,
        // in process_nodes_top.  Which means, we will go through these
        // toplevel attributes twice.
        // (A development version of the meta used handling
        // had a bug relating to this aliasing of the struct toplevel.)
        let mut cell_paths = cell_paths.borrow_mut();
        let aliased = cell_paths.len();
        cell_paths.extend_from_within(..);
        aliased
    } else {
        0
    };

    for (vindex, &vname) in variants.iter().enumerate() {
        let mut fs_toks = TokenStream::new();
        let vinfo = if vname != "" {
            let vident = mk_ident(vname);
            // trim test matrix:
            // multiple meta nodes only for the last variant
            let vmetas = add_metas("v", vindex == variants.len() - 1);
            Some((vident, vmetas))
        } else {
            None
        };
        for &fname in fields {
            let fident = mk_ident(fname);
            let fmetas = add_metas("f", true);
            fs_toks.extend(quote!(
                #fmetas
                #fident: (),
            ));
        }
        if let Some((vident, vmetas)) = &vinfo {
            fs_toks = quote!(
                #vmetas
                #vident { #fs_toks },
            )
        }
        vs_toks.extend(fs_toks);
    }
    let topkind: TokenTree = syn::parse_str(topkind).unwrap();
    let top_toks = quote!(
        #tmetas
        #topkind Data { #vs_toks }
    );
    eprintln!("{}", top_toks);

    let top: syn::DeriveInput = syn::parse2(top_toks.clone()).unwrap();

    let cell_paths = cell_paths.into_inner();

    // trim test matrix:
    // test both values and booleans only if we're short
    let allows = if cell_paths.len() <= 4 {
        const A: &[U] = &[U::BOOL_ONLY, U::VALUE];
        A
    } else {
        const A: &[U] = &[U::BOOL_ONLY];
        A
    };

    for scenario in cell_paths
        .iter()
        .map(|path| {
            chain!(
                [U::NONE], //
                allows.into_iter().copied(),
            )
            .map(move |ra| (path, ra))
        })
        .multi_cartesian_product()
    {
        let mut output_scenario;
        let output_scenario = if aliased == 0 {
            &scenario
        } else {
            output_scenario = scenario.clone();
            let (a, b) = output_scenario[0..aliased * 2].split_at_mut(aliased);
            for (a, b) in izip!(a, b) {
                let u: Usage = a.1 | b.1;
                a.1 = u;
                b.1 = u;
            }
            &output_scenario
        };
        roundtrip_scenario(&top, &scenario, output_scenario);
    }
}

#[test]
fn check_matching() {
    for topkind in ["struct", "enum"] {
        for variants in if topkind == "struct" {
            const VS: &[&[&str]] = &[&[""]];
            VS
        } else {
            const VS: &[&[&str]] = &[&[], &["V1"], &["V1", "V2"]];
            VS
        } {
            for fields in if variants.is_empty() {
                const FS: &[&[&str]] = &[&[]];
                FS
            } else {
                const FS: &[&[&str]] = &[&[], &["f1"], &["f1", "f2"]];
                FS
            } {
                check_matching_for(topkind, variants, fields);
            }
        }
    }
}
