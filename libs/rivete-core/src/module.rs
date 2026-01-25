use crate::boot::Ioc;
use derive_more::{Display, Error};
use exn::{ResultExt, bail};
use lifecycle::{Booting, Outliving};
use std::{any::TypeId, collections::HashSet, hash::Hash};

pub type Result<T> = exn::Result<T, ModuleError>;

#[derive(Debug, Copy, Clone, Eq)]
pub struct ModuleInfo {
    type_id: TypeId,
    name: &'static str,
}

impl Hash for ModuleInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.type_id.hash(state);
    }
}

impl PartialEq<Self> for ModuleInfo {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

#[derive(Debug)]
pub struct ModuleCtx {
    booted_modules: Vec<ModuleInfo>,
    booted_module_type_ids: HashSet<ModuleInfo>,
}

impl ModuleCtx {
    pub fn is_module_booted(&self, module: &ModuleInfo) -> bool {
        self.booted_module_type_ids.contains(module)
    }

    pub fn mark_module_as_booted(&mut self, module: ModuleInfo) {
        self.booted_modules.push(module);
        self.booted_module_type_ids.insert(module);
    }
}

pub struct BootingCtx<'a> {
    ctx: ModuleCtx,
    booting_chain: Vec<ModuleInfo>,
    booting_token: &'a mut Booting<Ioc>,
}

#[derive(Debug, Display, Error)]
pub enum ModuleError {
    #[display("Cyclic module dependency detected: {:?} -> {:?}", _0, _1)]
    CyclicDependency(#[error(not(source))] Vec<ModuleInfo>, ModuleInfo),

    #[display("Module booting error: {_0}")]
    BootingError(#[error(not(source))] String),
}

pub trait Module
where
    Self: Sized + 'static,
{
    fn info() -> ModuleInfo {
        ModuleInfo {
            type_id: TypeId::of::<Self>(),
            name: std::any::type_name::<Self>(),
        }
    }

    fn boot_dependencies(ctx: &mut BootingCtx) -> Result<()>;

    fn boot_self(ctx: &mut Booting<Ioc>) -> Result<()>;

    fn boot(ctx: &mut BootingCtx) -> Result<()> {
        let self_info = Self::info();
        if ctx.ctx.is_module_booted(&self_info) {
            Ok(())
        } else if ctx.booting_chain.contains(&self_info) {
            bail!(ModuleError::CyclicDependency(
                ctx.booting_chain.clone(),
                self_info
            ))
        } else {
            ctx.booting_chain.push(self_info);
            Self::boot_self(ctx.booting_token)
                .or_raise(|| ModuleError::BootingError("Boot self error".to_string()))?;
            Ok(ctx.ctx.mark_module_as_booted(self_info))
        }
    }

    unsafe fn destroy(phase: &mut Outliving<Ioc>);
}
