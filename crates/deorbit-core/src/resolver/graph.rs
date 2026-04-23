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
        if !matches!(bindings[cur_idx].kind, BindingKind::Alias { .. }) {
            cur_idx += 1;
            continue;
        }

        let cur_ty = bindings[cur_idx].ty.clone();
        let mut inner_idx = cur_idx + 1;

        while inner_idx < bindings.len() {
            let binding = &bindings[inner_idx];

            if cur_ty == binding.ty && matches!(binding.kind, BindingKind::Alias { .. }) {
                // Removing the duplicated alias
                let binding = bindings.remove(inner_idx);
                let impls = binding.kind.unwrap_alias();

                // Extending the original alias with the data from the duplicated one
                bindings[cur_idx].kind.unwrap_alias_mut().extend(impls);
            } else {
                inner_idx += 1;
            }
        }

        // Removing duplicated alias members
        let vec = bindings[cur_idx].kind.unwrap_alias_mut();
        remove_duplicates(vec, |(x, _)| x);

        cur_idx += 1;
    }

    bindings
}

fn remove_duplicates<T, K: Eq>(vec: &mut Vec<T>, selector: impl Fn(&T) -> &K) {
    let mut i = 0;

    while i < vec.len() {
        let mut j = i + 1;

        while j < vec.len() {
            if selector(&vec[i]) == selector(&vec[j]) {
                vec.remove(j);
            } else {
                j += 1;
            }
        }

        i += 1;
    }
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
            impls.iter().map(|(ty, _)| ty).try_for_each(visit)?;
        }
    }

    ordered.push(node.binding.take().unwrap());
    node.state.set(NodeState::Visited);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::ErasedUnsizer;
    use std::any::Any;
    use std::sync::Arc;

    #[test]
    fn test_merges_aliases() {
        let bindings = vec![
            Binding {
                ty: TypeMeta::of::<dyn Any>(),
                kind: BindingKind::Alias {
                    impls: vec![(
                        TypeMeta::of::<i32>(),
                        ErasedUnsizer::try_from(|x: Arc<i32>| x as Arc<dyn Any + Send + Sync>)
                            .unwrap(),
                    )],
                },
            },
            Binding {
                ty: TypeMeta::of::<dyn Any>(),
                kind: BindingKind::Alias {
                    impls: vec![(
                        TypeMeta::of::<i64>(),
                        ErasedUnsizer::try_from(|x: Arc<i64>| x as Arc<dyn Any + Send + Sync>)
                            .unwrap(),
                    )],
                },
            },
            Binding {
                ty: TypeMeta::of::<dyn Any>(),
                kind: BindingKind::Alias {
                    impls: vec![(
                        TypeMeta::of::<i128>(),
                        ErasedUnsizer::try_from(|x: Arc<i128>| x as Arc<dyn Any + Send + Sync>)
                            .unwrap(),
                    )],
                },
            },
        ];

        let bindings = merge_aliases(bindings);

        assert_eq!(bindings.len(), 1);

        if let BindingKind::Alias { ref impls } = bindings[0].kind {
            assert_eq!(impls[0].0, TypeMeta::of::<i32>());
            assert_eq!(impls[1].0, TypeMeta::of::<i64>());
            assert_eq!(impls[2].0, TypeMeta::of::<i128>());
        } else {
            panic!("Binding is not an alias");
        }
    }
}
