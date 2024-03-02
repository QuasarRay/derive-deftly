/// Forward/backward compatibility tests
/// See tests/compat/README.md

#[cfg(not(feature = "skip_imported_driver"))]
pub mod adhoc_template {
    use derive_adhoc::derive_adhoc;

    pub trait NumFields {
        fn num_fields() -> usize;
    }

    use pub_a::a_driver::*;

    derive_deftly_adhoc! {
        pub_a::ADriver:

        impl<$tgens> NumFields for $ttype {
            #[allow(clippy::identity_op)]
            fn num_fields() -> usize {
                $( let _: $ftype; )

                0 + ${for fields { 1 }}
            }
        }
    }
}

pub mod b_driver {
    use derive_adhoc::Deftly;

    pub struct BField<T: Default>(T);

    #[derive(Deftly)]
    #[derive_deftly(pub_a::IsEnum)]
    pub enum BDriver<T: Default> {
        Variant(BField<T>),
    }

    use pub_a::derive_deftly_template_IsEnum as derive_deftly_template_ImportedIsEnum;

    #[derive(Deftly)]
    #[derive_deftly(ImportedIsEnum)]
    pub struct BDriver2;
}

#[test]
fn invoke_template() {
    use pub_a::a_trait::IsEnum;
    assert!(b_driver::BDriver::<()>::is_enum().is_some());
}

#[cfg(not(feature = "skip_imported_driver"))]
#[test]
fn invoke_exported_driver() {
    use adhoc_template::NumFields;
    assert_eq!(pub_a::a_driver::ADriver::<()>::num_fields(), 1);
}
