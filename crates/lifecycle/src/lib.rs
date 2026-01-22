use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::mem::MaybeUninit;

/// A marker trait for types that can only have a single instance, representing the boot stage token.
///
/// This trait is used to enforce a linear lifecycle: booting → living → outliving.
/// A `Booting` instance serves as a unique token that controls access to associated [`Slots`](Slot).
///
/// # Safety
///
/// Implementing this trait is unsafe because it requires the following guarantees:
///
/// 1. **Singleton guarantee**: The implementing type must have at most one instance at any time.
///    Typically, this is enforced using a static `Once` or similar synchronization primitive.
///
/// 2. **Lifetime dominance**: The single instance must outlive all uses of `Slot` methods
///    that take references to it (`uninit`, `mut_deref`, `deref`, `drop_in_place`).
///
/// 3. **Linear lifecycle**: The instance must follow the sequence:
///    - Created once during boot phase
///    - Used to initialize [`Slots`](Slot) via `uninit`
///    - Converted to `Living` via `assume_booted` (consuming the instance)
///    - Eventually converted to `Outliving` via `outlive` (consuming the `Living`)
///    - Dropped after all [`Slots`](Slot) are destroyed
///
/// 4. **Exclusive access**: While a `Booting` instance exists (as `&mut C`), no other references
///    to the same [`Slots`](Slot) may exist. The instance serves as proof of exclusive access.
///
/// 5. **Proper initialization**: Before transitioning to `Living`, all [`Slots`](Slot) that will be
///    accessed must be initialized via `uninit().write(...)`.
///
/// # Example
///
/// ```rust
/// use std::sync::Once;
/// use lifecycle::Booting;
///
/// pub struct BootToken {
///     _private: ()
/// }
///
/// impl BootToken {
///     pub fn new() -> Self {
///         static ONCE: Once = Once::new();
///         let mut result = None;
///         ONCE.call_once(|| {
///             result = Some(BootToken { _private: () });
///         });
///         result.unwrap()
///     }
/// }
///
/// /// # Safety
/// /// - `BootToken::new()` uses `Once` to guarantee at most one instance
/// /// - The instance controls access to associated [`Slots`](Slot)
/// /// - The lifecycle follows boot → living → outliving sequence
/// unsafe impl Booting for BootToken {}
/// ```
pub unsafe trait Booting {}

/// Error type for lifecycle operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleError {
    /// The lifecycle instance has already been initialized.
    AlreadyInitialized,
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleError::AlreadyInitialized => {
                write!(f, "lifecycle instance already initialized")
            }
        }
    }
}

impl std::error::Error for LifecycleError {}

#[derive(Debug)]
pub struct Living<C: Booting> {
    _marker: PhantomData<C>,
}

impl<C: Booting> Living<C> {
    /// # Safety
    ///
    /// The caller must guarantee the following:
    ///
    /// 1. **Boot completion**: All boot steps are completed, meaning:
    ///    - All [`Slots`](Slot) that will be accessed have been initialized via `uninit().write(...)`
    ///    - No further initialization of [`Slots`](Slot) will occur
    ///
    /// 2. **Exclusive ownership**: The `booted` instance is the sole owner of all associated [`Slots`](Slot).
    ///    No other references (shared or mutable) to any `Slot` exist.
    ///
    /// 3. **Linear transition**: This method consumes the `Booting` instance, transitioning from boot phase
    ///    to living phase. After this call, the `Booting` instance no longer exists.
    ///
    /// 4. **Singleton validity**: The `booted` instance must be the unique singleton of type `C`,
    ///    as guaranteed by the `Booting` trait implementation.
    pub unsafe fn assume_booted(_booted: C) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

#[derive(Debug)]
pub struct Outliving<C: Booting> {
    _marker: PhantomData<C>,
}

impl<C: Booting> Outliving<C> {
    pub fn outlive(_living: Living<C>) -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

pub struct Slot<T, C: Booting> {
    slot: UnsafeCell<MaybeUninit<T>>,
    category: PhantomData<C>,
}

unsafe impl<T, C> Send for Slot<T, C>
where
    T: Send,
    C: Booting + Send,
{
}

unsafe impl<T, C> Sync for Slot<T, C>
where
    T: Send + Sync,
    C: Booting + Sync,
{
}

impl<T, C> Slot<T, C>
where
    C: Booting,
{
    pub const fn new() -> Self {
        Self {
            slot: UnsafeCell::new(MaybeUninit::uninit()),
            category: PhantomData,
        }
    }
}

impl<T, C> Default for Slot<T, C>
where
    C: Booting,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T, C> Slot<T, C>
where
    C: Booting,
{
    /// Returns a mutable reference to the uninitialized slot for writing.
    ///
    /// # Safety
    ///
    /// This method is safe because all necessary guarantees are enforced by the type system:
    ///
    /// 1. **Exclusive boot phase**: The `&mut C` parameter can only exist during the boot phase,
    ///    before `assume_booted` is called. Since `assume_booted` consumes the `C` instance,
    ///    it's impossible to obtain `&mut C` after boot phase, making calls after boot phase impossible.
    ///
    /// 2. **Singleton access**: The `C: Booting` bound guarantees that only one instance of type `C`
    ///    exists. Combined with Rust's borrowing rules, this ensures that only one `&mut C` reference
    ///    can exist at any time, preventing concurrent calls to `uninit` on the same `Slot`.
    ///
    /// 3. **Single initialization**: While technically safe to call `write` multiple times on a
    ///    `MaybeUninit<T>`, doing so may cause memory leaks if `T` implements `Drop`. However,
    ///    this is a logical error rather than memory unsafety. The type system cannot prevent
    ///    multiple writes, but the singleton nature of `C` makes it unlikely in practice.
    ///
    /// 4. **Lifetime connection**: The returned reference `&'a mut MaybeUninit<T>` is tied to the
    ///    lifetime `'a` of the `&'a mut C` parameter, ensuring it cannot outlive the `Booting` instance.
    ///
    /// 5. **No overlapping mutable references**: Rust's borrowing rules guarantee that the `&mut C`
    ///    is unique, and since all accesses to [`Slots`](Slot) go through this unique reference,
    ///    multiple mutable references to the same slot cannot be created.
    pub fn uninit<'a>(&'static self, _: &'a mut C) -> &'a mut MaybeUninit<T> {
        unsafe { &mut *self.slot.get() }
    }

    /// Returns a mutable reference to the initialized value.
    ///
    /// # Safety
    ///
    /// This method is safe because all necessary guarantees are enforced by the type system:
    ///
    /// 1. **Living phase**: The `&mut Living<C>` parameter can only exist during the living phase,
    ///    after `assume_booted` has been called and before `outlive` is called. The `Living<C>`
    ///    type serves as a proof token that boot phase is complete.
    ///
    /// 2. **Exclusive access**: Rust's borrowing rules guarantee that the `&mut Living<C>` reference
    ///    is unique, providing exclusive access to all associated [`Slots`](Slot). No other references
    ///    (shared or mutable) to this `Slot` can exist while this borrow is held.
    ///
    /// 3. **Proper initialization**: The `Living<C>` instance can only be created via `assume_booted`,
    ///    which is `unsafe` and requires the caller to guarantee that all slots are initialized.
    ///    Therefore, the existence of a `Living<C>` value proves initialization has occurred.
    ///
    /// 4. **Lifetime validity**: The returned reference's lifetime `'a` is tied to the
    ///    `&mut Living<C>` borrow, ensuring it cannot outlive the borrow.
    #[inline(always)]
    pub fn mut_deref<'a>(&'static self, _: &'a mut Living<C>) -> &'a mut T {
        let ptr = self.slot.get();
        unsafe { (&mut *ptr).assume_init_mut() }
    }

    /// Returns a shared reference to the initialized value.
    ///
    /// # Safety
    ///
    /// This method is safe because all necessary guarantees are enforced by the type system:
    ///
    /// 1. **Living phase**: The `&Living<C>` parameter can only exist during the living phase,
    ///    after `assume_booted` has been called and before `outlive` is called. The `Living<C>`
    ///    type serves as a proof token that boot phase is complete.
    ///
    /// 2. **No mutable aliases**: While a `&Living<C>` exists, a `&mut Living<C>` cannot exist
    ///    due to Rust's borrowing rules. Since all mutable access to [`Slots`](Slot) requires
    ///    `&mut Living<C>`, this guarantees no mutable references to the slot exist while
    ///    the returned shared reference is alive.
    ///
    /// 3. **Proper initialization**: The `Living<C>` instance can only be created via `assume_booted`,
    ///    which is `unsafe` and requires the caller to guarantee that all slots are initialized.
    ///    Therefore, the existence of a `Living<C>` value proves initialization has occurred.
    #[inline(always)]
    pub fn deref<'a>(&'static self, _: &'a Living<C>) -> &'a T {
        let ptr = self.slot.get();
        unsafe { (&*ptr).assume_init_ref() }
    }

    /// Drops the initialized value in place.
    ///
    /// # Safety
    ///
    /// The caller must guarantee:
    ///
    /// 1. **Single destruction**: The value in this slot must not have been already dropped.
    ///    Calling `drop_in_place` multiple times on the same slot is undefined behavior.
    ///
    /// While the `Outliving<C>` instance proves that:
    /// - The slot was properly initialized (via the chain `Outliving<C>` ← `Living<C>` ← `assume_booted`)
    /// - No `&Living<C>` or `&mut Living<C>` references exist (living phase has ended)
    ///
    /// The type system cannot prevent obtaining multiple `&mut Outliving<C>` references through reborrowing,
    /// which could lead to multiple calls to `drop_in_place`. Therefore, this method is `unsafe`.
    pub unsafe fn drop_in_place(&'static self, _: &mut Outliving<C>) {
        let ptr = self.slot.get();
        unsafe { (&mut *ptr).assume_init_drop() }
    }
}

/// Creates a new lifecycle type that automatically implements the `Booting` trait.
///
/// This macro generates a struct with a private field to prevent direct instantiation,
/// a `new()` method that uses `Once` to guarantee singleton behavior, and an
/// `unsafe impl Booting` for the type.
///
/// # Example
///
/// ```rust
/// use lifecycle::lifecycle;
///
/// // Creates a `pub struct AppLifecycle { _private: () }` with singleton guarantees
/// lifecycle!(AppLifecycle);
///
/// // Now you can use AppLifecycle as a Booting type:
/// use lifecycle::Slot;
/// static COMPONENT: Slot<usize, AppLifecycle> = Slot::new();
/// ```
#[macro_export]
macro_rules! lifecycle {
    ($name:ident) => {
        /// A lifecycle type created by the `lifecycle!` macro.
        ///
        /// This type implements the `Booting` trait with singleton guarantees
        /// enforced by a static `Once`.
        #[derive(Debug)]
        pub struct $name {
            _private: (),
        }

        impl $name {
            /// Returns the singleton instance of this lifecycle type.
            ///
            /// This method uses a static `Once` to ensure that only one instance
            /// is ever created, satisfying the singleton requirement of `Booting`.
            /// Returns the singleton instance of this lifecycle type.
            ///
            /// This method uses a static `Once` to ensure that only one instance
            /// is ever created, satisfying the singleton requirement of `Booting`.
            ///
            /// # Errors
            ///
            /// Returns `Err(LifecycleError::AlreadyInitialized)` if the lifecycle
            /// instance has already been created.
            pub fn new() -> Result<Self, $crate::LifecycleError> {
                use std::sync::Once;

                static ONCE: Once = Once::new();

                let mut instance: Option<Self> = None;

                ONCE.call_once(|| {
                    instance = Some(Self { _private: () });
                });

                instance
                    .take()
                    .ok_or($crate::LifecycleError::AlreadyInitialized)
            }
        }

        /// # Safety
        /// This implementation is safe because:
        /// - `$name::new()` uses `Once` to guarantee at most one instance
        /// - The instance controls access to associated [`Slots`](Slot)
        /// - The lifecycle follows boot → living → outliving sequence
        unsafe impl $crate::Booting for $name {}
    };
}

#[cfg(test)]
mod tests {
    use crate::{Booting, LifecycleError, Slot};
    use std::sync::Once;

    struct ComLifecycle {
        _private: (),
    }
    impl ComLifecycle {
        #[allow(static_mut_refs)]
        fn new() -> Result<Self, LifecycleError> {
            static ONCE: Once = Once::new();
            let mut instance: Option<ComLifecycle> = None;

            ONCE.call_once(|| {
                instance = Some(ComLifecycle { _private: () });
            });

            instance.take().ok_or(LifecycleError::AlreadyInitialized)
        }
    }

    unsafe impl Booting for ComLifecycle {}

    struct A(usize);

    static SLOT: Slot<A, ComLifecycle> = Slot::new();

    static SLOT2: Slot<A, ComLifecycle> = Slot::new();

    struct C {
        a: A,
        b: A,
    }

    #[test]
    fn it_works() {
        let mut boot = ComLifecycle::new().unwrap();
        SLOT.uninit(&mut boot).write(A(42));
        SLOT2.uninit(&mut boot).write(A(12));

        let mut ctx = unsafe { crate::Living::assume_booted(boot) };

        {
            SLOT.mut_deref(&mut ctx).0 = SLOT2.mut_deref(&mut ctx).0;
        }

        let mut c = C { a: A(42), b: A(12) };

        {
            let a = &mut c.a;
            let b = &mut c.b;
            a.0 += b.0;
        }
    }

    #[test]
    fn macro_creates_lifecycle_type() {
        // Use the macro to create a new lifecycle type
        lifecycle!(TestLifecycle);

        // Verify the type implements Booting
        let result = TestLifecycle::new();
        assert!(result.is_ok());
        let mut boot = result.unwrap();

        // Create a static slot using the macro-generated type
        static TEST_SLOT: Slot<usize, TestLifecycle> = Slot::new();

        // Test the full lifecycle
        TEST_SLOT.uninit(&mut boot).write(42);

        let mut living = unsafe { crate::Living::assume_booted(boot) };

        assert_eq!(*TEST_SLOT.deref(&living), 42);

        *TEST_SLOT.mut_deref(&mut living) = 100;
        assert_eq!(*TEST_SLOT.deref(&living), 100);

        let mut outliving = crate::Outliving::outlive(living);

        // Drop the value (in real usage, this would be called once per slot)
        unsafe { TEST_SLOT.drop_in_place(&mut outliving) };

        let result = TestLifecycle::new();
        assert_eq!(result.unwrap_err(), LifecycleError::AlreadyInitialized);
    }
}
