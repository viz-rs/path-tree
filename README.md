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
</div>

## Features

- **Fast**: See benchmark

- **Micro**: The [src/lib.rs](src/lib.rs) file is ~405 lines of code (Includes comments)

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

/*
/ •0
├── api/
│   └── + •13
├── login •1
├── public/
│   └── ** •7
├── s
│   ├── ettings •3
│   │   └── /
│   │       └── : •4
│   └── ignup •2
└── : •5
    └── /
        └── : •6
            └── /
                ├── actions/
                │   └── :
                │       └── \:
                │           └── : •10
                ├── releases/download/
                │   └── :
                │       └── /
                │           └── :
                │               └── .
                │                   └── : •8
                ├── tags/
                │   └── :
                │       └── -
                │           └── :
                │               └── -
                │                   └── : •9
                ├── : •11
                └── ** •12
*/
let mut tree = PathTree::new();

tree.insert("/", 0)
    .insert("/login", 1)
    .insert("/signup", 2)
    .insert("/settings", 3)
    .insert("/settings/:page", 4)
    .insert("/:user", 5)
    .insert("/:user/:repo", 6)
    .insert("/public/:any*", 7)
    .insert("/:org/:repo/releases/download/:tag/:filename.:ext", 8)
    .insert("/:org/:repo/tags/:day-:month-:year", 9)
    .insert("/:org/:repo/actions/:name\\::verb", 10)
    .insert("/:org/:repo/:page", 11)
    .insert("/:org/:repo/*", 12)
    .insert("/api/+", 13);

let r = tree.find("/").unwrap();
assert_eq!(r.value, &0);
assert_eq!(r.params(), vec![]);

let r = tree.find("/login").unwrap();
assert_eq!(r.value, &1);
assert_eq!(r.params(), vec![]);

let r = tree.find("/settings/admin").unwrap();
assert_eq!(r.value, &4);
assert_eq!(r.params(), vec![("page", "admin")]);

let r = tree.find("/viz-rs").unwrap();
assert_eq!(r.value, &5);
assert_eq!(r.params(), vec![("user", "viz-rs")]);

let r = tree.find("/viz-rs/path-tree").unwrap();
assert_eq!(r.value, &6);
assert_eq!(r.params(), vec![("user", "viz-rs"), ("repo", "path-tree")]);

let r = tree.find("/rust-lang/rust-analyzer/releases/download/2022-09-12/rust-analyzer-aarch64-apple-darwin.gz").unwrap();
assert_eq!(r.value, &8);
assert_eq!(
    r.params(),
    vec![
        ("org", "rust-lang"),
        ("repo", "rust-analyzer"),
        ("tag", "2022-09-12"),
        ("filename", "rust-analyzer-aarch64-apple-darwin"),
        ("ext", "gz")
    ]
);

let r = tree.find("/rust-lang/rust-analyzer/tags/2022-09-12").unwrap();
assert_eq!(r.value, &9);
assert_eq!(
    r.params(),
    vec![
        ("org", "rust-lang"),
        ("repo", "rust-analyzer"),
        ("day", "2022"),
        ("month", "09"),
        ("year", "12")
    ]
);

let r = tree.find("/rust-lang/rust-analyzer/actions/ci:bench").unwrap();
assert_eq!(r.value, &10);
assert_eq!(
    r.params(),
    vec![
        ("org", "rust-lang"),
        ("repo", "rust-analyzer"),
        ("name", "ci"),
        ("verb", "bench"),
    ]
);

let r = tree.find("/rust-lang/rust-analyzer/stargazers").unwrap();
assert_eq!(r.value, &11);
assert_eq!(r.params(), vec![("org", "rust-lang"), ("repo", "rust-analyzer"), ("page", "stargazers")]);

let r = tree.find("/rust-lang/rust-analyzer/stargazers/404").unwrap();
assert_eq!(r.value, &12);
assert_eq!(r.params(), vec![("org", "rust-lang"), ("repo", "rust-analyzer"), ("*1", "stargazers/404")]);

let r = tree.find("/public/js/main.js").unwrap();
assert_eq!(r.value, &7);
assert_eq!(r.params(), vec![("any", "js/main.js")]);

let r = tree.find("/api/v1").unwrap();
assert_eq!(r.value, &13);
assert_eq!(r.params(), vec![("+1", "v1")]);
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
- [gofiber] router
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
[gofiber]: https://github.com/gofiber/fiber
[trekjs]: https://github.com/trekjs/router
