use crate::Service;
use crate::builder::ServicesBuilder;
use crate::error::Error;
use crate::from_di::FromDi;
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

    builder.bind::<i32>().singleton().from(10);
    builder.bind::<Bar>().singleton().from_di();
    builder.bind::<Foo>().singleton().from_di();

    let services = builder.build()?;

    services.resolve::<Foo>().expect("Failed to resolve Foo");
    services.resolve::<Bar>().expect("Failed to resolve Bar");

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
