use crate::{FromDi, Services, TypeMeta};
use std::sync::Arc;

pub type ResolvedMany<T> = Vec<Arc<T>>;
pub type Resolved<T> = Arc<T>;

impl<T: ?Sized + 'static> FromDi for Resolved<T> {
    fn depends_on() -> &'static [TypeMeta] {
        &[]
    }

    fn produce(services: &Services) -> Self {
        services
            .resolve::<T>()
            .expect("Required dependency is missing")
    }
}

impl<T: ?Sized + 'static> FromDi for ResolvedMany<T> {
    fn depends_on() -> &'static [TypeMeta] {
        &[]
    }

    fn produce(services: &Services) -> Self {
        services
            .resolve_all::<T>()
            .expect("Required dependency is missing")
            .collect()
    }
}
