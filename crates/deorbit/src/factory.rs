use crate::Services;
use crate::arc::ErasedArc;
use crate::error::Error;
use crate::from_di::{DiFactory, FromDi};
use std::any::Any;
use std::fmt::{Debug, Formatter};

pub type ManagedService = ErasedArc;

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
            ServiceAllocator::Default { .. } => "Default",
        };

        write!(f, "{}", stringified)
    }
}

impl ServiceFactory {
    pub fn from_container<T: Any + FromDi>() -> Self {
        let wrapper = |x: &_| Ok(ManagedService::from_instance(T::produce(x)));

        Self {
            alloc: ServiceAllocator::Container { fun: wrapper },
        }
    }

    pub fn from_fn<T: Any, Args>(allocator: impl DiFactory<T, Args>) -> Self {
        let wrapper = move |x: &_| Ok(ManagedService::from_instance(allocator.produce(x)));

        Self {
            alloc: ServiceAllocator::Function {
                fun: Box::new(wrapper),
            },
        }
    }

    pub fn from_default<T: Any + Default>() -> Self {
        Self {
            alloc: ServiceAllocator::Default {
                fun: || ManagedService::from_instance(T::default()),
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
