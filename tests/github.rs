use path_tree::PathTree;

#[allow(dead_code)]
#[path = "fixtures/github.rs"]
mod github;

use github::{ROUTES_URLS, ROUTES_WITH_COLON};

#[test]
fn github() {
    let mut tree: PathTree<usize> = PathTree::new();
    for (i, r) in ROUTES_WITH_COLON.iter().enumerate() {
        tree.insert(r, i);
    }

    // println!("tree: {:#?}", tree);

    for (i, r) in ROUTES_URLS.iter().enumerate() {
        let n = tree.find(r).unwrap();
        assert_eq!(*n.0, i);
        // println!("route params: {:#?}", n.1);
    }
}
