## About
Deorbit is a dependency framework for Rust. It aims for maximum automatization, so user does less to achieve the same result. 

## Why?
There are currently no DI frameworks for Rust that could fulfill all needs for advanced modular applications.
Deorbit comes to fix this by providing a wide range of tools like automatic service wiring and trait resolution. 

## How does it look?
Deorbit utilizes chained builders to provide scalability and extensibility, so anyone can add finalizers besides from_di, from etc.
Below you'll find examples for different cases.

### Example 1 
This is the simplest case, we're binding the service from a manually created instance:
```rust
struct Foo {
    a: i32
}

fn bind() {
    let mut builder = ServicesBuilder::new();

    // Binding the service
    builder.bind().singleton().from(Foo { a: 10 }); 

    // Creating a Services instance
    let services = builder.build().unwrap();

    // Resolving bound dependency
    let foo = services.resolve::<Foo>().expect("Failed to resolve Foo!");
}
```

### Example 2
With the help of FromDi trait we can make any service constructable from a DI instance. This enables instance-less wiring:
```rust
#[derive(FromDi)]
struct Foo {
    // Must either be a service or a type implementing Default
    a: Service<i32>,
    // Will use the Default trait
    #[di(default)]
    b: i32,
}

fn bind() {
    let mut builder = ServicesBuilder::new();

    // Binding an integer
    builder.bind().singleton().from(10);

    // Binding the service itself. This call is eligible only for types implementing FromDi
    builder.bind::<Foo>().singleton().from_di(); 

    // Creating a Services instance
    let services = builder.build().unwrap();

    // Resolving bound dependency
    let foo = services.resolve::<Foo>().expect("Failed to resolve Foo!");
}
```

## Example 3
One of the main features of the framework is an ability to work with traits, so here it is:
```rust
trait Printable {
    fn print(&self);
}

impl Printable for i32 {
    fn print(&self) {
        println!("i32: {}", self)
    }
}

impl Printable for i64 {
    fn print(&self) {
        println!("i64: {}", self)
    }
}

fn bind() {
    let mut builder = ServicesBuilder::new();

    builder.bind::<i32>().singleton().from(10);
    builder.bind::<i64>().singleton().from(20);

    // Binding i32 and i64 under Printable
    builder.bind_alias::<dyn Printable>()
        .to::<i32>(|x| x)
        .to::<i64>(|x| x)
        .done();

    let services = builder.build().unwrap();

    // Resolving all implementations of Printable
    let nums = services.resolve_all::<dyn Printable>().unwrap();

    for num in nums {
        num.print();
    }

    // Calling resolve on bindings with multiple underlying types
    // returns the last bound type (so if we bind Foo and then Bar, it will return Bar)
    let num = services.resolve::<dyn Printable>().unwrap();
    num.print();
}
```
This code will output
```
i32: 10
i64: 20
i64: 20
```

## A bit about architecture
You might notice that to bind a trait you need to use a closure (e.g. `bind_alias::<dyn Any>().to::<i32>(|x| x).done()`). But isn't it weird? 
Well, this design decision comes from how rust does things. The main problem is, there is no stable api to turn a conrete instance into an unsized dyn yet
(look for `Unsized` trait for more details). Currently the only way to unsize an object is to implicitly coerce it in-place, so `let unsized: Arc<dyn Any> = Arc::new(10)`
will turn an instance of `Arc<i32>` into `Arc<dyn Any>`. And that's why closures are there, when you call `.to::<Type>(|x| x)`, the closure receives a conrete type and
returns a coerced instance. To add a bit of clarity, the previous snippet is semantically similar to `.to(|x: Arc<Type>| x as Arc<dyn Any>)`, it's just the rust compiler
that eliminates the need of specifying types manually by inferring them from previous calls.
This syntax could be replaced in future releases if `Unsized` will be become stable.

## Why the name? 
Well, initially this framework was planned as actix-specific. I wanted to bind the name with some of the existing frameworks and haven't came up with anything better than relating to Rocket.
Since Rocket directly relates to space, I've chosen the name "Orbit" as it would be a base of the application. During development I've got hit with a thought: "What if this solution would cover ALL the cases?". 
And as the project switched targeting to a much wider audience, I've added a "di" prefix, pointing that this is a dependency injection framework. But "Diorbit" sounded a bit weird so I've changed it to "Deorbit". That's it.

## Roadmap
Currently the project is rather an MVP than something production-ready, so here's a roadmap for further changes:
- [x] Add service lifetimes (singleton, transient)
- [x] Add abstraction bindinds
- [ ] Add support for actix and axum frameworks as a separate feature
