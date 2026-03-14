use crate::binding::{Binding, TypeMeta};
use crate::error::Error;
use crate::graph::NodeState::{Visited, Visiting};
use std::any::TypeId;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

#[derive(Eq, PartialEq, Copy, Clone)]
enum NodeState {
    Unvisited,
    Visiting,
    Visited,
}

pub fn resolve_order(bindings: Vec<Binding>) -> Result<Vec<Binding>, Error> {
    let bindings: HashMap<_, _> = bindings
        .into_iter()
        .map(|x| {
            (
                x.ty,
                (RefCell::new(Some(x)), Cell::new(NodeState::Unvisited)),
            )
        })
        .collect();

    let mut ordered = Vec::new();

    for ty in bindings.keys() {
        resolve_recursive(ty, &bindings, &mut ordered)?;
    }

    Ok(ordered)
}

fn resolve_recursive(
    ty: &TypeMeta,
    keyed: &HashMap<TypeMeta, (RefCell<Option<Binding>>, Cell<NodeState>)>,
    ordered: &mut Vec<Binding>,
) -> Result<(), Error> {
    let (binding, state) = keyed.get(ty).ok_or_else(|| Error::Missing {
        type_meta: ty.clone(),
    })?;

    // This node was already handled, ignoring
    if state.get() == Visited {
        return Ok(());
    }

    // Means that the node in the current stack
    if state.get() == Visiting {
        return Err(Error::Circular { path: vec![] });
    }

    // Setting a temporary state so in case of a circular dependency
    // we'll see this node as being handled
    state.set(Visiting);

    for dep in binding.borrow().as_ref().unwrap().deps {
        resolve_recursive(&dep, keyed, ordered)?;
    }

    ordered.push(binding.take().unwrap());
    state.set(Visited);

    Ok(())
}
