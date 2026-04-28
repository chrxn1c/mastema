use std::{cell::UnsafeCell, sync::atomic::AtomicU32};

use crate::guard::{read::ReadGuard, write::WriteGuard};

pub struct RwLock<T> {
    /// The number of readers times two, plus one if there's a writer waiting.
    /// e.g. 6 represents 3 active read-locks, 7 represents 3 active read-locks and 1 write-lock.
    /// [`u32::MAX`] if currently write-locked, because [`u32::MAX`] is an odd number.
    ///
    /// The usage is the following: readers acquire the lock when the lock_state is even,
    /// but have to block if lock_state is odd.
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
            if lock_state % 2 == 0 {
                // Even, have a go to read-lock.
                assert!(lock_state < u32::MAX - 2, "too many readers");
                match self.lock_state.compare_exchange_weak(
                    lock_state,
                    lock_state + 2,
                    std::sync::atomic::Ordering::Acquire,
                    std::sync::atomic::Ordering::Relaxed,
                ) {
                    Ok(_) => return ReadGuard::new(self),
                    Err(current_lock_state) => lock_state = current_lock_state,
                }
            }
            // Odd otherwise, have to wait
            else {
                atomic_wait::wait(&self.lock_state, lock_state);
                lock_state = self.lock_state.load(std::sync::atomic::Ordering::Relaxed);
            }
        }
    }

    pub fn write(&self) -> WriteGuard<'_, T> {
        let mut lock_state = self.lock_state.load(std::sync::atomic::Ordering::Relaxed);
        loop {
            // Try to write-lock instantnly (if the lock is unlocked right now).
            if lock_state <= 1 {
                match self.lock_state.compare_exchange(
                    lock_state,
                    u32::MAX,
                    std::sync::atomic::Ordering::Acquire,
                    std::sync::atomic::Ordering::Relaxed,
                ) {
                    Ok(_) => return WriteGuard::new(self),
                    Err(current_lock_state) => {
                        lock_state = current_lock_state;
                        continue;
                    }
                }
            }
            // Rwlock is currently locked.
            // Block new readers from acquiring the read-lock (by setting the lock_state to
            // odd number).
            if lock_state % 2 == 0 {
                match self.lock_state.compare_exchange(
                    lock_state,
                    lock_state + 1,
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                ) {
                    Ok(_) => {}
                    Err(current_lock_state) => {
                        lock_state = current_lock_state;
                        continue;
                    }
                }
            }

            // Rwlock is currently locked.
            // Wait until it is unlocked (readers won't acquire the lock from now on).
            let writers_counter = self
                .writers_counter
                .load(std::sync::atomic::Ordering::Acquire);
            lock_state = self.lock_state.load(std::sync::atomic::Ordering::Relaxed);
            if lock_state >= 2 {
                atomic_wait::wait(&self.writers_counter, writers_counter);
                lock_state = self.lock_state.load(std::sync::atomic::Ordering::Relaxed);
            }

            // The next iteration of the loop will insta-write-lock the RwLock if all already
            // existing readers are gone (this will not allow for new readers to intervene in
            // acquiring the write-lock).
        }
    }
}
