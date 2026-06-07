use std::{cell::UnsafeCell, mem::MaybeUninit};

pub struct LazyStatic<T> {
    inner: UnsafeCell<MaybeUninit<T>>,
}

impl<T: Copy> LazyStatic<T> {
    pub const fn new() -> Self {
        let inner = UnsafeCell::new(MaybeUninit::uninit());
        Self { inner }
    }

    /// # Safety
    /// Since this struct is [`Sync`] yet contains no actual locking,
    /// it is your responsibility to ensure no other threads might be
    /// reading this value while this method is overwriting it.
    ///
    /// However, since `T` is [`Copy`], it is safe to call this method
    /// multiple times, if that is for whichever reason necessary.
    pub unsafe fn init(&self, value: T) {
        // SAFETY: Since T is Copy, it's also not Drop, so we
        // can overwrite the value inside the cell at will.
        let r = unsafe { self.inner.get().as_mut_unchecked() };
        r.write(value);
    }

    /// # Safety
    /// It is only safe to call this method after [`Self::init`]
    /// has been called.
    pub unsafe fn get(&self) -> T {
        let mu = unsafe { *self.inner.get() };
        unsafe { mu.assume_init() }
    }
}

unsafe impl<T> Sync for LazyStatic<T> {}
