use crate::from_di::{DiFactory, FromDi};
use crate::resolver::Error;
use crate::runtime::TypeMeta;
use crate::runtime::{ErasedArc, ErasedUnsizer};
use crate::runtime::{ServiceFactory, ServiceFactoryOnce};
use crate::{DiFactoryOnce, Services};
use std::fmt::Debug;

#[derive(Debug)]
pub struct Binding {
    pub ty: TypeMeta,
    pub kind: BindingKind,
}

#[derive(Debug)]
pub enum BindingKind {
    Type {
        /// Whether this type should be exposed directly.
        bind_self: bool,
        lifetime: BindingLifetime,
        deps: &'static [TypeMeta],
    },
    Alias {
        impls: Vec<(TypeMeta, ErasedUnsizer)>,
    },
}

impl BindingKind {
    pub fn unwrap_alias(self) -> Vec<(TypeMeta, ErasedUnsizer)> {
        match self {
            BindingKind::Type { .. } => {
                panic!("Called unwrap_alias on a Type binding")
            }
            BindingKind::Alias { impls } => impls,
        }
    }

    pub fn unwrap_alias_mut(&mut self) -> &mut Vec<(TypeMeta, ErasedUnsizer)> {
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
    Factory(ServiceFactoryOnce),
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
pub enum BindingLifetime {
    Singleton(SingletonProvider),
    Transient(ServiceFactory),
}

impl BindingLifetime {
    pub fn singleton_from<T: 'static>(service: T) -> Self {
        let arc = ErasedArc::from_instance(service);

        Self::Singleton(SingletonProvider::Instance(arc))
    }

    pub fn singleton_from_di<T: 'static + FromDi>() -> Self {
        Self::Singleton(SingletonProvider::Factory(
            ServiceFactoryOnce::from_container::<T>(),
        ))
    }

    pub fn singleton_from_default<T: 'static + Default>() -> Self {
        Self::Singleton(SingletonProvider::Factory(
            ServiceFactoryOnce::from_default::<T>(),
        ))
    }

    pub fn singleton_from_fn<T: 'static, Args>(factory: impl DiFactoryOnce<T, Args>) -> Self {
        Self::Singleton(SingletonProvider::Factory(
            ServiceFactoryOnce::from_fn_once(factory),
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
