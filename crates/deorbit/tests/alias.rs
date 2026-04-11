use deorbit::{Error, FromDi, Resolved, ServicesBuilder};
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
fn maintains_alias_order() {
    let mut builder = ServicesBuilder::new();

    builder.bind::<i32>().singleton().from(10);
    builder.bind::<i64>().singleton().from(20);
    builder.bind::<i128>().singleton().from(30);
    builder
        .bind_alias::<dyn Any + Send + Sync>()
        .to::<i32>(|x| x)
        .to::<i64>(|x| x)
        .to::<i128>(|x| x)
        .done();

    let res = builder.build().unwrap();
    let items: Vec<_> = res.resolve_all::<dyn Any + Send + Sync>().unwrap().collect();

    assert!(items[0].clone().downcast::<i32>().is_ok_and(|x| *x == 10));
    assert!(items[1].clone().downcast::<i64>().is_ok_and(|x| *x == 20));
    assert!(items[2].clone().downcast::<i128>().is_ok_and(|x| *x == 30));
}

#[test]
fn maintains_alias_distinction() {
    let mut builder = ServicesBuilder::new();

    builder.bind::<i32>().singleton().from(0);
    builder
        .bind_alias::<dyn Any + Send + Sync>()
        .to::<i32>(|x| x)
        .to::<i32>(|x| x)
        .done();

    let res = builder.build().unwrap();
    let items: Vec<_> = res.resolve_all::<dyn Any + Send + Sync>().unwrap().collect();

    assert_eq!(items.len(), 1);
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
