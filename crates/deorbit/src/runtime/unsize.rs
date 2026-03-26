use crate::runtime::TypeMeta;
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
    fun: *const (),
}

impl ErasedUnsizer {
    pub fn try_from<T: 'static, K: ?Sized + 'static>(fun: fn(Arc<T>) -> Arc<K>) -> Option<Self> {
        // Arc is either a fat or a thin pointer. While fat pointers are not guaranteed
        // to be exactly 2 * usize, thin pointers are always the same size as usize,
        // meaning we can determine whether this arc is a fat or a thin pointer easily
        if size_of::<Arc<K>>() == size_of::<usize>() {
            return None;
        }

        Some(Self {
            type_in: TypeMeta::of::<T>(),
            type_out: TypeMeta::of::<K>(),
            fun: fun as *const (),
        })
    }

    pub fn unsize<T: 'static, K: ?Sized + 'static>(&self, arc: Arc<T>) -> Result<Arc<K>, Error> {
        if self.type_in != TypeMeta::of::<T>() || self.type_out != TypeMeta::of::<K>() {
            return Err(Error::MismatchedTypes);
        }

        let arc_ptr = Arc::as_ptr(&arc) as *const ();

        let fun: fn(Arc<T>) -> Arc<K> = unsafe { mem::transmute(self.fun) };
        let unsized_arc = fun(arc);

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
        let arc = Arc::new(10);
        let unsizer = ErasedUnsizer::try_from(|x: Arc<i32>| x as Arc<dyn Any>).unwrap();

        assert!(matches!(unsizer.unsize::<_, dyn Any>(arc), Ok(..)));
    }

    #[test]
    fn test_fails_invalid_types() {
        let arc = Arc::new(10i64);
        let unsizer = ErasedUnsizer::try_from(|x: Arc<i32>| x as Arc<dyn Any>).unwrap();

        assert!(matches!(
            unsizer.unsize::<_, dyn Any>(arc),
            Err(Error::MismatchedTypes)
        ));
    }

    #[test]
    fn test_fails_external_data() {
        let arc = Arc::new(10);
        let unsizer = ErasedUnsizer::try_from(|x: Arc<i32>| Arc::new(1) as Arc<dyn Any>).unwrap();

        assert!(matches!(
            unsizer.unsize::<_, dyn Any>(arc),
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
