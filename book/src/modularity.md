# Using templates from other crates and modules

In the previous section,
we learned how to crate our first template.
We used `derive-deftly` to define a template called `HelloWorld`
that could print the name of a type.

Before we go any further,
we'll discuss how to use that template
from other modules within the same crate,
and how to expose that template so that other crates can use it.

We'll also discuss a [convenient way]
to define a template that only needs to be used once.

[convenient way]: ./adhoc.md
