# Other features

`derive-deftly` has many more features,
that aren't yet explained in this tutorial.
For example:

 * [`fvis`, `tvis`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#fvis-tvis-fdefvis--test-for-public-visibility):
   and [`approx_equal`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#is_empty-approx_equalarg1-arg2--equality-comparison-token-comparison):
   more conditions for dealing with various cases by hand.

 * [`$tdeftype`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#ftype-vtype-ttype-tdeftype--types);
   for defining a new data structure
   in terms of features of the input data structure,
   and
   [`$Xattrs`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#fattrs--vattrs--tattrs---other-attributes)
   for passing through attributes.

Full details are in the [reference],
which also has a brief example demonstrating each construct.

<!--

 TODO:

> ## Links
>
>  - Link to more worked and commented examples.

-->

[reference]: crate::doc_reference
[README]: crate
[`paste`]: https://docs.rs/paste/latest/paste/
