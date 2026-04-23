use crate::DiFactoryOnce;
use crate::builder::BindingLifetime;
use crate::builder::ServicesBuilder;
use crate::from_di::{DiFactory, FromDi};
use std::marker::PhantomData;

/// The outermost builder containing no info but the type.
#[derive(Debug)]
pub struct ConcreteBuilder<'a, T: 'static> {
    builder: &'a mut ServicesBuilder,
    bind_self: bool,
    ph: PhantomData<T>,
}

impl<'a, T: 'static> ConcreteBuilder<'a, T> {
    pub(crate) fn from_builder(builder: &'a mut ServicesBuilder) -> Self {
        Self {
            builder,
            bind_self: true,
            ph: PhantomData,
        }
    }

    /// Excludes the type itself from the binding. Useful when
    /// you need to bind a trait but not the type itself.
    pub fn not_self(mut self) -> Self {
        self.bind_self = false;
        self
    }

    /// Transfers the flow to a singleton builder.
    pub fn singleton(self) -> SingletonConcreteBuilder<'a, T> {
        SingletonConcreteBuilder { builder: self }
    }

    /// Transfers the flow to a transient builder.
    pub fn transient(self) -> TransientConcreteBuilder<'a, T> {
        TransientConcreteBuilder { builder: self }
    }
}

/// A singleton lifetime builder for a type.
pub struct SingletonConcreteBuilder<'a, T: 'static> {
    builder: ConcreteBuilder<'a, T>,
}

impl<'a, T: Send + Sync + 'static> SingletonConcreteBuilder<'a, T> {
    /// Finalizes the binding with an instance.
    pub fn from(self, instance: T) {
        self.builder.builder.add_type_binding::<T>(
            self.builder.bind_self,
            BindingLifetime::singleton_from(instance),
            &[],
        );
    }
    /// Finalizes the binding with an instance resolved via the specified factory.
    pub fn from_fn<F: DiFactoryOnce<T, Args>, Args>(self, factory: F) {
        self.builder.builder.add_type_binding::<T>(
            self.builder.bind_self,
            BindingLifetime::singleton_from_fn(factory),
            F::depends_on(),
        );
    }
}

#[allow(clippy::wrong_self_convention)]
impl<'a, T: Send + Sync + Default + 'static> SingletonConcreteBuilder<'a, T> {
    /// Finalizes the binding with a default value.
    pub fn from_default(self) {
        self.builder.builder.add_type_binding::<T>(
            self.builder.bind_self,
            BindingLifetime::singleton_from_default::<T>(),
            &[],
        );
    }
}

#[allow(clippy::wrong_self_convention)]
impl<'a, T: Send + Sync + FromDi + 'static> SingletonConcreteBuilder<'a, T> {
    /// Finalizes the binding with an automatically resolved instance.
    pub fn from_di(self) {
        self.builder.builder.add_type_binding::<T>(
            self.builder.bind_self,
            BindingLifetime::singleton_from_di::<T>(),
            T::depends_on(),
        );
    }
}

/// A transient lifetime builder for a type.
pub struct TransientConcreteBuilder<'a, T: 'static> {
    builder: ConcreteBuilder<'a, T>,
}

#[allow(clippy::wrong_self_convention)]
impl<'a, T: Send + Sync + 'static> TransientConcreteBuilder<'a, T> {
    /// Finalizes the binding with an instance resolved via the specified factory.
    pub fn from_fn<F: DiFactory<T, Args>, Args>(self, factory: F) {
        self.builder.builder.add_type_binding::<T>(
            self.builder.bind_self,
            BindingLifetime::transient_from_fn(factory),
            F::depends_on(),
        );
    }
}

#[allow(clippy::wrong_self_convention)]
impl<'a, T: Send + Sync + FromDi + 'static> TransientConcreteBuilder<'a, T> {
    /// Finalizes the binding with an automatically resolved instance.
    pub fn from_di(self) {
        self.builder.builder.add_type_binding::<T>(
            self.builder.bind_self,
            BindingLifetime::transient_from_di::<T>(),
            T::depends_on(),
        );
    }
}

#[allow(clippy::wrong_self_convention)]
impl<'a, T: Send + Sync + Default + 'static> TransientConcreteBuilder<'a, T> {
    /// Finalizes the binding with a default value.
    pub fn from_default(self) {
        self.builder.builder.add_type_binding::<T>(
            self.builder.bind_self,
            BindingLifetime::transient_from_default::<T>(),
            &[],
        );
    }
}
