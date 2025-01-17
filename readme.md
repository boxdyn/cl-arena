# A general-purpose arena allocator for `std` Rust projects

`cl-arena` implements a generic bump allocator without piecemeal deallocation. This means that references to elements are valid for the lifetime of the allocator.

## Why would you want this?

It can be difficult to create self-referential data structures in Rust without jumping through hoops to satisfy the borrow checker. Giving all of your nodes the same lifetime would dramatically simplify your graph's construction, as long as you can wrap your back edges in a [`cell`][cell] of some kind.

Additionally, having an allocator that spits out references with an identical lifetime can facilitate the creation of [object interners][intern], which, due to their uniqueness characteristic, allow cheap referential comparison between objects.

## How do I use these allocators?

### TypedArena

The `TypedArena` stores elements of a single type, but will [Drop][Drop] its elements when the arena goes out of scope.

```rust
// Create a new TypedArena
let string_arena = TypedArena::new();

// Allocate some `String`s onto the arena.
// Note that allocating onto the arena gives you a *mutable* reference (which you can )
let elem_one: &mut String = string_arena.alloc(String::from("Hello, world!"));
let elem_two: &mut String = string_arena.alloc(String::from("Goodbye, world!"));

println!("{elem_one}\n{elem_two}");

// Drop the TypedArena, dropping its contents
drop(string_arena); // drops elem_one and elem_two
```

### DroplessArena

The `DroplessArena` stores data of any type that doesn't implement [Drop][Drop]. It has helper methods for allocating slices [`[T]`][slice] and [`str`][str].

```rust
    // Create a new DroplessArena
    let dropless_arena = DroplessArena::new();

    // Allocate some data on the arena
    // Note that alloc and alloc_slice return mutable references to the data...
    let just1_i32: &mut i32 = dropless_arena.alloc(1);
    let slice_i32: &mut [i32] = dropless_arena.alloc_slice(&[0, 2, 3, 4]);

    // ...and that alloc_str returns an *immutable* reference.
    let slice_str: &str = dropless_arena.alloc_str("Hello, world!");

    slice_i32[0] = *just1_i32;

    println!("just1_i32: {just1_i32:?}"); // just1_i32: 1
    println!("slice_i32: {slice_i32:?}"); // slice_i32: [1, 2, 3, 4]
    println!("slice_str: {slice_str:?}"); // str_slice: "Hello, world!"
```


[cell]: https://doc.rust-lang.org/core/cell/index.html
[Drop]: https://doc.rust-lang.org/core/ops/trait.Drop.html
[slice]: https://doc.rust-lang.org/core/primitive.slice.html
[str]: https://doc.rust-lang.org/core/primitive.str.html

[intern]: https://en.wikipedia.org/wiki/String_interning
