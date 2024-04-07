//! Test cases for conditions
//!
//! This gives a basic demonstration of how to handle an enum.

use derive_deftly::{define_derive_deftly, derive_deftly_adhoc, Deftly};

trait Trait {
    fn shape_top(&self) -> &'static str;
    fn shape_fields(&self) -> &'static str;
    fn has_tmeta(&self) -> bool;
    fn has_vmeta(&self) -> bool;
    fn has_tgens(&self) -> bool;
}

define_derive_deftly! {
    Trait =

    impl<$tgens> Trait for $ttype {
        fn shape_top(&self) -> &'static str {
            ${select1
              is_struct { "struct" }
              is_enum { "enum" }
              is_union { "union" }
            }
        }
        fn shape_fields(&self) -> &'static str {
            #[allow(unused_unsafe)]
            unsafe {
                match self { $(
                    #[allow(unused_variables)]
                    $vpat => ${select1
                               v_is_unit { "unit" }
                               v_is_tuple { "tuple" }
                               v_is_named { "named" }
                    },
                ) }
            }
        }
        fn has_tgens(&self) -> bool {
            ${if tgens
              { true }
              else { false }}
        }
        fn has_tmeta(&self) -> bool {
            ${if tmeta(hi(ferris))
              { true }
              else { false }}
        }
        fn has_vmeta(&self) -> bool {
            #[allow(unused_unsafe)]
            unsafe {
                match self { $(
                    #[allow(unused_variables)]
                    // TODO maybe ${bool COND} for ${if ... true ... false} ?
                    $vpat => ${if vmeta(hello(there))
                               { true }
                               else { false }},
                ) }
            }
        }
    }
}

trait GetUsize {
    fn get_usize(&self) -> Option<usize>;
}

define_derive_deftly! {
    GetUsize =

    impl GetUsize for $ttype {
        fn get_usize(&self) -> Option<usize> {
            match self { $(
                #[allow(unused_variables)]
                $vpat => { $(
                    ${when approx_equal($ftype, usize)}
                    return Some(*$fpatname);
                ) }
            ) };
            #[allow(unreachable_code)]
            None
        }
    }
}

#[derive(Deftly)]
#[derive_deftly(Trait, GetUsize)]
#[derive_deftly_adhoc]
struct Unit;

#[derive(Deftly)]
#[derive_deftly(Trait, GetUsize)]
#[deftly(hi(ferris))]
struct Tuple(usize);

#[derive(Deftly)]
#[derive_deftly(Trait, GetUsize)]
#[derive_deftly_adhoc] // allow this, which is matched only as bool
#[deftly(hello(there = 42))]
struct Struct {
    field: usize,
}

#[derive(Deftly)]
#[derive_deftly(Trait, GetUsize)]
#[derive_deftly_adhoc] // allow hello(there(nner)) which is never matched
enum Enum {
    #[deftly(hello(there))]
    Unit,
    #[deftly(hello(there(inner)))]
    Tuple(usize),
    Named {
        field: u32,
    },
}

#[derive(Deftly)]
#[derive_deftly(Trait)]
union Union {
    field: usize,
}

#[derive(Deftly)]
#[derive_deftly(Trait)]
struct Generic<T>(T);

derive_deftly_adhoc! {
    Unit:

    fn static_test() {
        // bad is an error
        ${if false             { bad }}
        ${if true              {} else { bad }}
        ${if not(false)        {} else { bad }}
        ${if not(true)         { bad }}
        ${if any()             { bad }}
        ${if any(false)        { bad }}
        ${if any(true)         {} else { bad }}
        ${if any(true,false)   {} else { bad }}
        ${if any(false,true)   {} else { bad }}
        ${if all()             {} else { bad }}
        ${if all(false)        { bad }}
        ${if all(true)         {} else { bad }}
        ${if all(true,false)   { bad }}
        ${if all(false,true)   { bad }}
    }
}

fn test(top: &str, tgens: bool, fields: &str, tmeta: bool, vmeta: bool, v: impl Trait) {
    if !(v.shape_top() == top
        && v.shape_fields() == fields
        && v.has_tgens() == tgens
        && v.has_tmeta() == tmeta
        && v.has_vmeta() == vmeta)
    {
        panic!()
    }
}

fn test_get_usize(some_usize: Option<usize>, v: impl GetUsize) {
    if !(v.get_usize() == some_usize) {
        panic!()
    }
}

fn main() {
    static_test();

    test("struct", false, "unit", false, false, Unit);
    test("struct", false, "tuple", true, false, Tuple(0));
    test("struct", false, "named", false, true, Struct { field: 0 });
    test("enum", false, "unit", false, true, Enum::Unit);
    test("enum", false, "tuple", false, true, Enum::Tuple(0));
    test("enum", false, "named", false, false, Enum::Named { field: 0 });
    test("union", false, "named", false, false, Union { field: 0 });
    test("struct", true, "tuple", false, false, Generic(""));

    test_get_usize(None, Unit);
    test_get_usize(Some(0), Tuple(0));
    test_get_usize(Some(0), Struct { field: 0 });
    test_get_usize(None, Enum::Unit);
    test_get_usize(Some(0), Enum::Tuple(0));
    test_get_usize(None, Enum::Named { field: 0u32 });
}
