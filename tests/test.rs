#[enum_struct::fields {
    /// some docs
    x: i32,
    t: T,
}]
#[derive(Debug)]
pub enum Foo<T: Copy> {
    Record { y: i32 },
    RecordHasGeneric { y: i32, z: T },
    Tuple(i32, i8),
    Unit,
}

#[test]
fn test_simple() {
    let foo = Foo::Record { x: 2, t: 'm', y: 4 };
    assert_eq!(foo.x(), &2);
    assert_eq!(foo.t(), &'m');
}

#[test]
fn test_simple_generic() {
    let foo = Foo::RecordHasGeneric { x: 2, t: 'm', y: 4, z: 'o' };
    assert_eq!(foo.x(), &2);
    assert_eq!(foo.t(), &'m');
}

#[test]
fn test_unit() {
    let foo = Foo::Unit { x: 3, t: 'm' };
    assert_eq!(foo.x(), &3);
    assert_eq!(foo.t(), &'m');
}

#[test]
fn test_tuple() {
    let foo = Foo::Tuple(4, 'm', 2, 5);
    assert_eq!(foo.x(), &4);
    assert_eq!(foo.t(), &'m');
}

#[enum_struct::fields { x: i32, t: () }]
enum EmptyEnum { }

#[enum_struct::fields {
    x: i32,
    #[cfg(false)]
    y: i32,
}]
enum HasCfg {
    A,
    B(i8),
}
