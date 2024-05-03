#### A note on syntax

In this last section,
you've seen a new syntax for the first time.
Both `${paste ident ident..}` and `${snake_case ident}`
are special cases of the following meta-syntax,
which `derive_deftly` uses everywhere:

[`${KEYWORD ARGS.. }`](crate::doc_reference#named-and-positional-template-arguments-to-expansions-and-conditions)

In fact, if you want,
you can use this format
for all of the expansion macros you have already seen:
`$ttype` is just a shortened form for `${ttype}`,
`$fname` is just `${fname}`,
and so on.

Some keywords,
including some of those we've already seen,
can take named arguments.
The syntax for this is:

`${KEYWORD ARGNAME=VALUE ARGNAME=VALUE...}`

> For example, we can use this syntax to give optional arguments to `$vpat`;
> see the template syntax reference for more information.

If you ever need to write a literal `$`
(say, if you want to confuse yourself
by making derive-deftly-generated pattern macros)
you can write [`$$`](crate::doc_reference#dollar-dollar).

