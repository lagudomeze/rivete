use std::{cell::UnsafeCell, marker::PhantomData, mem::MaybeUninit};

pub struct StaticCell<T> {
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for StaticCell<T> where T: Sync {}

impl<T> StaticCell<T> {
    pub const fn new() -> Self {
        Self {
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    #[inline]
    pub const fn as_ptr(&self) -> StaticPtr<T> {
        StaticPtr {
            ptr: self.value.get(),
        }
    }
}

pub struct StaticPtr<T> {
    ptr: *mut MaybeUninit<T>,
}

impl<T> StaticPtr<T> {
    pub const fn new(ptr: *mut MaybeUninit<T>) -> Self {
        Self { ptr }
    }

    /// # Safety
    /// This function is unsafe because it creates a new [`StaticPtr<T>`] from a raw pointer.
    /// You must ensure that the pointer is valid and properly aligned, and that the value it points to is properly initialized.
    #[inline]
    unsafe fn as_ref(&self) -> &T {
        (&*self.ptr).assume_init_ref()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Inited<T> {
    _marker: PhantomData<T>,
}

/// A type that represents a value that is properly initialized.
/// It's zero-cost to use, copy, and clone.
impl<T> Inited<T> {
    /// Creates a new [`Inited<T>`].
    ///
    /// # Safety
    /// This function is unsafe because it creates a new [`Inited<T>`] without initializing it.
    /// You must ensure that the value is properly initialized before using it.
    /// .
    pub const unsafe fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    #[inline]
    pub fn as_ref(&self) -> &T::Value
    where
        T: StaticInit,
    {
        // SAFETY: `Inited` is only created when the value is initialized.
        // And T::HOLDER is a valid pointer to static value.
        unsafe { T::HOLDER.as_ref() }
    }
}

pub trait StaticInit {
    type Value;

    const HOLDER: StaticPtr<Self::Value>;

    /// .
    ///
    /// # Safety
    /// This function is unsafe because it initializes the static value.
    /// You must ensure that the value is initialized only once, aka, it is called only once.
    /// Then you can safely use the value.
    /// .
    unsafe fn init(value: Self::Value) -> Inited<Self> where Self: Sized {
        // SAFETY: `HOLDER` is a valid pointer to static value.
        (&mut *Self::HOLDER.ptr).write(value);
        Inited::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct A(u8);

    static A_HOLDER: StaticCell<A> = StaticCell::new();

    impl StaticInit for A {
        type Value = A;

        // here you can not set value except from static variable
        // constant expression deny Box::new(...), so the only choice is static variable.
        // And when you initialize the static variable from the holder, you can use the value from holder everywhere.
        // It's safe and zero-cost.
        const HOLDER: StaticPtr<Self::Value> = A_HOLDER.as_ptr();
    }

    #[test]
    fn it_works() {
        let inited = unsafe { A::init(A(3)) };

        let a = inited.as_ref() as *const A;
        let b = unsafe { A::HOLDER.as_ref() } as *const A;

        assert_eq!(a, b);

        assert_eq!(inited.as_ref().0, 3);

    }
}
