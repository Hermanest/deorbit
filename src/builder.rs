use crate::arc::TypedArc;
use crate::binder::{Binder, ResolutionKind};
use crate::from_di::{FromDi, FromDiVtable};
use crate::services::Services;
use std::any::{Any, TypeId, type_name};

/// A builder for ServiceCollection.
pub struct ServicesBuilder {
    pub(crate) binders: Vec<Binder>,
}

impl Default for ServicesBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ServicesBuilder {
    pub fn new() -> Self {
        Self { binders: vec![] }
    }

    pub fn build(self) -> Result<Services, String> {
        Services::from_builder(self)
    }

    /// Binds a service using automatic instantiation.
    pub fn bind<T: Any + FromDi>(&mut self) {
        self.binders.push(Binder {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            resolution_kind: ResolutionKind::Automatic {
                type_size: size_of::<T>(),
                di_vtable: FromDiVtable::for_type::<T>()
            },
        })
    }

    /// Binds a service using a provided instance.
    pub fn bind_from<T: Any>(&mut self, instance: T) {
        let arc = TypedArc::from(instance);

        self.binders.push(Binder {
            type_id: TypeId::of::<T>(),
            type_name: type_name::<T>(),
            resolution_kind: ResolutionKind::Manual(arc),
        })
    }
}
