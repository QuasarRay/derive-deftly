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
            ("$< ${tmeta(plain)} _ ${tmeta(raw)} >", Err(r#""plain_r#for""#)),
            ("$< ${tmeta(raw)} _ ${tmeta(plain)} >", Ok("r#for_plain")),
            ("${tmeta(raw) as ident}", Ok("r#for")),
            ("$< ${tmeta(raw) as ident} >", Ok("r#for")),
            ("$< ${tmeta(plain)} _ ${tmeta(raw) as ident} >", Ok("plain_for")),
            ("$< ${tmeta(raw) as ident} _ ${tmeta(plain)} >", Ok("for_plain")),
        ],
    );
}
