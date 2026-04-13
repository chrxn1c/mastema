use std::sync::atomic::AtomicUsize;

use arc::arc::Arc;

#[test]
fn upgrading_with_drop_detection_ok() {
    static NUMBER_OF_DROPS: AtomicUsize = AtomicUsize::new(0);

    // A struct which will signal when it has been dropped.
    struct DropDetector;

    impl Drop for DropDetector {
        fn drop(&mut self) {
            NUMBER_OF_DROPS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    // Create an Arc with two weak pointers.
    let arc = Arc::new(("hello", DropDetector));
    let first_weak = Arc::downgrade(&arc);
    let second_weak = Arc::downgrade(&arc);

    let background_thread_handle = std::thread::spawn(move || {
        // Weak pointer should be upgradable at this point.
        let arc = first_weak.upgrade().unwrap();
        assert_eq!(arc.0, "hello");
    });
    assert_eq!(arc.0, "hello");
    background_thread_handle.join().unwrap();

    // The data shouldn't be dropped yet,
    // and the weak pointer should be upgradable.
    assert_eq!(
        NUMBER_OF_DROPS.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
    assert!(second_weak.upgrade().is_some());

    drop(arc);

    // Now, the data should be dropped, and the
    // weak pointer should no longer be upgradable.
    assert_eq!(
        NUMBER_OF_DROPS.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
    assert!(second_weak.upgrade().is_none());
}
