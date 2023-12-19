use derive_adhoc::{derive_adhoc, Adhoc};

#[derive(Adhoc)]
#[adhoc(wrong = "{")]
#[adhoc(dot = ".")]
//
//
#[adhoc(several = "first")]
#[adhoc(several = "second")]
#[adhoc(several_inner(several = "first", several = "second"))]
struct DataType {}

derive_adhoc! {
    DataType:

    const K: () = ${tmeta(wrong)}
}
// Invoke derive_adhoc! a second time, because (empirically) the
// attempt to parse "{" causes some kind of nonlocal exit:
// syn::LitStr::parse is called but doesn't return.  And then, something
// arranges to report only the *last* such error.  But we want to see
// all the errors, so invoke the macro separately.

derive_adhoc! {
    DataType:

    struct ${paste Bad ${tmeta(dot) as str}};
    // fails because `dot = ...` is not helpful
    struct ${paste Bad ${tmeta(dot(has_equals)) as str}};

    //
    //
    //

    struct ${paste Bad ${tmeta(several) as str}};
    struct ${paste Bad ${tmeta(several_inner(several)) as str}};
}

// Something somewhere deduplicates errors, so if we want to see them
// all we need to re-invoke derive-adhoc:
derive_adhoc! {
    DataType:

    // fails because `barelit("42")` is not helpful
    struct ${paste Bad ${tmeta(barelit(is_value)) as str}};
}

derive_adhoc! {
    DataType:

    // fails because `several_inner(...)` is not helpful
    struct ${paste Bad ${tmeta(several_inner) as str}};
}

#[derive(Adhoc)]
#[adhoc("forbidden")]
struct TopLevelMetaLitForbidden;
derive_adhoc! { TopLevelMetaLitForbidden: }

#[derive(Adhoc)]
#[adhoc(value = wrong)]
struct MetaValueMustBeLit;
derive_adhoc! { MetaValueMustBeLit: }

#[derive(Adhoc)]
#[adhoc(barelit("42"))]
struct WithBareLit;

#[derive(Adhoc)]
#[adhoc(multi("1", "2"))]
struct WithBareLitMulti;

derive_adhoc! {
    WithBareLit:
    struct ${paste Bad ${tmeta(barelit) as str}};
}

derive_adhoc! {
    WithBareLitMulti:
    struct ${paste Bad ${tmeta(multi) as str}};
}

fn main() {}
