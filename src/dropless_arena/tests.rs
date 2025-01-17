use super::DroplessArena;
extern crate std;
use core::alloc::Layout;
use std::{prelude::rust_2021::*, vec};

#[test]
fn readme_example() {
    // Create a new DroplessArena
    let dropless_arena = DroplessArena::new();

    // Allocate some data on the arena
    // Note that alloc and alloc_slice return mutable references to the data...
    let just1_i32: &mut i32 = dropless_arena.alloc(1);
    let slice_i32: &mut [i32] = dropless_arena.alloc_slice(&[0, 2, 3, 4]);

    // ...and that alloc_str returns an *immutable* reference.
    let slice_str: &str = dropless_arena.alloc_str("Hello, world!");

    slice_i32[0] = *just1_i32;

    std::println!("just1_i32: {just1_i32:?}"); // just1_i32: 1
    std::println!("slice_i32: {slice_i32:?}"); // slice_i32: [1, 2, 3, 4]
    std::println!("slice_str: {slice_str:?}"); // str_slice: "Hello, world!"
}

#[test]
fn alloc_raw() {
    let arena = DroplessArena::new();
    let bytes = arena.alloc_raw(Layout::for_value(&0u128));
    let byte2 = arena.alloc_raw(Layout::for_value(&0u128));

    assert_ne!(bytes, byte2);
}

#[test]
fn alloc() {
    let arena = DroplessArena::new();
    let mut allocations = vec![];
    for i in 0..0x400 {
        allocations.push(arena.alloc(i));
    }
}

#[test]
fn alloc_strings() {
    const KW: &[&str] = &["pub", "mut", "fn", "mod", "conlang", "sidon", "🦈"];
    let arena = DroplessArena::new();
    let mut allocations = vec![];
    for _ in 0..100 {
        for kw in KW {
            allocations.push(arena.alloc_str(kw));
        }
    }
}

#[test]
#[should_panic]
fn alloc_zsts() {
    struct Zst;
    let arena = DroplessArena::new();
    arena.alloc(Zst);
}
