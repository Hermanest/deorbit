use crate::TypeMeta;
use crate::arc::ErasedArc;
use crate::factory::ServiceFactory;
use crate::from_di::{DiFactory, FromDi};
use std::fmt::Debug;

#[derive(Debug)]
pub struct Binding {
    pub ty: TypeMeta,
    pub lifetime: ServiceLifetime,
    pub deps: &'static [TypeMeta],
}

#[derive(Debug)]
pub enum SingletonProvider {
    Instance(ErasedArc),
    Factory(ServiceFactory),
}

#[derive(Debug)]
pub enum ServiceLifetime {
    Singleton(SingletonProvider),
    Transient(ServiceFactory),
}

impl ServiceLifetime {
    pub fn singleton_from<T: 'static>(service: T) -> Self {
        let arc = ErasedArc::from_instance(service);

        Self::Singleton(SingletonProvider::Instance(arc))
    }

    pub fn singleton_from_di<T: 'static + FromDi>() -> Self {
        Self::Singleton(SingletonProvider::Factory(
            ServiceFactory::from_container::<T>(),
        ))
    }

    pub fn singleton_from_default<T: 'static + Default>() -> Self {
        Self::Singleton(SingletonProvider::Factory(
            ServiceFactory::from_default::<T>(),
        ))
    }

    pub fn transient_from_di<T: 'static + FromDi>() -> Self {
        Self::Transient(ServiceFactory::from_container::<T>())
    }

    pub fn transient_from_default<T: 'static + Default>() -> Self {
        Self::Transient(ServiceFactory::from_default::<T>())
    }

    pub fn transient_from_fn<T: 'static, Args>(factory: impl DiFactory<T, Args>) -> Self {
        Self::Transient(ServiceFactory::from_fn(factory))
    }
}
