use std::{cell::UnsafeCell, sync::atomic::AtomicBool};

use crate::guard::Guard;

pub struct SpinLock<T> {
    pub(crate) is_locked: AtomicBool,
    pub(crate) inner: UnsafeCell<T>,
}

// SAFETY: SpinLock, when locking, can be used to move values of type T
// from one thread to another. Therefore, SpinLock<T> is Sync only
// is T is send.
//
// NOTE: We don't require SpinLock<T> to be Sync if T: Sync,
// because we promise that at each time, only one thread
// may access the inner T.
unsafe impl<T> Sync for SpinLock<T> where T: Send {}

impl<T> SpinLock<T> {
    pub const fn new(inner: T) -> Self {
        Self {
            is_locked: AtomicBool::new(false),
            inner: UnsafeCell::new(inner),
        }
    }

    // NOTE: Unlocking & Locking must establish happens-before relationship
    pub fn lock(&self) -> Guard<'_, T> {
        while self
            .is_locked
            .swap(true, std::sync::atomic::Ordering::Acquire)
        {
            std::hint::spin_loop();
        }
        Guard::new(self)
    }
}
