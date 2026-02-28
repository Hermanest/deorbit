use crate::arc::TypedArc;
use crate::binder::Binder;
use crate::builder::ServicesBuilder;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::Arc;

pub type Service<T> = Arc<T>;

type ServiceMap = HashMap<TypeId, (TypedArc, Binder)>;
type ServiceError = String;

/// A collection of services.
pub struct Services {
    services: ServiceMap,
}

impl Services {
    pub fn from_builder(builder: ServicesBuilder) -> Result<Self, ServiceError> {
        let bindings = Self::build_template_map(builder)?;
        let this = Self { services: bindings };

        for (_, (arc, binder)) in &this.services {
            binder.finalize(arc.clone(), &this)?;
        }

        Ok(this)
    }

    fn build_template_map(builder: ServicesBuilder) -> Result<ServiceMap, ServiceError> {
        let mut map = HashMap::new();

        for binding in builder.binders {
            let entry = map.entry(binding.type_id);

            if matches!(entry, Entry::Occupied(_)) {
                return Err(format!(
                    "Type {} was registered more than once",
                    binding.type_name
                ));
            } else {
                let instance = binding.get_template();

                entry.insert_entry((instance, binding));
            }
        }

        Ok(map)
    }
}

impl Services {
    pub fn resolve<T: Any>(&self) -> Option<Service<T>> {
        let type_id = TypeId::of::<T>();

        self.services
            .get(&type_id)
            .map(|x| x.0.downcast::<T>().unwrap())
    }
}
