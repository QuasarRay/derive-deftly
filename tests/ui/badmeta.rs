use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(wrong = "{")]
#[deftly(dot = ".")]
#[deftly(several = "first")]
#[deftly(several = "second")]
#[deftly(several_inner(several = "first", several = "second"))]
struct DataType {}

derive_deftly_adhoc! {
    DataType:

    const K: () = ${tmeta(wrong)}
}
// Invoke derive_deftly_adhoc! a second time, because (empirically) the
// attempt to parse "{" causes some kind of nonlocal exit:
// syn::LitStr::parse is called but doesn't return.  And then, something
// arranges to report only the *last* such error.  But we want to see
// all the errors, so invoke the macro separately.

derive_deftly_adhoc! {
    DataType:

    struct ${paste Bad ${tmeta(dot) as str}};
    // fails because `dot = ...` is not helpful
    struct ${paste Bad ${tmeta(dot(has_equals)) as str}};

    struct ${paste Bad ${tmeta(several) as str}};
    struct ${paste Bad ${tmeta(several_inner(several)) as str}};
}

// Something somewhere deduplicates errors, so if we want to see them
// all we need to re-invoke derive-deftly:
derive_deftly_adhoc! {
    DataType:

    // fails because `barelit("42")` is not helpful
    struct ${paste Bad ${tmeta(barelit(is_value)) as str}};
}

derive_deftly_adhoc! {
    DataType:

    // fails because `several_inner(...)` is not helpful
    struct ${paste Bad ${tmeta(several_inner) as str}};
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly("forbidden")]
struct TopLevelMetaLitForbidden;
derive_deftly_adhoc! { TopLevelMetaLitForbidden: }

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(value = wrong)]
struct MetaValueMustBeLit;
derive_deftly_adhoc! { MetaValueMustBeLit: }

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(barelit("42"))]
struct WithBareLit;

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(multi("1", "2"))]
struct WithBareLitMulti;

derive_deftly_adhoc! {
    WithBareLit:
    struct ${paste Bad ${tmeta(barelit) as str}};
}

derive_deftly_adhoc! {
    WithBareLitMulti:
    struct ${paste Bad ${tmeta(multi) as str}};
}

define_derive_deftly! {
    TestFunkyPath =
    struct ${paste Bad ${tmeta(std::cell::Cell::<String>::ne)w as str}};
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[derive_deftly(TestFunkyPath)]
#[deftly(std::cell::Cell::<String>::new)]
struct WithFunkyPath;

derive_deftly_adhoc! {
    WithFunkyPath:
}

#[derive(Deftly)]
#[derive_deftly(TestFunkyPath)]
struct WithoutfunkyPath;

fn main() {}
