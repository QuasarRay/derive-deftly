# A first example: derive_deftly(HelloWorld)

Here we'll make an example project
with a simple `derive-deftly` template.
Our template won't do much,
but it will get us started using the system.

To begin with, we'll create a new project:

> ```shell
> $ cargo init dd-hello-world
> $ cd dd-hello-world
> ```

Next, we'll add the `derive-deftly` crate as a new dependency:

```shell,ignore
$ cargo add derive-deftly
```

> Note: You could also add the dependency
> by adding a line like this to the `[dependencies]` section
> in your `Cargo.toml`:
>
> ```toml
> derive-deftly = "0.10"
> ```
>
> (You should replace `0.10` with the real latest version.)


Now that we've set up our crate,
we'll write our code.

We replace our `src/main.rs` code with the following:

```rust
use derive_deftly::{Deftly,define_derive_deftly};

define_derive_deftly!{
    HelloWorld =
    impl $tname {
        fn greet() {
            println!("Hello from {}!", stringify!($tname));
        }
    }
}

#[derive(Deftly)]
#[derive_deftly(HelloWorld)]
struct MyStruct {}

#[derive(Deftly)]
#[derive_deftly(HelloWorld)]
struct OtherStruct {}

fn main() {
    MyStruct::greet();
    OtherStruct::greet();
}
```

Now we can run the code with:

```shell
$ cargo run
```

If everything works, we'll expect to see the following output:

```text
Hello from MyStruct!
Hello from OtherStruct!
```

## Understanding what we did

Let's look more closely at the code we wrote above, piece by piece.

```rust
use derive_deftly::{Deftly,define_derive_deftly};
```

Here, we're importing the pieces of `derive_deftly` we need
into our namespace.

```rust,ignore
define_derive_deftly!{
    HelloWorld =
       ...
}
```

This part defines a new _template_, called `HelloWorld`.
A template defines a block of code that will later be expanded
when we _apply_ that template to one of our types.

```rust,ignore
    impl $tname {
        fn greet() {
            println!("Hello from {}!", stringify!($tname));
        }
    }
```

This is the _body_ of our template.
When `derive-deftly` applies our template to our type,
it will copy out the elements of this template,
and _expand_ any elements that start with the `$` syntax.

The `$tname` ("type name") expansion
will turn into the name of the type
to which we're applying our template.
For example, when we apply the `HelloWorld` template
to `MyStruct`, `$tname` will become `MyStruct`.

We use the built-in [`stringify!`] macro
to convert the second instance of `$tname`
into a string, so that we can pass it to `println!`.

```rust,ignore
#[derive(Deftly)]
#[derive_deftly(HelloWorld)]
struct MyStruct {}

#[derive(Deftly)]
#[derive_deftly(HelloWorld)]
struct OtherStruct {}
```

Here we declare a pair of structs,
and apply our `HelloWorld` template to each.

We say `#[derive(Defly)]` to invoke `derive-deftly` on our struct,
and `#[derive_deftly(HelloWorld)]` to tell it which template to apply.

After `derive-deftly` has run, this code will expand do something like:

```rust
struct MyStruct {}

impl MyStruct {
    fn greet() {
        println!("Hello from {}!", stringify!(MyStruct));
    }
}

struct OtherStruct {}

impl OtherStruct {
    fn greet() {
        println!("Hello from {}!", stringify!(OtherStruct));
    }
}
```

Finally, we invoke the new `greet()` methods,
with a nice simple `main()`:

```rust,ignore
fn main() {
    MyStruct::greet();
    OtherStruct::greet();
}
```

## Limitations of our macro

Our template won't work for every type!
For example, suppose that we try to apply it to a generic type:

```rust,ignore
#[derive(Deftly)]
#[derive_deftly(HelloWorld)]
struct Pair<T> {
    first: T,
    second: T,
}
```

When we do this, `derive-deftly` will expand our template
to something like this:

```rust,ignore
impl Pair {
    fn greet() {
        println!("Hello from {}!", stringify!(Pair));
    }
}
```

But that won't work in Rust.
Instead, we'll need to say something more like this:
```rust,ignore
impl<T> Pair<T> {
    ...
}
```

We'll talk about how to make templates work for types like this
in later chapters.

## What's next?

In coming chapters,
we'll look over a more complex example,
and learn more features of `derive-deftly`.
We'll write a mode useful template,
and have it apply to more types.


[`stringify!`]: (https://doc.rust-lang.org/std/macro.stringify.html)

