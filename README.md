# path-tree
Path-based routing tree.

A compressing dynamic trie ([radix tree]) structure is used for efficient matching.

## Usage

```rust
use path_tree::PathTree;

let mut tree = PathTree::<usize>::new(
    "/",
    NodeMetadata {
        kind: NodeKind::Root,
        key: false,
        data: None,
        params: None,
    },
);

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
assert_eq!(res.0.path, ['/']);
assert_eq!(res.0.data.is_some(), true);
if let Some(meta) = &res.0.data {
  assert_eq!(meta.data.unwrap(), 0);
}
assert_eq!(res.1, None); // Params

// Matched "/:username"
let node = tree.find("/username");
assert_eq!(node.is_some(), true);
let res = node.unwrap();
assert_eq!(res.0.path, [':']);
if let Some(meta) = &res.0.data {
  assert_eq!(meta.data.unwrap(), 7); // Data
}
assert_eq!(res.1.unwrap(), [("username", "username")]); // Params


// Matched "/*any"
let node = tree.find("/user/s");
let res = node.unwrap();
assert_eq!(res.0.path, ['*']);
if let Some(meta) = &res.0.data {
  assert_eq!(meta.data.unwrap(), 8); // Data
}
assert_eq!(res.1.unwrap(), [("any", "user/s")]);

// Matched "/users/:id"
let node = tree.find("/users/fundon");
let res = node.unwrap();
assert_eq!(res.0.path, [':']);
if let Some(meta) = &res.0.data {
  assert_eq!(meta.data.unwrap(), 2); // Data
}
assert_eq!(res.1.unwrap(), [("id", "fundon")]); // Params

// Matched "/users/:user_id/repos/:id"
let node = tree.find("/users/fundon/repos/trek-rs");
let res = node.unwrap();
assert_eq!(res.0.path, [':']);
if let Some(meta) = &res.0.data {
  assert_eq!(meta.data.unwrap(), 5); // Data
}
assert_eq!(res.1.unwrap(), [("user_id", "fundon"), ("id", "trek-rs")]); //
Params

// Matched "/users/:user_id/repos/:id/*any"
let node = tree.find("/users/fundon/repos/trek-rs/noder/issues");
let res = node.unwrap();
assert_eq!(res.0.path, ['*']);
if let Some(meta) = &res.0.data {
  assert_eq!(meta.data.unwrap(), 6); // Data
}
assert_eq!(
    res.1.unwrap(),
    [
        ("user_id", "fundon"),
        ("id", "trek-rs"),
        ("any", "noder/issues"),
    ]
); // Params


// Matched "/users/repos/*any"
let node = tree.find("/users/repos/");
let res = node.unwrap();
assert_eq!(res.0.path, "*".chars().collect::<Vec<char>>());
if let Some(meta) = &res.0.data {
  assert_eq!(meta.data.unwrap(), 12); // Data
}
assert_eq!(res.1.is_none(), true);
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
