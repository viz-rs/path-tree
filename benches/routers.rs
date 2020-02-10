#[path = "../tests/fixtures/github.rs"]
mod github;

use actix_router::{Path as ActixPath, Router as ActixRouter};
use criterion::*;
use path_table::PathTable;
use path_tree::PathTree;
use route_recognizer::Router as RRRouter;

use github::*;

fn bench_path_insert(c: &mut Criterion) {
    c.bench(
        "path_insert",
        Benchmark::new("path_tree_insert", |b| {
            let mut tree: PathTree<usize> = PathTree::new();
            b.iter(|| {
                for (i, r) in ROUTES_WITH_COLON.iter().enumerate() {
                    tree.insert(r, i);
                }
            })
        })
        .with_function("route_recognizer_add", |b| {
            let mut router = RRRouter::<usize>::new();
            b.iter(|| {
                for (i, r) in ROUTES_WITH_COLON.iter().enumerate() {
                    router.add(r, i);
                }
            })
        })
        .with_function("path_table_setup", |b| {
            let mut table: PathTable<usize> = PathTable::new();
            b.iter(|| {
                for (i, r) in ROUTES_WITH_BRACES.iter().enumerate() {
                    *table.setup(r) = i;
                }
            })
        })
        .with_function("actix_router_path", |b| {
            let mut router = ActixRouter::<usize>::build();
            b.iter(|| {
                for (i, r) in ROUTES_WITH_BRACES.iter().enumerate() {
                    router.path(*r, i);
                }
            })
        })
        .sample_size(50),
    );
}

fn bench_path_find(c: &mut Criterion) {
    c.bench(
        "path_find",
        Benchmark::new("path_tree_find", |b| {
            let mut tree: PathTree<usize> = PathTree::new();
            for (i, r) in ROUTES_WITH_COLON.iter().enumerate() {
                tree.insert(r, i);
            }
            b.iter(|| {
                for (i, r) in ROUTES_URLS.iter().enumerate() {
                    let n = tree.find(r).unwrap();
                    assert_eq!(*n.0, i);
                }
            })
        })
        .with_function("route_recognizer_recognize", |b| {
            let mut router = RRRouter::<usize>::new();
            for (i, r) in ROUTES_WITH_COLON.iter().enumerate() {
                router.add(r, i);
            }
            b.iter(|| {
                for (i, r) in ROUTES_URLS.iter().enumerate() {
                    let n = router.recognize(r).unwrap();
                    assert_eq!(*n.handler, i);
                }
            })
        })
        .with_function("path_table_route", |b| {
            let mut table: PathTable<usize> = PathTable::new();
            for (i, r) in ROUTES_WITH_BRACES.iter().enumerate() {
                *table.setup(r) = i;
            }
            b.iter(|| {
                for (i, r) in ROUTES_URLS.iter().enumerate() {
                    let n = table.route(r).unwrap();
                    assert_eq!(*n.0, i);
                }
            })
        })
        .with_function("actix_router_recognize", |b| {
            let mut router = ActixRouter::<usize>::build();
            for (i, r) in ROUTES_WITH_BRACES.iter().enumerate() {
                router.path(*r, i);
            }
            let router = router.finish();
            b.iter(|| {
                for (i, r) in ROUTES_URLS.iter().enumerate() {
                    let mut path = ActixPath::new(*r);
                    let n = router.recognize(&mut path).unwrap();
                    assert_eq!(*n.0, i);
                }
            })
        })
        .sample_size(50),
    );
}

criterion_group!(benches, bench_path_insert, bench_path_find,);
criterion_main!(benches);
