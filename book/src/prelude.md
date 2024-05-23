# Introduction to derive-deftly

`derive-deftly` allows you to write `#[derive]` macros
-- macros driven by Rust data structures -
by writing templates in a fairly straightforward template language.

This is the friendly introduction.
Of course there is a [reference manual](https://docs.rs/derive-deftly/latest/derive_deftly/doc_reference/index.html) too.

<!--##toc##-->
   * [Getting started with `derive_deftly`.](basic-template.md)
      * [Templates inside modules](templates-in-modules.md)
      * [Exporting templates](exporting-templates.md)
      * [If you're only deriving once...](adhoc.md)
   * [A brief tutorial: How to write your own Clone](clone-intro.md)
      * [Simple templates: fields and repetition](clone-fields.md)
      * [Making MyClone apply to generics](clone-generics.md)
      * [Making MyClone apply conditionally](clone-conditionally.md)
      * [Deriving for enumerations](clone-enums.md)
   * [Some more advanced topics](advanced-intro.md)
      * [Transforming names and strings](paste-and-case.md)
   * [Another example: Defining a constructor function.](constructor-intro.md)
      * [Marking a template's limitations](constructor-limitations.md)
      * [Debugging templates](constructor-debugging.md)
      * [Working with visibility](constructor-visibility.md)
      * [Using attributes to make a template take arguments](constructor-attrs.md)
      * [Getting started with conditionals](constructor-conditionals.md)
      * [More complicated conditionals](constructor-advanced-conditionals.md)
   * [Other features](other-features.md)

