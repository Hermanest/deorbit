use crate::runtime::ErasedUnsizer;
use crate::{ServicesBuilder, TypeMeta};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Debug)]
pub struct AliasBuilder<'a, Trait: ?Sized + 'static> {
    builder: &'a mut ServicesBuilder,
    impls: HashMap<TypeMeta, ErasedUnsizer>,
    ph: PhantomData<Trait>,
}

impl<'a, Trait: ?Sized + 'static> AliasBuilder<'a, Trait> {
    pub(crate) fn from_builder(builder: &'a mut ServicesBuilder) -> Self {
        Self {
            builder,
            impls: HashMap::new(),
            ph: PhantomData,
        }
    }

    pub fn to<T: 'static>(mut self, unsize: fn(Arc<T>) -> Arc<Trait>) -> Self {
        // TODO: add proper option handling
        self.impls
            .entry(TypeMeta::of::<T>())
            .or_insert_with(|| ErasedUnsizer::try_from(unsize).unwrap());

        self
    }

    pub fn done(self) {
        self.builder.add_alias_binding::<Trait>(self.impls);
    }
}
