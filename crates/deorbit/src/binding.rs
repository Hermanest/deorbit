use crate::factory::{ManagedService, ServiceFactory};
use std::any::{type_name, TypeId};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Binding {
    pub ty: TypeMeta,
    pub lifetime: ServiceLifetime,
    pub deps: &'static [TypeId],
}

#[derive(Clone, Debug, Copy)]
pub struct TypeMeta {
    pub type_id: TypeId,
    pub type_name: &'static str,
}

impl TypeMeta {
    pub fn of<T: 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>()
        }
    }
}

impl Eq for TypeMeta {}

impl PartialEq<Self> for TypeMeta {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl PartialOrd for TypeMeta {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.type_id.partial_cmp(&other.type_id)
    }
}

impl Ord for TypeMeta {
    fn cmp(&self, other: &Self) -> Ordering {
        self.type_id.cmp(&other.type_id)
    }
}

impl Hash for TypeMeta {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state)
    }
}

#[derive(Debug)]
pub enum SingletonProvider {
    Instance(ManagedService),
    Factory(ServiceFactory),
}

#[derive(Debug)]
pub enum ServiceLifetime {
    Singleton(SingletonProvider),
    Transient(ServiceFactory),
}

impl ServiceLifetime {
    pub fn singleton_from(service: ManagedService) -> Self {
        Self::Singleton(SingletonProvider::Instance(service))
    }

    pub fn singleton_resolved(factory: ServiceFactory) -> Self {
        Self::Singleton(SingletonProvider::Factory(factory))
    }
}
