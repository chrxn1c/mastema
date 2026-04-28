use std::ops::Deref;

use crate::lock::RwLock;

pub struct ReadGuard<'a, T> {
    rwlock: &'a RwLock<T>,
}

impl<'a, T> ReadGuard<'a, T> {
    pub(crate) const fn new(rwlock: &'a RwLock<T>) -> Self {
        ReadGuard { rwlock }
    }
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY:
        // User cannot instantiate this guard or change its inner contens.
        // Therefore, the only remaining way to get this guard, is by read-locking the RwLock,
        // which means that RwLock has been successfully read-locked, and it's safe to get
        // the inner contens of this lock (RwLock guarantees multiple read-access)
        //
        // NOTE: Until we get a shared reference to the inner T -- it's fine. But getting a mutable
        // reference to the inner T is forbidden, because there are multiple readers, which is why
        // ReadGuard cannot be DerefMut'ed to &mut T.
        unsafe { &*self.rwlock.value().get() }
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        // Decrement the state by 2, which represents 1 reader gone.
        if self
            .rwlock
            .lock_state()
            .fetch_sub(2, std::sync::atomic::Ordering::Release)
            == 3
        {
            // If we decremented from 3 to 1 -- it means RwLock is now unlocked **AND** there's a
            // waiting writer. Time to wake him up.
            // NOTE: This must establish happens-before relationship (incrementing writers_counter
            // when dropping the read-guard happens before checking for writers_counter in
            // RwLock::write. RwLock::write must see both updated
            // writers_counter and lock_state, otherwise writer would conclude RwLock is still
            // read-locked while having missed the notification.

            self.rwlock
                .writers_counter()
                .fetch_add(1, std::sync::atomic::Ordering::Release);
            atomic_wait::wake_one(self.rwlock.writers_counter());
        }
    }
}
