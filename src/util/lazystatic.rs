use std::{cell::UnsafeCell, mem::MaybeUninit};

pub struct LazyStatic<T> {
    inner: UnsafeCell<MaybeUninit<T>>,
}

impl<T> LazyStatic<T> {
    pub const fn new() -> Self {
        let inner = UnsafeCell::new(MaybeUninit::uninit());
        Self { inner }
    }

    pub fn init(&self, value: T) {
        let mu = unsafe { self.inner.get().as_mut_unchecked() };
        mu.write(value);
    }

    pub fn drop(&self) {
        unsafe {
            let mu = self.inner.get().as_mut_unchecked();
            mu.assume_init_drop();
        }
    }

    pub fn get(&self) -> &T {
        let mu = unsafe { self.inner.get().as_ref_unchecked() };
        unsafe { mu.assume_init_ref() }
    }

    pub fn get_mut(&self) -> &mut T {
        let mu = unsafe { self.inner.get().as_mut_unchecked() };
        unsafe { mu.assume_init_mut() }
    }
}

unsafe impl<T> Send for LazyStatic<T> {}
unsafe impl<T> Sync for LazyStatic<T> {}
