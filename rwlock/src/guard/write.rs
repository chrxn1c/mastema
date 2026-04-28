use std::ops::{Deref, DerefMut};

use crate::lock::RwLock;

pub struct WriteGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}

impl<'a, T> WriteGuard<'a, T> {
    pub(crate) const fn new(rwlock: &'a RwLock<T>) -> Self {
        WriteGuard { rwlock }
    }
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY:
        // User cannot instantiate this guard or change its inner contens.
        // Therefore, the only remaining way to get this guard, is by write-locking the RwLock,
        // which means that RwLock has been successfully locked, and it's safe to get
        // the inner contens of this lock (RwLock guarantees that write-access in unique)
        unsafe { &*self.rwlock.value().get() }
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY:
        // User cannot instantiate this guard or change its inner contens.
        // Therefore, the only remaining way to get this guard, is by write-locking the RwLock,
        // which means that RwLock has been successfully locked, and it's safe to get
        // the inner contens of this lock (RwLock guarantees that write-access in unique)
        unsafe { &mut *self.rwlock.value().get() }
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        // Reset the lock_state to `not write-locked anymore`.
        self.rwlock
            .lock_state()
            .store(0, std::sync::atomic::Ordering::Release);
        // Increment the writers_counter, so that currently awaiting writers could be woken up.
        self.rwlock
            .writers_counter()
            .fetch_add(1, std::sync::atomic::Ordering::Release);

        // NOTE: We don't know if there are any writers or readers waiting, so we have to try to wake one
        // awaiting writer, or all readers.

        // Try to wake up one awaiting writer (if it's present).
        atomic_wait::wake_one(self.rwlock.writers_counter());
        // Try to wake up all waiting readers (be it readers or writers)
        atomic_wait::wake_all(self.rwlock.lock_state());
    }
}
