use crate::binding::{Binding, TypeMeta};
use crate::error::Error;
use crate::graph::NodeState::{Visited, Visiting};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

#[derive(Eq, PartialEq, Copy, Clone)]
enum NodeState {
    Unvisited,
    Visiting,
    Visited,
}

struct Node {
    binding: RefCell<Option<Binding>>,
    state: Cell<NodeState>,
}

pub fn resolve_order(bindings: Vec<Binding>) -> Result<Vec<Binding>, Error> {
    let bindings: HashMap<_, _> = bindings
        .into_iter()
        .map(|x| {
            (
                x.ty,
                Node {
                    binding: RefCell::new(Some(x)),
                    state: Cell::new(NodeState::Unvisited),
                },
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
    keyed: &HashMap<TypeMeta, Node>,
    ordered: &mut Vec<Binding>,
) -> Result<(), Error> {
    let node = keyed.get(ty).ok_or_else(|| Error::Missing {
        type_meta: ty.clone(),
    })?;

    // This node was already handled, ignoring
    if node.state.get() == Visited {
        return Ok(());
    }

    // Means that the node in the current stack
    if node.state.get() == Visiting {
        return Err(Error::Circular { path: vec![] });
    }

    // Setting a temporary state so in case of a circular dependency
    // we'll see this node as being handled
    node.state.set(Visiting);

    for dep in node.binding.borrow().as_ref().unwrap().deps {
        resolve_recursive(&dep, keyed, ordered)?;
    }

    ordered.push(node.binding.take().unwrap());
    node.state.set(Visited);

    Ok(())
}
