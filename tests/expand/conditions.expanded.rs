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
trait GetUsize {
    fn get_usize(&self) -> Option<usize>;
}
#[derive_deftly(Trait, GetUsize)]
#[derive_deftly_adhoc]
struct Unit;
impl Trait for Unit {
    fn shape_top(&self) -> &'static str {
        "struct"
    }
    fn shape_fields(&self) -> &'static str {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Unit {} => "unit",
            }
        }
    }
    fn has_tgens(&self) -> bool {
        let tgens = false;
        let is_empty = true;
        if tgens != !is_empty {
            ::core::panicking::panic("explicit panic");
        }
        tgens
    }
    fn has_tmeta(&self) -> bool {
        false
    }
    fn has_vmeta(&self) -> bool {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Unit {} => false,
            }
        }
    }
}
impl GetUsize for Unit {
    fn get_usize(&self) -> Option<usize> {
        match self {
            #[allow(unused_variables)]
            Unit {} => {}
        };
        #[allow(unreachable_code)] None
    }
}
#[derive_deftly(Trait, GetUsize)]
#[deftly(hi(ferris))]
struct Tuple(usize);
impl Trait for Tuple {
    fn shape_top(&self) -> &'static str {
        "struct"
    }
    fn shape_fields(&self) -> &'static str {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Tuple { 0: f_0 } => "tuple",
            }
        }
    }
    fn has_tgens(&self) -> bool {
        let tgens = false;
        let is_empty = true;
        if tgens != !is_empty {
            ::core::panicking::panic("explicit panic");
        }
        tgens
    }
    fn has_tmeta(&self) -> bool {
        true
    }
    fn has_vmeta(&self) -> bool {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Tuple { 0: f_0 } => false,
            }
        }
    }
}
impl GetUsize for Tuple {
    fn get_usize(&self) -> Option<usize> {
        match self {
            #[allow(unused_variables)]
            Tuple { 0: f_0 } => {
                return Some(*f_0);
            }
        };
        #[allow(unreachable_code)] None
    }
}
#[derive_deftly(Trait, GetUsize)]
#[derive_deftly_adhoc]
#[deftly(hello(there = 42))]
struct Struct {
    field: usize,
}
impl Trait for Struct {
    fn shape_top(&self) -> &'static str {
        "struct"
    }
    fn shape_fields(&self) -> &'static str {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Struct { field: f_field } => "named",
            }
        }
    }
    fn has_tgens(&self) -> bool {
        let tgens = false;
        let is_empty = true;
        if tgens != !is_empty {
            ::core::panicking::panic("explicit panic");
        }
        tgens
    }
    fn has_tmeta(&self) -> bool {
        false
    }
    fn has_vmeta(&self) -> bool {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Struct { field: f_field } => true,
            }
        }
    }
}
impl GetUsize for Struct {
    fn get_usize(&self) -> Option<usize> {
        match self {
            #[allow(unused_variables)]
            Struct { field: f_field } => {
                return Some(*f_field);
            }
        };
        #[allow(unreachable_code)] None
    }
}
#[derive_deftly(Trait, GetUsize)]
#[derive_deftly_adhoc]
enum Enum {
    #[deftly(hello(there))]
    Unit,
    #[deftly(hello(there(inner)))]
    Tuple(usize),
    Named { field: u32 },
}
impl Trait for Enum {
    fn shape_top(&self) -> &'static str {
        "enum"
    }
    fn shape_fields(&self) -> &'static str {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Enum::Unit {} => "unit",
                #[allow(unused_variables)]
                Enum::Tuple { 0: f_0 } => "tuple",
                #[allow(unused_variables)]
                Enum::Named { field: f_field } => "named",
            }
        }
    }
    fn has_tgens(&self) -> bool {
        let tgens = false;
        let is_empty = true;
        if tgens != !is_empty {
            ::core::panicking::panic("explicit panic");
        }
        tgens
    }
    fn has_tmeta(&self) -> bool {
        false
    }
    fn has_vmeta(&self) -> bool {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Enum::Unit {} => true,
                #[allow(unused_variables)]
                Enum::Tuple { 0: f_0 } => true,
                #[allow(unused_variables)]
                Enum::Named { field: f_field } => false,
            }
        }
    }
}
impl GetUsize for Enum {
    fn get_usize(&self) -> Option<usize> {
        match self {
            #[allow(unused_variables)]
            Enum::Unit {} => {}
            #[allow(unused_variables)]
            Enum::Tuple { 0: f_0 } => {
                return Some(*f_0);
            }
            #[allow(unused_variables)]
            Enum::Named { field: f_field } => {}
        };
        #[allow(unreachable_code)] None
    }
}
#[derive_deftly(Trait)]
union Union {
    field: usize,
}
impl Trait for Union {
    fn shape_top(&self) -> &'static str {
        "union"
    }
    fn shape_fields(&self) -> &'static str {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Union { field: f_field } => "named",
            }
        }
    }
    fn has_tgens(&self) -> bool {
        let tgens = false;
        let is_empty = true;
        if tgens != !is_empty {
            ::core::panicking::panic("explicit panic");
        }
        tgens
    }
    fn has_tmeta(&self) -> bool {
        false
    }
    fn has_vmeta(&self) -> bool {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Union { field: f_field } => false,
            }
        }
    }
}
#[derive_deftly(Trait)]
struct Generic<T>(T);
impl<T> Trait for Generic<T> {
    fn shape_top(&self) -> &'static str {
        "struct"
    }
    fn shape_fields(&self) -> &'static str {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Generic { 0: f_0 } => "tuple",
            }
        }
    }
    fn has_tgens(&self) -> bool {
        let tgens = true;
        let is_empty = false;
        if tgens != !is_empty {
            ::core::panicking::panic("explicit panic");
        }
        tgens
    }
    fn has_tmeta(&self) -> bool {
        false
    }
    fn has_vmeta(&self) -> bool {
        #[allow(unused_unsafe)]
        unsafe {
            match self {
                #[allow(unused_variables)]
                Generic { 0: f_0 } => false,
            }
        }
    }
}
fn static_test_unit() {}
fn static_test_struct() {}
fn test(top: &str, tgens: bool, fields: &str, tmeta: bool, vmeta: bool, v: impl Trait) {
    if !(v.shape_top() == top && v.shape_fields() == fields && v.has_tgens() == tgens
        && v.has_tmeta() == tmeta && v.has_vmeta() == vmeta)
    {
        ::core::panicking::panic("explicit panic")
    }
}
fn test_get_usize(some_usize: Option<usize>, v: impl GetUsize) {
    if !(v.get_usize() == some_usize) {
        ::core::panicking::panic("explicit panic")
    }
}
fn main() {
    static_test_unit();
    static_test_struct();
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
