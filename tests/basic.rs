extern crate path_tree;

use path_tree::{NodeMetadata, PathTree};

#[test]
fn new_tree() {
    let mut tree = PathTree::<usize>::new("/", NodeMetadata::new());
    assert_eq!(tree.tree.path.len(), 1);

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

    // println!("{:#?}", tree);

    let node = tree.find("/");
    // println!("/ {:#?}", node);
    assert_eq!(node.is_some(), true);
    let res = node.unwrap();
    assert_eq!(res.0.path, ['/']);
    if let Some(meta) = &res.0.data {
        assert_eq!(meta.data.unwrap(), 0);
    }
    assert_eq!(res.1, None);

    let node = tree.find("/users");
    // println!("/users: {:#?}", node);
    assert_eq!(node.is_some(), true);
    let res = node.unwrap();
    assert_eq!(res.0.path, ['u', 's', 'e', 'r', 's']);
    assert_eq!(res.1, None);

    let node = tree.find("/about");
    // println!("/about {:#?}", node);
    assert_eq!(node.is_some(), true);
    let res = node.unwrap();
    assert_eq!(res.0.path, ['a', 'b', 'o', 'u', 't']);
    assert_eq!(res.1, None);

    let node = tree.find("/about/");
    // println!("/about/ {:#?}", node);
    assert_eq!(node.is_some(), true);
    let res = node.unwrap();
    assert_eq!(res.0.path, ['/']);
    assert_eq!(res.1, None);

    let node = tree.find("/about/us");
    // println!("/about/us {:#?}", node);
    assert_eq!(node.is_some(), true);
    let res = node.unwrap();
    assert_eq!(res.0.path, ['u', 's']);
    assert_eq!(res.1, None);

    let node = tree.find("/username");
    // println!("/username {:#?}", node);
    assert_eq!(node.is_some(), true);
    let res = node.unwrap();
    assert_eq!(res.0.path, [':']);
    if let Some(meta) = &res.0.data {
        assert_eq!(meta.data.unwrap(), 7);
    }
    assert_eq!(res.1.unwrap(), [("username", "username")]);

    let node = tree.find("/user/s");
    // println!("/user/s {:#?}", node);
    let res = node.unwrap();
    assert_eq!(res.0.path, ['*']);
    if let Some(meta) = &res.0.data {
        assert_eq!(meta.data.unwrap(), 8); // Data
    }
    assert_eq!(res.1.unwrap(), [("any", "user/s")]);

    let node = tree.find("/users/fundon/repo");
    // println!("/users/fundon/repo {:#?}", node);
    let res = node.unwrap();
    assert_eq!(res.0.path, [':']);
    if let Some(meta) = &res.0.data {
        assert_eq!(meta.data.unwrap(), 3);
    }
    assert_eq!(res.1.unwrap(), [("id", "fundon"), ("org", "repo")]);

    let node = tree.find("/users/fundon/repos");
    // println!("/users/fundon/repos {:#?}", node);
    let res = node.unwrap();
    assert_eq!(res.0.path, "repos".chars().collect::<Vec<char>>());
    assert_eq!(res.1.unwrap(), [("user_id", "fundon")]);

    let node = tree.find("/users/fundon/repos/trek-rs");
    // println!("/users/fundon/repos/233 {:#?}", node);
    let res = node.unwrap();
    assert_eq!(res.0.path, [':']);
    if let Some(meta) = &res.0.data {
        assert_eq!(meta.data.unwrap(), 5); // Data
    }
    assert_eq!(res.1.unwrap(), [("user_id", "fundon"), ("id", "trek-rs"),]);

    let node = tree.find("/users/fundon/repos/trek-rs/");
    // println!("/users/fundon/repos/233/ {:#?}", node);
    let res = node.unwrap();
    assert_eq!(res.0.path, ['*']);
    assert_eq!(res.1.unwrap(), [("user_id", "fundon"), ("id", "trek-rs"),]);

    let node = tree.find("/users/fundon/repos/trek-rs/noder");
    // println!("/users/fundon/repos/trek-rs/noder {:#?}", node);
    let res = node.unwrap();
    assert_eq!(res.0.path, ['*']);
    assert_eq!(
        res.1.unwrap(),
        [("user_id", "fundon"), ("id", "trek-rs"), ("any", "noder"),]
    );

    let node = tree.find("/users/fundon/repos/trek-rs/noder/issues");
    // println!("/users/fundon/repos/trek-rs/noder/issues {:#?}", node);
    let res = node.unwrap();
    assert_eq!(res.0.path, ['*']);
    if let Some(meta) = &res.0.data {
        assert_eq!(meta.data.unwrap(), 6); // Data
    }
    assert_eq!(
        res.1.unwrap(),
        [
            ("user_id", "fundon"),
            ("id", "trek-rs"),
            ("any", "noder/issues"),
        ]
    );

    let node = tree.find("/users/repos/");
    // println!("/users/repos/ {:#?}", node);
    let res = node.unwrap();
    assert_eq!(res.0.path, "*".chars().collect::<Vec<char>>());
    if let Some(meta) = &res.0.data {
        assert_eq!(meta.data.unwrap(), 12); // Data
    }
    assert_eq!(res.1.is_none(), true);

    let node = tree.find("/about/as");
    // println!("/about/as {:#?}", node);
    let res = node.unwrap();
    assert_eq!(res.0.path, ['*']);
    assert_eq!(res.1.unwrap(), [("any", "about/as")]);
}

#[test]
fn statics() {
    let mut tree = PathTree::new("/", NodeMetadata::new());
    let nodes = [
        "/hi",
        "/contact",
        "/co",
        "/c",
        "/a",
        "/ab",
        "/doc/",
        "/doc/go_faq.html",
        "/doc/go1.html",
        "/α",
        "/β",
    ];
    let mut i = 0;
    for node in &nodes {
        tree.insert(node, i);
        i += 1;
    }

    // println!("tree {:#?}", tree);

    let node = tree.find("/a");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, ['a']);

    let node = tree.find("/");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/hi");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, ['h', 'i']);

    let node = tree.find("/contact");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, "ntact".chars().collect::<Vec<char>>());

    let node = tree.find("/co");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, "o".chars().collect::<Vec<char>>());

    let node = tree.find("/con");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/cona");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/no");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/ab");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, ['b']);

    let node = tree.find("/α");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, ['α']);

    let node = tree.find("/β");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, ['β']);
}

#[test]
fn wildcards() {
    let mut tree = PathTree::new("/", NodeMetadata::new());
    let nodes = [
        "/",
        "/cmd/:tool/:sub",
        "/cmd/:tool/",
        "/cmd/vet",
        "/src/*filepath",
        "/src1/",
        "/src1/*filepath",
        "/src2*filepath",
        "/search/",
        "/search/:query",
        "/search/invalid",
        "/user_:name",
        "/user_:name/about",
        "/user_x",
        "/files/:dir/*filepath",
        "/doc/",
        "/doc/rust_faq.html",
        "/doc/rust1.html",
        "/info/:user/public",
        "/info/:user/project/:project",
    ];
    let mut i = 0;
    for node in &nodes {
        tree.insert(node, i);
        i += 1;
    }

    // println!("tree {:#?}", tree);

    let node = tree.find("/");
    // println!("/ {:#?}", node);
    assert!(node.is_some());

    let node = tree.find("/cmd/test/");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, ['/']);
    assert_eq!(res.1.unwrap(), [("tool", "test")]);

    let node = tree.find("/cmd/test");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/cmd/test/3");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, [':']);
    assert_eq!(res.1.unwrap(), [("tool", "test"), ("sub", "3")]);

    let node = tree.find("/src/");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, ['*']);
    assert_eq!(res.1, None);

    let node = tree.find("/src/some/file.png");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, ['*']);
    assert_eq!(res.1.unwrap(), [("filepath", "some/file.png")]);

    let node = tree.find("/search/");
    // println!("/ {:#?}", node);
    assert!(node.is_some());

    let node = tree.find("/search/someth!ng+in+ünìcodé");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, [':']);
    assert_eq!(res.1.unwrap(), [("query", "someth!ng+in+ünìcodé")]);

    let node = tree.find("/search/someth!ng+in+ünìcodé/");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/user_rust");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, [':']);
    assert_eq!(res.1.unwrap(), [("name", "rust")]);

    let node = tree.find("/user_rust/about");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, "/about".chars().collect::<Vec<char>>());
    assert_eq!(res.1.unwrap(), [("name", "rust")]);

    let node = tree.find("/files/js/inc/framework.js");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, ['*']);
    assert_eq!(
        res.1.unwrap(),
        [("dir", "js"), ("filepath", "inc/framework.js")]
    );

    let node = tree.find("/info/gordon/public");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, "ublic".chars().collect::<Vec<char>>());
    assert_eq!(res.1.unwrap(), [("user", "gordon")]);

    let node = tree.find("/info/gordon/project/rust");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, [':']);
    assert_eq!(res.1.unwrap(), [("user", "gordon"), ("project", "rust")]);
}

#[test]
fn single_named_parameter() {
    //  Pattern: /users/:id
    //
    //      /users/gordon              match
    //      /users/you                 match
    //      /users/gordon/profile      no match
    //      /users/                    no match
    let mut tree = PathTree::new("/", NodeMetadata::new());

    tree.insert("/users/:id", 0);

    // println!("tree {:#?}", tree);

    let node = tree.find("/users/");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/users/gordon/profile");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/users/gordon");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, [':']);
    if let Some(data) = &node.data {
        assert_eq!(data.key, true);
    }
    assert_eq!(res.1.unwrap(), [("id", "gordon")]);

    let node = tree.find("/users/you");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    let node = &res.0;
    assert_eq!(node.path, [':']);
    if let Some(data) = &node.data {
        assert_eq!(data.key, true);
    }
    assert_eq!(res.1.unwrap(), [("id", "you")]);
}

#[test]
fn static_and_named_parameter() {
    //  Pattern: /a/b/c
    //  Pattern: /a/c/d
    //  Pattern: /a/c/a
    //  Pattern: /:id/c/e
    //
    //      /a/b/c                  match
    //      /a/c/d                  match
    //      /a/c/a                  match
    //      /a/c/e                  match
    let mut tree = PathTree::new("/", NodeMetadata::new());

    tree.insert("/a/b/c", "/a/b/c");
    tree.insert("/a/c/d", "/a/c/d");
    tree.insert("/a/c/a", "/a/c/a");
    tree.insert("/:id/c/e", "/:id/c/e");

    // println!("tree {:#?}", tree);

    let node = tree.find("/");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/a/b/c");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['b', '/', 'c']);
    assert_eq!(res.1, None);

    let node = tree.find("/a/c/d");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['d']);
    assert_eq!(res.1, None);

    let node = tree.find("/a/c/a");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['a']);
    assert_eq!(res.1, None);

    let node = tree.find("/a/c/e");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['/', 'c', '/', 'e']);
    assert_eq!(res.1.unwrap(), [("id", "a")]);
}

#[test]
fn multi_named_parameters() {
    //  Pattern: /:lang/:keyword
    //  Pattern: /:id
    //
    //      /rust                     match
    //      /rust/let                 match
    //      /rust/let/const           no match
    //      /rust/let/                no match
    //      /rust/                    no match
    //      /                         no match
    let mut tree = PathTree::new("/", NodeMetadata::new());

    tree.insert("/:lang/:keyword", true);
    tree.insert("/:id", true);
    // tree.insert("/:id/:post_id", NodeMetadata::new());

    // println!("tree {:#?}", tree);

    let node = tree.find("/");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/rust/");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/rust/let/");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/rust/let/const");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/rust/let");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, [':']);
    assert_eq!(res.1.unwrap(), [("lang", "rust"), ("keyword", "let")]);

    let node = tree.find("/rust");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, [':']);
    assert_eq!(res.1.unwrap(), [("id", "rust")]);
}

#[test]
fn catch_all_parameter() {
    //  Pattern: /src/*filepath
    //
    //      /src                      no match
    //      /src/                     match
    //      /src/somefile.go          match
    //      /src/subdir/somefile.go   match
    let mut tree = PathTree::new("/", NodeMetadata::new());

    tree.insert("/src/*filepath", "* files");

    let node = tree.find("/src");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/src/");
    // println!("/src/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['*']);
    assert!(res.1.is_none());

    let node = tree.find("/src/somefile.rs");
    // println!("/src/somefile.rs {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['*']);
    assert_eq!(res.1.unwrap(), [("filepath", "somefile.rs")]);

    let node = tree.find("/src/subdir/somefile.rs");
    // println!("/src/subdir/somefile.rs {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['*']);
    assert_eq!(res.1.unwrap(), [("filepath", "subdir/somefile.rs")]);

    let node = tree.find("/src.rs");
    // println!("/src.rs {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/rust");
    // println!("/rust {:#?}", node);
    assert!(node.is_none());

    // split node, 'src/' is key node
    tree.insert("/src/", "dir");

    let node = tree.find("/src/");
    // println!("/src/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, "src/".chars().collect::<Vec<char>>());
    assert!(res.1.is_none());
}

#[test]
fn static_and_catch_all_parameter() {
    //  Pattern: /a/b/c
    //  Pattern: /a/c/d
    //  Pattern: /a/c/a
    //  Pattern: /a/*c
    //
    //      /a/b/c                  match
    //      /a/c/d                  match
    //      /a/c/a                  match
    //      /a/c/e                  match
    let mut tree = PathTree::new("/", NodeMetadata::new());

    tree.insert("/a/b/c", "/a/b/c");
    tree.insert("/a/c/d", "/a/c/d");
    tree.insert("/a/c/a", "/a/c/a");
    tree.insert("/a/*c", "/a/*c");

    // println!("tree {:#?}", tree);

    let node = tree.find("/");
    // println!("/ {:#?}", node);
    assert!(node.is_none());

    let node = tree.find("/a/b/c");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['b', '/', 'c']);
    assert_eq!(res.1, None);

    let node = tree.find("/a/c/d");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['d']);
    assert_eq!(res.1, None);

    let node = tree.find("/a/c/a");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['a']);
    assert_eq!(res.1, None);

    let node = tree.find("/a/c/e");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['*']);
    assert_eq!(res.1.unwrap(), [("c", "c/e")]);
}

#[test]
fn root_catch_all_parameter() {
    //  Pattern: /
    //  Pattern: /*
    //  Pattern: /users/*
    //
    //      /                  match *
    //      /download          match *
    //      /users/fundon      match users *
    let mut tree = PathTree::<fn() -> usize>::new("/", NodeMetadata::new());

    tree.insert("/", || 1);
    tree.insert("/*", || 2);
    tree.insert("/users/*", || 3);

    // println!("tree {:#?}", tree);

    let node = tree.find("/");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['/']);
    assert_eq!(res.0.data.is_some(), true);
    if let Some(meta) = &res.0.data {
        assert_eq!(meta.data.unwrap()(), 1);
    }

    let node = tree.find("/download");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['*']);
    assert_eq!(res.0.data.is_some(), true);
    if let Some(meta) = &res.0.data {
        assert_eq!(meta.data.unwrap()(), 2);
    }
    assert_eq!(res.1.unwrap(), [("", "download")]);

    let node = tree.find("/users/fundon");
    // println!("/ {:#?}", node);
    assert!(node.is_some());
    let res = node.unwrap();
    assert_eq!(res.0.path, ['*']);
    assert_eq!(res.0.data.is_some(), true);
    if let Some(meta) = &res.0.data {
        assert_eq!(meta.data.unwrap()(), 3);
    }
    assert_eq!(res.1.unwrap(), [("", "fundon")]);
}
