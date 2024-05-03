#### Will MyClone apply to other kinds of struct?

Rust defines several kinds of struct:
structs with fields (`struct Foo {...};`),
tuple structs (`struct Foo(...);`),
and unit structs (`struct Foo;`).

If you try to apply the `MyClone` template above
to a struct with fields,
it will work just fine.
But with a tuple struct, or a unit struct,
you might expect it to fail.

Surprisingly, it will still work fine!
This isn't because of any clever trickery
from `derive_deftly`:
it's just how Rust works.
When you use it on tuple or unit structs,
the `MyClone` template we wrote above will expand
to code like this...
which happens to be valid syntax!

```
struct TupleStruct(String, Option<String>);
impl Clone for TupleStruct {
    fn clone(&self) -> Self {
        Self {
            0: self.0.clone(),
            1: self.1.clone(),
        }
    }
}

struct UnitStruct;
impl Clone for UnitStruct {
    fn clone(&self) -> Self {
        Self {
        }
    }
}
```

This will be a common theme in what follows:
Rust often lets you use a slightly unidiomatic syntax
so that you can handle many different cases
in the same way.
