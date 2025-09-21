Add shared fields to each variant of the enumeration

For example:

```rust
struct Foo {
    pub id: u64,
    pub data: FooData,
}
enum FooData {
    Named(String),
    Complex { name: String, age: u32 },
    Empty,
}
```

Change to using `enum_struct::fields`:

```rust
#[enum_struct::fields { id: u64 }]
enum Foo {
    Named(String),
    Complex { name: String, age: u32 },
    Empty,
}
```

**Expand into**:

```rust,ignore
enum Foo {
    Named(u64, String),
    Complex { id: u64, name: String, age: u32 },
    Empty { id: u64 },
}
impl Foo {
    fn id(&self) -> &u64 { /*...*/ }
    fn id_mut(&self) -> &mut u64 { /*...*/ }
    fn into_id(&self) -> u64 { /*...*/ }
}
```


# Example

```rust
#[enum_struct::fields {
    id: u64,
}]
#[derive(Debug, PartialEq)]
enum Foo {
    Named(String),
    Complex { name: String, age: u32 },
    Empty,
}

let named = Foo::Named(2, "jack".into());
let complex = Foo::Complex { id: 3, name: "john".into(), age: 22 };
let empty = Foo::Empty { id: 4 };

assert_eq!(named.id(), &2);
assert_eq!(complex.id(), &3);
assert_eq!(empty.id(), &4);

let mut named = named;

*named.id_mut() = 8;
assert_eq!(named.id(), &8);
assert_eq!(named, Foo::Named(8, "jack".into()));
```
