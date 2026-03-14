use crate::binding::{Binding, ServiceLifetime, TypeMeta};
use crate::error::Error;
use crate::factory::{ManagedService, ServiceFactory};
use crate::services::Services;
use std::any::Any;

/// Represents an object that's capable of building T from a DI instance.
pub trait FromDi: Sized {
    fn depends_on() -> &'static [TypeMeta];
    fn produce(services: &Services) -> Self;
}

/// Represents an object that's capable of building itself from a DI instance.
pub trait FromDiFactory<T>: 'static {
    fn depends_on() -> &'static [TypeMeta];
    fn produce(&self, services: &Services) -> T;
}

/// Represents an object that's capable of building itself from a DI instance.
pub trait FromDiFactoryOnce<T, D>: 'static {
    fn depends_on() -> &'static [TypeMeta];
    fn produce(self, services: &Services) -> T;
}

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

    pub fn bind_singleton_from<T: Any, F: FromDiFactoryOnce<T, D>, D>(&mut self, instance: F) {
        let instance = ManagedService::from(instance);
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

    pub fn bind_transient_from_fn<T: Any, K: FromDiFactory<T>>(&mut self, factory: K) {
        let factory = ServiceFactory::from_fn(factory);
        let lifetime = ServiceLifetime::Transient(factory);

        let binding = Self::make_binding::<T>(lifetime, K::depends_on());

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
