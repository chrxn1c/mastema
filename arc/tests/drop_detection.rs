use std::sync::atomic::AtomicUsize;

use arc::arc::Arc;

#[test]
fn basic_drop_detection_ok() {
    static NUMBER_OF_DROPS: AtomicUsize = AtomicUsize::new(0);

    // A struct which will signal when it has been dropped.
    struct DropDetector;

    impl Drop for DropDetector {
        fn drop(&mut self) {
            NUMBER_OF_DROPS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }

    // Create two Arcs sharing an object containing a string
    // and a DropDetector to detect when it's dropped.
    let arc_content = ("hello", DropDetector);
    let first = Arc::new(arc_content);
    let second = first.clone();

    // Send `first` to another thread, and use it there.
    let background_thread_handle = std::thread::spawn(move || {
        assert_eq!(first.0, "hello");
    });

    // In parallel, `second` should still be usable here.
    assert_eq!(second.0, "hello");

    // Wait for the thread to finish.
    background_thread_handle.join().unwrap();

    // One Arc, the `first` one, should be dropped by now, as the thread it was in was joined.
    // We still have `second`, so the object shouldn't have been dropped yet.
    assert_eq!(
        NUMBER_OF_DROPS.load(std::sync::atomic::Ordering::Relaxed),
        0
    );

    // Drop the remaining `Arc`.
    drop(second);

    // Now that `second` is dropped too,
    // the object must be dropped.
    assert_eq!(
        NUMBER_OF_DROPS.load(std::sync::atomic::Ordering::Relaxed),
        1
    );
}
