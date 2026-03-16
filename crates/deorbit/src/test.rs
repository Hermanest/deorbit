use crate::builder::ServicesBuilder;
use crate::error::Error;
use crate::from_di::FromDi;
use crate::services::{Service, Services};
use deorbit_macro::FromDi;

#[test]
fn binds_concrete() -> Result<(), Error> {
    #[derive(FromDi)]
    struct Foo {
        a: Service<i32>,
    }

    #[derive(FromDi)]
    struct Bar {
        a: Service<Foo>,
    }

    let mut builder = ServicesBuilder::new();

    builder.bind_singleton_from(10i32);
    builder.bind_singleton::<Bar>();
    builder.bind_singleton::<Foo>();

    let services = builder.build()?;

    services.resolve::<Foo>().expect("Failed to resolve Foo");
    services.resolve::<Bar>().expect("Failed to resolve Bar");

    Ok(())
}

#[test]
fn fails_duplicated() {
    let mut builder = ServicesBuilder::new();

    builder.bind_singleton_from(10i32);
    builder.bind_singleton_from(10i32);

    let res = builder.build();

    assert!(matches!(res, Err(Error::Duplicated { .. })));
}

#[test]
fn fails_missing() {
    let mut builder = ServicesBuilder::new();

    builder.bind_singleton_from(10i32);
    builder.bind_singleton_from::<i128, _, _>(|_: Service<i64>| 0i128);

    let res = builder.build();

    assert!(matches!(res, Err(Error::Missing { .. })));
}

#[test]
fn fails_circular() {
    #[derive(FromDi)]
    struct BarCirc {
        a: Service<FooCirc>,
    }

    #[derive(FromDi)]
    struct FooCirc {
        a: Service<BarCirc>,
    }

    let mut builder = ServicesBuilder::new();

    builder.bind_singleton::<FooCirc>();
    builder.bind_singleton::<BarCirc>();

    let res = builder.build();

    assert!(matches!(res, Err(Error::Circular { .. })));
}
