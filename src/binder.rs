use crate::arc::TypedArc;
use crate::from_di::FromDiVtable;
use crate::services::Services;
use std::any::TypeId;

pub(crate) struct Binder {
    pub type_id: TypeId,
    pub type_name: &'static str,
    pub resolution_kind: ResolutionKind,
}

pub(crate) enum ResolutionKind {
    /// Load an instance provided manually.
    Manual(TypedArc),
    /// Create a new instance automatically.
    Automatic {
        type_size: usize,
        di_vtable: FromDiVtable,
    },
}

impl Binder {
    pub fn get_template(&self) -> TypedArc {
        match self.resolution_kind {
            ResolutionKind::Manual(ref x) => x.clone(),

            ResolutionKind::Automatic { type_size, .. } => {
                TypedArc::from_size(self.type_id, type_size)
            }
        }
    }

    pub fn finalize(&self, arc: TypedArc, services: &Services) -> Result<(), String> {
        match &self.resolution_kind {
            ResolutionKind::Automatic { di_vtable, .. } => unsafe {
                (di_vtable.inject)(arc.to_uninit(), services)
            },
            _ => Ok(()),
        }
    }
}
