### Marking a template's limitations

The template above doesn't work for enumerations.
If you try to apply it to one, you'll get
a not-entirely-helpful error message.

In earlier examples,
we've shown how to make templates
that apply to enums as well as structs.
But let's say that in this case,
we want our template to be struct-only.

We can tell `derive_deftly` about this restriction,
to help it generate more useful error messages:

```
# use derive_deftly::define_derive_deftly;
define_derive_deftly! {
   Constructor for struct = // (1)

   impl<$tgens> $ttype where $twheres {
      pub fn new( $( $fname: $ftype , ) ) -> Self {
          Self {
              $( $fname , )
          }
      }
   }
}
```

(Note the use of
[`for struct`](crate::doc_reference#for-struct-for-enum-for-union--insist-on-a-particular-driver-kind)
above at `// (1)`.)

Now if we try to apply our template to an enum,
we'll get a more useful error:


```text,ignore
error: template defined for struct, but applied to enum
```

