use crate::Services;
use crate::builder::{Binding, BindingKind, ServiceLifetime};
use crate::builder::alias::AliasBuilder;
use crate::builder::bind::BindingBuilder;
use crate::resolver::Error;
use crate::runtime::{ErasedUnsizer, TypeMeta};
use std::collections::HashMap;

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

    pub fn bind_alias<T: ?Sized + 'static>(&mut self) -> AliasBuilder<'_, T> {
        AliasBuilder::from_builder(self)
    }

    pub(crate) fn add_type_binding<T: 'static>(
        &mut self,
        lifetime: ServiceLifetime,
        deps: &'static [TypeMeta],
    ) {
        let binding = Binding {
            ty: TypeMeta::of::<T>(),
            kind: BindingKind::Type { lifetime, deps },
        };

        self.bindings.push(binding);
    }

    pub(crate) fn add_alias_binding<T: ?Sized + 'static>(
        &mut self,
        impls: HashMap<TypeMeta, ErasedUnsizer>,
    ) {
        let binding = Binding {
            ty: TypeMeta::of::<T>(),
            kind: BindingKind::Alias { impls },
        };

        self.bindings.push(binding);
    }

    pub(crate) fn to_vec(self) -> Vec<Binding> {
        self.bindings
    }
}
