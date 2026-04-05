use crate::binding::{BindingKind, ServiceLifetime, SingletonProvider};
use crate::builder::ServicesBuilder;
use crate::resolver::Error;
use crate::resolver::graph;
use crate::runtime::ServiceFactory;
use crate::runtime::TypeMeta;
use crate::runtime::{ErasedArc, ErasedUnsizer};
use std::collections::HashMap;
use std::sync::Arc;

pub type Service<T> = Arc<T>;

/// A collection of services.
#[derive(Debug)]
pub struct Services {
    services: HashMap<TypeMeta, ImmutableBinding>,
}

#[derive(Debug)]
enum ImmutableBinding {
    Type {
        binding: ImmutableTypeBinding,
    },
    Alias {
        // TODO: change to OneOrMany to remove a level of indirection
        impls: Vec<(ErasedUnsizer, ImmutableTypeBinding)>,
    },
}

impl ImmutableBinding {
    pub fn unwrap_type(&self) -> &ImmutableTypeBinding {
        match self {
            ImmutableBinding::Type { binding } => binding,
            ImmutableBinding::Alias { .. } => {
                panic!("Called unwrap_type on an Alias binding")
            }
        }
    }
}

#[derive(Debug, Clone)]
enum ImmutableTypeBinding {
    Singleton(ErasedArc),
    Transient(ServiceFactory),
}

impl ImmutableTypeBinding {
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
        let bindings = builder.to_vec();

        let bindings = graph::merge_aliases(bindings);
        let bindings = graph::resolve_order(bindings)?;

        let mut services = Self {
            services: HashMap::new(),
        };

        for binding in bindings {
            let ty = binding.ty;

            let binding = match binding.kind {
                BindingKind::Type { lifetime, .. } => ImmutableBinding::Type {
                    binding: ImmutableTypeBinding::from(lifetime, &services)?,
                },
                BindingKind::Alias { impls } => {
                    let impls = impls
                        .iter()
                        .map(|(meta, unsizer)| {
                            let binding = services.services.get(meta).unwrap();

                            (unsizer.clone(), binding.unwrap_type().clone())
                        })
                        .collect();

                    ImmutableBinding::Alias { impls }
                }
            };

            services.services.insert(ty, binding);
        }

        Ok(services)
    }
}

impl Services {
    pub fn resolve<T: ?Sized + 'static>(&self) -> Option<Service<T>> {
        let type_meta = TypeMeta::of::<T>();

        self.services.get(&type_meta).map(|x| match x {
            ImmutableBinding::Type { binding } => self.resolve_type(binding).coerce::<T>().unwrap(),
            ImmutableBinding::Alias { impls } => {
                // Takes the last implementation
                let (unsizer, binding) = impls.last().unwrap();
                let arc = self.resolve_type(binding);

                unsizer.unsize(arc).expect("Failed while trying to unsize")
            }
        })
    }

    fn resolve_type(&self, binding: &ImmutableTypeBinding) -> ErasedArc {
        match binding {
            ImmutableTypeBinding::Singleton(arc) => arc.clone(),
            // Transient is designed to panic because dependencies are validated beforehand,
            // so if an error has occurred after validation it's not a dependency resolution issue anymore
            ImmutableTypeBinding::Transient(factory) => factory
                .produce(self)
                .expect("Failed while trying to instantiate Transient"),
        }
    }
}
