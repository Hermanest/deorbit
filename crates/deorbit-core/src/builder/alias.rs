use crate::runtime::ErasedUnsizer;
use crate::{ServicesBuilder, TypeMeta};
use std::marker::PhantomData;
use std::sync::Arc;

#[derive(Debug)]
pub struct AliasBuilder<'a, Trait: ?Sized + 'static> {
    builder: &'a mut ServicesBuilder,
    impls: Vec<(TypeMeta, ErasedUnsizer)>,
    ph: PhantomData<Trait>,
}

impl<'a, Trait: ?Sized + Send + Sync + 'static> AliasBuilder<'a, Trait> {
    pub(crate) fn from_builder(builder: &'a mut ServicesBuilder) -> Self {
        Self {
            builder,
            impls: vec![],
            ph: PhantomData,
        }
    }

    pub fn to<T: Send + Sync + 'static>(mut self, unsize: fn(Arc<T>) -> Arc<Trait>) -> Self {
        // TODO: add proper option handling
        self.impls.push((
            TypeMeta::of::<T>(),
            ErasedUnsizer::try_from(unsize).unwrap(),
        ));

        self
    }

    pub fn done(self) {
        self.builder.add_alias_binding::<Trait>(self.impls);
    }
}
