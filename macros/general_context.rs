#![allow(unused)] // XXXX

use super::framework::*;

impl<'c> Context<'c> {
    pub fn defs(&self) -> &DefinitionsContext<'c> {
        &self.defs
    }
}

impl<'c> AsMut<DefinitionsContext<'c>> for Context<'c> {
    fn as_mut(&mut self) -> &mut DefinitionsContext<'c> {
        &mut self.defs
    }
}
