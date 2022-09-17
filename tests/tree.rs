use path_tree::*;

#[test]
fn tree() {
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/", 1);
    tree.insert("/api/v1/:param/*", 2);
    tree.insert("/api", 3);
    dbg!(tree);

    // assert!(tree.find("/api/v1/entity").is_none());
    // assert!(tree.find("/api/v1/entity/").is_some());
    // assert!(tree.find("/api/v1/entity/1").is_some());
    // assert!(tree.find("/api/v1/entity/1/2").is_some());
    // assert!(tree.find("/api/v").is_none());
    // assert!(tree.find("/api/v1").is_none());
    // assert!(tree.find("/api/v2/").is_none());
    // assert!(tree.find("/api/v1/entities").is_none());
}
