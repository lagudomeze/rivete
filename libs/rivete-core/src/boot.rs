use crate::{
    config::{CfgSource, ConfigSource, IsConfig},
    Result,
};
use lifecycle::{mono, Booting, Living, Outliving, Slot};
use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

mono!(Ioc);

/// Bootstrap context for initializing beans during the booting phase.
///
/// This struct holds the [`Booting<Ioc>`] context and configuration source,
/// allowing beans to be initialized and providing access to configuration.
#[derive(Debug)]
pub struct Bootstrap {
    booting: Booting<Ioc>,
    config: CfgSource,
}

impl Bootstrap {
    /// Creates a new bootstrap context.
    ///
    /// # Arguments
    ///
    /// * `ioc` - The I/O controller instance.
    /// * `config` - Configuration source for beans.
    pub fn new(ioc: Ioc, config: CfgSource) -> Self {
        Self {
            booting: Booting::new(ioc),
            config,
        }
    }

    /// Completes the booting phase and transitions to the active phase.
    ///
    /// This consumes the bootstrap context and returns a [`Living<Ioc>`] instance,
    /// which represents the active runtime phase.
    pub fn complete(self) -> Living<Ioc> {
        unsafe { Living::assume_booted(self.booting) }
    }
}

impl ConfigSource for Bootstrap {
    fn get_config<T: IsConfig>(&self, key: impl AsRef<str>) -> Result<T> {
        self.config.get_config(key)
    }
    fn get_config_or<T: IsConfig>(&self, key: impl AsRef<str>, default: T) -> Result<T> {
        self.config.get_config_or(key, default)
    }
}

impl Deref for Bootstrap {
    type Target = Booting<Ioc>;

    fn deref(&self) -> &Self::Target {
        &self.booting
    }
}

impl DerefMut for Bootstrap {
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
    K: Bean,
{
    pub fn get<'a>(&self, living: &'a Living<Ioc>) -> &'a K::Bean {
        K::PLACE.deref(living)
    }

    pub fn get_mut<'a>(&self, living: &'a mut Living<Ioc>) -> &'a mut K::Bean {
        K::PLACE.deref_mut(living)
    }

    /// Deinitializes the bean during the drop phase.
    ///
    /// # Safety
    ///
    /// The caller must ensure calling this method only once for type `K`
    /// (even though there may be multiple `Inited<K>` instances).
    ///
    /// # Note
    ///
    /// `Inited<K>` guarantees that the bean has been initialized,
    /// so there is no risk of accessing uninitialized memory.
    pub unsafe fn drop_in_place(self, outliving: &mut Outliving<Ioc>) {
        unsafe { K::PLACE.drop_in_place(outliving) }
    }
}

/// A trait for types that can be initialized during the booting phase ([`Booting<Ioc>`]).
///
/// Implementors must provide a static storage location for the type,
/// as well as a method to construct an instance of the type.
///
/// # Safety
///
/// Duplicate initialization of the same storage location is safe but may cause memory leaks
/// (memory leaks are not considered unsafe in Rust). However, it is recommended to avoid
/// duplicate initialization.
///
/// When [`init`](Bean::init) is called, it returns an [`Inited<Self>`] witness that can be
/// cloned and shared freely. However, to access the bean, a [`Living<Ioc>`] instance is
/// required, which is only available after transitioning from the booting phase to the
/// active phase. This ensures the bean's lifetime is tied to the active phase, guaranteeing
/// safety.
pub trait Bean {
    const PLACE: &'static Slot<Self::Bean, Ioc>;
    type Bean: 'static + Sized;

    #[inline(always)]
    fn init(ctx: &mut Bootstrap) -> Result<Inited<Self>>
    where
        Self: Sized,
    {
        let bean = Self::construct(ctx)?;

        Self::PLACE.uninit(&mut ctx.booting).write(bean);

        // SAFETY: Although multiple `Inited<Self>` witnesses may be created for the same type,
        // accessing the bean requires a `&Living<Ioc>` (for shared access) or `&mut Living<Ioc>`
        // (for mutable access). This ties the bean's lifetime to the active phase, ensuring safety.
        // The `Inited<Self>` witness can be cloned and shared freely.
        Ok(Inited(PhantomData))
    }

    /// Constructs an instance of the bean.
    ///
    /// This method is called during initialization to create the bean instance.
    /// The provided [`Bootstrap`] context can be used to access configuration
    /// and other beans that have already been initialized.
    fn construct(ctx: &mut Bootstrap) -> Result<Self::Bean>;
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

    impl Bean for TestStruct {
        const PLACE: &'static Slot<TestStruct, Ioc> = &STORAGE;

        type Bean = TestStruct;

        fn construct(_: &mut Bootstrap) -> Result<Self::Bean> {
            Ok(TestStruct(42, "Hello".to_string(), "world"))
        }
    }

    #[test]
    fn lifecycle_management() {
        let ioc = Ioc::new().expect("should create ioc");

        let mut init_ctx = Bootstrap::new(
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
