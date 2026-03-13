use crate::binding::{Binding, TypeMeta};
use crate::graph::NodeState::{Visited, Visiting};
use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    Circular { path: Vec<TypeMeta> },
    Missing { type_meta: TypeMeta },
    Duplicated { type_meta: TypeMeta },
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Circular { path } => {
                let separator = " -> ";

                let avg_name_len = 8;
                let est_capacity = size_of_val(separator) + avg_name_len * path.len();

                // Trying to avoid redundant allocs as much as possible
                let mut buffer = String::with_capacity(est_capacity);

                for (idx, meta) in path.iter().enumerate() {
                    if idx > 0 {
                        buffer += separator;
                    }

                    buffer += meta.type_name;
                }

                write!(
                    f,
                    "Failed to resolve the graph due to a circular dependency ({})",
                    buffer
                )
            }
            Error::Missing { type_meta } => {
                write!(
                    f,
                    "Failed to resolve the graph due to a missing service of type {}",
                    type_meta.type_name
                )
            }
            Error::Duplicated { type_meta } => {
                write!(
                    f,
                    "Failed to resolve the graph due to a duplicated binding of type {}",
                    type_meta.type_name
                )
            }
        }
    }
}

/// A struct responsible for service resolution logic.
pub struct ServiceGraph;

#[derive(Eq, PartialEq, Copy, Clone)]
enum NodeState {
    Unvisited,
    Visiting,
    Visited,
}

impl ServiceGraph {
    pub fn build(bindings: Vec<Binding>) -> Result<Vec<Binding>, Error> {
        let bindings: HashMap<_, _> = bindings
            .into_iter()
            .map(|x| (x.ty.type_id, (RefCell::new(Some(x)), Cell::new(NodeState::Unvisited))))
            .collect();

        let mut ordered = Vec::new();

        for ty in bindings.keys() {
            Self::build_recursive(ty, &bindings, &mut ordered)?;
        }

        Ok(ordered)
    }

    fn build_recursive(
        ty: &TypeId,
        keyed: &HashMap<TypeId, (RefCell<Option<Binding>>, Cell<NodeState>)>,
        ordered: &mut Vec<Binding>,
    ) -> Result<(), Error> {
        let (binding, state) = keyed.get(ty).unwrap();

        if state.get() == Visited {
            return Ok(());
        }

        if state.get() == Visiting {
            return Err(Error::Circular { path: vec![] });
        }

        state.set(Visiting);

        for dep in binding.borrow().as_ref().unwrap().deps {
            Self::build_recursive(&dep, keyed, ordered)?;
        }

        ordered.push(binding.take().unwrap());
        state.set(Visited);

        Ok(())
    }
}