use std::any::{TypeId, type_name};
use std::cmp::Ordering;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};

/// A wrapper over TypeId that also stores a type name, providing better debugging experience.
#[derive(Clone, Copy)]
pub struct TypeMeta {
    type_id: TypeId,
    type_name: TypeName,
}

#[derive(Clone, Copy)]
enum TypeName {
    Hardcoded(&'static str),
    Dynamic(fn() -> &'static str),
}

impl TypeMeta {
    pub const fn of<T: ?Sized + 'static>() -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            // Const type_name is unstable so using this workaround
            type_name: TypeName::Dynamic(|| type_name::<T>()),
        }
    }

    pub const fn of_name<T: ?Sized + 'static>(name: &'static str) -> Self {
        Self {
            type_id: TypeId::of::<T>(),
            type_name: TypeName::Hardcoded(name),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self.type_name {
            TypeName::Hardcoded(x) => x,
            TypeName::Dynamic(x) => x(),
        }
    }
}

impl Debug for TypeMeta {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.type_name())
    }
}

impl Eq for TypeMeta {}

impl PartialEq<Self> for TypeMeta {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl PartialOrd for TypeMeta {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.type_id.partial_cmp(&other.type_id)
    }
}

impl Ord for TypeMeta {
    fn cmp(&self, other: &Self) -> Ordering {
        self.type_id.cmp(&other.type_id)
    }
}

impl Hash for TypeMeta {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.type_id.hash(state)
    }
}
