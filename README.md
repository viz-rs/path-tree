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

## Examples

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

```rust
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use path_tree::PathTree;

static NOT_FOUND: &[u8] = b"Not Found";

type Params = Vec<(String, String)>;

trait Handler: Send + Sync + 'static {
    fn call<'a>(&'a self, req: Request<Body>) -> Pin<Box<dyn Future<Output = Body> + Send + 'a>>;
}

impl<F, R> Handler for F
where
    F: Send + Sync + 'static + Fn(Request<Body>) -> R,
    R: Future<Output = Body> + Send + 'static,
{
    fn call<'a>(&'a self, req: Request<Body>) -> Pin<Box<dyn Future<Output = Body> + Send + 'a>> {
        let fut = (self)(req);
        Box::pin(async move { fut.await })
    }
}

async fn index(_: Request<Body>) -> Body {
    Body::from("Hello, Web!")
}

async fn hello_world(req: Request<Body>) -> Body {
    let params = req.extensions().get::<Params>().unwrap();
    let mut s = String::new();
    s.push_str("Hello, World!\n");
    for (_, v) in params {
        s.push_str(&format!("param = {}", v));
    }
    Body::from(s)
}

async fn hello_user(req: Request<Body>) -> Body {
    let params = req.extensions().get::<Params>().unwrap();
    let mut s = String::new();
    s.push_str("Hello, ");
    for (k, v) in params {
        s.push_str(&format!("{} = {}", k, v));
    }
    s.push_str("!");
    Body::from(s)
}

async fn hello_rust(_: Request<Body>) -> Body {
    Body::from("Hello, Rust!")
}

async fn login(_req: Request<Body>) -> Body {
    Body::from("I'm logined!")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    let mut tree = PathTree::<Box<dyn Handler>>::new();
    tree.insert("/GET/", Box::new(index));
    tree.insert("/GET/*", Box::new(hello_world));
    tree.insert("/GET/hello/:name", Box::new(hello_user));
    tree.insert("/GET/rust", Box::new(hello_rust));
    tree.insert("/POST/login", Box::new(login));

    let tree = Arc::new(tree);

    let make_service = make_service_fn(move |_| {
        let router = Arc::clone(&tree);

        async move {
            Ok::<_, Infallible>(service_fn(move |mut req| {
                let router = router.clone();
                let path = "/".to_owned() + req.method().as_str() + req.uri().path();
                let builder = Response::builder();

                async move {
                    Ok::<_, Infallible>(
                        match router.find(&path) {
                            Some((handler, params)) => {
                                let p = params
                                    .iter()
                                    .map(|p| (p.0.to_owned(), p.1.to_owned()))
                                    .collect::<Params>();
                                req.extensions_mut().insert(p);
                                builder.body(handler.call(req).await)
                            }
                            None => builder.status(StatusCode::NOT_FOUND).body(NOT_FOUND.into()),
                        }
                        .unwrap(),
                    )
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
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
