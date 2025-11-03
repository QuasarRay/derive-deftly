//! Contexts that might not include a driver

use super::framework::*;

use GeneralContext as GC;
use GeneralContextBuf as GCB;

macro_rules! GeneralContext { { $($life:tt)* } => {
    GeneralContext<$($life)*>
} }


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
#[derive(Clone, Copy)]
pub enum GeneralContext<'c> {
    Full(&'c Context<'c>),
    NoDriver(&'c DriverlessContext<'c>),
}

/// Probably a `Context`, but might not be - there might be no driver
///
/// Definitely contains a `DefinitionsContext`.
#[derive(Clone, Copy)]
pub enum GeneralContextBuf<'c> {
    Full(Context<'c>),
    NoDriver(DriverlessContext<'c>),
}

#[derive(Clone, Copy)]
pub struct DriverlessContext<'c> {
    defs: DefinitionsContext<'c>,
    driver_needed: ErrorHandler<'c>,
}
 
//==================== methods on GeneralContext ====================

impl<'c> GeneralContext<'c> {
    pub fn full_ctx_raw(&self) -> Result<&Context<'c>, ErrorHandler<'c>> {
        match self {
            GC::Full(ctx) => Ok(ctx),
            GC::NoDriver(dc) => Err(dc.driver_needed),
        }
    }

    /// Obtain a full `Context`, in situations where a driver is needed
    pub fn full_ctx(&self, span: Span) -> FullContextResult<'_, 'c> {
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
    pub fn defs(&self) -> &DefinitionsContext<'c> {
        self.as_ref()
    }

    // XXXX rename to error_loc, now it's not coexisting with an inherent
    pub fn error_loc_general(&self) -> Option<ErrorLoc<'c>> {
        match self {
            GC::Full(ctx) => Some(ctx.error_loc()),
            GC::NoDriver { .. } => None,
        }
    }

    pub fn prepend_error_loc(
        &self,
        rest: &'_ [ErrorLoc<'c>],
    ) -> Vec<ErrorLoc<'c>> {
        chain!(
            self.error_loc_general(), //
            rest.iter().copied(),
        )
        .collect_vec()
    }

    pub fn clone_buf(&self) -> GeneralContextBuf<'c> {
        match self {
            GC::Full(ctx) => GCB::Full((*ctx).clone()),
            GC::NoDriver(dc) => GCB::NoDriver((*dc).clone()),
        }
    }
}

impl<'c> GeneralContext<'c> {
    // This is here so that it can be a trait method on GeneralContext.
    // Otherwise it would have to be a free function or in an extension trait.
    // XXXX ^ no longer true, put it back
    pub fn find_definition<B>(
        &'c self,
        call: &'c DefinitionName,
    ) -> syn::Result<Option<(&'c Definition<B>, GeneralContextBuf<'c>)>>
    where
        Definitions<'c>: AsRef<[&'c Definition<B>]>,
        B: 'static,
    {
        let def = match self.defs().defs.find_raw(call) {
            Some(y) => y,
            None => return Ok(None),
        };

        let defs = self.defs().find_definition_deeper(&def.name, call)?;
        let mut self_ = self.clone_buf();
        *self_.as_mut() = defs;
        Ok(Some((def, self_)))
    }
}

impl<'c> AsRef<DefinitionsContext<'c>> for GeneralContext<'c> {
    fn as_ref(&self) -> &DefinitionsContext<'c> {
        match self {
            GC::Full(ctx) => &ctx.defs,
            GC::NoDriver(dc) => &dc.defs,
        }
    }
}

impl<'c> AsMut<DefinitionsContext<'c>> for GeneralContextBuf<'c> {
    fn as_mut(&mut self) -> &mut DefinitionsContext<'c> {
        match self {
            GCB::Full(ctx) => &mut ctx.defs,
            GCB::NoDriver(dc) => &mut  dc.defs,
        }
    }
}

//==================== display_for_dbg ====================

struct ContextDbgDisplayAdapter<'a>(Option<&'a Context<'a>>);

impl Display for ContextDbgDisplayAdapter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Some(ctx) = self.0 else {
            return write!(f, "[no driver]");
        };

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

impl<'c> Context<'c> {
    pub fn display_for_dbg(&self) -> impl Display + '_ {
        ContextDbgDisplayAdapter(Some(self))
    }
}

impl<'c> GeneralContext<'c> {
    pub fn display_for_dbg(&self) -> impl Display + '_ {
        ContextDbgDisplayAdapter(match &self {
            GC::Full(ctx) => Some(ctx),
            GC::NoDriver { .. } => None,
        })
    }
}

//==================== conversions to/from Context ====================

impl<'c> Context<'c> {
    pub fn as_general(&'c self) -> GeneralContext<'c> {
        GC::Full(self)
    }
}

impl<'c> GeneralContextBuf<'c> {
    pub fn as_ref(&'c self) -> GeneralContext<'c> {
        match self {
            GCB::Full(ctx) => GC::Full(ctx),
            GCB::NoDriver(dc) => GC::NoDriver(dc),
        }
    }
}

//==================== remaining impl(s) ====================

impl<'c> From<MissingContextError<'c>> for syn::Error {
    fn from(e: MissingContextError<'c>) -> syn::Error {
        (e.error_handler)(e.span)
    }
}
