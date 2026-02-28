## About
Deorbit is a dependency framework for Rust. It aims for maximum automatization, user does less to achieve the same result.

## How does it look?

### Example 1 
This is the simplest case, we're binding the service from a manually created instance:
```rust
struct Foo {
    a: i32
}

fn bind() {
    let mut builder = ServicesBuilder::new();

    // Binding the service
    builder.bind_from(Foo { a: 10 }); 

    // Creating a Services instance
    let services = builder.build().unwrap();

    // Resolving bound dependency
    let foo = services.resolve::<Foo>().expect("Failed to resolve Foo!");
}
```

### Example 2
With the help of FromDi trait we can make any service constructable from a DI instance. This enables instance-less wiring:
```rust
#[derive(FromDi)
struct Foo {
    // Must either a service or a type implementing Default
    a: Service<i32>,
    // Will use the Default trait
    #[di(default)]
    b: i32,
}

fn bind() {
    let mut builder = ServicesBuilder::new();

    // Binding an integer
    builder.bind_from(10);

    // Binding the service itself. This call is eligible only for types implementing FromDi
    builder.bind::<Foo>(); 

    // Creating a Services instance
    let services = builder.build().unwrap();

    // Resolving bound dependency
    let foo = services.resolve::<Foo>().expect("Failed to resolve Foo!");
}
```

### Example 3
Sometimes things can get really tricky. Imagine you need to inject one service into another, pretty complicated, ain't it?
Well, Deorbit handles this without hesitation:
```rust
#[derive(FromDi)]
struct Foo {
    a: Service<i32>,
    bar: Service<Bar>,
}

#[derive(FromDi)]
struct Bar {
    a: Service<i32>,
    foo: Service<Foo>,
}

fn bind() {
    let mut builder = ServicesBuilder::new();

    // Binding all necessary services
    builder.bind_from(10i32); 
    builder.bind::<Foo>();
    builder.bind::<Bar>();

    // Creating a Services instance
    let services = builder.build().unwrap();

    services.resolve::<Foo>().expect("Failed to resolve Foo");
    services.resolve::<Bar>().expect("Failed to resolve Bar");
}
```

## Why?
There are currently no DI frameworks for Rust that could fulfill all needs for advanced and modular applications: automatic service wiring, circular dependecy resolution, abstractions.
Deorbit comes to fix this by providing a wide range of tools hence covering most cases. 

## Why the name? 
Well, initially this framework was planned as actix-specific. I wanted to bind the name with some of the existing frameworks and haven't came up with anything better than relating to Rocket.
Since Rocket directly relates to space, I've chosen the name "Orbit" as it would be a base of the application. During development I've got hit with a thought: "What if this solution would cover ALL the cases?". 
And as the project switched targeting to a much wider audience, I've added a "di" prefix, pointing that this is a dependency injection framework. But "Diorbit" sounded a bit weird so I've changed it to "Deorbit". That's it.

## Roadmap
Currently the project is rather an MVP than something production-ready, so here's a roadmap for further changes:
- [ ] Add service lifetimes (singleton, transient)
- [ ] Add abstraction bindinds
- [ ] Add support for actix and axum frameworks as a separate feature
