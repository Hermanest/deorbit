use crate::Service;
use crate::builder::ServicesBuilder;
use crate::from_di::FromDi;
use crate::resolver::Error;
use deorbit_macro::FromDi;
use std::any::Any;
use std::sync::Arc;

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

    builder.bind::<i32>().not_self().singleton().from(10);
    builder.bind_alias::<dyn Any>().to::<i32>(|x| x).done();

    builder.bind::<Bar>().singleton().from_di();
    builder.bind::<Foo>().singleton().from_di();

    let services = builder.build()?;

    services.resolve::<Foo>().expect("Failed to resolve Foo");
    services.resolve::<Bar>().expect("Failed to resolve Bar");
    services
        .resolve::<dyn Any>()
        .expect("Failed to resolve dyn Any");

    Ok(())
}

#[test]
fn fails_duplicated() {
    let mut builder = ServicesBuilder::new();

    builder.bind().singleton().from(10i32);
    builder.bind().singleton().from(10i32);

    let res = builder.build();

    assert!(matches!(res, Err(Error::Duplicated { .. })));
}

#[test]
fn fails_missing() {
    #[derive(FromDi)]
    struct Foo {
        a: Service<i64>,
    }

    let mut builder = ServicesBuilder::new();

    builder.bind().singleton().from(10i32);
    builder.bind::<Foo>().singleton().from_di();

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

    builder.bind::<FooCirc>().singleton().from_di();
    builder.bind::<BarCirc>().singleton().from_di();

    let res = builder.build();

    assert!(matches!(res, Err(Error::Circular { .. })));
}

#[test]
fn fails_circular_dyn() {
    trait IBarCirc {}
    impl IBarCirc for BarCirc {}

    #[derive(FromDi)]
    struct BarCirc {
        a: Service<FooCirc>,
    }

    #[derive(FromDi)]
    struct FooCirc {
        a: Service<dyn IBarCirc>,
    }

    let mut builder = ServicesBuilder::new();

    builder.bind::<FooCirc>().singleton().from_di();
    builder.bind::<BarCirc>().not_self().singleton().from_di();
    builder
        .bind_alias::<dyn IBarCirc>()
        .to::<BarCirc>(|x| x)
        .done();

    let res = builder.build();

    assert!(matches!(res, Err(Error::Circular { .. })));
    println!("{}", res.unwrap_err());
}

#[test]
fn resolves_last_alias() {
    trait Service {}
    impl Service for Bar {}
    impl Service for Foo {}

    #[derive(FromDi)]
    struct Bar {}

    #[derive(FromDi)]
    struct Foo {}

    let mut builder = ServicesBuilder::new();

    builder.bind::<Foo>().singleton().from_di();
    builder.bind::<Bar>().singleton().from_di();
    builder
        .bind_alias::<dyn Service>()
        .to::<Foo>(|x| x)
        .to::<Bar>(|x| x)
        .done();

    let res = builder.build().unwrap();

    let last = res
        .resolve_all::<dyn Service>()
        .unwrap()
        .next_back()
        .unwrap();
    let single = res.resolve::<dyn Service>().unwrap();

    assert_eq!(Arc::as_ptr(&last), Arc::as_ptr(&single));
}

#[test]
fn fails_if_not_exposed() {
    #[derive(FromDi)]
    struct Foo {}

    let mut builder = ServicesBuilder::new();

    builder.bind::<Foo>().not_self().singleton().from_di();
    builder.bind_alias::<dyn Any>().to::<Foo>(|x| x).done();

    let res = builder.build().unwrap();

    let dy = res.resolve::<dyn Any>();
    let concrete = res.resolve::<Foo>();

    assert!(matches!(dy, Some(..)));
    assert!(matches!(concrete, None));
}
