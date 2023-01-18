use std::{convert::Infallible, future::Future, pin::Pin, sync::Arc};

use hyper::{
    server::Server,
    service::{make_service_fn, service_fn},
    Body, Request, Response, StatusCode,
};
use path_tree::PathTree;

static NOT_FOUND: &[u8] = b"Not Found";

type Params = Vec<(String, String)>;

trait Handler: Send + Sync + 'static {
    fn call<'a>(
        &'a self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Response<Body>> + Send + 'a>>;
}

impl<F, R> Handler for F
where
    F: Send + Sync + 'static + Fn(Request<Body>) -> R,
    R: Future<Output = Response<Body>> + Send + 'static,
{
    fn call<'a>(
        &'a self,
        req: Request<Body>,
    ) -> Pin<Box<dyn Future<Output = Response<Body>> + Send + 'a>> {
        let fut = (self)(req);
        Box::pin(async move { fut.await })
    }
}

async fn index(_: Request<Body>) -> Response<Body> {
    Response::new(Body::from("Hello, Web!"))
}

async fn hello_world(req: Request<Body>) -> Response<Body> {
    let params = req.extensions().get::<Params>().unwrap();
    let mut s = String::new();
    s.push_str("Hello, World!\n");
    for (_, v) in params {
        s.push_str(&format!("param = {v}"));
    }
    Response::new(Body::from(s))
}

async fn hello_user(req: Request<Body>) -> Response<Body> {
    let params = req.extensions().get::<Params>().unwrap();
    let mut s = String::new();
    s.push_str("Hello, ");
    for (k, v) in params {
        s.push_str(&format!("{k} = {v}"));
    }
    s.push('!');
    Response::new(Body::from(s))
}

async fn hello_rust(_: Request<Body>) -> Response<Body> {
    Response::new(Body::from("Hello, Rust!"))
}

async fn login(_req: Request<Body>) -> Response<Body> {
    Response::new(Body::from("I'm logined!"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr = ([127, 0, 0, 1], 3000).into();

    // /
    // ├── GET/ •0
    // │   ├── hello/
    // │   │   └── : •2
    // │   ├── rust •3
    // │   └── ** •1
    // └── POST/login •4
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

                async move {
                    Ok::<_, Infallible>(match router.find(&path) {
                        Some((handler, route)) => {
                            let p = route
                                .params()
                                .iter()
                                .map(|p| (p.0.to_string(), p.1.to_string()))
                                .collect::<Params>();
                            req.extensions_mut().insert(p);
                            handler.call(req).await
                        }
                        None => Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(NOT_FOUND.into())
                            .unwrap(),
                    })
                }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_service);

    println!("Listening on http://{addr}");

    server.await?;

    Ok(())
}
