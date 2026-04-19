use crate::builder::ServicesBuilder;
use crate::builder::{BindingKind, BindingLifetime};
use crate::either_iter::EitherIter;
use crate::mbmany::OneOrMany;
use crate::resolver::Error;
use crate::resolver::graph;
use crate::resolver::resolved::Resolved;
use crate::runtime::ServiceFactory;
use crate::runtime::TypeMeta;
use crate::runtime::{ErasedArc, ErasedUnsizer};
use std::collections::HashMap;
use std::iter;

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
        impls: OneOrMany<(ErasedUnsizer, ImmutableTypeBinding)>,
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
    fn from(value: BindingLifetime, services: &Services) -> Result<Self, Error> {
        let binding = match value {
            BindingLifetime::Singleton(x) => Self::Singleton(x.to_instance(services)?),
            BindingLifetime::Transient(x) => Self::Transient(x),
        };

        Ok(binding)
    }
}

impl Services {
    /// Attempts to make an instance of Services from a ServiceBuilder instance.
    pub fn from_builder(builder: ServicesBuilder) -> Result<Self, Error> {
        let bindings = builder.to_vec();

        let bindings = graph::merge_aliases(bindings);
        let bindings = graph::resolve_order(bindings)?;

        // Bindings that shouldn't be exposed under their types
        // will be removed after construction
        let unbind: Vec<_> = bindings
            .iter()
            .filter_map(|x| match x.kind {
                BindingKind::Type { bind_self, .. } if !bind_self => Some(x.ty),
                _ => None,
            })
            .collect();

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
                    ImmutableBinding::Alias {
                        impls: impls
                            .iter()
                            .map(|(meta, unsizer)| {
                                // Bindings are sorted, so it's guaranteed that all
                                // underlying types are already presented in the map
                                let binding = services.services.get(meta).unwrap();

                                (unsizer.clone(), binding.unwrap_type().clone())
                            })
                            .collect(),
                    }
                }
            };

            services.services.insert(ty, binding);
        }

        // Removing unexposed bindings as they're
        // not needed for resolution anymore
        for ty in unbind {
            services.services.remove(&ty);
        }

        Ok(services)
    }
}

impl Services {
    /// Returns a lazy iterator over all members of type T.
    /// If T is concrete, the iter will always contain a single element.
    pub fn resolve_all<T>(&self) -> Option<impl DoubleEndedIterator<Item = Resolved<T>>>
    where
        T: ?Sized + 'static,
    {
        let type_meta = TypeMeta::of::<T>();

        self.services.get(&type_meta).map(|x| match x {
            ImmutableBinding::Type { binding } => {
                let iter = iter::once(binding).map(|binding| {
                    self.get_instance(binding)
                        .coerce::<T>()
                        .expect("Failed to coerce. This is a bug!")
                });

                EitherIter::Left(iter)
            }
            ImmutableBinding::Alias { impls } => {
                let iter = impls.into_iter().map(|(unsizer, binding)| {
                    let arc = self.get_instance(binding);

                    unsizer
                        .unsize::<T>(arc)
                        .expect("Failed to unsize. This is a bug!")
                });

                EitherIter::Right(iter)
            }
        })
    }

    /// Returns the last binding of type T. For concrete T, there is always a single instance.
    pub fn resolve<T: ?Sized + 'static>(&self) -> Option<Resolved<T>> {
        self.resolve_all().map(|mut x| x.next_back().unwrap())
    }

    fn get_instance(&self, binding: &ImmutableTypeBinding) -> ErasedArc {
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
