use crate::builder::ServicesBuilder;
use crate::from_di::FromDi;
use crate::services::{Service, Services};
use std::any::type_name;
use std::mem::MaybeUninit;

struct Foo {
    a: Service<i32>,
    b: Service<Bar>,
}

struct Bar {
    a: Service<i32>,
    b: Service<Foo>,
}

impl FromDi for Foo {
    fn inject(instance: &mut MaybeUninit<Self>, services: &Services) -> Result<(), String> {
        let a = services
            .resolve()
            .ok_or_else(|| format!("Failed to resolve {}", type_name::<i32>()))?;

        let b = services
            .resolve()
            .ok_or_else(|| format!("Failed to resolve {}", type_name::<Bar>()))?;

        instance.write(Self { a, b });

        Ok(())
    }
}

impl FromDi for Bar {
    fn inject(instance: &mut MaybeUninit<Self>, services: &Services) -> Result<(), String> {
        let a = services
            .resolve()
            .ok_or_else(|| format!("Failed to resolve {}", type_name::<i32>()))?;

        let b = services
            .resolve()
            .ok_or_else(|| format!("Failed to resolve {}", type_name::<Foo>()))?;

        instance.write(Self { a, b });

        Ok(())
    }
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
