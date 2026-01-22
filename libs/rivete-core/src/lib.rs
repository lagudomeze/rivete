extern crate core;

use crate::{
    init::{Init, InitCtx, Inited},
};
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

pub mod config;
pub mod error;
pub mod init;

pub type Result<T> = exn::Result<T, error::Error>;

pub trait Alias<Name> {
    type Key;
}

pub trait WithMod<C> {
    fn get(ctx: &C) -> &Self;
}

pub trait WithInited<C> {
    fn witness(ctx: &C) -> Inited<Self>
    where
        Self: Sized;
}

pub trait Module {
    unsafe fn build(ctx: &mut InitCtx) -> Result<Self>
    where
        Self: Sized;

    fn init(mut ctx: InitCtx) -> Result<Context<Self>>
    where
        Self: Sized,
    {
        let module = unsafe { Self::build(&mut ctx) }?;
        let phase = ctx.complete();
        Ok(Context {
            inner: ManuallyDrop::new((module, phase)),
        })
    }

    fn drop_in_place(self, phase: &mut DropPhase);
}

pub struct Context<M: Module> {
    inner: ManuallyDrop<(M, ActivePhase)>,
}

impl<M> Context<M>
where
    M: Module,
{
    pub fn get<K>(&self) -> &K::Bean
    where
        K: WithInited<M> + Init,
    {
        let (module, phase) = self.inner.deref();
        <K as WithInited<M>>::witness(module).get(phase)
    }

    pub fn get_mut<K>(&mut self) -> &mut K::Bean
    where
        K: WithInited<M> + Init,
    {
        let (module, phase) = self.inner.deref_mut();
        <K as WithInited<M>>::witness(module).get_mut(phase)
    }
}

impl<M> Drop for Context<M>
where
    M: Module,
{
    fn drop(&mut self) {
        // Safety: We are taking ownership of the phase to drop it properly.
        // the ActivePhase is really once owned here.
        let (module, phase) = unsafe { ManuallyDrop::take(&mut self.inner) };
        let mut phase = phase.into_drop();
        // Safety: Context owned ActivePhase which is global singleton, Context instance is unique here.
        // So dropping phase here is safe.
        module.drop_in_place(&mut phase);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {}
}
