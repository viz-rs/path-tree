use path_tree::PathTree;
use rand::seq::SliceRandom;

#[allow(dead_code)]
#[path = "fixtures/github.rs"]
mod github;

use github::{ROUTES_URLS, ROUTES_WITH_COLON};

#[test]
fn github() {
    let mut routes = ROUTES_WITH_COLON
        .iter()
        .zip(ROUTES_URLS.iter())
        .map(|(a, b)| (*a, *b))
        .collect::<Vec<_>>();

    routes.shuffle(&mut rand::thread_rng());

    let mut tree: PathTree<usize> = PathTree::new();

    for (i, (r, ..)) in routes.iter().enumerate() {
        tree.insert(r, i);
    }

    // println!("tree: {:#?}", tree);

    for (i, (_, r)) in routes.iter().enumerate() {
        let n = tree.find(r).unwrap();
        assert_eq!(*n.0, i);
        // println!("route params: {:#?}", n.1);
    }
}
