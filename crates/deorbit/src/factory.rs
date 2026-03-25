use crate::arc::ErasedArc;
use crate::error::Error;
use crate::from_di::{DiFactory, FromDi};
use crate::Services;
use std::fmt::{Debug, Formatter};

pub struct ServiceFactory {
    alloc: ServiceAllocator,
}

enum ServiceAllocator {
    Container {
        fun: fn(&Services) -> Result<ErasedArc, Error>,
    },
    Function {
        fun: Box<dyn Fn(&Services) -> Result<ErasedArc, Error>>,
    },
    Default {
        fun: fn() -> ErasedArc,
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
    pub fn from_container<T: 'static + FromDi>() -> Self {
        let wrapper = |x: &_| Ok(ErasedArc::from_instance(T::produce(x)));

        Self {
            alloc: ServiceAllocator::Container { fun: wrapper },
        }
    }

    pub fn from_fn<T: 'static, Args>(allocator: impl DiFactory<T, Args>) -> Self {
        let wrapper = move |x: &_| Ok(ErasedArc::from_instance(allocator.produce(x)));

        Self {
            alloc: ServiceAllocator::Function {
                fun: Box::new(wrapper),
            },
        }
    }

    pub fn from_default<T: 'static + Default>() -> Self {
        Self {
            alloc: ServiceAllocator::Default {
                fun: || ErasedArc::from_instance(T::default()),
            },
        }
    }

    pub fn produce(&self, services: &Services) -> Result<ErasedArc, Error> {
        match &self.alloc {
            ServiceAllocator::Container { fun } => fun(services),
            ServiceAllocator::Function { fun } => fun(services),
            ServiceAllocator::Default { fun } => Ok(fun()),
        }
    }
}
