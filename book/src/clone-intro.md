# A brief tutorial: Writing your own Clone

In the previous chapters,
we've learned how to define a simple `derive-deftly` template,
and how to export it to the rest of the world.

In this chapter, we'll use derive-deftly
to implement our own version of Rust's standard
`#[derive(Clone)]` macro.
At first, it will be very simple;
later on, we'll add a few features
that Rust's `derive(Clone)` doesn't have.

With these examples,
we'll learn more feature of derive-deftly,
including:

* Writing templates that apply to structs and enums.
* Writing templates that apply to generic types.
* Using a variety of expansions, including ones
  for top-level types, variants, and fields.

> Aside:
>
> We've picked a simple trait to derive on purpose,
> so that we can focus on the features of derive-deftly
> without the additional complexity
> of introducing an unfamiliar trait as well.
>
> Please let us know if this approach works for you!
> We're learning how to explain these concepts
> as we go along.

