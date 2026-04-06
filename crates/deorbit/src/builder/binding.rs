use crate::from_di::{DiFactory, FromDi};
use crate::runtime::ServiceFactory;
use crate::runtime::TypeMeta;
use crate::runtime::{ErasedArc, ErasedUnsizer};
use std::collections::HashMap;
use std::fmt::Debug;
use crate::resolver::Error;
use crate::Services;

#[derive(Debug)]
pub struct Binding {
    pub ty: TypeMeta,
    pub kind: BindingKind,
}

#[derive(Debug)]
pub enum BindingKind {
    Type {
        lifetime: ServiceLifetime,
        deps: &'static [TypeMeta],
    },
    Alias {
        impls: HashMap<TypeMeta, ErasedUnsizer>,
    },
}

impl BindingKind {
    pub fn unwrap_alias(self) -> HashMap<TypeMeta, ErasedUnsizer> {
        match self {
            BindingKind::Type { .. } => {
                panic!("Called unwrap_alias on a Type binding")
            }
            BindingKind::Alias { impls } => impls,
        }
    }

    pub fn unwrap_alias_mut(&mut self) -> &mut HashMap<TypeMeta, ErasedUnsizer> {
        match self {
            BindingKind::Type { .. } => {
                panic!("Called unwrap_alias on a Type binding")
            }
            BindingKind::Alias { impls } => impls,
        }
    }
}

#[derive(Debug)]
pub enum SingletonProvider {
    Instance(ErasedArc),
    Factory(ServiceFactory),
}

impl SingletonProvider {
    pub fn to_instance(self, services: &Services) -> Result<ErasedArc, Error> {
        match self {
            SingletonProvider::Instance(x) => Ok(x),
            SingletonProvider::Factory(x) => x.produce(services),
        }
    }
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
