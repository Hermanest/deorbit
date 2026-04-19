use crate::{Error, FromDi, Services, TypeMeta};
use std::sync::Arc;

pub type ResolvedMany<T> = Vec<Arc<T>>;
pub type Resolved<T> = Arc<T>;

impl<T: ?Sized + 'static> FromDi for Resolved<T> {
    fn depends_on() -> &'static [TypeMeta] {
        &[]
    }

    fn produce(services: &Services) -> Result<Self, Error> {
        services.resolve::<T>().ok_or(Error::missing::<T>())
    }
}

impl<T: ?Sized + 'static> FromDi for ResolvedMany<T> {
    fn depends_on() -> &'static [TypeMeta] {
        &[]
    }

    fn produce(services: &Services) -> Result<Self, Error> {
        services
            .resolve_all::<T>()
            .ok_or(Error::missing::<T>())
            .map(|x| x.collect())
    }
}
