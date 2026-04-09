use deorbit::{Error, FromDi, Resolved, ResolvedMany, ServicesBuilder};
use std::any::Any;
use std::fmt::Debug;
use std::sync::Arc;

#[test]
fn binds_dyn() {
    let mut builder = ServicesBuilder::new();

    let num = 10;

    builder.bind::<i32>().singleton().from(num);
    builder
        .bind_alias::<dyn Any + Send + Sync>()
        .to::<i32>(|x| x)
        .done();

    let services = builder.build().unwrap();
    let res = services.resolve::<dyn Any + Send + Sync>().unwrap();

    assert!(res.downcast::<i32>().is_ok_and(|x| *x == num));
}

#[test]
fn resolves_same_singleton_dyn() {
    let mut builder = ServicesBuilder::new();

    builder.bind::<i32>().singleton().from(10);
    builder.bind_alias::<dyn Any>().to::<i32>(|x| x).done();

    let services = builder.build().unwrap();

    let res1 = services.resolve::<dyn Any>().unwrap();
    let res2 = services.resolve::<dyn Any>().unwrap();

    assert_eq!(Arc::as_ptr(&res1), Arc::as_ptr(&res2));
}

#[test]
fn resolves_new_transient_dyn() {
    let mut builder = ServicesBuilder::new();

    builder.bind::<i32>().transient().from_fn(|| 10);
    builder.bind_alias::<dyn Any>().to::<i32>(|x| x).done();

    let services = builder.build().unwrap();

    let res1 = services.resolve::<dyn Any>().unwrap();
    let res2 = services.resolve::<dyn Any>().unwrap();

    assert_ne!(Arc::as_ptr(&res1), Arc::as_ptr(&res2));
}

#[test]
fn resolves_last_alias() {
    let mut builder = ServicesBuilder::new();

    builder.bind::<i32>().singleton().from(10);
    builder.bind::<i64>().singleton().from(20);
    builder
        .bind_alias::<dyn Any>()
        .to::<i32>(|x| x)
        .to::<i64>(|x| x)
        .done();

    let res = builder.build().unwrap();

    let last = res.resolve_all::<dyn Any>().unwrap().next_back().unwrap();
    let single = res.resolve::<dyn Any>().unwrap();

    assert_eq!(Arc::as_ptr(&last), Arc::as_ptr(&single));
}

#[test]
fn resolves_many() {
    #[derive(FromDi)]
    struct Foo {
        other: ResolvedMany<dyn Any + Send + Sync>,
    }

    let mut builder = ServicesBuilder::new();

    builder.bind::<Foo>().singleton().from_di();
    builder.bind::<i32>().singleton().from(10);
    builder.bind::<i64>().singleton().from(20);
    builder
        .bind_alias::<dyn Any + Send + Sync>()
        .to::<i32>(|x| x)
        .to::<i64>(|x| x)
        .done();

    let res = builder.build().unwrap();
    let items = &res.resolve::<Foo>().unwrap().other;

    assert!(items[0].clone().downcast::<i64>().is_ok_and(|x| *x == 20));
    assert!(items[1].clone().downcast::<i32>().is_ok_and(|x| *x == 10));
}

#[test]
fn fails_circular_dyn() {
    trait IBarCirc {}
    impl IBarCirc for BarCirc {}

    #[derive(FromDi)]
    struct BarCirc {
        a: Resolved<FooCirc>,
    }

    #[derive(FromDi)]
    struct FooCirc {
        a: Resolved<dyn IBarCirc>,
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
fn fails_if_not_exposed() {
    let mut builder = ServicesBuilder::new();

    builder.bind::<i32>().not_self().singleton().from(10);
    builder.bind_alias::<dyn Any>().to::<i32>(|x| x).done();

    let res = builder.build().unwrap();

    let dy = res.resolve::<dyn Any>();
    let concrete = res.resolve::<i32>();

    assert!(matches!(dy, Some(..)));
    assert!(matches!(concrete, None));
}
