use derive_deftly::{derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(something = "Box<char>")]
struct DataType {
    foo: u8,
    bar: Vec<String>,
}

derive_deftly_adhoc! {
    DataType:

    ${for fields {
        ${paste $ttype _ $ftype}
    }}
}

derive_deftly_adhoc! {
    DataType:

    struct ${paste $tname _ 42};
}

derive_deftly_adhoc! {
    DataType:

    struct ${paste $ttype ${tmeta(something) as ty}};
}

derive_deftly_adhoc! {
    DataType:

    // This expands to "r#struct Broken { }"
    ${paste tdefkwed} Broken { }
}

derive_deftly_adhoc! {
    DataType:

    // This attribute isn't actually provided, but that doesn't matter
    // for this test since it fails before even looking for it.
    ${define VIA_DEFINE ${tmeta(t_frag) as ident}}

    struct $<ViaDefine $VIA_DEFINE>;
}

derive_deftly_adhoc! {
    DataType beta_deftly:
    ${kebab_case $tname}
}
derive_deftly_adhoc! {
    DataType beta_deftly:
    $<${kebab_case $tname}>
}

derive_deftly_adhoc! {
    DataType beta_deftly:
    { $< ${define OK {<}} > }
    { $< ${define OK {>}} > }
    { $< }
    { $< < }
    ${paste < }
    ${paste > }
}

fn main() {}
