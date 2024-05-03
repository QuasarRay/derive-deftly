### Transforming names and strings

Often, it's useful to define new identifiers
based on existing ones,
or to convert identifiers into strings.

You _could_ use the existing [`paste`] crate for this,
or you can use a native facility provided by `derive_deftly`.

For example, suppose that you want
to define a template that makes a "discriminant" type
for your enumerations.
You want the new type to be named `FooDiscriminant`,
where `Foo` is the name of your existing type.
While you're at it, you want to add an `is_` function
to detect each variant.

You can do that like this:
```
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
    Discriminant =

    #[derive(Copy,Clone,Eq,PartialEq,Debug)]
    enum ${paste $tname Discriminant} {
        $(
            $vname,
        )
    }

    impl<$tgens> $ttype where $twheres {
       fn discriminant(&self) -> ${paste $tname Discriminant} {
          match self {
              $(
                  $vpat => ${paste $tname Discriminant}::$vname,
              )
          }
        }

        $(
            fn ${paste is_ ${snake_case $vname}} (&self) -> bool {
                self.discriminant() ==
                    ${paste $tname Discriminant} ::$vname
            }
        )
    }
}
```

Here we see a couple of new constructs.

First, we're using
[`${paste}`](crate::doc_reference#-paste---identifier-pasting)
to glue several identifiers together into one.
When we say `${paste $tname Discriminant}`,
we are generating a new identifier from `$tname`
(the type name)
and the word Discriminant.
So if the type name is `Foo`,
the new type will be called `FooDiscriminant`.

Second, we're using
[`${snake_case}`](crate::doc_reference#case-changing)
to transform an identifier into `snake_case`
(that is, lowercase words separated by underscores).
We use this to turn the name of each variant (`$vname`)
into a name suitable for use in a function name.
So if a variant is called `ExampleVariant`,
`${snake_case $vname}`  will be `example_variant`,
and `${paste is_ ${snake_case $vname}}` will be
`is_example_variant`.

There are [other case-changers](crate::doc_reference#case-changing):
  * `${pascal_case my_ident}` becomes `MyIdent`.
    You can also write this as
    `${upper_camel_case ..}`.
  * `${lower_camel_case my_ident}` becomes `myIdent`.
  * `${shouty_snake_case MyIdent}` becomes `MY_IDENT`.
  * `${snake_case MyIdent}` becomes `my_ident`, as you've already seen.

You can abbreviate `${paste ...}` as `$<...>`.

<!--
  TODO: Should we show `$<...>` instead of `${paste ...}`?
  Maybe we should principally advertise `$<...>`, everywhere,
  and de-emphasise `${paste ...}`.
 -->

#### Pasting onto types

Here's a side-note:
You can use `${paste}` to append identifiers to a type
(like `$ttype` or `$ftype`),
not just an identifier.
When you do this, `${paste}` will
[do the right thing](crate::doc_reference#-paste---identifier-pasting)
even if the type is generic.

For example, if `$ftype` is `Vec<u8>`,
then `${paste $ftype Builder}` will expand to
`VecBuilder<u8>`, not `Vec<u8>Builder`.
