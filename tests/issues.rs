use path_tree::PathTree;

#[test]
fn test_44() {
    let mut tree = PathTree::new();
    let _ = tree.insert("/test/:me", 0);
    let _ = tree.insert("/test/:me?", 1);
    let _ = tree.insert("/test/:me/now", 2);
    let _ = tree.insert("/test/:me?/now", 3);

    let (value, path) = tree.find("/test/").unwrap();
    assert_eq!(value, &1);
    assert_eq!(path.params(), &[("me", "")]);

    let (value, path) = tree.find("/test/now").unwrap();
    assert_eq!(value, &0);
    assert_eq!(path.params(), &[("me", "now")]);

    // not found
    let result = tree.find("/test//");
    assert!(result.is_none());

    let (value, path) = tree.find("/test//now").unwrap();
    assert_eq!(value, &3);
    assert_eq!(path.params(), &[("me", "")]);

    let (value, path) = tree.find(r"/test/\/now").unwrap();
    assert_eq!(value, &2);
    assert_eq!(path.params(), &[("me", r"\")]);

    // trim `/`
    let trimmed = "/test//now"
        .split('/')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>();
    let mut path = trimmed.join("/");
    // add lead with `/`
    path.insert(0, '/');
    assert_eq!(path, "/test/now");
    let (value, path) = tree.find(&path).unwrap();
    assert_eq!(value, &0);
    assert_eq!(path.params(), &[("me", "now")]);
}
