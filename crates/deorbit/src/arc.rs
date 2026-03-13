use std::any::{Any, TypeId};
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

        Self {
            type_id: TypeId::of::<T>(),
            data: unsafe { Self::cast_arc(arc) },
        }
    }

    pub fn from_generic<T: Any>() -> Self {
        let arc = Arc::<T>::new_uninit();

        Self {
            type_id: TypeId::of::<T>(),
            data: unsafe { Self::cast_arc(arc) },
        }
    }

    pub fn from_size(type_id: TypeId, size: usize) -> Self {
        let arc = Arc::<[u8]>::new_uninit_slice(size);

        Self {
            type_id,
            data: unsafe { Self::cast_arc(arc) },
        }
    }

    pub fn downcast<T: Any>(&self) -> Option<Arc<T>> {
        if self.type_id == TypeId::of::<T>() {
            let data = self.data.clone();
            let coerced = unsafe { Self::cast_arc(data) };

            Some(coerced)
        } else {
            None
        }
    }

    unsafe fn cast_arc<T: ?Sized, K>(from: Arc<T>) -> Arc<K> {
        unsafe { Arc::from_raw(Arc::into_raw(from).cast()) }
    }
}
