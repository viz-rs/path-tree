<h1 align="center">path-tree</h1>

<div align="center">
  <p><strong>A lightweight high performance HTTP request router for Rust</strong></p>
</div>

<div align="center">
  <!-- Safety docs -->
  <a href="/">
    <img src="https://img.shields.io/badge/-safety!-success?style=flat-square" alt="Safety!" /></a>
  <!-- Docs.rs docs -->
  <a href="https://docs.rs/path-tree">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="Docs.rs docs" /></a>
  <!-- Crates version -->
  <a href="https://crates.io/crates/path-tree">
    <img src="https://img.shields.io/crates/v/path-tree.svg?style=flat-square"
    alt="Crates.io version" /></a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/path-tree">
    <img src="https://img.shields.io/crates/d/path-tree.svg?style=flat-square"
      alt="Download" /></a>
  <!-- Twitter -->
  <a href="https://twitter.com/_fundon">
    <img src="https://img.shields.io/badge/twitter-@__fundon-blue.svg?style=flat-square" alt="Twitter: @_fundon" /></a>
</div>

## Features

- **Fast**: See benchmark

- **Micro**: The [src/lib.rs](src/lib.rs) file is ~407 lines of code (Includes comments)

- **Flexible**:

  - _**Static**_ segment. e.g. `/users`.

  - _**Named**_ parameters. e.g. `:name`.

  - _**Catch-All**_ parameters. e.g. `*any`, it must always be at the end of the pattern.

  - Supports multiple naming for the same path segment. e.g. `/users/:id` and `/users/:user_id/repos`.

  - Don't care about routes orders, recursive lookup, `Static` -> `Named` -> `Catch-All`.

## Examples

- [hello-hyper](examples/hello.rs)

```rust
use path_tree::PathTree;

let mut tree = PathTree::<usize>::new();

tree.insert("/", 0);
tree.insert("/users", 1);
tree.insert("/users/:id", 2);
tree.insert("/users/:id/:org", 3);
tree.insert("/users/:user_id/repos", 4);
tree.insert("/users/:user_id/repos/:id", 5);
tree.insert("/users/:user_id/repos/:id/*any", 6);
tree.insert("/:username", 7);
tree.insert("/*any", 8);
tree.insert("/about", 9);
tree.insert("/about/", 10);
tree.insert("/about/us", 11);
tree.insert("/users/repos/*any", 12);

// Matched "/"
let node = tree.find("/");
assert_eq!(node.is_some(), true);
let res = node.unwrap();
assert_eq!(*res.0, 0);
assert_eq!(res.1, []); // Params

// Matched "/:username"
let node = tree.find("/username");
assert_eq!(node.is_some(), true);
let res = node.unwrap();
assert_eq!(*res.0, 7);
assert_eq!(res.1, [("username", "username")]); // Params


// Matched "/*any"
let node = tree.find("/user/s");
let res = node.unwrap();
assert_eq!(*res.0, 8);
assert_eq!(res.1, [("any", "user/s")]);

// Matched "/users/:id"
let node = tree.find("/users/fundon");
let res = node.unwrap();
assert_eq!(*res.0, 2);
assert_eq!(res.1, [("id", "fundon")]); // Params

// Matched "/users/:user_id/repos/:id"
let node = tree.find("/users/fundon/repos/viz-rs");
let res = node.unwrap();
assert_eq!(*res.0, 5);
assert_eq!(res.1, [("user_id", "fundon"), ("id", "viz-rs")]); // Params

// Matched "/users/:user_id/repos/:id/*any"
let node = tree.find("/users/fundon/repos/viz-rs/noder/issues");
let res = node.unwrap();
assert_eq!(*res.0, 6);
assert_eq!(
    res.1,
    [
        ("user_id", "fundon"),
        ("id", "viz-rs"),
        ("any", "noder/issues"),
    ]
); // Params


// Matched "/users/repos/*any"
let node = tree.find("/users/repos/");
let res = node.unwrap();
assert_eq!(*res.0, 12);
assert_eq!(res.1, []);
```

## Benchmark

```shell
$ cargo bench
```

## Acknowledgements

It is inspired by the:

- [rax]
- [httprouter]
- [echo] router
- [trekjs] router

## Other languages

Wrappers for path-tree in other languages:

- Python: https://github.com/adriangb/routrie

## License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>

[radix tree]: https://github.com/viz-rs/radix-tree
[rax]: https://github.com/antirez/rax
[httprouter]: https://github.com/julienschmidt/httprouter
[echo]: https://github.com/labstack/echo
[trekjs]: https://github.com/trekjs/router
