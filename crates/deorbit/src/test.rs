use crate::builder::{FromDi, ServicesBuilder};
use crate::services::{Service, Services};
use std::any::TypeId;

struct Foo {
    a: Service<i32>,
    b: Service<Bar>,
}

struct Bar {
    a: Service<Foo>,
}

const BAR_DEPS: &[TypeId] = &[TypeId::of::<Foo>()];

impl FromDi for Bar {
    fn depends_on() -> &'static [TypeId] {
        BAR_DEPS
    }

    fn produce(services: &Services) -> Self {
        Self {
            a: services.resolve().expect(""),
        }
    }
}

#[test]
fn binds_concrete() -> Result<(), String> {
    let mut builder = ServicesBuilder::new();

    builder.bind_singleton_from(10i32);
    builder.bind_singleton::<Bar>();
    builder.bind_singleton_from::<Foo, _, _>(|x: Service<i32>, y: Service<Bar>| Foo { a: x, b: y });
    
    let built = builder.build();

    println!("{:?}", built);

    Ok(())
}
