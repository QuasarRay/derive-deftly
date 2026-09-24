use derive_deftly::{
    define_derive_deftly, define_derive_deftly_module, derive_deftly_adhoc,
    Deftly,
};

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(unit)]
#[deftly(wrong = "{")]
#[deftly(dot = ".")]
#[deftly(several = "first")]
#[deftly(several = "second")]
#[deftly(several_inner(several = "first", several = "second"))]
#[deftly(integer = "42", with_suffix = "42"x)]
#[deftly(int128 = "0x12345678_12345678_12345678_12345678")]
struct DataType {}

derive_deftly_adhoc! {
    DataType:

    const K: () = ${tmeta(wrong) as expr}
    const L: () = ${tmeta(dot) as expr}
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
derive_deftly_adhoc! {
    MetaValueMustBeLit:
    ${tmeta(value) as token_stream}
}

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(value = very < wrong)]
struct MetaValueBareForbidden;

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(value2 = )]
struct MetaValueBareForbidden2;

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(value3 = setting = inner_value)]
struct MetaValueBareForbidden3;

define_derive_deftly! {
    export MetaQuotedWithoutBeta:
    ${tmeta(value) as expr, quoted strip}
    ${tmeta(value) as expr, quoted retain}
}

define_derive_deftly! { export MetaQuotedOptionNoBeta1 meta_quoted strip: }
define_derive_deftly! { export MetaQuotedOptionNoBeta2 meta_quoted retain: }
define_derive_deftly_module! { export MetaQuotedModule meta_quoted retain: }

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
    TestFunkyPath:
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

define_derive_deftly! {
    WrongRequestSyntax:

    $tmeta(something)
    ${if tmeta}

    ${tmeta(something) unknown_option}
    ${if tmeta(something) unknown_option {}}
}

derive_deftly_adhoc! {
    DataType:

    ${tmeta(dot)} // forgot `.. as`, so `"."` not even parsed
}

derive_deftly_adhoc! {
    DataType:
    ${if approx_equal("42", ${tmeta(with_suffix) as str}) {}}
}
derive_deftly_adhoc! {
    DataType:
    ${if approx_equal(0x12345678_12345678_12345678_aaaaaaaa,
                      ${tmeta(int128) as token_stream}) {}}
}
derive_deftly_adhoc! {
    DataType:
    ${fmeta(missing) default { 42 }}
}
derive_deftly_adhoc! {
    DataType beta_deftly:
    // This might be allowed to work in future?  See
    //  https://gitlab.torproject.org/Diziet/rust-derive-deftly/-/issues/48#note_3284656
    ${tmeta(unit) as token_stream, default {}}
}





define_derive_deftly! {
    UnusedUnit beta_deftly:
    ${tmeta(unit) as token_stream, default { }}
}
#[derive(Deftly)]
#[derive_deftly(UnusedUnit)]
#[deftly(unit)]
struct UnusedUnit;

define_derive_deftly! {
    UnusedDueToError beta_deftly:
    ${error "force failure"}
}
#[derive(Deftly)]
#[derive_deftly(UnusedDueToError)]
#[deftly(unused)] // shouldn't be reported, since template has error
struct UnusedDueToError;

define_derive_deftly! {
    UnusedDueToSyntax beta_deftly, expect items:
    not item;
}
#[derive(Deftly)]
#[derive_deftly(UnusedDueToSyntax)]
#[deftly(ignored)]
struct UnusedDueToSyntax;

define_derive_deftly! {
    IgnoredDueToCondition:

    $(
        ${when not(fmeta(thing(okay)))}
        ${if fmeta(thing(x)) { }}
    )
}
#[derive(Deftly)]
#[derive_deftly(IgnoredDueToCondition)]
struct IgnoredDueToCondition {
    #[deftly(thing(okay))]
    #[deftly(thing(x))]
    a: bool,
}
#[derive(Deftly)]
#[derive_deftly(IgnoredDueToCondition)]
struct IgnoredDueToCondition2 {
    #[deftly(thing(okay))]
    #[deftly(thing(x = "x"))]
    a: bool,
}

fn main() {}
