use crate::Services;
use crate::builder::alias::AliasBuilder;
use crate::builder::concrete::ConcreteBuilder;
use crate::builder::{Binding, BindingKind, BindingLifetime};
use crate::resolver::Error;
use crate::runtime::{ErasedUnsizer, TypeMeta};

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
    pub fn bind<T: 'static>(&mut self) -> ConcreteBuilder<'_, T> {
        ConcreteBuilder::from_builder(self)
    }

    pub fn bind_alias<T: ?Sized + Send + Sync + 'static>(&mut self) -> AliasBuilder<'_, T> {
        AliasBuilder::from_builder(self)
    }

    pub(crate) fn add_type_binding<T: 'static>(
        &mut self,
        bind_self: bool,
        lifetime: BindingLifetime,
        deps: &'static [TypeMeta],
    ) {
        let binding = Binding {
            ty: TypeMeta::of::<T>(),
            kind: BindingKind::Type {
                lifetime,
                deps,
                bind_self,
            },
        };

        self.bindings.push(binding);
    }

    pub(crate) fn add_alias_binding<T: ?Sized + 'static>(
        &mut self,
        impls: Vec<(TypeMeta, ErasedUnsizer)>,
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
