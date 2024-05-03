## A brief tutorial: How to write your own Clone

For the next few sections, for a toy example,
we'll be using `derive_deftly` to define
our own version of `derive(Clone)`.
At first, it will be very simple;
later on, we'll add a few features
that Rust's `derive(Clone)` doesn't have.

> Aside:
>
> We've picked a simple trait to derive on purpose,
> so that we can focus on the features of `derive_deftly`
> without the additional complexity
> of introducing an unfamiliar trait as well.
>
> Please let us know if this approach works for you!
> We're learning how to explain these concepts
> as we go along.
