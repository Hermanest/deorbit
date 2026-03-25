use crate::binding::{Binding, ServiceLifetime};
use crate::builder::bind::BindingBuilder;
use crate::error::Error;
use crate::{Services, TypeMeta};

/// A builder for Services.
#[derive(Default, Debug)]
pub struct ServicesBuilder {
    bindings: Vec<Binding>,
}

impl ServicesBuilder {
    pub fn new() -> Self {
        Self { bindings: vec![] }
    }

    pub fn build(self) -> Result<Services, Error> {
        Services::from_builder(self)
    }

    /// Binds a service using automatic instantiation.
    pub fn bind<T: 'static>(&mut self) -> BindingBuilder<'_, T> {
        BindingBuilder::from_builder(self)
    }

    pub(crate) fn add_binding<T: 'static>(&mut self, lifetime: ServiceLifetime, deps: &'static [TypeMeta]) {
        let binding = Binding {
            ty: TypeMeta::of::<T>(),
            lifetime,
            deps,
        };

        self.bindings.push(binding);
    }
    
    pub(crate) fn to_vec(self) -> Vec<Binding> {
        self.bindings
    }
}