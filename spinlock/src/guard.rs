use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::lock::SpinLock;

/// RAII struct that establishes unlocking the [`SpinLock`] when dropped.
pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
    // Guard<T> is Send only if &mut T: Send, not if SpinLock<T>: Send.
    // Guard<T> is Sync only if &mut T: Sync, not if SpinLock<T>: Sync.
    phantom_data: PhantomData<&'a mut T>,
}

impl<'a, T> Guard<'a, T> {
    pub(crate) fn new(lock: &'a SpinLock<T>) -> Self {
        Self {
            lock,
            phantom_data: PhantomData,
        }
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY:
        //  1) Guard has pub(crate) fields (user cannot instantiate this guard or change its inner
        //     contens)
        //  2) Guard cannot be instantiated via constructor by user (constructor is pub(crate))
        //
        // Therefore, the only remaining way to get this guard, is by locking the SpinLock,
        // which means that SpinLock has been successfully locked, and it's safe to get
        // the inner contens of this lock (SpinLock guarantees a single access to the
        // underlying data at any time)
        unsafe { &*self.lock.inner.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY:
        //  1) Guard has pub(crate) fields (user cannot instantiate this guard or change its inner
        //     contens)
        //  2) Guard cannot be instantiated via constructor by user (constructor is pub(crate))
        //
        // Therefore, the only remaining way to get this guard, is by locking the SpinLock,
        // which means that SpinLock has been successfully locked, and it's safe to get
        // the inner contens of this lock (SpinLock guarantees a single access to the
        // underlying data at any time)
        unsafe { &mut *self.lock.inner.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        // NOTE: Unlocking & Locking must establish happens-before relationship
        self.lock
            .is_locked
            .store(false, std::sync::atomic::Ordering::Release);
    }
}
