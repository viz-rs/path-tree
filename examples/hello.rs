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
