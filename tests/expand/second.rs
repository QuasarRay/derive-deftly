//! Example including attribute filtering

use derive_deftly::{derive_deftly_adhoc, Deftly};

#[derive(Deftly, Default)]
#[derive_deftly_adhoc]
/// Some docs
pub struct ChannelsParams {
    /// thing
    ///
    /// paragraph
    #[allow(dead_code)]
    padding_enable: bool,

    #[allow(dead_code)]
    padding_parameters: usize,
}

derive_deftly_adhoc! {
    ChannelsParams:

    #[derive(Debug, Default, Clone, Eq, PartialEq)]
    pub struct ChannelsParamsUpdates {
        $(
            // ${fattrs doc[0]}
            ${fattrs serde}
            ///
            /// New value, if it has changed.
            //
            // ${fattrs doc[+]}
            pub(crate) $fname: Option<$ftype>,
        )
    }
}

derive_deftly_adhoc! {
    ChannelsParams:

    #[allow(dead_code)]
    ${tattrs doc, serde}
    struct ChannelsParamsDupliate {
        $(
            ${fattrs ! serde}
            $fname: $ftype,
        )
    }
}

derive_deftly_adhoc! {
    ChannelsParams:
    type Wombat = $ tname;
}

type K = Wombat;

fn main() {
    let _: K = ChannelsParams::default();

    derive_deftly_adhoc! {
        ChannelsParams:
        $(
            println!("field name {:?}", stringify!($fname));
        )
    }

    let u = ChannelsParamsUpdates::default();
    println!("updates = {:?}", &u);
}
