//! Examples involving searching over possible outputs for a particular input
//!
//! Where the input file doesn't specify
//! which driver (or part of driver) generates the output.
//! (Or maybe where it limits it according to `limit`).

use super::*;

pub struct PossibilitiesExample {
    loc: DocLoc,
    /// A derive-deftly template fragment
    input: TokenStream,
    /// Limit on which contexts to consider
    limit: Limit,
    /// Expected output
    output: ExpectedOutput,
    /// Are *all* the possibilities (subject to `limit`) supposed to match?
    all_must_match: bool,
    /// Does the `output` faithfully represent `None`-delimited `Group`s?
    grouping: ExpectedOutputNoneGrouping,
}

struct Tracker {
    all_must_match: bool,
    matching_outputs: usize,
    other_outputs: Vec<Mismatch>,
    skipped_context_descs: Vec<String>,
}

impl Tracker {
    fn finish_ok(&self) -> Result<(), String> {
        if self.all_must_match {
            if !self.other_outputs.is_empty() {
                return Err(
 "at least one actual output doesn't match the documented output".into()
                );
            }
        }
        if self.matching_outputs == 0 {
            return Err(
                "documented output does not match any of the actual outputs"
                    .into(),
            );
        }
        Ok(())
    }

    fn should_continue(&self) -> bool {
        if !self.all_must_match && self.matching_outputs != 0 {
            return false;
        }
        true
    }

    fn note(&mut self, matched: Result<(), Mismatch>) {
        match matched {
            Ok(()) => self.matching_outputs += 1,
            Err(got) => self.other_outputs.push(got),
        }
    }

    fn note_skip(&mut self, context_desc: String) {
        self.skipped_context_descs.push(context_desc);
    }
}

impl Example for PossibilitiesExample {
    fn print_checking(&self) {
        println!(
            "checking :{} {} => {}",
            self.loc,
            &self.input,
            self.output,
        );
    }

    fn check(&self, errs: &mut Errors, drivers: &[syn::DeriveInput]) {
        let mut tracker = Tracker {
            all_must_match: self.all_must_match,
            matching_outputs: 0,
            other_outputs: vec![],
            skipped_context_descs: vec![],
        };
        //println!("  LIMIT {:?}", &self.limit);

        for_every_example_context(drivers, |ctx| {
            self.compare_one_output(&mut tracker, &ctx);
            Ok::<_, Void>(())
        })
        .void_unwrap();

        match tracker.finish_ok() {
            Ok(()) => {}
            Err(m) => {
                eprintln!();
                eprintln!("========================================");
                errs.wrong(self.loc, "example mismatch");
                eprintln!(
                    r"{}
input: {}
limit: {:?}
documented: {}",
                    m,
                    self.input,
                    self.limit,
                    self.output,
                );
                for got in &tracker.other_outputs {
                    got.eprintln_initial();
                }
                eprintln!("matched: {}", tracker.matching_outputs);
                eprint!("skipped:");
                for skip in tracker.skipped_context_descs {
                    eprint!(" [{}]", skip);
                }
                eprintln!("");
                for got in &tracker.other_outputs {
                    got.eprintln_extended();
                }
                eprintln!("========================================");
            }
        }
    }
}

impl PossibilitiesExample {
    pub fn new(
        loc: DocLoc,
        input: &str,
        limit: Limit,
        all_must_match: bool,
        output: Result<&str, &str>,
        grouping: ExpectedOutputNoneGrouping,
    ) -> Result<Box<PossibilitiesExample>, String> {
        let input = parse_str(input, "input")?;
        let output = ExpectedOutput::parse(output, grouping)?;
        Ok(Box::new(PossibilitiesExample {
            loc,
            input,
            limit,
            all_must_match,
            output,
            grouping,
        }))
    }

    fn compare_one_output(&self, tracker: &mut Tracker, ctx: &Context<'_>) {
        if !tracker.should_continue() {
            return;
        };
        let limit = &self.limit;

        let context_desc = ctx.desc_for_tests();

        if !limit
            .matches(ctx)
            // Treat errors "skip".
            .unwrap_or_default()
        {
            //println!("  INAPPLICABLE {:?}", &context_desc);
            tracker.note_skip(context_desc);
            return;
        }

        let input = &self.input;

        let mut out = TokenAccumulator::new();
        let got = (|| {
            let template: Template<TokenAccumulator> =
                syn::parse2(input.clone())?;

            template.expand(ctx, &mut out);
            let got = out.tokens()?;

            Ok(got)
        })();

        let matched = self.output.compare(
            got,
            self.grouping,
            &context_desc,
        );

        tracker.note(matched);
    }
}

#[test]
fn poc() {
    let driver: syn::DeriveInput = parse_quote! {
        pub(crate) enum Enum<'a, 'l: 'a, T: Display = usize,
                             const C: usize = 1>
        where T: 'l, T: TryInto<u8>
        {
            UnitVariant,
            TupleVariant(std::iter::Once::<T>),
            NamedVariant {
                field: &'l &'a T,
                field_b: String,
                field_e: <T as TryInto<u8>>::Error,
             },
        }
    };
    let input = quote! { $($vname,) };
    let limit = Limit::Name("Enum".into());
    let output = quote! { UnitVariant, TupleVariant, NamedVariant, };
    let output = ExpectedOutput {
        exp: Ok(output),
    };
    PossibilitiesExample {
        all_must_match: false,
        loc: 42,
        input: input,
        limit: limit,
        output,
        grouping: ExpectedOutputNoneGrouping::Flatten,
    }
    .check(&mut Errors::new(), &[driver]);
}
