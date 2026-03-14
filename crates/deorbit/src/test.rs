use crate::builder::{FromDi, ServicesBuilder};
use crate::error::Error;
use crate::services::{Service, Services};
use deorbit_macro::FromDi;

#[derive(FromDi)]
struct Foo {
    a: Service<i32>,
}

#[derive(FromDi)]
struct Bar {
    a: Service<Foo>,
}

#[test]
fn binds_concrete() -> Result<(), Error> {
    let mut builder = ServicesBuilder::new();

    builder.bind_singleton_from(10i32);
    builder.bind_singleton::<Bar>();
    builder.bind_singleton::<Foo>();

    let services = builder.build()?;

    services.resolve::<Foo>().expect("Failed to resolve Foo");
    services.resolve::<Bar>().expect("Failed to resolve Bar");

    println!("{:?}", services);

    Ok(())
}
