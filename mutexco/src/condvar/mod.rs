//! NOTE: Condition Variable doesn't need other than `Relaxed` memory ordering,
//! because Happens-Before relationship is provided from the mutex, on which Condition Variable
//! is waiting.
use std::sync::atomic::{AtomicU32, AtomicUsize};

use crate::mutex::guard::MutexGuard;

/// A condition variable.
/// It is used together with a mutex to wait until the mutex-protected data matches some condition.
#[derive(Default)]
pub struct Condvar {
    counter: AtomicU32,
    number_of_waiting_threads: AtomicUsize,
}

impl Condvar {
    /// Constructor for Condition Variable.
    #[inline]
    pub const fn new() -> Self {
        Self {
            counter: AtomicU32::new(0),
            number_of_waiting_threads: AtomicUsize::new(0),
        }
    }

    /// Sends notification to one thread that is waiting on the same Condition Variable, if there
    /// are any.
    pub fn notify_one(&self) {
        if self
            .number_of_waiting_threads
            .load(std::sync::atomic::Ordering::Relaxed)
            > 0
        {
            self.counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            atomic_wait::wake_one(&self.counter);
        }
    }

    /// Sends notification to all threads which are waiting on the same Condition Variable, if there
    /// are any.
    pub fn notify_all(&self) {
        if self
            .number_of_waiting_threads
            .load(std::sync::atomic::Ordering::Relaxed)
            > 0
        {
            self.counter
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            atomic_wait::wake_all(&self.counter);
        }
    }

    /// Wait until other threads wake you up.
    pub fn wait<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        self.number_of_waiting_threads
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let counter_value = self.counter.load(std::sync::atomic::Ordering::Relaxed);

        // Unlock the mutex by dropping the guard,
        // but remember the mutex so we can lock it again later.
        let mutex = guard.mutex;
        drop(guard);

        // Wait, but only if the counter hasn't changed since unlocking.
        atomic_wait::wait(&self.counter, counter_value);

        self.number_of_waiting_threads
            .fetch_sub(1, std::sync::atomic::Ordering::Relaxed);

        // Relock the mutex
        mutex.lock()
    }
}
