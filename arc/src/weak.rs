use std::{ptr::NonNull, sync::atomic::fence};

use crate::arc::{Arc, data::ArcData};

pub struct Weak<T> {
    pub(crate) pointer: NonNull<ArcData<T>>,
}

impl<T> Weak<T> {
    pub(crate) fn new(data: T) -> Weak<T> {
        Self {
            pointer: std::ptr::NonNull::from(Box::leak(Box::new(ArcData::new(data)))),
        }
    }

    pub(crate) fn data(&self) -> &ArcData<T> {
        // SAFETY: if &self still exists, so does the pointer to the ArcData<T>, and this pointer is
        // always NonNull.
        unsafe { self.pointer.as_ref() }
    }

    /// A method, though which `Weak<T>` may be upgraded into `Arc<T>`.
    ///
    /// It is only done if T still exists (returns None otherwise).
    pub fn upgrade(&self) -> Option<Arc<T>> {
        let mut data_reference_count = self
            .data()
            .data_reference_count()
            .load(std::sync::atomic::Ordering::Relaxed);

        loop {
            if data_reference_count == 0 {
                return None;
            }
            // Check for possibility of being close to overflow
            assert!(data_reference_count <= usize::MAX / 2);

            if let Err(current_data_reference_count) =
                self.data().data_reference_count().compare_exchange_weak(
                    data_reference_count,
                    data_reference_count + 1,
                    std::sync::atomic::Ordering::Relaxed,
                    std::sync::atomic::Ordering::Relaxed,
                )
            {
                data_reference_count = current_data_reference_count;
                continue;
            }

            return Some(Arc::new_with_weak(self.clone()));
        }
    }
}

// SAFETY: If we move Weak<T> into another thread, we share T across thread boundaries.
// Therefore, Weak<T>: Send if T: Sync.
//
// Furthermore, if we move Weak<T> into another thread, that thread may try to drop the T, as it may be the
// last thread to hold the reference to T via Weak<T>. Therefore, Weak<T>: Send if T: Sync + Send.
unsafe impl<T: Sync + Send> Send for Weak<T> {}

// Same holds for Weak<T>: Sync. It is only if T: Send + Sync (Weak<T> may be `upgrade`-d into Arc<T>).
unsafe impl<T: Sync + Send> Sync for Weak<T> {}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        // Check if overflow is not even close, since
        // that may happen if user is `std::mem::forget`ting.
        if self
            .data()
            .allocations_reference_count()
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            > usize::MAX / 2
        {
            std::process::abort();
        }
        Self {
            pointer: self.pointer,
        }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self
            .data()
            .allocations_reference_count()
            .fetch_sub(1, std::sync::atomic::Ordering::Release)
            == 1
        {
            // NOTE: The last drop of Arc<T> must happen before all the previous Arcs were dropped,
            // so synchronization is needed. We only care about synchronization when we are dropping
            // the *LAST* Arc (we are the last owner), so we can use fence with Acquire ordering in this scenario, leaving
            // dropping of non-last Arc<T> (reference_count > 1) with Release Ordering.
            fence(std::sync::atomic::Ordering::Acquire);

            // SAFETY: We only drop the inner T if we're the last owner of Arc<T>, which we have
            // already checked via `fetch_sub`.
            std::mem::drop(unsafe { Box::from_raw(self.pointer.as_ptr()) })
        }
    }
}
