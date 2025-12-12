use path_tree::PathTree;

#[test]
fn test_44() {
    let mut tree = PathTree::new();
    let _ = tree.insert("/test/:me", 0);
    let _ = tree.insert("/test/:me?", 1);
    let _ = tree.insert("/test/:me/now", 2);
    let _ = tree.insert("/test/:me?/now", 3);
    let _ = tree.insert("/test/:this+/now", 4);
    let _ = tree.insert("/test/:this+/*", 5);
    let _ = tree.insert("/test/:this+/now/*", 6);

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
    // add leading `/`
    path.insert(0, '/');
    assert_eq!(path, "/test/now");
    let (value, path) = tree.find(&path).unwrap();
    assert_eq!(value, &0);
    assert_eq!(path.params(), &[("me", "now")]);

    let (value, path) = tree.find("/test/multiple/paths/now").unwrap();
    assert_eq!(value, &4);
    assert_eq!(path.params(), &[("this", "multiple/paths")]);

    let (value, path) = tree.find("/test/multiple/paths/noww").unwrap();
    assert_eq!(value, &5);
    assert_eq!(path.params(), &[("this", "multiple"), ("*1", "paths/noww")]);

    let (value, path) = tree.find("/test/multiple/paths/now/12h").unwrap();
    assert_eq!(value, &6);
    assert_eq!(path.params(), &[("this", "multiple/paths"), ("*1", "12h")]);

    let (value, path) = tree.find("/test/multiple/paths/today/12h").unwrap();
    assert_eq!(value, &5);
    assert_eq!(
        path.params(),
        &[("this", "multiple"), ("*1", "paths/today/12h")]
    );
}
