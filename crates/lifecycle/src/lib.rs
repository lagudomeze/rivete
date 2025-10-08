use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

/// A label for types that can only have a single instance.
/// # Safety
/// It is unsafe to implement this trait because it is a marker trait.
/// Users must ensure that the type they are implementing this trait for can only have a single instance.  
/// In most cases, you should use a static [`Once`](std::sync::Once) to ensure that the type is only initialized once.  For example:
/// ```rust
/// use std::sync::Once;
///
/// use lifecycle::Singleton;
///
/// pub struct A {
///   _private: ()
/// }
///
/// impl A {
///  pub fn new() -> Self {
///    static ONCE: Once = Once::new();
///
///    let mut result = None;
///
///    ONCE.call_once(|| {
///      result = Some(A { _private: () });
///    });
///
///    result.unwrap()
///  }
/// }
///
/// /// # Safety
/// /// This is safe because A can only be initialized once.
/// unsafe impl Singleton for A {}
/// ```
pub unsafe trait Singleton {}

/// # Safety
/// This is unsafe because the caller must ensure:
/// 1. The Booting is Singleton
/// 2. You can only create Living from Living::assume_booted
/// 3. You can only create Outliving from Living::outliving
pub trait Living {
    type Booting: Singleton;

    unsafe fn assume_booted(from: Self::Booting) -> Self;

    type Outliving;

    fn outliving(self) -> Self::Outliving;
}

/// # Safety
/// This is safe because any type that implements Living can only be created from a Booting instance, which is Singleton.
unsafe impl<T> Singleton for T where T: Living {}

pub struct StaticSlot<T, L> {
    slot: UnsafeCell<MaybeUninit<T>>,
    life: PhantomData<L>,
}

unsafe impl<T, L> Send for StaticSlot<T, L>
where
    T: Send,
    L: Send,
{
}

unsafe impl<T, L> Sync for StaticSlot<T, L>
where
    T: Send,
    L: Send,
{
}

impl<T, L> StaticSlot<T, L> {
    pub const fn new() -> Self {
        StaticSlot {
            slot: UnsafeCell::new(MaybeUninit::uninit()),
            life: PhantomData,
        }
    }
}

impl<T, L: Living> StaticSlot<T, L> {
    /// # Safety
    /// This is safe because the `Booting` instance is Singleton, so you can not create multiple mutable references to the same `StaticSlot`.
    pub fn uninit<'a>(&'static self, _: &'a mut L::Booting) -> &'a mut MaybeUninit<T> {
        unsafe { &mut *self.slot.get() }
    }

    /// # Safety
    /// This is safe because the `T` impl Living , it is Singleton, so you can not create multiple mutable references to the same `StaticSlot`.
    /// Also, Living is an unsafe trait, so when you get a Living instance, you can be sure the slot is initialized.
    #[inline(always)]
    pub fn mut_deref<'a, 'b>(&'static self, _: &'a mut L) -> &'b mut T
    where
        'a: 'b,
    {
        let ptr = self.slot.get();
        unsafe { (&mut *ptr).assume_init_mut() }
    }

    /// # Safety
    /// This is safe because the `L` impl Living , it is Singleton, so you can safely create references to the same `StaticSlot`.
    /// Also, Living is an unsafe trait, so when you get a Living instance, you can be sure the slot is initialized.
    #[inline(always)]
    pub fn deref<'a>(&'static self, _: &'a L) -> &'a T {
        let ptr = self.slot.get();
        unsafe { (&*ptr).assume_init_ref() }
    }

    /// # Safety
    /// This is safe because the `L` impl Living , it is Singleton, so Outliving is also Singleton. There is not any ref to this slot, so you call drop_in_place safely.
    pub fn drop_in_place(&'static self, _: &L::Outliving) {
        let ptr = self.slot.get();
        unsafe { (&mut *ptr).assume_init_drop() }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Singleton, StaticSlot};
    use std::sync::Once;

    // #[singleton]
    struct Bootstrap {
        _private: (),
    }

    struct Living {
        _private: (),
    }

    struct Outliving {
        _private: (),
    }

    impl Bootstrap {
        fn new() -> Self {
            static ONCE: Once = Once::new();

            let mut ret = None;

            ONCE.call_once(|| {
                ret = Some(Self { _private: () });
            });

            ret.unwrap()
        }
    }

    unsafe impl Singleton for Bootstrap {}

    impl super::Living for Living {
        type Booting = Bootstrap;

        unsafe fn assume_booted(_: Self::Booting) -> Self {
            Self { _private: () }
        }

        type Outliving = Outliving;

        fn outliving(self) -> Self::Outliving {
            Self::Outliving { _private: () }
        }
    }

    struct A(usize);

    static SLOT: StaticSlot<A, Living> = StaticSlot::new();

    static SLOT2: StaticSlot<A, Living> = StaticSlot::new();

    struct C {
        a: A,
        b: A,
    }

    #[test]
    fn it_works() {
        let mut boot = Bootstrap::new();
        SLOT.uninit(&mut boot).write(A(42));
        SLOT2.uninit(&mut boot).write(A(12));

        let mut ctx = unsafe { crate::Living::assume_booted(boot) };

        {
            SLOT.mut_deref(&mut ctx).0 =  SLOT2.mut_deref(&mut ctx).0 ;
        }

        let mut c = C { a: A(42), b: A(12) };

        {
            let a = &mut c.a;
            let b = &mut c.b;
            a.0 += b.0;
        }
    }
}
