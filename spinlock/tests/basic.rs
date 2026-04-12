use spinlock::lock::SpinLock;

#[test]
fn basic_test_ok() {
    let data_locked = SpinLock::new(Vec::new());
    std::thread::scope(|s| {
        s.spawn(|| data_locked.lock().push(1));
        s.spawn(|| {
            let mut data = data_locked.lock();
            data.push(2);
            data.push(3);
        });
    });
    let data = data_locked.lock();
    // Either the first thread was the first one to access lock (hence [1 2 3]), or the second one
    // ([3 2 1])
    assert!(data.as_slice() == [1, 2, 3] || data.as_slice() == [3, 2, 1]);
}
