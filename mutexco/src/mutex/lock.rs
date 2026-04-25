use std::{cell::UnsafeCell, sync::atomic::AtomicU32};

use crate::mutex::guard::MutexGuard;

pub(crate) const STATE_UNLOCKED: u32 = 0;
pub(crate) const STATE_LOCKED_WITHOUT_WAITERS: u32 = 1;
pub(crate) const STATE_LOCKED_WITH_WAITERS: u32 = 2;

pub struct Mutex<T> {
    // The state of lock, which may be [STATE_UNLOCKED], [STATE_LOCKED_WITHOUT_WAITERS], or
    // [STATE_LOCKED_WITH_WAITERS]
    lock_state: AtomicU32,
    // The value behind the lock
    value: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    /// Construct a new [`Mutex`] guarding the provided T.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            lock_state: AtomicU32::new(STATE_UNLOCKED),
            value: UnsafeCell::new(value),
        }
    }

    // NOTE: Unlocking & Locking must establish happens-before relationship.
    // Trying to lock when already locked -- really doesn't.
    #[inline]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        // If we fail to change state to `LOCKED` (mutex is already locked)
        if self
            .lock_state
            .compare_exchange(
                STATE_UNLOCKED,
                STATE_LOCKED_WITHOUT_WAITERS,
                std::sync::atomic::Ordering::Acquire,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {
            self.lock_contended();
        }
        MutexGuard::new(self)
    }

    #[inline]
    pub(crate) fn lock_state(&self) -> &AtomicU32 {
        &self.lock_state
    }

    #[inline]
    pub(crate) fn value(&self) -> &UnsafeCell<T> {
        &self.value
    }

    // What to do in case of trying to lock an already locked mutex.
    #[cold]
    fn lock_contended(&self) {
        // Spin-loop for luck, avoiding expensive syscalls (if there's little contention, this may
        // be worthy).
        let mut spin_count = 0;

        while self.lock_state.load(std::sync::atomic::Ordering::Relaxed)
            == STATE_LOCKED_WITHOUT_WAITERS
            && spin_count < 100
        {
            spin_count += 1;
            std::hint::spin_loop();
        }

        // If we grabbed the lock while spin-looping, and other threads are not waiting for a lock -- no need to do any work.
        if self
            .lock_state
            .compare_exchange(
                STATE_UNLOCKED,
                STATE_LOCKED_WITHOUT_WAITERS,
                std::sync::atomic::Ordering::Acquire,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_ok()
        {
            return;
        }

        // If we didn't -- time to block. Try to signal that state now is `LOCKED_WITH_WAITERS` (we're waiting as well).
        //
        //  - If `swap` returns 0 -- we change the state to `LOCKED_WITH_WAITERS` (it may be
        //  unnecessary, but it is unavoidable, see Note in [`MutexGuard::Drop`])
        //
        //  - If `swap` returns 1 or 2 -- we change the state to `LOCKED_WITH_WAITERS` and block.
        //
        // NOTE: Spurrious wake-up is real, so `wait` may return even when `lock_state` is still
        // locked. Which is why while loop is here.
        while self.lock_state.swap(
            STATE_LOCKED_WITH_WAITERS,
            std::sync::atomic::Ordering::Acquire,
        ) != STATE_UNLOCKED
        {
            atomic_wait::wait(self.lock_state(), STATE_LOCKED_WITH_WAITERS);
        }
    }
}

// SAFETY: Mutex<T>, when locking, can be used to move values of type T
// from one thread to another. Therefore, Mutex<T> is Sync only
// is T is send.
//
// NOTE: We don't require Mutex<T> to be Sync if T: Sync,
// because we promise that at each time, only one thread
// may access the inner T.
unsafe impl<T> Sync for Mutex<T> where T: Send {}
