use crate::binding::{ServiceLifetime, SingletonProvider};
use crate::builder::ServicesBuilder;
use crate::factory::{ManagedService, ServiceFactory};
use crate::graph;
use crate::graph::ServiceGraph;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

pub type Service<T> = Arc<T>;

/// A collection of services.
pub struct Services {
    services: HashMap<TypeId, ServiceEntry>,
}

/// A bound service.
enum ServiceEntry {
    Singleton(ManagedService),
    Transient(ServiceFactory),
}

impl Services {
    pub fn from_builder(builder: ServicesBuilder) -> Result<Self, graph::Error> {
        let sorted_bindings = ServiceGraph::build(builder.bindings)?;

        let mut services = Self {
            services: HashMap::new(),
        };

        for binding in sorted_bindings {
            let entry = match binding.lifetime {
                ServiceLifetime::Singleton(provider) => match provider {
                    SingletonProvider::Instance(instance) => ServiceEntry::Singleton(instance),

                    SingletonProvider::Factory(factory) => {
                        let instance =
                            factory
                                .produce(&services)
                                .map_err(|_| graph::Error::Missing {
                                    type_meta: binding.ty.clone(),
                                })?;

                        ServiceEntry::Singleton(instance)
                    }
                },

                ServiceLifetime::Transient(factory) => ServiceEntry::Transient(factory),
            };

            services.services.insert(binding.ty.type_id, entry);
        }

        Ok(services)
    }
}

impl Services {
    pub fn resolve<T: Any>(&self) -> Option<Service<T>> {
        let type_id = TypeId::of::<T>();

        self.services
            .get(&type_id)
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
            .map(|x| x.downcast::<T>().unwrap())
    }
}
