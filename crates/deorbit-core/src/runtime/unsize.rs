use crate::runtime::{ErasedArc, TypeMeta};
use std::mem;
use std::sync::Arc;

#[derive(Debug)]
pub enum Error {
    MismatchedData,
    MismatchedTypes,
}

#[derive(Clone, Debug)]
pub struct ErasedUnsizer {
    type_in: TypeMeta,
    type_out: TypeMeta,
    typed_func: *const (),
    erased_func: *const (),
}

type TypedFunc<T, K> = fn(Arc<T>) -> Arc<K>;
type ErasedFunc<K> = fn(ErasedArc, typed: *const ()) -> Arc<K>;

impl ErasedUnsizer {
    pub fn try_from<T, K>(func: TypedFunc<T, K>) -> Option<Self>
    where
        T: Send + Sync + 'static,
        K: ?Sized + Send + Sync + 'static,
    {
        // Arc is either a fat or a thin pointer. While fat pointers are not guaranteed
        // to be exactly 2 * usize, thin pointers are always the same size as usize,
        // meaning we can determine whether this arc is a fat or a thin pointer easily
        if size_of::<Arc<K>>() == size_of::<usize>() {
            return None;
        }

        let erased: ErasedFunc<K> = |arc: ErasedArc, typed: *const ()| {
            let arc = arc.coerce::<T>().unwrap();
            let typed: TypedFunc<T, K> = unsafe { mem::transmute(typed) };

            typed(arc)
        };

        Some(Self {
            type_in: TypeMeta::of::<T>(),
            type_out: TypeMeta::of::<K>(),
            typed_func: func as *const (),
            erased_func: erased as *const (),
        })
    }

    pub fn type_in(&self) -> TypeMeta {
        self.type_in
    }

    pub fn type_out(&self) -> TypeMeta {
        self.type_out
    }

    pub fn unsize<K>(&self, arc: ErasedArc) -> Result<Arc<K>, Error>
    where
        K: ?Sized + Send + Sync + 'static,
    {
        if self.type_in != arc.ty() || self.type_out != TypeMeta::of::<K>() {
            return Err(Error::MismatchedTypes);
        }

        let arc_ptr = ErasedArc::as_ptr(&arc);

        let erased: ErasedFunc<K> = unsafe { mem::transmute(self.erased_func) };
        let unsized_arc = erased(arc, self.typed_func);

        // Check data pointers on both arcs to ensure that user has provided an arc
        // pointing to the same location in memory
        if arc_ptr != Arc::as_ptr(&unsized_arc) as *const () {
            return Err(Error::MismatchedData);
        }

        Ok(unsized_arc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    #[test]
    fn test_unsizes() {
        let arc = ErasedArc::from_instance(10);
        let unsizer =
            ErasedUnsizer::try_from(|x: Arc<i32>| x as Arc<dyn Any + Send + Sync>).unwrap();

        assert!(matches!(
            unsizer.unsize::<dyn Any + Send + Sync>(arc),
            Ok(..)
        ));
    }

    #[test]
    fn test_fails_invalid_types() {
        let arc = ErasedArc::from_instance(10i64);
        let unsizer =
            ErasedUnsizer::try_from(|x: Arc<i32>| x as Arc<dyn Any + Send + Sync>).unwrap();

        assert!(matches!(
            unsizer.unsize::<dyn Any + Send + Sync>(arc),
            Err(Error::MismatchedTypes)
        ));
    }

    #[test]
    fn test_fails_external_data() {
        let arc = ErasedArc::from_instance(10);
        let unsizer =
            ErasedUnsizer::try_from(|x: Arc<i32>| Arc::new(1) as Arc<dyn Any + Send + Sync>)
                .unwrap();

        assert!(matches!(
            unsizer.unsize::<dyn Any + Send + Sync>(arc),
            Err(Error::MismatchedData)
        ));
    }

    #[test]
    fn test_fails_to_sized() {
        let arc = Arc::new(10);
        let unsizer = ErasedUnsizer::try_from(|x: Arc<i32>| x);

        assert!(matches!(unsizer, None));
    }
}
