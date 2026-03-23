use crate::binding::{Binding, ServiceLifetime, TypeMeta};
use crate::error::Error;
use crate::factory::{ManagedService, ServiceFactory};
use crate::services::Services;
use std::any::Any;
use crate::from_di::{DiFactory, DiFactoryOnce, FromDi};

/// A builder for Services.
#[derive(Default)]
pub struct ServicesBuilder {
    pub(crate) bindings: Vec<Binding>,
}

impl ServicesBuilder {
    pub fn new() -> Self {
        Self { bindings: vec![] }
    }

    pub fn build(self) -> Result<Services, Error> {
        Services::from_builder(self)
    }

    /// Binds a service using automatic instantiation.
    pub fn bind_singleton<T: Any + FromDi>(&mut self) {
        let factory = ServiceFactory::from_container::<T>();
        let lifetime = ServiceLifetime::singleton_resolved(factory);

        let binding = Self::make_binding::<T>(lifetime, T::depends_on());

        self.bindings.push(binding)
    }

    pub fn bind_singleton_from<T: Any, F: DiFactoryOnce<T, Args>, Args>(&mut self, instance: F) {
        let instance = ManagedService::from_instance(instance);
        let lifetime = ServiceLifetime::singleton_from(instance);

        let binding = Self::make_binding::<T>(lifetime, F::depends_on());

        self.bindings.push(binding)
    }

    pub fn bind_transient<T: Any + FromDi>(&mut self) {
        let factory = ServiceFactory::from_container::<T>();
        let lifetime = ServiceLifetime::Transient(factory);

        let binding = Self::make_binding::<T>(lifetime, T::depends_on());

        self.bindings.push(binding)
    }

    pub fn bind_transient_from<T: Any, F: DiFactory<T, Args>, Args>(&mut self, factory: F) {
        let factory = ServiceFactory::from_fn(factory);
        let lifetime = ServiceLifetime::Transient(factory);

        let binding = Self::make_binding::<T>(lifetime, F::depends_on());

        self.bindings.push(binding)
    }

    pub fn bind_transient_from_default<T: Any + Default>(&mut self) {
        let factory = ServiceFactory::from_default::<T>();
        let lifetime = ServiceLifetime::Transient(factory);

        let binding = Self::make_binding::<T>(lifetime, &[]);

        self.bindings.push(binding)
    }

    fn make_binding<T: Any>(lifetime: ServiceLifetime, deps: &'static [TypeMeta]) -> Binding {
        Binding {
            ty: TypeMeta::of::<T>(),
            lifetime,
            deps,
        }
    }
}
