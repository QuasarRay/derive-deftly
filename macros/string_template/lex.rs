use super::*;

#[derive(Debug)]
pub struct LexingContext<'s> {
    span: Span,
    whole: &'s str,
    i: usize,
}

#[derive(Debug)]
pub struct HadDollar<'l, 's> {
    pub lc: &'l mut LexingContext<'s>,
    dollar_i: usize,
}

impl<'s> LexingContext<'s> {
    pub fn new(span: Span, whole: &'s str) -> Self {
        LexingContext {
            span,
            whole: &whole,
            i: 0,
        }
    }

    fn rhs(&self) -> &'s str {
        &self.whole[self.i..]
    }

    pub fn peek_char(&self) -> Option<char> {
        self.rhs().chars().next()
    }

    pub fn next_char(&mut self) -> Option<char> {
        let mut chars = self.rhs().chars();
        let c = chars.next()?;
        self.i = self.whole.len() - chars.as_str().len();
        Some(c)
    }

    /// Returns the part before the dollar, and advances to just after it
    pub fn find_dollar(&mut self) -> (&'s str, Option<HadDollar<'_, 's>>) {
        let l_end_i;
        let new_i;
        let dollar_i;
        match self.rhs().find('$') {
            Some(dollar_offset) => {
                l_end_i = self.i + dollar_offset;
                new_i = l_end_i + 1;
                dollar_i = Some(l_end_i);
            }
            None => {
                l_end_i = self.whole.len();
                new_i = l_end_i;
                dollar_i = None;
            }
        }
        let l = &self.whole[self.i..l_end_i];
        self.i = new_i;
        let had_dollar =
            dollar_i.map(|dollar_i| HadDollar { lc: self, dollar_i });
        (l, had_dollar)
    }
}

impl<'l, 's> HadDollar<'l, 's> {
    pub fn chunk_after_dollar(&self) -> &'s str {
        &self.lc.whole[self.dollar_i + 1..self.lc.i]
    }

    pub fn error(&self, m: impl Display) -> syn::Error {
        self.lc.span.error(format_args!(
            "{}; in string template, error in fragment {:?}, near this location {:?}<-HERE",
            m,
            &self.lc.whole[self.dollar_i .. self.lc.i],
            &self.lc.whole[0 .. self.lc.i],
        ))
    }
    pub fn map_syn_err(&self, e: syn::Error) -> syn::Error {
        self.error(e)
    }
}
