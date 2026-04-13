use std::{cell::UnsafeCell, sync::atomic::AtomicUsize};

pub(crate) struct ArcData<T> {
    // The number of references to `Arc<T>` active.
    data_reference_count: AtomicUsize,
    // The number of references to `Arc<T>` + the number of references to `ArcData<T>` active.
    allocations_reference_count: AtomicUsize,
    // The inner data. It is only `None` if all Arc's have been dropped, and only Weak<T>'s are
    // left active.
    pub(crate) data: UnsafeCell<Option<T>>,
}

impl<T> ArcData<T> {
    pub(crate) fn new(data: T) -> Self {
        Self {
            data_reference_count: AtomicUsize::new(1),
            allocations_reference_count: AtomicUsize::new(1),
            data: UnsafeCell::new(Some(data)),
        }
    }

    pub(crate) fn allocations_reference_count(&self) -> &AtomicUsize {
        &self.allocations_reference_count
    }

    pub(crate) fn data_reference_count(&self) -> &AtomicUsize {
        &self.data_reference_count
    }
}
