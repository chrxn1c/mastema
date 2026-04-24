use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::lock::{Mutex, STATE_LOCKED_WITH_WAITERS, STATE_UNLOCKED};

/// RAII struct that establishes unlocking the [`Mutex`] when dropped.
pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
    // MutexGuard<T> is Send only if &mut T: Send, not if Mutex<T>: Send.
    // MutexGuard<T> is Sync only if &mut T: Sync, not if Mutex<T>: Sync.
    phantom_data: PhantomData<&'a mut T>,
}

impl<'a, T> MutexGuard<'a, T> {
    #[inline]
    pub(crate) const fn new(mutex: &'a Mutex<T>) -> Self {
        Self {
            mutex,
            phantom_data: PhantomData,
        }
    }
}

unsafe impl<T> Send for MutexGuard<'_, T> where T: Send {}
unsafe impl<T> Sync for MutexGuard<'_, T> where T: Sync {}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY:
        //  1) User does not have access to MutexGuard's fields, and user cannot instantiate MutexGuard or change its inner
        //     contens
        //  2) MutexGuard cannot be instantiated via constructor
        //
        // Therefore, the only remaining way to get this MutexGuard, is by locking the Mutex,
        // which means that Mutex has been successfully locked, and it's safe to get
        // the inner contens of this mutex(Mutex guarantees a single access to the
        // underlying data at any time)

        unsafe { &*self.mutex.value().get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY:
        //  1) User does not have access to MutexGuard's fields, and user cannot instantiate MutexGuard or change its inner
        //     contens
        //  2) MutexGuard cannot be instantiated via constructor
        //
        // Therefore, the only remaining way to get this MutexGuard, is by locking the Mutex,
        // which means that Mutex has been successfully locked, and it's safe to get
        // the inner contens of this mutex(Mutex guarantees a single access to the
        // underlying data at any time)

        unsafe { &mut *self.mutex.value().get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // Set the lock_state back to `unlocked`, if there are any threads waiting -- wake them,
        // otherwise -- don't.
        //
        // NOTE: the woken thread must set `lock_state` back to `locked_with_waiters` to ensure
        // waiting threads are not forgotten.
        if self
            .mutex
            .lock_state()
            .swap(STATE_UNLOCKED, std::sync::atomic::Ordering::Release)
            == STATE_LOCKED_WITH_WAITERS
        {
            atomic_wait::wake_one(self.mutex.lock_state());
        }
    }
}
