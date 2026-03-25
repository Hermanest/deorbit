use crate::TypeMeta;
use crate::arc::ErasedArc;
use crate::binding::{ServiceLifetime, SingletonProvider};
use crate::builder::ServicesBuilder;
use crate::error::Error;
use crate::factory::ServiceFactory;
use crate::graph;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

pub type Service<T> = Arc<T>;

/// A collection of services.
#[derive(Debug)]
pub struct Services {
    services: HashMap<TypeMeta, ServiceEntry>,
}

/// A bound service.
#[derive(Debug)]
enum ServiceEntry {
    Singleton(ErasedArc),
    Transient(ServiceFactory),
}

impl ServiceEntry {
    fn from(value: ServiceLifetime, services: &Services) -> Result<Self, Error> {
        let entry = match value {
            ServiceLifetime::Singleton(x) => {
                let instance = match x {
                    SingletonProvider::Instance(x) => x,
                    SingletonProvider::Factory(x) => x.produce(services)?,
                };

                Self::Singleton(instance)
            }
            ServiceLifetime::Transient(x) => Self::Transient(x),
        };

        Ok(entry)
    }
}

impl Services {
    /// Attempts to make an instance of Services from a ServiceBuilder instance.
    pub fn from_builder(builder: ServicesBuilder) -> Result<Self, Error> {
        let sorted_bindings = graph::resolve_order(builder.to_vec())?;

        let mut services = Self {
            services: HashMap::new(),
        };

        for binding in sorted_bindings {
            let entry = ServiceEntry::from(binding.lifetime, &services)?;

            services.services.insert(binding.ty, entry);
        }

        Ok(services)
    }
}

impl Services {
    pub fn resolve<T: Any>(&self) -> Option<Service<T>> {
        let type_meta = TypeMeta::of::<T>();

        self.services
            .get(&type_meta)
            .map(|x| {
                match x {
                    ServiceEntry::Singleton(x) => x.clone(),

                    // Transient is designed to panic because dependencies are validated beforehand,
                    // so if an error has occurred after validation it's not a dependency resolution issue anymore
                    ServiceEntry::Transient(factory) => factory
                        .produce(self)
                        .expect("The binding has failed while trying to instantiate Transient"),
                }
            })
            .map(|x| x.coerce::<T>().unwrap())
    }
}
