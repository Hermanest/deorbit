use std::any::TypeId;
use std::mem;
use std::sync::Arc;

/// Allows storing heterogeneous data in the same collection.
#[derive(Debug)]
pub(crate) struct ErasedArc {
    type_id: TypeId,
    // Here Arc might have a size of 16 bytes hence not safe to be stored
    // as a plain Arc because fat pointers don't have a guaranteed layout
    data: [usize; 2],
    // Arc is completely erased so we have to drop it manually
    drop_fn: unsafe fn([usize; 2]),
    // Same as for drop
    inc_fn: unsafe fn([usize; 2]),
}

impl ErasedArc {
    pub fn from_instance<T: 'static>(instance: T) -> Self {
        Self::from(Arc::new(instance))
    }

    pub fn from<T: ?Sized + 'static>(instance: Arc<T>) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            data: unsafe {
                let raw = Arc::into_raw(instance);

                mem::transmute_copy(&raw)
            },
            drop_fn: |x| unsafe {
                let ptr = Self::cast_ptr::<T>(&x);

                Arc::decrement_strong_count(ptr);
            },
            inc_fn: |x| unsafe {
                let ptr = Self::cast_ptr::<T>(&x);

                Arc::increment_strong_count(ptr);
            },
        }
    }

    pub fn coerce<T: ?Sized + 'static>(&self) -> Option<Arc<T>> {
        if self.type_id == TypeId::of::<T>() {
            let coerced = unsafe {
                let ptr = Self::cast_ptr::<T>(&self.data);

                Arc::increment_strong_count(ptr);
                Arc::from_raw(ptr)
            };

            Some(coerced)
        } else {
            None
        }
    }

    unsafe fn cast_ptr<T: ?Sized>(from: &[usize; 2]) -> *const T {
        // Note that from is the initial arc pointer, so we wrap it into
        // ManuallyDrop to prevent it from being dropped
        unsafe { mem::transmute_copy(&from) }
    }
}

impl Clone for ErasedArc {
    fn clone(&self) -> Self {
        unsafe {
            (self.inc_fn)(self.data);
        }

        Self {
            type_id: self.type_id,
            data: self.data,
            drop_fn: self.drop_fn,
            inc_fn: self.inc_fn,
        }
    }
}

impl Drop for ErasedArc {
    fn drop(&mut self) {
        unsafe {
            (self.drop_fn)(self.data);
        }
    }
}
