use std::time::Duration;

use mutexco::{condvar::Condvar, mutex::lock::Mutex};

#[test]
fn condvar_waiting_ok() {
    let mutex = Mutex::new(0);
    let condvar = Condvar::new();

    let mut number_of_wakeups = 0;

    std::thread::scope(|s| {
        s.spawn(|| {
            std::thread::sleep(Duration::from_secs(1));
            *mutex.lock() = 123;
            condvar.notify_one();
        });

        let mut mutex_guard = mutex.lock();
        while *mutex_guard < 100 {
            mutex_guard = condvar.wait(mutex_guard);
            number_of_wakeups += 1;
        }

        // Check that the value is the one set by another thread
        assert_eq!(*mutex_guard, 123);
    });

    // Check that the main thread actually *DID* wait (not busy-loop),
    // while still allowing for a few spurious wake ups.
    assert!(number_of_wakeups < 10);
}
