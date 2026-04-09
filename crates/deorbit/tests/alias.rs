use std::any::Any;
use std::sync::Arc;
use deorbit::{Error, FromDi, Service, ServicesBuilder};

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

    builder.bind().singleton().from(Foo {});
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
