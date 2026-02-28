use std::any::{Any, TypeId};
use std::mem::{MaybeUninit, transmute};
use std::sync::Arc;

/// Allows storing heterogeneous data in the same collection.
#[derive(Clone, Debug)]
pub(crate) struct TypedArc {
    type_id: TypeId,
    data: Arc<()>,
}

impl TypedArc {
    pub fn from<T: Any>(instance: T) -> Self {
        let arc = Arc::new(instance);
        let arc = unsafe { transmute(arc) };

        Self {
            type_id: TypeId::of::<T>(),
            data: arc,
        }
    }

    pub fn from_generic<T: Any>() -> Self {
        let arc = Arc::<T>::new_uninit();
        let arc = unsafe { transmute(arc) };

        Self {
            type_id: TypeId::of::<T>(),
            data: arc,
        }
    }

    pub fn from_size(type_id: TypeId, size: usize) -> Self {
        let arc = Arc::<[u8]>::new_uninit_slice(size);

        let arc = unsafe {
            let raw = Arc::into_raw(arc) as *const ();
            Arc::from_raw(raw)
        };

        Self { type_id, data: arc }
    }

    pub fn downcast<T: Any>(&self) -> Option<Arc<T>> {
        if self.type_id == TypeId::of::<T>() {
            let data = self.data.clone();
            let coerced = unsafe { transmute(data) };

            Some(coerced)
        } else {
            None
        }
    }

    pub unsafe fn to_uninit(&self) -> &mut MaybeUninit<()> {
        // This is generally not a good practice, but in this case the method
        // is used in synchronous context only, so there won't be a race condition
        unsafe { transmute(Arc::as_ptr(&self.data)) }
    }
}
