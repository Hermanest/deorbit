use crate::builder::ServicesBuilder;
use crate::from_di::FromDi;
use crate::services::{Service, Services};
use deorbit_macro::FromDi;
use std::any::type_name;
use std::mem::MaybeUninit;

#[derive(FromDi)]
struct Foo {
    a: Service<i32>,
    b: Service<Bar>,
}

#[derive(FromDi)]
struct Bar {
    a: Service<i32>,
    b: Service<Foo>,
}

#[test]
fn binds_concrete() -> Result<(), String> {
    let mut builder = ServicesBuilder::new();

    builder.bind_from(10i32);
    builder.bind::<Foo>();
    builder.bind::<Bar>();

    let services = builder.build()?;

    services.resolve::<Foo>().expect("Failed to resolve Foo");
    services.resolve::<Bar>().expect("Failed to resolve Bar");

    Ok(())
}
