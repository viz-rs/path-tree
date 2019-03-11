extern crate path_tree;

use path_tree::PathTree;

#[test]
fn new_tree() {
    let mut tree: PathTree<usize> = PathTree::default();

    const ROUTES: [&'static str; 13] = [
        "/",
        "/users",
        "/users/:id",
        "/users/:id/:org",
        "/users/:user_id/repos",
        "/users/:user_id/repos/:id",
        "/users/:user_id/repos/:id/*any",
        "/:username",
        "/*any",
        "/about",
        "/about/",
        "/about/us",
        "/users/repos/*any",
    ];

    for (i, u) in ROUTES.iter().enumerate() {
        tree.insert(u, i);
    }

    const VALID_URLS: [&'static str; 13] = [
        "/",
        "/users",
        "/users/fundon",
        "/users/fundon/trek-rs",
        "/users/fundon/repos",
        "/users/fundon/repos/path-tree",
        "/users/fundon/repos/trek-rs/trek",
        "/fundon",
        "/fundon/trek-rs/trek",
        "/about",
        "/about/",
        "/about/us",
        "/users/repos/trek-rs/trek",
    ];

    let valid_res = vec![
        vec![],
        vec![],
        vec![("id", "fundon")],
        vec![("id", "fundon"), ("org", "trek-rs")],
        vec![("user_id", "fundon")],
        vec![("user_id", "fundon"), ("id", "path-tree")],
        vec![("user_id", "fundon"), ("id", "trek-rs"), ("any", "trek")],
        vec![("username", "fundon")],
        vec![("any", "fundon/trek-rs/trek")],
        vec![],
        vec![],
        vec![],
        vec![("any", "trek-rs/trek")],
    ];

    for (i, u) in VALID_URLS.iter().enumerate() {
        let res = tree.find(u).unwrap();
        // println!("{}, {}, {:#?}", i, r, res);
        assert_eq!(*res.0, i);
        assert_eq!(res.1, valid_res[i]);
    }
}

#[test]
fn statics() {
    let mut tree = PathTree::<usize>::new();

    const ROUTES: [&'static str; 11] = [
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

    for (i, u) in ROUTES.iter().enumerate() {
        tree.insert(u, i);
    }

    for (i, u) in ROUTES.iter().enumerate() {
        let res = tree.find(u).unwrap();
        // println!("{}, {}, {:#?}", i, r, res);
        assert_eq!(*res.0, i);
    }
}

#[test]
fn wildcards() {
    let mut tree = PathTree::<usize>::new();

    const ROUTES: [&'static str; 20] = [
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

    for (i, u) in ROUTES.iter().enumerate() {
        tree.insert(u, i);
    }

    // println!("tree: {:#?}", tree);

    let valid_res = vec![
        ("/", 0, vec![]),
        ("/cmd/test/", 2, vec![("tool", "test")]),
        ("/cmd/test/3", 1, vec![("tool", "test"), ("sub", "3")]),
        ("/src/", 4, vec![]),
        ("/src/some/file.png", 4, vec![("filepath", "some/file.png")]),
        (
            "/search/someth!ng+in+ünìcodé",
            9,
            vec![("query", "someth!ng+in+ünìcodé")],
        ),
        ("/user_rust", 11, vec![("name", "rust")]),
        ("/user_rust/about", 12, vec![("name", "rust")]),
        (
            "/files/js/inc/framework.js",
            14,
            vec![("dir", "js"), ("filepath", "inc/framework.js")],
        ),
        ("/info/gordon/public", 18, vec![("user", "gordon")]),
        (
            "/info/gordon/project/rust",
            19,
            vec![("user", "gordon"), ("project", "rust")],
        ),
    ];

    for (u, h, p) in valid_res {
        let res = tree.find(u).unwrap();
        // println!("{}, {:#?}", r, res);
        assert_eq!(*res.0, h);
        assert_eq!(res.1, p);
    }
}

#[test]
fn single_named_parameter() {
    //  Pattern: /users/:id
    //
    //      /users/gordon              match
    //      /users/you                 match
    //      /users/gordon/profile      no match
    //      /users/                    no match
    let mut tree = PathTree::new();

    tree.insert("/users/:id", 0);

    let res = vec![
        ("/", false),
        ("/users/gordon", true),
        ("/users/you", true),
        ("/users/gordon/profile", false),
        ("/users/", false),
        ("/users", false),
    ];

    for (u, b) in res {
        let r = tree.find(u);
        assert_eq!(r.is_some(), b);
    }
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
    let mut tree = PathTree::new();

    tree.insert("/a/b/c", "/a/b/c");
    tree.insert("/a/c/d", "/a/c/d");
    tree.insert("/a/c/a", "/a/c/a");
    tree.insert("/:id/c/e", "/:id/c/e");

    let res = vec![
        ("/", false, "", vec![]),
        ("/a/b/c", true, "/a/b/c", vec![]),
        ("/a/c/d", true, "/a/c/d", vec![]),
        ("/a/c/a", true, "/a/c/a", vec![]),
        ("/a/c/e", true, "/:id/c/e", vec![("id", "a")]),
    ];

    for (u, b, a, p) in res {
        let r = tree.find(u);
        assert_eq!(r.is_some(), b);
        if r.is_some() {
            let res = r.unwrap();
            assert_eq!(*res.0, a);
            assert_eq!(res.1, p);
        }
    }
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
    let mut tree = PathTree::new();

    tree.insert("/:lang/:keyword", true);
    tree.insert("/:id", true);

    let res = vec![
        ("/", false, false, vec![]),
        ("/rust/", false, false, vec![]),
        ("/rust/let/", false, false, vec![]),
        ("/rust/let/const", false, false, vec![]),
        (
            "/rust/let",
            true,
            true,
            vec![("lang", "rust"), ("keyword", "let")],
        ),
        ("/rust", true, true, vec![("id", "rust")]),
    ];

    for (u, b, a, p) in res {
        let r = tree.find(u);
        assert_eq!(r.is_some(), b);
        if r.is_some() {
            let res = r.unwrap();
            assert_eq!(*res.0, a);
            assert_eq!(res.1, p);
        }
    }
}

#[test]
fn catch_all_parameter() {
    //  Pattern: /src/*filepath
    //
    //      /src                      no match
    //      /src/                     match
    //      /src/somefile.go          match
    //      /src/subdir/somefile.go   match
    let mut tree = PathTree::new();

    tree.insert("/src/*filepath", "* files");

    let res = vec![
        ("/src", false, vec![]),
        ("/src/", true, vec![]),
        ("/src/somefile.rs", true, vec![("filepath", "somefile.rs")]),
        (
            "/src/subdir/somefile.rs",
            true,
            vec![("filepath", "subdir/somefile.rs")],
        ),
        ("/src.rs", false, vec![]),
        ("/rust", false, vec![]),
    ];

    for (u, b, p) in res {
        let r = tree.find(u);
        assert_eq!(r.is_some(), b);
        if r.is_some() {
            let res = r.unwrap();
            assert_eq!(*res.0, "* files");
            assert_eq!(res.1, p);
        }
    }

    tree.insert("/src/", "dir");

    let r = tree.find("/src/");
    assert_eq!(r.is_some(), true);
    if r.is_some() {
        let res = r.unwrap();
        assert_eq!(*res.0, "dir");
        assert_eq!(res.1, vec![]);
    }
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
    let mut tree = PathTree::new();

    tree.insert("/a/b/c", "/a/b/c");
    tree.insert("/a/c/d", "/a/c/d");
    tree.insert("/a/c/a", "/a/c/a");
    tree.insert("/a/*c", "/a/*c");

    let res = vec![
        ("/", false, "", vec![]),
        ("/a/b/c", true, "/a/b/c", vec![]),
        ("/a/c/d", true, "/a/c/d", vec![]),
        ("/a/c/a", true, "/a/c/a", vec![]),
        ("/a/c/e", true, "/a/*c", vec![("c", "c/e")]),
    ];

    for (u, b, a, p) in res {
        let r = tree.find(u);
        assert_eq!(r.is_some(), b);
        if r.is_some() {
            let res = r.unwrap();
            assert_eq!(*res.0, a);
            assert_eq!(res.1, p);
        }
    }
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
    let mut tree = PathTree::<fn() -> usize>::new();

    tree.insert("/", || 1);
    tree.insert("/*", || 2);
    tree.insert("/users/*", || 3);

    let res = vec![
        ("/", true, 1, vec![]),
        ("/download", true, 2, vec![("", "download")]),
        ("/users/fundon", true, 3, vec![("", "fundon")]),
    ];

    for (u, b, a, p) in res {
        let r = tree.find(u);
        assert_eq!(r.is_some(), b);
        if r.is_some() {
            let res = r.unwrap();
            assert_eq!(res.0(), a);
            assert_eq!(res.1, p);
        }
    }
}

#[test]
fn root_catch_all_parameter_1() {
    //  Pattern: /*
    //
    //      /                  match *
    //      /download          match *
    //      /users/fundon      match *
    let mut tree = PathTree::<fn() -> usize>::new();

    tree.insert("/*", || 1);

    let res = vec![
        ("/", true, 1, vec![]),
        ("/download", true, 1, vec![("", "download")]),
        ("/users/fundon", true, 1, vec![("", "users/fundon")]),
    ];

    // println!("tree: {:#?}", tree);

    for (u, b, a, p) in res {
        let r = tree.find(u);
        //println!("route: {:#?}", r);
        assert_eq!(r.is_some(), b);
        if r.is_some() {
            let res = r.unwrap();
            assert_eq!(res.0(), a);
            assert_eq!(res.1, p);
        }
    }

    tree.insert("/", || 0);
    let r = tree.find("/");
    //println!("route: {:#?}", r);
    assert!(r.is_some());
    if r.is_some() {
        let res = r.unwrap();
        assert_eq!(res.0(), 0);
        assert_eq!(res.1, []);
    }
}
