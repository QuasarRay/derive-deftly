### Using attributes to make a template take arguments

Let's suppose we want to make our `Constructor` template
a little more flexible:
we'd like to be able to give the `new` function a different name.

We could do this as follows:

```
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
   Constructor for struct =

   impl<$tgens> $ttype where $twheres {
      pub fn ${tmeta(newfn)} // (1)
      ( $( $fname: $ftype , ) ) -> Self {
          Self {
              $( $fname , )
          }
      }
   }
}

use derive_deftly::Deftly;
#[derive(Deftly)]
#[derive_deftly(Constructor)]
#[deftly(newfn="construct_example")]
struct Example {
    a: f64,
    b: String
}
```

Here, instead of specifying "new"
for the method name in our template,
we give the name as `${tmeta(newfn)}`.
This tells the template to look for an
[`#[deftly(newfn="...")]`](Deftly#deftly-attribute)
attribute on the type,
and to use the value of that attribute
in place of the keyword.

<span id="meta-attr-scope">If
we expect our template to be widely used,
we should namespace our attributes,
by saying something like
`${tmeta(constructor(newfn = "..."))}`,
in our `Constructor` template.
Then, the template will look for an attribute like
`#[deftly(constructor(newfn = "..."))]`.
This avoids name clashes with other templates.</span>

The
[`$tmeta`](crate::doc_reference#tmeta-vmeta-fmeta--deftly-attributes)
keyword that we used here
tells the template
to look at the `#[deftly]` attributes for the _type_.
We can, instead, use
[`$vmeta`](crate::doc_reference#tmeta-vmeta-fmeta--deftly-attributes)
to look for `#[deftly]` attributes for the current _variant_,
or
[`$fmeta`](crate::doc_reference#tmeta-vmeta-fmeta--deftly-attributes)
to
to look for `#[deftly]` attributes for the current _field_.

 <!--
> TODO: Is this the right way to talk about "as lit" and "as ty"?
> I'm thinking not yet.
 -->
