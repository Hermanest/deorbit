use crate::from_di::{DiFactory, FromDi};
use crate::resolver::Error;
use crate::runtime::ErasedArc;
use crate::{DiFactoryOnce, Services};
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub type ServiceFactory = Factory<Arc<dyn Fn(&Services) -> Result<ErasedArc, Error>>>;
pub type ServiceFactoryOnce = Factory<Box<dyn FnOnce(&Services) -> Result<ErasedArc, Error>>>;

#[derive(Clone)]
pub struct Factory<F> {
    alloc: ServiceAllocator<F>,
}

#[derive(Clone)]
enum ServiceAllocator<F> {
    Static {
        fun: fn(&Services) -> Result<ErasedArc, Error>,
    },
    Fn {
        fun: F,
    },
}

impl<T> Debug for Factory<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let stringified = match self.alloc {
            ServiceAllocator::Static { .. } => "Static",
            ServiceAllocator::Fn { .. } => "Closure",
        };

        write!(f, "{}", stringified)
    }
}

impl<F> Factory<F> {
    pub fn from_container<T: 'static + FromDi>() -> Self {
        let wrapper = move |x: &_| {
            let instance = T::produce(x)?;
            let erased = ErasedArc::from_instance(instance);

            Ok(erased)
        };

        Self {
            alloc: ServiceAllocator::Static { fun: wrapper },
        }
    }

    pub fn from_default<T: 'static + Default>() -> Self {
        Self {
            alloc: ServiceAllocator::Static {
                fun: |_| Ok(ErasedArc::from_instance(T::default())),
            },
        }
    }
}

impl ServiceFactory {
    pub fn from_fn<T: 'static, Args>(allocator: impl DiFactory<T, Args>) -> Self {
        let wrapper = move |x: &_| {
            let instance = DiFactory::<T, Args>::produce(&allocator, x)?;
            let erased = ErasedArc::from_instance(instance);

            Ok(erased)
        };

        Self {
            alloc: ServiceAllocator::Fn {
                fun: Arc::new(wrapper),
            },
        }
    }

    pub fn produce(&self, services: &Services) -> Result<ErasedArc, Error> {
        match &self.alloc {
            ServiceAllocator::Static { fun } => fun(services),
            ServiceAllocator::Fn { fun } => fun(services),
        }
    }
}

impl ServiceFactoryOnce {
    pub fn from_fn_once<T: 'static, Args>(allocator: impl DiFactoryOnce<T, Args>) -> Self {
        let wrapper = move |x: &_| {
            let instance = DiFactoryOnce::<T, Args>::produce(allocator, x)?;
            let erased = ErasedArc::from_instance(instance);

            Ok(erased)
        };

        Self {
            alloc: ServiceAllocator::Fn {
                fun: Box::new(wrapper),
            },
        }
    }

    pub fn produce(self, services: &Services) -> Result<ErasedArc, Error> {
        match self.alloc {
            ServiceAllocator::Static { fun } => fun(services),
            ServiceAllocator::Fn { fun } => fun(services),
        }
    }
}
