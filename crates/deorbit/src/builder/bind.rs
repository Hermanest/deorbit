use crate::binding::ServiceLifetime;
use crate::builder::services::ServicesBuilder;
use crate::from_di::{DiFactory, FromDi};
use crate::runtime::ErasedUnsizer;
use crate::runtime::TypeMeta;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

/// The outermost builder containing no info but the type.
#[derive(Debug)]
pub struct BindingBuilder<'a, T: 'static> {
    builder: &'a mut ServicesBuilder,
    bind_self: bool,
    traits: HashMap<TypeMeta, Option<ErasedUnsizer>>,
    ph: PhantomData<T>,
}

impl<'a, T: 'static> BindingBuilder<'a, T> {
    pub(crate) fn from_builder(builder: &'a mut ServicesBuilder) -> Self {
        Self {
            builder,
            bind_self: false,
            traits: HashMap::new(),
            ph: PhantomData,
        }
    }

    /// Excludes the type itself from the binding. Useful when
    /// you need to bind a trait but not the type itself.
    pub fn not_self(mut self) -> Self {
        self.bind_self = false;
        self
    }

    /// Maps this type to the specified trait.
    pub fn to<Trait: ?Sized + 'static>(mut self, eval: fn(Arc<T>) -> Arc<Trait>) -> Self {
        self.traits
            .entry(TypeMeta::of::<Trait>())
            .or_insert_with(|| ErasedUnsizer::try_from(eval));

        self
    }

    /// Transfers the flow to a singleton builder.
    pub fn singleton(self) -> SingletonBindingBuilder<'a, T> {
        SingletonBindingBuilder { builder: self }
    }

    /// Transfers the flow to a transient builder.
    pub fn transient(self) -> TransientBindingBuilder<'a, T> {
        TransientBindingBuilder { builder: self }
    }
}

/// A singleton lifetime builder for a type.
pub struct SingletonBindingBuilder<'a, T: 'static> {
    builder: BindingBuilder<'a, T>,
}

impl<'a, T: 'static> SingletonBindingBuilder<'a, T> {
    /// Finalizes the binding with an instance.
    pub fn from(self, instance: T) {
        self.builder
            .builder
            .add_binding::<T>(ServiceLifetime::singleton_from(instance), &[]);
    }
}

#[allow(clippy::wrong_self_convention)]
impl<'a, T: Default + 'static> SingletonBindingBuilder<'a, T> {
    /// Finalizes the binding with a default value.
    pub fn from_default(self) {
        self.builder
            .builder
            .add_binding::<T>(ServiceLifetime::singleton_from_default::<T>(), &[]);
    }
}

#[allow(clippy::wrong_self_convention)]
impl<'a, T: FromDi + 'static> SingletonBindingBuilder<'a, T> {
    /// Finalizes the binding with an automatically resolved instance.
    pub fn from_di(self) {
        self.builder
            .builder
            .add_binding::<T>(ServiceLifetime::singleton_from_di::<T>(), T::depends_on());
    }
}

/// A transient lifetime builder for a type.
pub struct TransientBindingBuilder<'a, T: 'static> {
    builder: BindingBuilder<'a, T>,
}

#[allow(clippy::wrong_self_convention)]
impl<'a, T: 'static> TransientBindingBuilder<'a, T> {
    /// Finalizes the binding with an instance resolved via the specified factory.
    pub fn from_fn<F: DiFactory<T, Args>, Args>(self, factory: F) {
        self.builder
            .builder
            .add_binding::<T>(ServiceLifetime::transient_from_fn(factory), F::depends_on());
    }
}

#[allow(clippy::wrong_self_convention)]
impl<'a, T: FromDi + 'static> TransientBindingBuilder<'a, T> {
    /// Finalizes the binding with an automatically resolved instance.
    pub fn from_di(self) {
        self.builder
            .builder
            .add_binding::<T>(ServiceLifetime::transient_from_di::<T>(), T::depends_on());
    }
}

#[allow(clippy::wrong_self_convention)]
impl<'a, T: Default + 'static> TransientBindingBuilder<'a, T> {
    /// Finalizes the binding with a default value.
    pub fn from_default(self) {
        self.builder
            .builder
            .add_binding::<T>(ServiceLifetime::transient_from_default::<T>(), &[]);
    }
}
