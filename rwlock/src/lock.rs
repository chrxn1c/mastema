use std::{cell::UnsafeCell, sync::atomic::AtomicU32};

use crate::guard::{read::ReadGuard, write::WriteGuard};

pub struct RwLock<T> {
    /// The number of readers, or [`u32::MAX`] if write-locked.
    lock_state: AtomicU32,
    /// The counter for writers notification. It only goes up, never down,
    /// overflows to 0 when having [`u32::MAX`] value.
    writers_counter: AtomicU32,
    /// The value behind the lock.
    value: UnsafeCell<T>,
}

// SAFETY: RwLock, when locking, can be used to move values of type T
// from one thread to another. Therefore, RwLock<T> is Sync only
// is T is Send.
//
// Furthermore, there may be multiple readers of T (each in different thread),
// which is why RwLock: Send if T: Send
unsafe impl<T> Sync for RwLock<T> where T: Send + Sync {}

impl<T> RwLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            lock_state: AtomicU32::new(0),      // Unlocked.
            writers_counter: AtomicU32::new(0), // No writers for now.
            value: UnsafeCell::new(value),
        }
    }

    pub(crate) fn value(&self) -> &UnsafeCell<T> {
        &self.value
    }

    pub(crate) fn lock_state(&self) -> &AtomicU32 {
        &self.lock_state
    }

    pub(crate) fn writers_counter(&self) -> &AtomicU32 {
        &self.writers_counter
    }

    pub fn read(&self) -> ReadGuard<'_, T> {
        let mut lock_state = self.lock_state.load(std::sync::atomic::Ordering::Relaxed);
        loop {
            // Try to lock the state, if it is not write-locked already
            if lock_state < u32::MAX {
                assert!(lock_state < u32::MAX - 1, "too many readers");
                match self.lock_state.compare_exchange_weak(
                    lock_state,
                    lock_state + 1,
                    std::sync::atomic::Ordering::Acquire,
                    std::sync::atomic::Ordering::Relaxed,
                ) {
                    Ok(_) => return ReadGuard::new(self),
                    Err(current_lock_state) => lock_state = current_lock_state,
                }
            }

            // Wait otherwise
            if lock_state == u32::MAX {
                atomic_wait::wait(&self.lock_state, u32::MAX);
                lock_state = self.lock_state.load(std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    pub fn write(&self) -> WriteGuard<'_, T> {
        // Try to set the lock_state to "write-locked"
        while self
            .lock_state
            .compare_exchange(
                0,
                u32::MAX,
                std::sync::atomic::Ordering::Acquire,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {
            // If failed -- check the current writers_counter.
            let writers_counter = self
                .writers_counter
                .load(std::sync::atomic::Ordering::Acquire);
            // If it's more than 0 -- wait for wake notifications.
            // This way, read-contention does not result in Self::write busy-looping constantly.
            if self.lock_state.load(std::sync::atomic::Ordering::Relaxed) != 0 {
                atomic_wait::wait(&self.lock_state, writers_counter);
            }
        }
        WriteGuard::new(self)
    }
}
