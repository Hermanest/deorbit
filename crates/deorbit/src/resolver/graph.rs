use crate::TypeMeta;
use crate::builder::binding::{Binding, BindingKind};
use crate::resolver::Error;
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

pub fn merge_aliases(mut bindings: Vec<Binding>) -> Vec<Binding> {
    let mut cur_idx = 0;

    while cur_idx < bindings.len() {
        // Skip this binding if it's not an alias
        if matches!(bindings[cur_idx].kind, BindingKind::Alias { .. }) {
            cur_idx += 1;
            continue;
        }

        let cur_ty = bindings[cur_idx].ty.clone();
        let mut inner_idx = cur_idx + 1;

        while inner_idx < bindings.len() {
            let binding = &bindings[inner_idx];

            if cur_ty == binding.ty
                && let BindingKind::Alias { .. } = binding.kind
            {
                // Removing the duplicated alias
                let binding = bindings.remove(inner_idx);
                let impls = binding.kind.unwrap_alias();

                // Extending the original alias with the data from the duplicated one
                bindings[cur_idx].kind.unwrap_alias_mut().extend(impls);
            } else {
                inner_idx += 1;
            }
        }

        cur_idx += 1;
    }

    bindings
}

pub fn resolve_order(mut bindings: Vec<Binding>) -> Result<Vec<Binding>, Error> {
    let mut mapped = HashMap::new();

    for binding in bindings.drain(0..) {
        let ty = binding.ty.clone();

        let node = Node {
            binding: RefCell::new(Some(binding)),
            state: Cell::new(NodeState::Unvisited),
        };

        if mapped.insert(ty, node).is_some() {
            return Err(Error::Duplicated { type_meta: ty });
        }
    }

    for ty in mapped.keys() {
        resolve_recursive(ty, &mapped, &mut bindings)?;
    }

    Ok(bindings)
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
    if node.state.get() == NodeState::Visited {
        return Ok(());
    }

    // Means that the node in the current stack
    if node.state.get() == NodeState::Visiting {
        return Err(Error::Circular { path: vec![*ty] });
    }

    // Setting a temporary state so in case of a circular dependency
    // we'll see this node as being handled
    node.state.set(NodeState::Visiting);

    let visit = |dep: &TypeMeta| {
        let result = resolve_recursive(&dep, keyed, ordered);

        result.map_err(|x| match x {
            Error::Circular { mut path } => {
                path.insert(0, *ty);

                Error::Circular { path }
            }
            x => x,
        })
    };

    match &node.binding.borrow().as_ref().unwrap().kind {
        BindingKind::Type { deps, .. } => {
            deps.iter().try_for_each(visit)?;
        }
        BindingKind::Alias { impls } => {
            impls.keys().try_for_each(visit)?;
        }
    }

    ordered.push(node.binding.take().unwrap());
    node.state.set(NodeState::Visited);

    Ok(())
}
