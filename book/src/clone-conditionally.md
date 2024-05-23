# Making MyClone apply conditionally 

Now, for the first time, we will make MyClone do something
that Rust's `#[derive(Clone)]` does not:
it will apply only when the fields of a struct are `Clone`.

For example, suppose have a struct like this:
```
# use std::sync::Arc;
struct Indirect<T>(Arc<T>, u16);
```
If you try to derive `Clone` on it,
the compiler will generate code something like this:

```rust,ignore
impl<T: Clone> Clone for Indirect<T> { ... }
```

But that `T: Clone` constraint isn't necessary: `Arc<T>` always
implements `Clone`, so your struct could be clone unconditionally.

But using `derive_deftly`,
you can define a template
that derives `Clone` only for the cases
where the _actual_ required constraints are met:

```
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
    MyClone =

    impl<$tgens> Clone for $ttype
    where $twheres
          $( $ftype : Clone , )
    {
        fn clone(&self) -> Self {
            Self {
                $( $fname : self.$fname.clone() , )
            }
        }
    }
}
```

Here, we are using
[`$ftype`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#ftype-vtype-ttype-tdeftype--types),
the type of a field.
Since we're repeating it with `$( ... )`,
we are requiring every field to be `Clone`.

Will this work with non-generic fields,
or if the same field is used more than once?
Once again, yes!
To Rust, this is a perfectly valid example:

```rust,ignore
impl<T> Clone for Direct
where
    T: Clone,
    T: Clone,
    String: Clone
{
    ...
}
```


> This time,
> `derive_deftly` has exactly _one_ piece cleverness at work.
> It makes sure that either `$twheres` is empty,
> or that it ends with a comma.
> That way, your template won't expand to something like
> `where U: Debug + Clone T: Clone`
> (which is a syntax eror)
> or soemthing like
> `where ,`
> (which is also a syntax error).

