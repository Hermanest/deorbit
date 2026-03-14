use std::fmt::{Display, Formatter};
use crate::binding::TypeMeta;

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