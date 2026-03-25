use crate::arc::ErasedArc;
use crate::factory::ServiceFactory;
use crate::from_di::{DiFactory, FromDi};
use std::any::{TypeId, type_name};
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Binding {
    pub ty: TypeMeta,
    pub lifetime: ServiceLifetime,
    pub deps: &'static [TypeMeta],
}

#[derive(Clone, Copy)]
pub struct TypeMeta {
    pub type_id: TypeId,
    pub type_name: MetaName,
}

#[derive(Clone, Copy)]
enum MetaName {
    Hardcoded(&'static str),
    Dynamic(fn() -> &'static str),
}

impl TypeMeta {
    pub const fn of<T: ?Sized + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            // Const type_name is unstable so using this workaround
            type_name: MetaName::Dynamic(|| type_name::<T>()),
        }
    }

    pub const fn of_name<T: ?Sized + 'static>(name: &'static str) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: MetaName::Hardcoded(name),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self.type_name {
            MetaName::Hardcoded(x) => x,
            MetaName::Dynamic(x) => x(),
        }
    }
}

impl Debug for TypeMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.type_name())
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
