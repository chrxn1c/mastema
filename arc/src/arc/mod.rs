use std::sync::atomic::fence;

use crate::weak::Weak;

pub(crate) mod data;

pub struct Arc<T> {
    weak_pointer: Weak<T>,
}

// SAFETY: If we move Arc<T> into another thread, we share T across thread boundaries.
// Therefore, Arc<T>: Send if T: Sync.
//
// Furthermore, if we move Arc<T> into another thread, that thread may drop the T, as it may be the
// last thread to hold the reference to T via Arc<T>. Therefore, Arc<T>: Send if T: Sync + Send.
unsafe impl<T: Send + Sync> Send for Arc<T> {}

// Same holds for Arc<T>: Sync. It is only if T: Send + Sync (&Arc<T> may be cloned into Arc<T>).
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Arc<T> {
    /// Construct an instance of Arc<T>, moving the T into the Arc.
    pub fn new(data: T) -> Self {
        Self {
            weak_pointer: Weak::new(data),
        }
    }

    pub(crate) fn new_with_weak(weak_pointer: Weak<T>) -> Self {
        Self { weak_pointer }
    }

    /// Get the exclusive access to the inner T of Arc<T>.
    /// This only returns Some(&mut T) if caller is holding the LAST clone of Arc<T>
    /// (with reference_count = 1).
    pub fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc
            .weak_pointer
            .data()
            .allocations_reference_count()
            .load(std::sync::atomic::Ordering::Relaxed)
            == 1
        {
            // NOTE: We must be sure that we're really the owner of the last clone of Arc<T> in this
            // scenario, which means that synchronization is needed between every Drop of Arc and
            // this method (every drop of Arc<T> must happen before user did `.get_mut`).
            fence(std::sync::atomic::Ordering::Acquire);

            // SAFETY: Nothing is accessing the inner T (we have exclusive access, no Arc's and
            // Weak's).
            let arc_data = unsafe { arc.weak_pointer.pointer.as_mut() };
            let option = arc_data.data.get_mut();

            // SAFETY: as we still have &self, Option is always Some.
            let data = option.as_mut().unwrap();
            Some(data)
        } else {
            None
        }
    }

    /// Downgrade `Arc` into `Weak` pointer.
    pub fn downgrade(arc: &Self) -> Weak<T> {
        arc.weak_pointer.clone()
    }
}

impl<T> std::ops::Deref for Arc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let pointer = self.weak_pointer.data().data.get();
        // SAFETY: While &self is alive, there's always data to be accessed.
        // Therefore, Option is always Some.
        unsafe { (*pointer).as_ref().unwrap() }
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Self {
        let weak = self.weak_pointer.clone();

        // Check if overflow is not even close, since
        // that may happen if user is `std::mem::forget`ting.
        if weak
            .data()
            .data_reference_count()
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            > usize::MAX / 2
        {
            std::process::abort();
        }
        Self { weak_pointer: weak }
    }
}

impl<T> Drop for Arc<T> {
    fn drop(&mut self) {
        // NOTE: Dropping Arc<T> will automatically invoke Drop implementation of Weak<T>, so we
        // don't need to take care of that.
        if self
            .weak_pointer
            .data()
            .data_reference_count()
            .fetch_sub(1, std::sync::atomic::Ordering::Release)
            == 1
        {
            // NOTE: The last drop of Arc<T> must happen before all the previous Arcs were dropped,
            // so synchronization is needed. We only care about synchronization when we are dropping
            // the *LAST* Arc (we are the last owner), so we can use fence with Acquire ordering in this scenario, leaving
            // dropping of non-last Arc<T> (reference_count > 1) with Release Ordering.
            fence(std::sync::atomic::Ordering::Acquire);

            let pointer = self.weak_pointer.data().data.get();
            // SAFETY: Data reference count is zero, which means nothing will ever access the data.
            unsafe {
                (*pointer) = None;
            }
        }
    }
}
