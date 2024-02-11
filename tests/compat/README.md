# Introduction

These three crates test forward/backward compatibility with the
version of derive-adhoc prior at the  most recent flag day.

## Tests

I. Forward compatibility (higher layer crate is more up to date)

  new-b -> new derive_adhoc (bizarre version)
    |
    V
  old-a -> old derive_adhoc

This reuses pub-b.rs (which expects bizarre derive-adhoc)
and pub-a.rs.

2. Backward compatibility (lower layer crate is more up to date)

  old-b -> old derive_adhoc
    |
    V
  pub-a -> current derive_adhoc

This reuses pub-a.rs (which expects vanilla derive-adhoc)
and a filtered version of pub-b.rs called old-b.rs.
pub-b.rs expects bizarre derive-adhoc but there is no published
old version, so we have maint/update-bizarre filter out the _bizarres.

## Two sets of these tests

There are `new-b`, `old-a` and `old-b`.
These test compatibility of *template exports* only.

There are also `edrv-new-b`, `edrv-old-a` and `edrv-old-b`.
These test compatibility of exporting driver structs.
