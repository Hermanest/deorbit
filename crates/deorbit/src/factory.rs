use crate::Services;
use crate::arc::TypedArc;
use crate::builder::{FromDi, FromDiFactory};
use std::any::Any;
use std::fmt::{Debug, Formatter};
use crate::error::Error;

pub type ManagedService = TypedArc;

pub struct ServiceFactory {
    alloc: ServiceAllocator,
}

enum ServiceAllocator {
    Container {
        fun: fn(&Services) -> Result<ManagedService, Error>,
    },
    Function {
        fun: Box<dyn Fn(&Services) -> Result<ManagedService, Error>>,
    },
    Default {
        fun: fn() -> ManagedService,
    },
}

impl Debug for ServiceFactory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let stringified = match self.alloc {
            ServiceAllocator::Container { .. } => "Container",
            ServiceAllocator::Function { .. } => "Function",
            ServiceAllocator::Default { .. } => "Default"
        };
        
        write!(f, "{}", stringified)
    }
}

impl ServiceFactory {
    pub fn from_container<T: Any + FromDi>() -> Self {
        let wrapper = |x: &_| Ok(ManagedService::from(T::produce(x)));

        Self {
            alloc: ServiceAllocator::Container { fun: wrapper },
        }
    }

    pub fn from_fn<T: Any>(allocator: impl FromDiFactory<T>) -> Self {
        let wrapper = move |x: &_| Ok(ManagedService::from(allocator.produce(x)));

        Self {
            alloc: ServiceAllocator::Function {
                fun: Box::new(wrapper),
            },
        }
    }

    pub fn from_default<T: Any + Default>() -> Self {
        Self {
            alloc: ServiceAllocator::Default {
                fun: || ManagedService::from(T::default()),
            },
        }
    }

    pub fn produce(&self, services: &Services) -> Result<ManagedService, Error> {
        match &self.alloc {
            ServiceAllocator::Container { fun } => fun(services),
            ServiceAllocator::Function { fun } => fun(services),
            ServiceAllocator::Default { fun } => Ok(fun()),
        }
    }
}
