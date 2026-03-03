use crate::services::Services;
use std::any::Any;
use std::mem::{MaybeUninit, transmute};

/// Represents an object that can be built from a DI instance.
pub trait FromDi: Sized {
    fn inject(instance: &mut MaybeUninit<Self>, services: &Services) -> Result<(), String>;
}

pub(crate) struct FromDiVtable {
    pub inject: fn(ptr: &mut MaybeUninit<()>, services: &Services) -> Result<(), String>,
}

impl FromDiVtable {
    pub fn for_type<T: Any + FromDi>() -> Self {
        Self {
            inject: |ptr, services| unsafe { T::inject(transmute(ptr), services) },
        }
    }
}
