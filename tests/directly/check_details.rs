//! Tests of individual expansion details

use super::*;

use ExpectedOutputNoneGrouping as EONG;

fn tests_using_driver(
    driver: TokenStream,
    grouping: ExpectedOutputNoneGrouping,
    cases: &[(&str, Result<&str, &str>)],
) {
    eprintln!("----- driver -----\n{}\n----- cases -----", driver);
    let driver: syn::DeriveInput = syn::parse2(driver).unwrap();

    for (input_s, exp) in cases {
        eprintln!(" case {}", input_s);
        let input: Template<_> = syn::parse_str(input_s).unwrap();
        let exp = ExpectedOutput::parse(*exp, grouping).unwrap();

        let got = Context::call(&driver, &parse_quote!(crate), None, |ctx| {
            let mut ts = TokenAccumulator::new();
            input.expand(&ctx, &mut ts);
            ts.tokens()
        });

        exp.compare(got, input_s).unwrap_or_else(|m| {
            m.eprintln_initial();
            m.eprintln_extended();
            panic!("test case mismatch");
        });
    }
}

#[test]
fn check_details_one() {
    tests_using_driver(
        quote!(
            struct S {
                f: Option<u32>,
            }
        ),
        EONG::Explicit,
        &[
            //
            ("$vpat", Ok("S { f: f_f, }")),
            ("$( $ftype )", Ok("« Option::<u32> »")),
        ],
    );
}

#[test]
fn check_details_idents() {
    tests_using_driver(
        quote! {
            #[deftly(raw = "r#for", plain = "plain")]
            struct S;
        },
        EONG::Explicit,
        &[
            ("${tmeta(raw) as str}", Ok(r#""r#for""#)),
            ("${tmeta(raw) as ident}", Ok("r#for")),
            ("$< ${tmeta(raw)} >", Ok("r#for")),
            (
                "$< ${tmeta(plain)} _ ${tmeta(raw)} >",
                Err(r#""plain_r#for""#),
            ),
            ("$< ${tmeta(raw)} _ ${tmeta(plain)} >", Ok("r#for_plain")),
            ("${tmeta(raw) as ident}", Ok("r#for")),
            ("$< ${tmeta(raw) as ident} >", Ok("r#for")),
            (
                "$< ${tmeta(plain)} _ ${tmeta(raw) as ident} >",
                Ok("plain_for"),
            ),
            (
                "$< ${tmeta(raw) as ident} _ ${tmeta(plain)} >",
                Ok("for_plain"),
            ),
        ],
    );
}

/// Returns `Ok(s)`, but checking that it parses as an item
fn item_ok(s: &str) -> Result<&str, &str> {
    syn::parse_str(s)
        .map(|_: syn::Item| s)
        .map_err(|e| panic!("parse failed: {:?}: {}", s, e))
}

// Test "redefinition keywords" and `$vpat`
//
// In particular, we test that
//  - you can define a new data structure
//  - of every kind
//  - with >1 of every kind of content (field, variant)
//    (proving that the commas are all there)
//  - and that it parses as an item
// This shows that our commas are all present and correct,
// avoiding a repetition of #74 for these cases.
// We use `Flatten` because any groups are buried in the middle, here.
//
// We also have handwritten expected outputs for $vpat in all cases.
#[test]
#[rustfmt::skip] // this test includes deliberately-missing commas in inputs
fn check_details_shapes() {
    let redefine = r#"
        $tdefkwd $tdeftype ${tdefvariants $(
             ${vdefbody $vname $(
                 ${fdefine $fname} $ftype,
             )}
         )}
    "#;
    tests_using_driver(
        quote!(
            struct S<'l, G: Bound = Def>;
        ),
        EONG::Flatten,
        &[
            ("$vpat", Ok("S {}")),
            (redefine, item_ok("struct S<'l, G: Bound = Def>;")),
        ],
    );
    tests_using_driver(
        quote!(
            struct S(A, B);

        ),
        EONG::Flatten,
        &[
            ("$vpat", Ok("S { 0: f_0, 1: f_1, }")),
            (redefine, item_ok("struct S(A, B,);")),
        ],
    );
    tests_using_driver(
        quote!(
            struct S { a: A, b: B }
        ),
        EONG::Flatten,
        &[
            //
            ("$vpat", Ok("S { a: f_a, b: f_b, }")),
            (redefine, item_ok("struct S { a: A, b: B, }")),
        ],
    );
    tests_using_driver(
        quote!(
            enum E {
                U,
                T(A, B),
                S { a: A, b: B }
            }
        ),
        EONG::Flatten,
        &[
            ("$( $vpat | )", Ok(r"E::U {} |
                                  E::T { 0: f_0, 1: f_1 ,} |
                                  E::S { a: f_a, b: f_b, } |")),
            ("$( ${vpat self=$<T $ttype> vname=$<V $vname> fprefix=g_} | )",
             Ok(r"TE::VU {} |
                  TE::VT { 0: g_0, 1: g_1, } |
                  TE::VS { a: g_a, b: g_b, } |")),
            (redefine, item_ok(r"enum E {
                                    U,
                                    T(A, B,),
                                    S { a: A, b: B, },
                                }")),
        ],
    );
}

#[test]
fn check_details_types() {
    tests_using_driver(
        quote! {
            struct T<'l0, 'l1: 'l0, AG: Bound = Def>(A<'l0, AG>);
        },
        EONG::Explicit,
        &[
            // Note lack of final `,` in generics inside A and T, below.
            // That's fine because one can't deconstruct this;
            // if we invent $fgens or something, it would need the `,`
            ("$( $ftype )", Ok("« A::<'l0, AG> »")),
            ("$ttype", Ok("T::<'l0, 'l1, AG>")),
            ("$vtype", Ok("T::<'l0, 'l1, AG>")),
            ("${vtype self=T2 vname=$<V $vname>}", Ok("T2")),
            (
                "${vtype self=$<T $ttype> vname=$<V $vname>}",
                Ok("TT::<'l0, 'l1, AG>"),
            ),
            ("$tdeftype", Ok("T<'l0, 'l1: 'l0, AG: Bound = Def>")),
        ],
    );

    tests_using_driver(
        quote! {
            enum E<'l0, 'l1: 'l0, AG: Bound> { U }
        },
        EONG::Explicit,
        &[
            //
            ("$( $vtype ) /", Ok("E::U::<'l0, 'l1, AG> /")),
            (
                "$( ${vtype self=$<T $ttype> vname=$<V $vname>} ) /",
                Ok("TE::VU::<'l0, 'l1, AG> /"),
            ),
        ],
    );

    tests_using_driver(
        quote! {
            enum E<'l0, 'l1: 'l0, AG: Bound, BG> {
                U,
                S {
                    a: A<'l0, AG>,
                    b: B::<'l1, BG>,
                },
            }
        },
        // Proper grouping is tested in the first block, above.
        EONG::Flatten,
        &[
            // Again, note lack of final `,` inside `$ftype` and `$vtype`
            ("$( $ftype / )", Ok("A::<'l0, AG> / B::<'l1, BG> /")),
            (
                "$( $vtype / )",
                Ok("E::U::<'l0, 'l1, AG, BG> / 
                                  E::S::<'l0, 'l1, AG, BG> /"),
            ),
        ],
    );
}
