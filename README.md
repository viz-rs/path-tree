# path-tree

Path-based routing tree.

A compressing dynamic trie ([radix tree]) structure is used for efficient matching.

[![Build Status](https://travis-ci.org/trek-rs/path-tree.svg?branch=master)](https://travis-ci.org/trek-rs/path-tree)
[![Latest version](https://img.shields.io/crates/v/path-tree.svg)](https://crates.io/crates/path-tree)
[![Documentation](https://docs.rs/path-tree/badge.svg)](https://docs.rs/path-tree)
![License](https://img.shields.io/crates/l/path-tree.svg)

## Features

- Fast!

- Named parameters. e.g. `:name`.

- Catch-All parameters. e.g. `*any`.

- Supports multiple naming for the same path segment. e.g. `/users/:id` and `/users/:user_id/repos`.

- Don't care about routes orders, we will automatically need to find them.

## Usage

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
assert_eq!(*res.0, 1);
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
let node = tree.find("/users/fundon/repos/trek-rs");
let res = node.unwrap();
assert_eq!(*res.0, 5);
assert_eq!(res.1, [("user_id", "fundon"), ("id", "trek-rs")]); // Params

// Matched "/users/:user_id/repos/:id/*any"
let node = tree.find("/users/fundon/repos/trek-rs/noder/issues");
let res = node.unwrap();
assert_eq!(*res.0, 6);
assert_eq!(
    res.1,
    [
        ("user_id", "fundon"),
        ("id", "trek-rs"),
        ("any", "noder/issues"),
    ]
); // Params


// Matched "/users/repos/*any"
let node = tree.find("/users/repos/");
let res = node.unwrap();
assert_eq!(*res.0, 12);
assert_eq!(res.1, []);
```

## Acknowledgements

It is inspired by the:

- [rax]
- [httprouter]
- [echo] router
- [trekjs] router

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

[radix tree]: https://github.com/trek-rs/radix-tree
[rax]: https://github.com/antirez/rax
[httprouter]: https://github.com/julienschmidt/httprouter
[echo]: https://github.com/labstack/echo
[trekjs]: https://github.com/trekjs/router
