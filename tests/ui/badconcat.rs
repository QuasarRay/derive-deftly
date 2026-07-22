use derive_deftly::{derive_deftly_adhoc, Deftly};

#[derive(Deftly)]
#[derive_deftly_adhoc]
#[deftly(something = "Box<char>")]
struct DataType {
    foo: u8,
    bar: Vec<String>,
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    ${concat forbidden}
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    ${concat ${tmeta(something) as token_stream}}
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    ${define NOT_CONCAT $tgens}
    ${concat $NOT_CONCAT}
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    ${paste ${concat "ok_so_far"}}
}

derive_deftly_adhoc! {
    DataType beta_deftly, dbg:

    const _: &str = ${concat ${dbg { "successful string" }}};
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    ${concat ${dbg { ${error "failure forced in dbg"} }}}
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    // You can't ${define } something containing concat-only expansions
    // without putting the ${concat } in the ${define }.
    //
    // And you can't ${concat } a ${define } unless that ${define }
    // is precisely the ${concat }.
    //
    // In principle we could have ${defconcat } but that would be more
    // confusing, I think.  We have ${defcond } because conditions
    // are a separate namespace.
    ${define BARE_KEBAB { ${kebab_case $tname} }}
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    ${error message=${concat $tname " is not used for span"}}
}

derive_deftly_adhoc! {
    DataType beta_deftly:

    ${define JUST_STRING "just string"}

    const $<bad_ident_ $JUST_STRING>: () = ();
}

fn main() {}
