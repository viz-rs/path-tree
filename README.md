<h1 align="center">path-tree</h1>
<div align="center">
  <p><strong>A lightweight high performance HTTP request router for Rust.</strong></p>
</div>

<br />

<div align="center">
  <!-- Crates version -->
  <a href="https://crates.io/crates/path-tree">
    <img src="https://img.shields.io/crates/v/path-tree.svg?style=flat-square"
    alt="Crates.io version" />
  </a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/path-tree">
    <img src="https://img.shields.io/crates/d/path-tree.svg?style=flat-square"
      alt="Download" />
  </a>
  <!-- docs.rs docs -->
  <a href="https://docs.rs/path-tree">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="docs.rs docs" />
  </a>
</div>

## Features

- **Fast**: See benchmark

- **Micro**: The [src/lib.rs](src/lib.rs) file is ~385 lines of code (Includes comments)

- **Flexible**:

  - Named parameters. e.g. `:name`.

  - Catch-All parameters. e.g. `*any`, it must always be at the end of the pattern.

  - Supports multiple naming for the same path segment. e.g. `/users/:id` and `/users/:user_id/repos`.

  - Don't care about routes orders, recursive lookup, `Static` -> `Named` -> `Catch-All`.

## Benchmark

```shell
$ cargo bench
```

<details>
    <summary>
        <a href="https://travis-ci.org/trek-rs/path-tree/builds/607606611" rel="nofollow">From Travis</a>
    </summary>

```
path_insert/path_tree_insert

                        time:   [285.97 us 286.32 us 286.73 us]

Found 6 outliers among 50 measurements (12.00%)

  4 (8.00%) high mild

  2 (4.00%) high severe

path_insert/route_recognizer_add

                        time:   [194.49 us 194.95 us 195.49 us]

Found 4 outliers among 50 measurements (8.00%)

  1 (2.00%) high mild

  3 (6.00%) high severe

path_insert/path_table_setup

                        time:   [94.507 us 94.655 us 94.847 us]

Found 9 outliers among 50 measurements (18.00%)

  3 (6.00%) high mild

  6 (12.00%) high severe

path_insert/actix_router_path

                        time:   [14.998 ms 15.013 ms 15.029 ms]

Found 8 outliers among 50 measurements (16.00%)

  2 (4.00%) low mild

  3 (6.00%) high mild

  3 (6.00%) high severe

path_find/path_tree_find

                        time:   [375.39 us 375.73 us 376.04 us]

Found 2 outliers among 50 measurements (4.00%)

  1 (2.00%) high mild

  1 (2.00%) high severe

path_find/route_recognizer_recognize

                        time:   [1.1090 ms 1.1110 ms 1.1138 ms]

Found 4 outliers among 50 measurements (8.00%)

  2 (4.00%) high mild

  2 (4.00%) high severe

path_find/path_table_route

                        time:   [158.79 us 159.96 us 161.24 us]

Found 4 outliers among 50 measurements (8.00%)

  4 (8.00%) high mild

path_find/actix_router_recognize

                        time:   [9.1690 ms 9.1891 ms 9.2135 ms]

Found 7 outliers among 50 measurements (14.00%)

  3 (6.00%) high mild

  4 (8.00%) high severe

```

</details>

### Path Find

![Path Find](resources/bench-find.svg)

### Path Insert

![Path Insert](resources/bench-insert.svg)

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

## Examples

### `async-await`

```rust
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Error, Request, Response, Server, StatusCode};
use path_tree::PathTree;
use std::sync::Arc;

static NOTFOUND: &[u8] = b"Not Found";

type Params<'a> = Vec<(&'a str, &'a str)>;

type Handler = fn(Request<Body>, Params) -> Body;

fn index(_: Request<Body>, _: Params) -> Body {
    Body::from("Hello, Web!")
}

fn hello_world(_: Request<Body>, params: Params) -> Body {
    let mut s = String::new();
    s.push_str("Hello, World!\n");
    for (_, v) in params {
        s.push_str(&format!("param = {}", v));
    }
    Body::from(s)
}

fn hello_user(_: Request<Body>, params: Params) -> Body {
    let mut s = String::new();
    s.push_str("Hello, ");
    for (k, v) in params {
        s.push_str(&format!("{} = {}", k, v));
    }
    s.push_str("!");
    Body::from(s)
}

fn hello_rust(_: Request<Body>, _: Params) -> Body {
    Body::from("Hello, Rust!")
}

fn login(_req: Request<Body>, _: Params) -> Body {
    Body::from("I'm logined!")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let mut tree = PathTree::<Handler>::new();
    tree.insert("/GET/", index);
    tree.insert("/GET/*", hello_world);
    tree.insert("/GET/hello/:name", hello_user);
    tree.insert("/GET/rust", hello_rust);
    tree.insert("/POST/login", login);

    let tree = Arc::new(tree);

    let make_service = make_service_fn(move |_| {
        let router = Arc::clone(&tree);

        async move {
            Ok::<_, Error>(service_fn(move |req| {
                let path = "/".to_owned() + req.method().as_str() + req.uri().path();

                dbg!(&path);

                let body = match router.find(&path) {
                    Some((handler, params)) => Response::new(handler(req, params)),
                    None => Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(NOTFOUND.into())
                        .unwrap(),
                };

                async move { Ok::<_, Error>(body) }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
```

### `normal`

```rust
extern crate futures;
extern crate hyper;
extern crate path_tree;

use futures::Future;
use hyper::server::Server;
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, StatusCode};
use path_tree::PathTree;
use std::sync::Arc;

type Params<'a> = Vec<(&'a str, &'a str)>;

type Handler = fn(Request<Body>, Params) -> Body;

fn index(_: Request<Body>, _: Params) -> Body {
    Body::from("Hello, Web!")
}

fn hello_world(_: Request<Body>, params: Params) -> Body {
    let mut s = String::new();
    s.push_str("Hello, World!\n");
    for (_, v) in params {
        s.push_str(&format!("param = {}", v));
    }
    Body::from(s)
}

fn hello_user(_: Request<Body>, params: Params) -> Body {
    let mut s = String::new();
    s.push_str("Hello, ");
    for (k, v) in params {
        s.push_str(&format!("{} = {}", k, v));
    }
    s.push_str("!");
    Body::from(s)
}

fn hello_rust(_: Request<Body>, _: Params) -> Body {
    Body::from("Hello, Rust!")
}

fn login(_req: Request<Body>, _: Params) -> Body {
    Body::from("I'm logined!")
}

fn main() {
    let addr = ([127, 0, 0, 1], 3000).into();

    let mut tree = PathTree::<Handler>::new();
    tree.insert("/GET/", index);
    tree.insert("/GET/*", hello_world);
    tree.insert("/GET/hello/:name", hello_user);
    tree.insert("/GET/rust", hello_rust);
    tree.insert("/POST/login", login);

    let tree = Arc::new(tree);

    let routing = move || {
        let router = Arc::clone(&tree);

        service_fn_ok(move |req| {
            let path = "/".to_owned() + req.method().as_str() + req.uri().path();

            dbg!(&path);

            match router.find(&path) {
                Some((handler, params)) => Response::new(handler(req, params)),
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("Not Found"))
                    .unwrap(),
            }
        })
    };

    let server = Server::bind(&addr)
        .serve(routing)
        .map_err(|e| eprintln!("server error: {}", e));

    hyper::rt::run(server);
}
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

at your option.

[radix tree]: https://github.com/trek-rs/radix-tree
[rax]: https://github.com/antirez/rax
[httprouter]: https://github.com/julienschmidt/httprouter
[echo]: https://github.com/labstack/echo
[trekjs]: https://github.com/trekjs/router
