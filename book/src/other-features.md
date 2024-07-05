# Other features

`derive-deftly` has many more features,
that aren't yet explained in this tutorial.
For example:

 * [`is_enum`, `is_struct`, `is_union`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#is_struct-is_enum-is_union);
   [`v_is_unit`, `is_named`, `is_tuple`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#v_is_unit-v_is_tuple-v_is_named);
   [`fvis`, `tvis`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#fvis-tvis-fdefvis--test-for-public-visibility):
   and [`approx_equal`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#is_empty-approx_equalarg1-arg2--equality-comparison-token-comparison):
   more conditions for dealing with various cases by hand.

 * [`$tdefkwd`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#tdefkwd--keyword-introducing-the-new-data-structure);
   [`$tdeftype`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#ftype-vtype-ttype-tdeftype--types);
   [`$fdefvis`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#fvis-tvis-fdefvis--visibility);
   [`$fdefgens`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#tgens-tgnames-twheres-tdefgens--generics);
   and
   [`$tdefvariants`, `$vdefbody`, `$fdefine`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#tdefvariants-vdefbody-fdefine--tools-for-defining-types)
   for defining a new data structure
   in terms of features of the input data structure,
   and
   [`$Xattrs`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#fattrs--vattrs--tattrs---other-attributes)
   for passing through attributes.

 * [`${select1}`](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html#select1-cond1----else-if-cond2----else-----expect-precisely-one-predicate)
   can help with writing careful templates
   that will reject incoherent inputs.

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
