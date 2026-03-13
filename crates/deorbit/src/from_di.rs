use crate::builder::FromDiFactoryOnce;
use crate::{Service, Services};
use std::any::{Any, TypeId};
use std::sync::OnceLock;

impl<T: 'static> FromDiFactoryOnce<T, ()> for T {
    fn depends_on() -> &'static [TypeId] {
        &[]
    }

    fn produce(self, _: &Services) -> T {
        self
    }
}

impl<F, O, T1> FromDiFactoryOnce<O, (T1,)> for F
where
    F: Fn(Service<T1>) -> O + 'static,
    O: 'static,
    T1: 'static,
{
    fn depends_on() -> &'static [TypeId] {
        static DEPS: OnceLock<[TypeId; 1]> = OnceLock::new();

        DEPS.get_or_init(|| [TypeId::of::<T1>()])
    }

    fn produce(self, services: &Services) -> O {
        self(services.resolve::<T1>().unwrap())
    }
}

impl<F, O, T1, T2> FromDiFactoryOnce<O, (T1, T2)> for F
where
    F: Fn(Service<T1>, Service<T2>) -> O + 'static,
    O: 'static,
    T1: 'static,
    T2: 'static,
{
    fn depends_on() -> &'static [TypeId] {
        static DEPS: OnceLock<[TypeId; 2]> = OnceLock::new();

        DEPS.get_or_init(|| [TypeId::of::<T1>(), TypeId::of::<T2>()])
    }

    fn produce(self, services: &Services) -> O {
        self(services.resolve().unwrap(), services.resolve().unwrap())
    }
}
