//! Contexts that might not include a driver

use super::framework::*;

//==================== types ====================

pub type ErrorHandler<'c> = &'c (dyn Fn(Span) -> syn::Error + 'c);

pub type FullContextResult<'s, 'c> = Result<
    &'s Context<'c>, //
    MissingContextError<'c>,
>;

/// Cheap type that embodies a failed requirement for a driver
///
/// This can be pre-obtained and bound to a local `ctx`, and then
/// converted conveniently to `syn::Error` with `ctx?` when actually
/// needed.
#[derive(Clone, Copy)]
pub struct MissingContextError<'c> {
    span: Span,
    error_handler: ErrorHandler<'c>,
}

/// Probably a `Context`, but might not be - there might be no driver
///
/// Definitely contains a `DefinitionsContext`.
pub trait GeneralContext<'c>:
    Clone + AsRef<DefinitionsContext<'c>> + AsMut<DefinitionsContext<'c>>
{
    /// This very type, but with a different lifetime
    ///
    /// A couple of places want to make a version of the context but
    /// with a shorter lifetime - but they only have the impl trait,
    /// not a concrete type.
    ///
    /// There isn't a way to declare in the trait definition that the
    /// type is covariant in its lifetime.  (It's hard even to *talk*
    /// about the implementing Context type's own lifetime.)
    ///
    /// Instead, we provide this type: it implements the same trait,
    /// but maybe with a shorter lifetime.  `reduce_lifetime` returns it.
    ///
    /// Implementations on `T<'a>` should make this be `T<'l>`.
    type Self_<'l>: GeneralContext<'l> + Copy
    where
        'c: 'l;

    fn reduce_lifetime<'s, 'r>(&'s self) -> &'s Self::Self_<'r>
    where
        'c: 'r;

    fn full_ctx_raw(&self) -> Result<&Context<'c>, ErrorHandler<'c>>;

    /// Driver `ErrorLoc` - if there is a driver
    ///
    /// Like `Context::error_loc` but can return `None`.
    ///
    fn error_loc_general(&self) -> Option<ErrorLoc<'c>>;

    fn display_for_dbg(&self) -> impl Display + '_;

    // ----- Provided -----

    /// Obtain a full `Context`, in situations where a driver is needed
    fn full_ctx(&self, span: Span) -> FullContextResult<'_, 'c> {
        self.full_ctx_raw()
            .map_err(move |error_handler| MissingContextError {
                span,
                error_handler,
            })
    }

    /// Convenience method.
    ///
    /// We could use `.as_ref()` but that is less clear (and risks
    /// future inference failures).
    ///
    /// We don't bother with `defs_mut`.
    fn defs(&self) -> &DefinitionsContext<'c> {
        self.as_ref()
    }

    fn prepend_error_loc(
        &self,
        rest: &'_ [ErrorLoc<'c>],
    ) -> Vec<ErrorLoc<'c>> {
        chain!(
            self.error_loc_general(), //
            rest.iter().copied(),
        )
        .collect_vec()
    }

    // This is here so that it can be a trait method on GeneralContext.
    // Otherwise it would have to be a free function or in an extension trait.
    fn find_definition<B>(
        &'c self,
        call: &'c DefinitionName,
    ) -> syn::Result<Option<(&'c Definition<B>, impl GeneralContext<'c>)>>
    where
        Definitions<'c>: AsRef<[&'c Definition<B>]>,
        B: 'static,
    {
        let def = match self.defs().defs.find_raw(call) {
            Some(y) => y,
            None => return Ok(None),
        };

        let defs = self.defs().find_definition_deeper(&def.name, call)?;
        let mut self_ = self.clone();
        *self_.as_mut() = defs;
        Ok(Some((def, self_)))
    }
}

impl<'c> AsRef<DefinitionsContext<'c>> for Context<'c> {
    fn as_ref(&self) -> &DefinitionsContext<'c> {
        &self.defs
    }
}

impl<'c> AsMut<DefinitionsContext<'c>> for Context<'c> {
    fn as_mut(&mut self) -> &mut DefinitionsContext<'c> {
        &mut self.defs
    }
}

impl<'c> GeneralContext<'c> for Context<'c> {
    type Self_<'l>
        = Context<'l>
    where
        'c: 'l;

    fn reduce_lifetime<'s, 'r>(&'s self) -> &'s Self::Self_<'r>
    where
        'c: 'r,
    {
        self
    }

    fn full_ctx_raw(&self) -> Result<&Context<'c>, ErrorHandler<'c>> {
        Ok(self)
    }

    fn error_loc_general(&self) -> Option<ErrorLoc<'c>> {
        Some(self.error_loc())
    }

    fn display_for_dbg(&self) -> impl Display + '_ {
        struct Adapter<'a>(&'a Context<'a>);

        impl Display for Adapter<'_> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let ctx = &self.0;
                write!(f, "for {}", &ctx.top.ident)?;
                if let Some(wv) = &ctx.variant {
                    if let Some(v) = &wv.variant {
                        write!(f, "::{}", &v.ident)?;
                    }
                }
                if let Some(wf) = &ctx.field {
                    // we're just using the Display impl, any span will do
                    let span = Span::call_site();
                    write!(f, ".{}", &wf.fname(span))?;
                }
                if let Some(templ) = &ctx.template_name {
                    let templ = templ.to_token_stream().to_string();
                    write!(f, " from {}", templ)?;
                }
                Ok::<_, fmt::Error>(())
            }
        }

        Adapter(self)
    }
}

impl<'c> Context<'c> {
    pub fn as_general(&'c self) -> &'c Context<'c> {
        self
    }
}

impl<'c> From<MissingContextError<'c>> for syn::Error {
    fn from(e: MissingContextError<'c>) -> syn::Error {
        (e.error_handler)(e.span)
    }
}
