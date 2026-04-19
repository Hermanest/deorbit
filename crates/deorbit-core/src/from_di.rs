use crate::resolver::Services;
use crate::runtime::TypeMeta;

/// Represents an object that's capable of building itself from a DI instance.
pub trait FromDi: Sized {
    fn depends_on() -> &'static [TypeMeta];
    fn produce(services: &Services) -> Self;
}

/// Represents an object that's capable of building T from a DI instance.
pub trait DiFactory<T, Args>: DiFactoryOnce<T, Args> {
    fn produce(&self, services: &Services) -> T;
}

/// Represents an object that's capable of building T from a DI instance.
pub trait DiFactoryOnce<T, Args>: 'static {
    fn depends_on() -> &'static [TypeMeta];
    fn produce(self, services: &Services) -> T;
}

macro_rules! impl_factory_once {
    ($($args:ident),*) => {
        impl<F, O, $($args),*> DiFactoryOnce<O, ($($args),*,)> for F
        where
            $($args: $crate::FromDi,)*
            F: FnOnce($($args),*) -> O + 'static,
            O: 'static,
            $($args: 'static),*
        {
            fn depends_on() -> &'static [$crate::runtime::TypeMeta] {
                const {
                    &[
                        $($crate::runtime::TypeMeta::of::<$args>()),*
                    ]
                }
            }

            fn produce(self, services: &$crate::Services) -> O {
                (self)(
                    $($args::produce(services)),*
                )
            }
        }
    };
}

macro_rules! impl_factory {
    ($($args:ident),*) => {
        impl<F, O, $($args),*> DiFactory<O, ($($args),*,)> for F
        where
            $($args: $crate::FromDi,)*
            F: Fn($($args),*) -> O + 'static,
            O: 'static,
            $($args: 'static),*
        {
            fn produce(&self, services: &$crate::Services) -> O {
                (&self)(
                    $($args::produce(services)),*
                )
            }
        }
    };
}

macro_rules! impl_all_for {
    ($macro:ident) => {
        $macro!(T1);
        $macro!(T1, T2);
        $macro!(T1, T2, T3);
        $macro!(T1, T2, T3, T4);
        $macro!(T1, T2, T3, T4, T5);
        $macro!(T1, T2, T3, T4, T5, T6);
        $macro!(T1, T2, T3, T4, T5, T6, T7);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8);
        $macro!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
    };
}

impl_all_for!(impl_factory_once);
impl_all_for!(impl_factory);

impl<F, O> DiFactoryOnce<O, ()> for F
where
    F: FnOnce() -> O + 'static,
    O: 'static,
{
    fn depends_on() -> &'static [TypeMeta] {
        &[]
    }

    fn produce(self, _: &Services) -> O {
        self()
    }
}


impl<F, O> DiFactory<O, ()> for F
where
    F: Fn() -> O + 'static,
    O: 'static,
{
    fn produce(&self, _: &Services) -> O {
        self()
    }
}
