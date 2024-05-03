##### A further note on reptition

Note that when we define our additional `where` clauses,
we don't have to specify separate of repetition
for variants and fields:
if we just have `$ftype` in a top-level repetition,
`derive_deftly` will iterate over all fields in all variants.

Sometimes, if you do something subtle,
derive-deftly may not be able to figure
out what you're trying to repeat over.
You can use 
[`${for ...}`](crate::doc_reference#repetition-and-nesting)
to specifiy explicitly:
so, above, instead, we could have written
```
# use derive_deftly::{Deftly, derive_deftly_adhoc};
# #[derive(Deftly)]
# #[derive_deftly_adhoc]
# enum TestCase<A> { Variant(A) }
# derive_deftly_adhoc! { TestCase:
#     fn testcase<$tgens>() where
${for fields { $ftype: Clone, }}
#     {}
# }
```

You can also use `${for ...}` rather than `$(...)`
in cases where you feel
it makes your macro code clearer.

