use super::TypedArena;
extern crate std;
use std::{prelude::rust_2021::*, print, vec};

#[test]
fn readme_example() {
    // Create a new TypedArena
    let string_arena = TypedArena::new();

    // Allocate some `String`s onto the arena.
    // Note that allocating onto the arena gives you a *mutable* reference (which you can )
    let elem_one: &mut String = string_arena.alloc(String::from("Hello, world!"));
    let elem_two: &mut String = string_arena.alloc(String::from("Goodbye, world!"));

    std::println!("{elem_one}\n{elem_two}");

    // Drop the TypedArena, dropping its contents
    drop(string_arena); // drops elem_one and elem_two
}

#[test]
fn pushing_to_arena() {
    let arena = TypedArena::new();
    let foo = arena.alloc("foo");
    let bar = arena.alloc("bar");
    let baz = arena.alloc("baz");

    assert_eq!("foo", *foo);
    assert_eq!("bar", *bar);
    assert_eq!("baz", *baz);
}

#[test]
fn pushing_vecs_to_arena() {
    let arena = TypedArena::new();

    let foo = arena.alloc(vec!["foo"]);
    let bar = arena.alloc(vec!["bar"]);
    let baz = arena.alloc(vec!["baz"]);

    assert_eq!("foo", foo[0]);
    assert_eq!("bar", bar[0]);
    assert_eq!("baz", baz[0]);
}

#[test]
fn pushing_zsts() {
    struct ZeroSized;
    impl Drop for ZeroSized {
        fn drop(&mut self) {
            print!("")
        }
    }

    let arena = TypedArena::new();

    for _ in 0..0x100 {
        arena.alloc(ZeroSized);
    }
}

#[test]
fn pushing_nodrop_zsts() {
    struct ZeroSized;
    let arena = TypedArena::new();

    for _ in 0..0x1000 {
        arena.alloc(ZeroSized);
    }
}
#[test]
fn resize() {
    let arena = TypedArena::new();

    for _ in 0..0x780 {
        arena.alloc(0u128);
    }
}
