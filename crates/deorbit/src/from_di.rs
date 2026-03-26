use crate::resolver::Services;
use crate::runtime::TypeMeta;

/// Represents an object that's capable of building itself from a DI instance.
pub trait FromDi: Sized {
    fn depends_on() -> &'static [TypeMeta];
    fn produce(services: &Services) -> Self;
}

/// Represents an object that's capable of building T from a DI instance.
pub trait DiFactory<T, Args>: 'static {
    fn depends_on() -> &'static [TypeMeta];
    fn produce(&self, services: &Services) -> T;
}

/// Represents an object that's capable of building T from a DI instance.
pub trait DiFactoryOnce<T, Args>: 'static {
    fn depends_on() -> &'static [TypeMeta];
    fn produce(self, services: &Services) -> T;
}

macro_rules! impl_factory {
    ($name:ident, $closure:ident, ($($this:tt)+), ($($this_arg:tt)+), $($args:ident),*) => {
        impl<F, O, $($args),*> $name<O, ($($args),*,)> for F
        where
            F: $closure($($crate::Service<$args>),*) -> O + 'static,
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

            fn produce($($this)+, services: &$crate::Services) -> O {
                ($($this_arg)+)(
                    $(services.resolve::<$args>().expect("")),*
                )
            }
        }
    };
}

macro_rules! impl_all_for {
    ($name:ident, $closure:ident, ($($this:tt)+), ($($this_arg:tt)+)) => {
        impl_factory!($name, $closure, ($($this)+), ($($this_arg)+), T1);
        impl_factory!($name, $closure, ($($this)+), ($($this_arg)+), T1, T2);
        impl_factory!($name, $closure, ($($this)+), ($($this_arg)+), T1, T2, T3);
        impl_factory!($name, $closure, ($($this)+), ($($this_arg)+), T1, T2, T3, T4);
        impl_factory!($name, $closure, ($($this)+), ($($this_arg)+), T1, T2, T3, T4, T5);
        impl_factory!($name, $closure, ($($this)+), ($($this_arg)+), T1, T2, T3, T4, T5, T6);
        impl_factory!($name, $closure, ($($this)+), ($($this_arg)+), T1, T2, T3, T4, T5, T6, T7);
        impl_factory!($name, $closure, ($($this)+), ($($this_arg)+), T1, T2, T3, T4, T5, T6, T7, T8);
        impl_factory!($name, $closure, ($($this)+), ($($this_arg)+), T1, T2, T3, T4, T5, T6, T7, T8, T9);
    };
}

impl_all_for!(DiFactoryOnce, FnOnce, (self), (self));
impl_all_for!(DiFactory, Fn, (&self), (self));

/// This impl allows passing plain objects as factories
impl<T: 'static> DiFactoryOnce<T, ()> for T {
    fn depends_on() -> &'static [TypeMeta] {
        &[]
    }

    fn produce(self, _: &Services) -> T {
        self
    }
}
