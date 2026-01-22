use crate::{
    Result,
    config::{CfgSource, ConfigSource, IsConfig},
};
use lifecycle::{Booting, Living, Outliving, Slot, mono};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

mono!(Ioc);

#[derive(Debug)]
pub struct InitCtx {
    booting: Booting<Ioc>,
    config: CfgSource,
}

impl InitCtx {
    pub fn new(ioc: Ioc, config: CfgSource) -> Self {
        Self {
            booting: Booting::new(ioc),
            config,
        }
    }

    pub fn complete(self) -> Living<Ioc> {
        unsafe { Living::assume_booted(self.booting) }
    }
}

impl ConfigSource for InitCtx {
    fn get_config<T: IsConfig>(&self, key: impl AsRef<str>) -> Result<T> {
        self.config.get_config(key)
    }
    fn get_config_or<T: IsConfig>(&self, key: impl AsRef<str>, default: T) -> Result<T> {
        self.config.get_config_or(key, default)
    }
}

impl Deref for InitCtx {
    type Target = Booting<Ioc>;

    fn deref(&self) -> &Self::Target {
        &self.booting
    }
}

impl DerefMut for InitCtx {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.booting
    }
}

/// A witness that a type `T` has been initialized during the [`InitPhase`].
/// Can not be constructed directly.
#[derive(Debug)]
pub struct Inited<T>(PhantomData<T>);

impl<T> Clone for Inited<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Inited(PhantomData)
    }
}

impl<K> Inited<K>
where
    K: Init,
{
    pub fn get<'a>(&self, living: &'a Living<Ioc>) -> &'a K::Bean {
        K::PLACE.deref(living)
    }

    pub fn get_mut<'a>(&self, living: &'a mut Living<Ioc>) -> &'a mut K::Bean {
        K::PLACE.deref_mut(living)
    }

    /// Deinitializes the bean during the [`DropPhase`].
    /// # Safety
    /// The caller must ensure calling this method only once for type [K] (This means you can only call once this method even if InitWitness<[K]> may have many instance).
    /// # tips
    /// InitWitness mean it has been initialized, so there is no free uninitialized problem.
    pub unsafe fn drop_in_place(self, outliving: &mut Outliving<Ioc>) {
        unsafe { K::PLACE.drop_in_place(outliving) }
    }
}

/// A trait for types that can be initialized during the [`InitPhase`].
/// Implementors must provide a static storage location for the type,
/// as well as a method to construct an instance of the type.
/// # Safety
/// Duplicate initialization of the same place is safe, but may cause memory leak (In rust leak is not memory unsafe),
/// but you better not do that.
/// When init called we get InitWitness, which can be cloned and spread around freely,
/// but to access the bean inside, we also need the [`token`](ActivePhase) instance, which is only create from InitPhase,
/// so the lifetime of the bean is tied to the InitPhase/ActivePhase, which is safe.
pub trait Init {
    const PLACE: &'static Slot<Self::Bean, Ioc>;
    type Bean: 'static + Sized;

    #[inline(always)]
    fn init(ctx: &mut InitCtx) -> Result<Inited<Self>>
    where
        Self: Sized,
    {
        let bean = Self::construct(ctx)?;

        Self::PLACE.uninit(&mut ctx.booting).write(bean);

        // SAFETY: Although we may have multiple InitWitness created for same type,
        // but to access the bean, we also need &Token (for shared access) or &mut Token (for mutable access)
        // so the lifetime of the bean is same as the Token (its mean safe),
        // and the InitWitness can be cloned freely and spread around.
        Ok(Inited(PhantomData))
    }

    fn construct(ctx: &mut InitCtx) -> Result<Self::Bean>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestStruct(u32, String, &'static str);

    impl Drop for TestStruct {
        fn drop(&mut self) {
            println!("Dropping TestStruct({}, {}, {})", self.0, self.1, self.2);
        }
    }
    static STORAGE: Slot<TestStruct, Ioc> = Slot::new();

    impl Init for TestStruct {
        const PLACE: &'static Slot<TestStruct, Ioc> = &STORAGE;

        type Bean = TestStruct;

        fn construct(_: &mut InitCtx) -> Result<Self::Bean> {
            Ok(TestStruct(42, "Hello".to_string(), "world"))
        }
    }

    #[test]
    fn lifecycle_management() {
        let ioc = Ioc::new().expect("should create ioc");

        let mut init_ctx = InitCtx::new(
            ioc,
            CfgSource::new(Default::default()).expect("should create config source"),
        );

        // Initialize during init phase and get InitWitness
        let inited = TestStruct::init(&mut init_ctx).unwrap();

        // Transition to active phase
        let mut active_phase = init_ctx.complete();

        // Access during active phase
        let shared_ref: &TestStruct = inited.get(&active_phase);
        assert_eq!(shared_ref.1, "Hello");

        let exclusive_ref: &mut TestStruct = inited.get_mut(&mut active_phase);
        exclusive_ref.0 += 1;
        exclusive_ref.1.push_str(", universe!");

        let mut drop_phase = Outliving::outlive(active_phase);

        // Transition to drop phase and deinitialize
        unsafe {
            inited.drop_in_place(&mut drop_phase);
        }

        assert!(Ioc::new().is_err());
    }
}
