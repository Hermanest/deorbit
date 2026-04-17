use deorbit::{Error, FromDi, ServicesBuilder, from_di};
use std::sync::Arc;

#[test]
fn binds_single() {
    let mut builder = ServicesBuilder::new();

    let num = 10;
    builder.bind::<i32>().singleton().from(num);

    let services = builder.build().unwrap();
    let res = services.resolve::<i32>();

    assert!(matches!(res, Some(x) if *x == num));
}

#[test]
fn binds_multiple() {
    let mut builder = ServicesBuilder::new();

    let num1 = 10;
    let num2 = 20;

    builder.bind::<i32>().singleton().from(num1);
    builder.bind::<i64>().singleton().from(num2);

    let services = builder.build().unwrap();

    let res1 = services.resolve::<i32>();
    let res2 = services.resolve::<i64>();

    assert!(matches!(res1, Some(x) if *x == num1));
    assert!(matches!(res2, Some(x) if *x == num2));
}

#[test]
fn resolves_same_singleton() {
    let mut builder = ServicesBuilder::new();

    builder.bind::<i32>().singleton().from(10);

    let services = builder.build().unwrap();

    let res1 = services.resolve::<i32>().unwrap();
    let res2 = services.resolve::<i32>().unwrap();

    assert_eq!(Arc::as_ptr(&res1), Arc::as_ptr(&res2));
}

#[test]
fn resolves_new_transient() {
    let mut builder = ServicesBuilder::new();

    builder.bind::<i32>().transient().from_fn(|| 10);

    let services = builder.build().unwrap();

    let res1 = services.resolve::<i32>().unwrap();
    let res2 = services.resolve::<i32>().unwrap();

    assert_ne!(Arc::as_ptr(&res1), Arc::as_ptr(&res2));
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
    #[from_di]
    struct Foo {
        a: i64,
    }

    let mut builder = ServicesBuilder::new();

    builder.bind().singleton().from(10i32);
    builder.bind::<Foo>().singleton().from_di();

    let res = builder.build();

    assert!(matches!(res, Err(Error::Missing { .. })));
}

#[test]
fn fails_circular() {
    #[from_di]
    struct BarCirc {
        a: FooCirc,
    }

    #[from_di]
    struct FooCirc {
        a: BarCirc,
    }

    let mut builder = ServicesBuilder::new();

    builder.bind::<FooCirc>().singleton().from_di();
    builder.bind::<BarCirc>().singleton().from_di();

    let res = builder.build();

    assert!(matches!(res, Err(Error::Circular { .. })));
}
