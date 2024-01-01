#![allow(unused_must_use)]
#![allow(clippy::too_many_lines)]

use path_tree::{Kind, PathTree, Piece, Position};
use rand::seq::SliceRandom;

#[test]
fn statics() {
    const ROUTES: [&str; 12] = [
        "/",
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

    let mut routes = ROUTES.to_vec();

    routes.shuffle(&mut rand::thread_rng());

    let mut tree = PathTree::<usize>::new();

    for (i, u) in routes.iter().enumerate() {
        tree.insert(u, i);
    }

    for (i, u) in routes.iter().enumerate() {
        let (h, _) = tree.find(u).unwrap();
        assert_eq!(h, &i);
    }
}

#[test]
fn wildcards() {
    const ROUTES: [&str; 20] = [
        "/",
        "/cmd/:tool/:sub",
        "/cmd/:tool/",
        "/cmd/vet",
        "/src/:filepath*",
        "/src1/",
        "/src1/:filepath*",
        "/src2:filepath*",
        "/search/",
        "/search/:query",
        "/search/invalid",
        "/user_:name",
        "/user_:name/about",
        "/user_x",
        "/files/:dir/:filepath*",
        "/doc/",
        "/doc/rust_faq.html",
        "/doc/rust1.html",
        "/info/:user/public",
        "/info/:user/project/:project",
    ];

    let mut routes = (0..20).zip(ROUTES.iter()).collect::<Vec<_>>();

    routes.shuffle(&mut rand::thread_rng());

    let mut tree = PathTree::<usize>::new();

    for (i, u) in &routes {
        tree.insert(u, *i);
    }

    let valid_res = vec![
        ("/", 0, vec![]),
        ("/cmd/test/", 2, vec![("tool", "test")]),
        ("/cmd/test/3", 1, vec![("tool", "test"), ("sub", "3")]),
        ("/src/", 4, vec![("filepath", "")]),
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

    for (u, v, p) in valid_res {
        let (h, r) = tree.find(u).unwrap();
        assert_eq!(*h, v);
        assert_eq!(r.params(), p);
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
        let n = tree.find(u);
        assert_eq!(n.is_some(), b);
    }
}

#[test]
fn repeated_single_named_param() {
    let mut tree = PathTree::new();

    tree.insert("/users/:id", 0);
    tree.insert("/users/:user_id", 1);

    let (h, r) = tree.find("/users/gordon").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(r.params(), vec![("user_id", "gordon")]);
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
        let n = tree.find(u);
        assert_eq!(n.is_some(), b);
        if let Some((h, r)) = n {
            assert_eq!(*h, a);
            assert_eq!(r.params(), p);
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
        let n = tree.find(u);
        assert_eq!(n.is_some(), b);
        if let Some((h, r)) = n {
            assert_eq!(*h, a);
            assert_eq!(r.params(), p);
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

    tree.insert("/src/:filepath*", "* files");

    let res = vec![
        ("/src", false, vec![]),
        ("/src/", true, vec![("filepath", "")]),
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
        let n = tree.find(u);
        assert_eq!(n.is_some(), b);
        if let Some((h, r)) = n {
            assert_eq!(*h, "* files");
            assert_eq!(r.params(), p);
        }
    }

    tree.insert("/src/", "dir");

    let n = tree.find("/src/");
    assert!(n.is_some());
    if let Some((h, r)) = n {
        assert_eq!(*h, "dir");
        assert_eq!(r.params(), vec![]);
    }
}

#[test]
fn catch_all_parameter_with_prefix() {
    //  Pattern: /commit_*sha
    //
    //      /commit                   no match
    //      /commit_                  match
    //      /commit_/                 match
    //      /commit_/foo              match
    //      /commit_123               match
    //      /commit_123/              match
    //      /commit_123/foo           match
    let mut tree = PathTree::new();

    tree.insert("/commit_:sha*", "* sha");
    tree.insert("/commit/:sha", "hex");
    tree.insert("/commit/:sha0/compare/:sha1", "compare");
    tree.insert("/src/", "dir");

    let n = tree.find("/src/");
    assert!(n.is_some());
    if let Some((h, r)) = n {
        assert_eq!(*h, "dir");
        assert_eq!(r.params(), vec![]);
    }

    let n = tree.find("/commit/123");
    assert!(n.is_some());
    if let Some((h, r)) = n {
        assert_eq!(*h, "hex");
        assert_eq!(r.params(), vec![("sha", "123")]);
    }

    let n = tree.find("/commit/123/compare/321");
    assert!(n.is_some());
    if let Some((h, r)) = n {
        assert_eq!(*h, "compare");
        assert_eq!(r.params(), vec![("sha0", "123"), ("sha1", "321")]);
    }

    let res = vec![
        ("/commit", false, vec![]),
        ("/commit_", true, vec![("sha", "")]),
        ("/commit_/", true, vec![("sha", "/")]),
        ("/commit_/foo", true, vec![("sha", "/foo")]),
        ("/commit123", false, vec![]),
        ("/commit_123", true, vec![("sha", "123")]),
        ("/commit_123/", true, vec![("sha", "123/")]),
        ("/commit_123/foo", true, vec![("sha", "123/foo")]),
    ];

    for (u, b, p) in res {
        let n = tree.find(u);
        assert_eq!(n.is_some(), b);
        if let Some((h, r)) = n {
            assert_eq!(*h, "* sha");
            assert_eq!(r.params(), p);
        }
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
    tree.insert("/a/*", "/a/*c");

    let res = vec![
        ("/", false, "", vec![]),
        ("/a/b/c", true, "/a/b/c", vec![]),
        ("/a/c/d", true, "/a/c/d", vec![]),
        ("/a/c/a", true, "/a/c/a", vec![]),
        ("/a/c/e", true, "/a/*c", vec![("*1", "c/e")]),
    ];

    for (u, b, a, p) in res {
        let n = tree.find(u);
        assert_eq!(n.is_some(), b);
        if let Some((h, r)) = n {
            assert_eq!(*h, a);
            assert_eq!(r.params(), p);
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
    //      /users/jordan      match users *
    let mut tree = PathTree::<fn() -> usize>::new();

    tree.insert("/", || 1);
    tree.insert("/*", || 2);
    tree.insert("/users/*", || 3);

    let res = vec![
        ("/", true, 1, vec![]),
        ("/download", true, 2, vec![("*1", "download")]),
        ("/users/jordan", true, 3, vec![("*1", "jordan")]),
    ];

    for (u, b, a, p) in res {
        let n = tree.find(u);
        assert_eq!(n.is_some(), b);
        if let Some((h, r)) = n {
            assert_eq!((h)(), a);
            assert_eq!(r.params(), p);
        }
    }
}

#[test]
fn root_catch_all_parameter_1() {
    //  Pattern: /*
    //
    //      /                  match *
    //      /download          match *
    //      /users/jordan      match *
    let mut tree = PathTree::<fn() -> usize>::new();

    tree.insert("/*", || 1);

    let res = vec![
        ("/", true, 1, vec![("*1", "")]),
        ("/download", true, 1, vec![("*1", "download")]),
        ("/users/jordan", true, 1, vec![("*1", "users/jordan")]),
    ];

    for (u, b, a, p) in res {
        let n = tree.find(u);
        assert_eq!(n.is_some(), b);
        if let Some((h, r)) = n {
            assert_eq!((h)(), a);
            assert_eq!(r.params(), p);
        }
    }

    tree.insert("/", || 0);
    let n = tree.find("/");
    assert!(n.is_some());
    if let Some((h, r)) = n {
        assert_eq!((h)(), 0);
        assert_eq!(r.params(), vec![]);
    }
}

#[test]
fn test_named_routes_with_non_ascii_paths() {
    let mut tree = PathTree::<usize>::new();
    tree.insert("/", 0);
    tree.insert("/*", 1);
    tree.insert("/matchme/:slug/", 2);

    // ASCII only (single-byte characters)
    let n = tree.find("/matchme/abc-s-def/");
    assert!(n.is_some());
    let (h, r) = n.unwrap();
    assert_eq!(*h, 2);
    assert_eq!(r.params(), vec![("slug", "abc-s-def")]);

    // with multibyte character
    let n = tree.find("/matchme/abc-ß-def/");
    assert!(n.is_some());
    let (h, r) = n.unwrap();
    assert_eq!(*h, 2);
    assert_eq!(r.params(), vec![("slug", "abc-ß-def")]);

    // with emoji (fancy multibyte character)
    let n = tree.find("/matchme/abc-⭐-def/");
    assert!(n.is_some());
    let (h, r) = n.unwrap();
    assert_eq!(*h, 2);
    assert_eq!(r.params(), vec![("slug", "abc-⭐-def")]);

    // with multibyte character right before the slash (char boundary check)
    let n = tree.find("/matchme/abc-def-ß/");
    assert!(n.is_some());
    let (h, r) = n.unwrap();
    assert_eq!(*h, 2);
    assert_eq!(r.params(), vec![("slug", "abc-def-ß")]);
}

#[test]
fn test_named_wildcard_collide() {
    let mut tree = PathTree::<usize>::new();
    tree.insert("/git/:org/:repo", 1);
    tree.insert("/git/*", 2);

    let n = tree.find("/git/rust-lang/rust");
    assert!(n.is_some());
    let (h, r) = n.unwrap();
    assert_eq!(*h, 1);
    assert_eq!(r.params(), vec![("org", "rust-lang"), ("repo", "rust")]);

    let n = tree.find("/git/rust-lang");
    assert!(n.is_some());
    let (h, r) = n.unwrap();
    assert_eq!(*h, 2);
    assert_eq!(r.params(), vec![("*1", "rust-lang")]);
}

#[test]
fn match_params() {
    // /
    // └── api/v1/
    //     └── :
    //         └── /
    //             └── ** •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/v1/:param/*", 1);

    assert_eq!(tree.find("/api/v1/entity"), None);
    let (h, p) = tree.find("/api/v1/entity/").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "entity"), ("*1", "")]);
    assert_eq!(p.pattern(), "/api/v1/:param/*");
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/api/v1/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMoreSegment),
        ]
    );

    let (h, p) = tree.find("/api/v1/entity/1").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "entity"), ("*1", "1")]);

    assert_eq!(tree.find("/api/v"), None);
    assert_eq!(tree.find("/api/v2"), None);
    assert_eq!(tree.find("/api/v1/"), None);

    let (h, p) = tree.find("/api/v1/entity/1/foo/bar").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "entity"), ("*1", "1/foo/bar")]);

    // /
    // └── api/v1/
    //     └── :
    //         └── /
    //             └── + •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/v1/:param/+", 1);

    assert_eq!(tree.find("/api/v1/entity"), None);
    assert_eq!(tree.find("/api/v1/entity/"), None);

    let (h, p) = tree.find("/api/v1/entity/1").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "entity"), ("+1", "1")]);

    assert_eq!(tree.find("/api/v"), None);
    assert_eq!(tree.find("/api/v2"), None);
    assert_eq!(tree.find("/api/v1/"), None);

    let (h, p) = tree.find("/api/v1/entity/1/foo/bar").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "entity"), ("+1", "1/foo/bar")]);

    // /
    // └── api/v1/
    //     └── ?? •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/v1/:param?", 1);

    let (h, p) = tree.find("/api/v1/").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "")]);
    assert_eq!(p.pattern(), "/api/v1/:param?");
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/api/v1/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::OptionalSegment),
        ]
    );

    assert_eq!(tree.find("/api/v1/entity/1/foo/bar"), None);
    assert_eq!(tree.find("/api/v"), None);
    assert_eq!(tree.find("/api/v2"), None);
    assert_eq!(tree.find("/api/xyz"), None);

    // /
    // └── v1/some/resource/name
    //     └── \:
    //         └── customVerb •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/v1/some/resource/name\\:customVerb", 1);

    let (h, p) = tree.find("/v1/some/resource/name:customVerb").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![]);
    assert_eq!(p.pattern(), "/v1/some/resource/name\\:customVerb");
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/v1/some/resource/name".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::String(b"customVerb".to_vec()),
        ]
    );
    assert_eq!(tree.find("/v1/some/resource/name:test"), None);

    // /
    // └── v1/some/resource/
    //     └── :
    //         └── \:
    //             └── customVerb •0
    let mut tree = PathTree::<usize>::new();

    tree.insert(r"/v1/some/resource/:name\:customVerb", 1);

    let (h, p) = tree.find("/v1/some/resource/test:customVerb").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("name", "test")]);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/v1/some/resource/".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
            Piece::String(b":".to_vec()),
            Piece::String(b"customVerb".to_vec()),
        ]
    );
    assert_eq!(tree.find("/v1/some/resource/test:test"), None);

    // /
    // └── v1/some/resource/name
    //     └── \:
    //         └── customVerb\?
    //             └── \?
    //                 └── /
    //                     └── :
    //                         └── /
    //                             └── ** •0
    let mut tree = PathTree::<usize>::new();

    tree.insert(r"/v1/some/resource/name\\\\:customVerb?\?/:param/*", 1);

    let (h, p) = tree
        .find("/v1/some/resource/name:customVerb??/test/optionalWildCard/character")
        .unwrap();
    assert_eq!(*h, 1);
    assert_eq!(
        p.params(),
        vec![("param", "test"), ("*1", "optionalWildCard/character")]
    );

    let (h, p) = tree
        .find("/v1/some/resource/name:customVerb??/test/")
        .unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "test"), ("*1", "")]);

    assert_eq!(tree.find("/v1/some/resource/name:customVerb??/test"), None);

    // /
    // └── api/v1/
    //     └── ** •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/v1/*", 1);

    assert_eq!(tree.find("/api/v1"), None);

    let (h, p) = tree.find("/api/v1/").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("*1", "")]);

    let (h, p) = tree.find("/api/v1/entity").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("*1", "entity")]);

    let (h, p) = tree.find("/api/v1/entity/1/2").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("*1", "entity/1/2")]);

    let (h, p) = tree.find("/api/v1/Entity/1/2").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("*1", "Entity/1/2")]);

    // /
    // └── api/v1/
    //     └── : •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/v1/:param", 1);

    assert_eq!(tree.find("/api/v1"), None);
    assert_eq!(tree.find("/api/v1/"), None);

    let (h, p) = tree.find("/api/v1/entity").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "entity")]);

    assert_eq!(tree.find("/api/v1/entity/1/2"), None);
    assert_eq!(tree.find("/api/v1/Entity/1/2"), None);

    // /
    // └── api/v1/
    //     └── :
    //         ├── -
    //         │   └── : •1
    //         ├── .
    //         │   └── : •3
    //         ├── \:
    //         │   └── : •5
    //         ├── _
    //         │   └── : •4
    //         ├── ~
    //         │   └── : •2
    //         └── /
    //             └── : •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/v1/:param/:param2", 3);
    tree.insert("/api/v1/:param-:param2", 1);
    tree.insert("/api/v1/:param~:param2", 2);
    tree.insert("/api/v1/:param.:param2", 4);
    tree.insert("/api/v1/:param\\_:param2", 5);
    tree.insert("/api/v1/:param\\::param2", 6);

    let (h, p) = tree.find("/api/v1/entity-entity2").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "entity"), ("param2", "entity2")]);

    let (h, p) = tree.find("/api/v1/entity~entity2").unwrap();
    assert_eq!(*h, 2);
    assert_eq!(p.params(), vec![("param", "entity"), ("param2", "entity2")]);

    let (h, p) = tree.find("/api/v1/entity.entity2").unwrap();
    assert_eq!(*h, 4);
    assert_eq!(p.params(), vec![("param", "entity"), ("param2", "entity2")]);

    let (h, p) = tree.find("/api/v1/entity_entity2").unwrap();
    assert_eq!(*h, 5);
    assert_eq!(p.params(), vec![("param", "entity"), ("param2", "entity2")]);

    let (h, p) = tree.find("/api/v1/entity:entity2").unwrap();
    assert_eq!(*h, 6);
    assert_eq!(p.params(), vec![("param", "entity"), ("param2", "entity2")]);

    let (h, p) = tree.find("/api/v1/entity/entity2").unwrap();
    assert_eq!(*h, 3);
    assert_eq!(p.params(), vec![("param", "entity"), ("param2", "entity2")]);

    assert_eq!(tree.find("/api/v1"), None);
    assert_eq!(tree.find("/api/v1/"), None);

    let (h, p) = tree.find("/api/v1/test.pdf").unwrap();
    assert_eq!(*h, 4);
    assert_eq!(p.params(), vec![("param", "test"), ("param2", "pdf")]);

    // /
    // └── api/v1/const •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/v1/const", 1);

    let (h, p) = tree.find("/api/v1/const").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert!(p.params().is_empty());
    assert_eq!(p.pattern(), "/api/v1/const");
    assert_eq!(p.pieces, vec![Piece::String(b"/api/v1/const".to_vec())]);

    assert_eq!(tree.find("/api/v1/cons"), None);
    assert_eq!(tree.find("/api/v1/conststatic"), None);
    assert_eq!(tree.find("/api/v1/let"), None);
    assert_eq!(tree.find("/api/v1/"), None);
    assert_eq!(tree.find("/api/v1"), None);

    // /
    // └── api/
    //     └── :
    //         └── /fixedEnd •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/:param/fixedEnd", 1);

    let (h, p) = tree.find("/api/abc/fixedEnd").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/api/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
            Piece::String(b"/fixedEnd".to_vec()),
        ]
    );
    assert_eq!(p.params(), vec![("param", "abc")]);
    assert_eq!(p.pattern(), "/api/:param/fixedEnd");

    assert_eq!(tree.find("/api/abc/def/fixedEnd"), None);

    // /
    // └── shop/product/
    //     └── \:
    //         └── :
    //             └── /color
    //                 └── \:
    //                     └── :
    //                         └── /size
    //                             └── \:
    //                                 └── : •0
    let mut tree = PathTree::<usize>::new();

    tree.insert(r"/shop/product/\::filter/color\::color/size\::size", 1);

    let (h, p) = tree.find("/shop/product/:test/color:blue/size:xs").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/shop/product/".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::Parameter(Position::Named(b"filter".to_vec()), Kind::Normal),
            Piece::String(b"/color".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::Parameter(Position::Named(b"color".to_vec()), Kind::Normal),
            Piece::String(b"/size".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::Parameter(Position::Named(b"size".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(
        p.pattern(),
        r"/shop/product/\::filter/color\::color/size\::size"
    );
    assert_eq!(
        p.params(),
        vec![("filter", "test"), ("color", "blue"), ("size", "xs")]
    );

    assert_eq!(tree.find("/shop/product/test/color:blue/size:xs"), None);

    // /
    // └── \:
    //     └── ? •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/\\::param?", 1);

    let (h, p) = tree.find("/:hello").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "hello")]);
    assert_eq!(p.pattern(), "/\\::param?");
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Optional),
        ]
    );

    let (h, p) = tree.find("/:").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "")]);

    assert_eq!(tree.find("/"), None);

    // /
    // └── test
    //     └── :
    //         └── : •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/test:sign:param", 1);

    let (h, p) = tree.find("/test-abc").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("sign", "-"), ("param", "abc")]);
    assert_eq!(p.pattern(), "/test:sign:param");
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/test".to_vec()),
            Piece::Parameter(Position::Named(b"sign".to_vec()), Kind::Normal),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
        ]
    );

    let (h, p) = tree.find("/test-_").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("sign", "-"), ("param", "_")]);

    assert_eq!(tree.find("/test-"), None);
    assert_eq!(tree.find("/test"), None);

    // /
    // └── :
    //     └── ?
    //         └── : •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/:param1:param2?:param3", 1);

    let (h, p) = tree.find("/abbbc").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(
        p.params(),
        vec![("param1", "a"), ("param2", "b"), ("param3", "bbc")]
    );
    assert_eq!(p.pattern(), "/:param1:param2?:param3");
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"param1".to_vec()), Kind::Normal),
            Piece::Parameter(Position::Named(b"param2".to_vec()), Kind::Optional),
            Piece::Parameter(Position::Named(b"param3".to_vec()), Kind::Normal),
        ]
    );

    let (h, p) = tree.find("/ab").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(
        p.params(),
        vec![("param1", "a"), ("param2", ""), ("param3", "b")]
    );

    assert_eq!(tree.find("/a"), None);

    // /
    // └── test
    //     └── ?
    //         └── : •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/test:optional?:mandatory", 1);

    let (h, p) = tree.find("/testo").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("optional", ""), ("mandatory", "o")]);
    assert_eq!(p.pattern(), "/test:optional?:mandatory");
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/test".to_vec()),
            Piece::Parameter(Position::Named(b"optional".to_vec()), Kind::Optional),
            Piece::Parameter(Position::Named(b"mandatory".to_vec()), Kind::Normal),
        ]
    );

    let (h, p) = tree.find("/testoaaa").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("optional", "o"), ("mandatory", "aaa")]);

    assert_eq!(tree.find("/test"), None);
    assert_eq!(tree.find("/tes"), None);

    // /
    // └── test
    //     └── ?
    //         └── ? •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/test:optional?:optional2?", 1);

    let (h, p) = tree.find("/testo").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("optional", "o"), ("optional2", "")]);
    assert_eq!(p.pattern(), "/test:optional?:optional2?");
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/test".to_vec()),
            Piece::Parameter(Position::Named(b"optional".to_vec()), Kind::Optional),
            Piece::Parameter(Position::Named(b"optional2".to_vec()), Kind::Optional),
        ]
    );

    let (h, p) = tree.find("/testoaaa").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("optional", "o"), ("optional2", "aaa")]);

    let (h, p) = tree.find("/test").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("optional", ""), ("optional2", "")]);

    assert_eq!(tree.find("/tes"), None);

    // /
    // └── foo
    //     └── ?
    //         └── bar •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/foo:param?bar", 1);

    let (h, p) = tree.find("/foofalsebar").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "false")]);
    assert_eq!(p.pattern(), "/foo:param?bar");
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/foo".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Optional),
            Piece::String(b"bar".to_vec()),
        ]
    );

    let (h, p) = tree.find("/foobar").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("param", "")]);

    assert_eq!(tree.find("/fooba"), None);
    assert_eq!(tree.find("/foo"), None);

    // /
    // └── foo
    //     └── *
    //         └── bar •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/foo*bar", 1);

    let (h, p) = tree.find("/foofalsebar").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("*1", "false")]);
    assert_eq!(p.pattern(), "/foo*bar");
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/foo".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMore),
            Piece::String(b"bar".to_vec()),
        ]
    );

    let (h, p) = tree.find("/foobar").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("*1", "")]);

    let (h, p) = tree.find("/foo/bar").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("*1", "/")]);

    let (h, p) = tree.find("/foo/baz/bar").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("*1", "/baz/")]);

    assert_eq!(tree.find("/fooba"), None);
    assert_eq!(tree.find("/foo"), None);

    // /
    // └── foo
    //     └── +
    //         └── bar •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/foo+bar", 1);

    let (h, p) = tree.find("/foofalsebar").unwrap();
    assert_eq!(*p.id, 0);
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("+1", "false")]);
    assert_eq!(p.pattern(), "/foo+bar");
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/foo".to_vec()),
            Piece::Parameter(Position::Index(1, b"+1".to_vec()), Kind::OneOrMore),
            Piece::String(b"bar".to_vec()),
        ]
    );

    assert_eq!(tree.find("/foobar"), None);

    let (h, p) = tree.find("/foo/bar").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("+1", "/")]);

    let (h, p) = tree.find("/foo/baz/bar").unwrap();
    assert_eq!(*h, 1);
    assert_eq!(p.params(), vec![("+1", "/baz/")]);

    assert_eq!(tree.find("/fooba"), None);
    assert_eq!(tree.find("/foo"), None);

    // /
    // └── a
    //     └── *
    //         └── cde
    //             └── *
    //                 └── g/ •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/a*cde*g/", 1);

    assert_eq!(tree.find("/abbbcdefffg"), None);

    let (h, p) = tree.find("/abbbcdefffg/").unwrap();
    assert_eq!(h, &1);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/a".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMore),
            Piece::String(b"cde".to_vec()),
            Piece::Parameter(Position::Index(2, b"*2".to_vec()), Kind::ZeroOrMore),
            Piece::String(b"g/".to_vec()),
        ]
    );
    assert_eq!(p.pattern(), "/a*cde*g/");
    assert_eq!(p.params(), vec![("*1", "bbb"), ("*2", "fff")]);

    let (_, p) = tree.find("/acdeg/").unwrap();
    assert_eq!(p.params(), vec![("*1", ""), ("*2", "")]);

    let (_, p) = tree.find("/abcdeg/").unwrap();
    assert_eq!(p.params(), vec![("*1", "b"), ("*2", "")]);

    let (_, p) = tree.find("/acdefg/").unwrap();
    assert_eq!(p.params(), vec![("*1", ""), ("*2", "f")]);

    let (_, p) = tree.find("/abcdefg/").unwrap();
    assert_eq!(p.params(), vec![("*1", "b"), ("*2", "f")]);

    let (_, p) = tree.find("/a/cde/g/").unwrap();
    assert_eq!(p.params(), vec![("*1", "/"), ("*2", "/")]);

    let (_, p) = tree.find("/a/b/cde/f/g/").unwrap();
    assert_eq!(p.params(), vec![("*1", "/b/"), ("*2", "/f/")]);

    // /
    // └── *
    //     └── v1
    //         └── *
    //             └── proxy/ •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/*v1*/proxy", 1);

    let (h, p) = tree.find("/customer/v1/cart/proxy").unwrap();
    assert_eq!(h, &1);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMore),
            Piece::String(b"v1".to_vec()),
            Piece::Parameter(Position::Index(2, b"*2".to_vec()), Kind::ZeroOrMore),
            Piece::String(b"/proxy".to_vec()),
        ]
    );
    assert_eq!(p.pattern(), "/*v1*/proxy");
    assert_eq!(p.params(), vec![("*1", "customer/"), ("*2", "/cart")]);

    let (_, p) = tree.find("/v1/proxy").unwrap();
    assert_eq!(p.params(), vec![("*1", ""), ("*2", "")]);

    assert_eq!(tree.find("/v1/"), None);

    // /
    // ├── -
    // │   └── : •2
    // ├── .
    // │   └── : •3
    // ├── @
    // │   └── : •1
    // ├── _
    // │   └── : •5
    // ├── name
    // │   └── \:
    // │       └── : •0
    // ├── ~
    // │   └── : •4
    // └── : •6
    let mut tree = PathTree::<usize>::new();

    tree.insert("/name\\::name", 1);
    tree.insert("/@:name", 2);
    tree.insert("/-:name", 3);
    tree.insert("/.:name", 4);
    tree.insert("/~:name", 5);
    tree.insert("/_:name", 6);
    tree.insert("/:name", 7);

    let (h, p) = tree.find("/name:john").unwrap();
    assert_eq!(h, &1);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/name".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(p.pattern(), "/name\\::name");
    assert_eq!(p.params(), vec![("name", "john")]);

    let (h, p) = tree.find("/@john").unwrap();
    assert_eq!(h, &2);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/@".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(p.pattern(), "/@:name");
    assert_eq!(p.params(), vec![("name", "john")]);

    let (h, p) = tree.find("/-john").unwrap();
    assert_eq!(h, &3);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/-".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(p.pattern(), "/-:name");
    assert_eq!(p.params(), vec![("name", "john")]);

    let (h, p) = tree.find("/.john").unwrap();
    assert_eq!(h, &4);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/.".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(p.pattern(), "/.:name");
    assert_eq!(p.params(), vec![("name", "john")]);

    let (h, p) = tree.find("/~john").unwrap();
    assert_eq!(h, &5);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/~".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(p.pattern(), "/~:name");
    assert_eq!(p.params(), vec![("name", "john")]);

    let (h, p) = tree.find("/_john").unwrap();
    assert_eq!(h, &6);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/_".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(p.pattern(), "/_:name");
    assert_eq!(p.params(), vec![("name", "john")]);

    let (h, p) = tree.find("/john").unwrap();
    assert_eq!(h, &7);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(p.pattern(), "/:name");
    assert_eq!(p.params(), vec![("name", "john")]);

    // /
    // └── api/v1/
    //     └── :
    //         └── /abc/
    //             └── ** •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/v1/:param/abc/*", 1);

    let (h, p) = tree.find("/api/v1/well/abc/wildcard").unwrap();
    assert_eq!(h, &1);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/api/v1/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
            Piece::String(b"/abc/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMoreSegment),
        ]
    );
    assert_eq!(p.pattern(), "/api/v1/:param/abc/*");
    assert_eq!(p.params(), vec![("param", "well"), ("*1", "wildcard")]);

    let (_, p) = tree.find("/api/v1/well/abc/").unwrap();
    assert_eq!(p.params(), vec![("param", "well"), ("*1", "")]);

    assert_eq!(tree.find("/api/v1/well/abc"), None);
    assert_eq!(tree.find("/api/v1/well/ttt"), None);

    // /
    // └── api/
    //     └── :
    //         └── /
    //             └── ??
    //                 └── /
    //                     └── ?? •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/:day/:month?/:year?", 1);

    assert_eq!(tree.find("/api/1"), None);

    let (h, p) = tree.find("/api/1/").unwrap();
    assert_eq!(h, &1);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/api/".to_vec()),
            Piece::Parameter(Position::Named(b"day".to_vec()), Kind::Normal),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"month".to_vec()), Kind::OptionalSegment),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"year".to_vec()), Kind::OptionalSegment),
        ]
    );
    assert_eq!(p.pattern(), "/api/:day/:month?/:year?");
    assert_eq!(p.params(), vec![("day", "1"), ("month", ""), ("year", "")]);

    let (_, p) = tree.find("/api/1//").unwrap();
    assert_eq!(p.params(), vec![("day", "1"), ("month", ""), ("year", "")]);

    let (_, p) = tree.find("/api/1/-/").unwrap();
    assert_eq!(p.params(), vec![("day", "1"), ("month", "-"), ("year", "")]);

    assert_eq!(tree.find("/api/1-"), None);

    let (_, p) = tree.find("/api/1-/").unwrap();
    assert_eq!(p.params(), vec![("day", "1-"), ("month", ""), ("year", "")]);

    let (_, p) = tree.find("/api/1/2").unwrap();
    assert_eq!(p.params(), vec![("day", "1"), ("month", "2"), ("year", "")]);

    let (_, p) = tree.find("/api/1/2/3").unwrap();
    assert_eq!(
        p.params(),
        vec![("day", "1"), ("month", "2"), ("year", "3")]
    );

    // /
    // └── api/
    //     └── :
    //         └── .
    //             └── ?
    //                 └── .
    //                     └── ? •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/:day.:month?.:year?", 1);
    tree.insert("/api/:day-:month?-:year?", 2);

    assert_eq!(tree.find("/api/1"), None);
    assert_eq!(tree.find("/api/1/"), None);
    assert_eq!(tree.find("/api/1."), None);

    let (h, p) = tree.find("/api/1..").unwrap();
    assert_eq!(h, &1);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/api/".to_vec()),
            Piece::Parameter(Position::Named(b"day".to_vec()), Kind::Normal),
            Piece::String(b".".to_vec()),
            Piece::Parameter(Position::Named(b"month".to_vec()), Kind::Optional),
            Piece::String(b".".to_vec()),
            Piece::Parameter(Position::Named(b"year".to_vec()), Kind::Optional),
        ]
    );
    assert_eq!(p.pattern(), "/api/:day.:month?.:year?");
    assert_eq!(p.params(), vec![("day", "1"), ("month", ""), ("year", "")]);

    let (h, p) = tree.find("/api/1.2.").unwrap();
    assert_eq!(h, &1);
    assert_eq!(p.params(), vec![("day", "1"), ("month", "2"), ("year", "")]);

    let (h, p) = tree.find("/api/1.2.3").unwrap();
    assert_eq!(h, &1);
    assert_eq!(
        p.params(),
        vec![("day", "1"), ("month", "2"), ("year", "3")]
    );

    let (h, p) = tree.find("/api/1--").unwrap();
    assert_eq!(h, &2);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/api/".to_vec()),
            Piece::Parameter(Position::Named(b"day".to_vec()), Kind::Normal),
            Piece::String(b"-".to_vec()),
            Piece::Parameter(Position::Named(b"month".to_vec()), Kind::Optional),
            Piece::String(b"-".to_vec()),
            Piece::Parameter(Position::Named(b"year".to_vec()), Kind::Optional),
        ]
    );
    assert_eq!(p.pattern(), "/api/:day-:month?-:year?");
    assert_eq!(p.params(), vec![("day", "1"), ("month", ""), ("year", "")]);

    let (h, p) = tree.find("/api/1-2-").unwrap();
    assert_eq!(h, &2);
    assert_eq!(p.params(), vec![("day", "1"), ("month", "2"), ("year", "")]);

    let (h, p) = tree.find("/api/1-2-3").unwrap();
    assert_eq!(h, &2);
    assert_eq!(
        p.params(),
        vec![("day", "1"), ("month", "2"), ("year", "3")]
    );

    assert_eq!(tree.find("/api/1.2-3"), None);

    // /
    // └── config/
    //     ├── abc.json •0
    //     ├── +
    //     │   └── .json •1
    //     └── *
    //         └── .json •2
    let mut tree = PathTree::<usize>::new();

    tree.insert("/config/abc.json", 1);
    tree.insert("/config/+.json", 2);
    tree.insert("/config/*.json", 3);

    let (h, p) = tree.find("/config/abc.json").unwrap();
    assert_eq!(h, &1);
    assert_eq!(p.pieces, vec![Piece::String(b"/config/abc.json".to_vec())]);
    assert_eq!(p.pattern(), "/config/abc.json");
    assert_eq!(p.params(), vec![]);

    let (h, p) = tree.find("/config/a.json").unwrap();
    assert_eq!(h, &2);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/config/".to_vec()),
            Piece::Parameter(Position::Index(1, b"+1".to_vec()), Kind::OneOrMore),
            Piece::String(b".json".to_vec()),
        ]
    );
    assert_eq!(p.pattern(), "/config/+.json");
    assert_eq!(p.params(), vec![("+1", "a")]);

    let (h, p) = tree.find("/config/ab.json").unwrap();
    assert_eq!(h, &2);
    assert_eq!(p.params(), vec![("+1", "ab")]);

    let (h, p) = tree.find("/config/a/b.json").unwrap();
    assert_eq!(h, &2);
    assert_eq!(p.params(), vec![("+1", "a/b")]);

    let (h, p) = tree.find("/config/a/b/abc.json").unwrap();
    assert_eq!(h, &2);
    assert_eq!(p.params(), vec![("+1", "a/b/abc")]);

    let (h, p) = tree.find("/config/.json").unwrap();
    assert_eq!(h, &3);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/config/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMore),
            Piece::String(b".json".to_vec()),
        ]
    );
    assert_eq!(p.pattern(), "/config/*.json");
    assert_eq!(p.params(), vec![("*1", "")]);

    // /
    // └── api/
    //     └── **
    //         └── /
    //             └── ?? •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/*/:param?", 1);

    let (h, p) = tree.find("/api/").unwrap();
    assert_eq!(h, &1);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/api/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMoreSegment),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::OptionalSegment),
        ]
    );
    assert_eq!(p.pattern(), "/api/*/:param?");
    assert_eq!(p.params(), vec![("*1", ""), ("param", "")]);

    let (_, p) = tree.find("/api/joker").unwrap();
    assert_eq!(p.params(), vec![("*1", ""), ("param", "joker")]);

    let (_, p) = tree.find("/api/joker/").unwrap();
    assert_eq!(p.params(), vec![("*1", "joker"), ("param", "")]);

    let (_, p) = tree.find("/api/joker/batman").unwrap();
    assert_eq!(p.params(), vec![("*1", "joker"), ("param", "batman")]);

    let (_, p) = tree.find("/api/joker/batman/robin").unwrap();
    assert_eq!(p.params(), vec![("*1", "joker/batman"), ("param", "robin")]);

    let (_, p) = tree.find("/api/joker/batman/robin/1").unwrap();
    assert_eq!(
        p.params(),
        vec![("*1", "joker/batman/robin"), ("param", "1")]
    );

    // /
    // └── api/
    //     └── **
    //         └── /
    //             └── : •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/*/:param", 1);

    let (h, p) = tree.find("/api/test/abc").unwrap();
    assert_eq!(h, &1);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/api/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMoreSegment),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(p.pattern(), "/api/*/:param");
    assert_eq!(p.params(), vec![("*1", "test"), ("param", "abc")]);

    let (_, p) = tree.find("/api/joker/batman/robin/1").unwrap();
    assert_eq!(
        p.params(),
        vec![("*1", "joker/batman/robin"), ("param", "1")]
    );

    let (_, p) = tree.find("/api//joker").unwrap();
    assert_eq!(p.params(), vec![("*1", ""), ("param", "joker")]);

    assert_eq!(tree.find("/api/joker"), None);
    assert_eq!(tree.find("/api/"), None);

    // /
    // └── api/
    //     └── +
    //         └── /
    //             └── : •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/+/:param", 1);

    let (h, p) = tree.find("/api/test/abc").unwrap();
    assert_eq!(h, &1);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/api/".to_vec()),
            Piece::Parameter(Position::Index(1, b"+1".to_vec()), Kind::OneOrMore),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(p.pattern(), "/api/+/:param");
    assert_eq!(p.params(), vec![("+1", "test"), ("param", "abc")]);

    let (_, p) = tree.find("/api/joker/batman/robin/1").unwrap();
    assert_eq!(
        p.params(),
        vec![("+1", "joker/batman/robin"), ("param", "1")]
    );

    assert_eq!(tree.find("/api/joker"), None);
    assert_eq!(tree.find("/api/"), None);

    // /
    // └── api/
    //     └── **
    //         └── /
    //             └── :
    //                 └── /
    //                     └── : •0
    let mut tree = PathTree::<usize>::new();

    tree.insert("/api/*/:param/:param2", 1);

    let (h, p) = tree.find("/api/test/abc/1").unwrap();
    assert_eq!(h, &1);
    assert_eq!(
        p.pieces,
        vec![
            Piece::String(b"/api/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMoreSegment),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"param2".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(p.pattern(), "/api/*/:param/:param2");
    assert_eq!(
        p.params(),
        vec![("*1", "test"), ("param", "abc"), ("param2", "1")]
    );

    assert_eq!(tree.find("/api/joker/batman"), None);

    let (_, p) = tree.find("/api/joker/batman-robin/1").unwrap();
    assert_eq!(
        p.params(),
        vec![("*1", "joker"), ("param", "batman-robin"), ("param2", "1")]
    );

    assert_eq!(tree.find("/api/joker-batman-robin-1"), None);
    assert_eq!(tree.find("/api/test/abc"), None);

    let (_, p) = tree.find("/api/joker/batman/robin").unwrap();
    assert_eq!(
        p.params(),
        vec![("*1", "joker"), ("param", "batman"), ("param2", "robin")]
    );

    let (_, p) = tree.find("/api/joker/batman/robin/1").unwrap();
    assert_eq!(
        p.params(),
        vec![("*1", "joker/batman"), ("param", "robin"), ("param2", "1")]
    );

    let (_, p) = tree.find("/api/joker/batman/robin/1/2").unwrap();
    assert_eq!(
        p.params(),
        vec![
            ("*1", "joker/batman/robin"),
            ("param", "1"),
            ("param2", "2")
        ]
    );

    assert_eq!(tree.find("/api"), None);
    assert_eq!(tree.find("/api/:test"), None);
}

#[test]
fn basic() {
    let mut tree = PathTree::<usize>::new();

    tree.insert("/", 0);
    tree.insert("/login", 1);
    tree.insert("/signup", 2);
    tree.insert("/settings", 3);
    tree.insert("/settings/:page", 4);
    tree.insert("/:user", 5);
    tree.insert("/:user/:repo", 6);
    tree.insert("/public/:any*", 7);
    tree.insert("/:org/:repo/releases/download/:tag/:filename.:ext", 8);
    tree.insert("/:org/:repo/tags/:day-:month-:year", 9);
    tree.insert("/:org/:repo/actions/:name\\::verb", 10);
    tree.insert("/:org/:repo/:page", 11);
    tree.insert("/:org/:repo/*", 12);
    tree.insert("/api/+", 13);

    assert_eq!(
        format!("{:?}", &tree.node),
        r"
/ •0
├── api/
│   └── + •13
├── login •1
├── public/
│   └── ** •7
├── s
│   ├── ettings •3
│   │   └── /
│   │       └── : •4
│   └── ignup •2
└── : •5
    └── /
        └── : •6
            └── /
                ├── actions/
                │   └── :
                │       └── \:
                │           └── : •10
                ├── releases/download/
                │   └── :
                │       └── /
                │           └── :
                │               └── .
                │                   └── : •8
                ├── tags/
                │   └── :
                │       └── -
                │           └── :
                │               └── -
                │                   └── : •9
                ├── : •11
                └── ** •12
"
    );

    let (h, p) = tree.find("/").unwrap();
    assert_eq!(h, &0);
    assert_eq!(p.params(), vec![]);

    tree.insert("", 14);
    let (h, p) = tree.find("/").unwrap();
    assert_eq!(h, &14);
    assert_eq!(p.params(), vec![]);

    tree.insert("/", 15);
    let (h, p) = tree.find("/").unwrap();
    assert_eq!(h, &15);
    assert_eq!(p.params(), vec![]);

    let (h, p) = tree.find("/login").unwrap();
    assert_eq!(h, &1);
    assert_eq!(p.params(), vec![]);

    let (h, p) = tree.find("/settings/admin").unwrap();
    assert_eq!(h, &4);
    assert_eq!(p.params(), vec![("page", "admin")]);

    let (h, p) = tree.find("/viz-rs").unwrap();
    assert_eq!(h, &5);
    assert_eq!(p.params(), vec![("user", "viz-rs")]);

    let (h, p) = tree.find("/viz-rs/path-tree").unwrap();
    assert_eq!(h, &6);
    assert_eq!(p.params(), vec![("user", "viz-rs"), ("repo", "path-tree")]);

    let (h, p) = tree.find("/rust-lang/rust-analyzer/releases/download/2022-09-12/rust-analyzer-aarch64-apple-darwin.gz").unwrap();
    assert_eq!(h, &8);
    assert_eq!(
        p.params(),
        vec![
            ("org", "rust-lang"),
            ("repo", "rust-analyzer"),
            ("tag", "2022-09-12"),
            ("filename", "rust-analyzer-aarch64-apple-darwin"),
            ("ext", "gz")
        ]
    );

    let (h, p) = tree
        .find("/rust-lang/rust-analyzer/tags/2022-09-12")
        .unwrap();
    assert_eq!(h, &9);
    assert_eq!(
        p.params(),
        vec![
            ("org", "rust-lang"),
            ("repo", "rust-analyzer"),
            ("day", "2022"),
            ("month", "09"),
            ("year", "12")
        ]
    );

    let (h, p) = tree
        .find("/rust-lang/rust-analyzer/actions/ci:bench")
        .unwrap();
    assert_eq!(h, &10);
    assert_eq!(
        p.params(),
        vec![
            ("org", "rust-lang"),
            ("repo", "rust-analyzer"),
            ("name", "ci"),
            ("verb", "bench"),
        ]
    );

    let (h, p) = tree.find("/rust-lang/rust-analyzer/stargazers").unwrap();
    assert_eq!(h, &11);
    assert_eq!(
        p.params(),
        vec![
            ("org", "rust-lang"),
            ("repo", "rust-analyzer"),
            ("page", "stargazers")
        ]
    );

    let (h, p) = tree
        .find("/rust-lang/rust-analyzer/stargazers/404")
        .unwrap();
    assert_eq!(h, &12);
    assert_eq!(
        p.params(),
        vec![
            ("org", "rust-lang"),
            ("repo", "rust-analyzer"),
            ("*1", "stargazers/404")
        ]
    );

    let (h, p) = tree.find("/public/js/main.js").unwrap();
    assert_eq!(h, &7);
    assert_eq!(p.params(), vec![("any", "js/main.js")]);

    let (h, p) = tree.find("/api/v1").unwrap();
    assert_eq!(h, &13);
    assert_eq!(p.params(), vec![("+1", "v1")]);
}

#[test]
fn github_tree() {
    let mut tree = PathTree::<usize>::new();

    tree.insert("/", 0);
    tree.insert("/api", 1);
    tree.insert("/about", 2);
    tree.insert("/login", 3);
    tree.insert("/signup", 4);
    tree.insert("/pricing", 5);

    tree.insert("/features", 6);
    tree.insert("/features/actions", 600);
    tree.insert("/features/packages", 601);
    tree.insert("/features/security", 602);
    tree.insert("/features/codespaces", 603);
    tree.insert("/features/copilot", 604);
    tree.insert("/features/code-review", 605);
    tree.insert("/features/issues", 606);
    tree.insert("/features/discussions", 607);

    tree.insert("/enterprise", 7);
    tree.insert("/team", 8);
    tree.insert("/customer-stories", 9);
    tree.insert("/sponsors", 10);
    tree.insert("/readme", 11);
    tree.insert("/topics", 12);
    tree.insert("/trending", 13);
    tree.insert("/collections", 14);
    tree.insert("/search", 15);
    tree.insert("/pulls", 16);
    tree.insert("/issues", 17);
    tree.insert("/marketplace", 18);
    tree.insert("/explore", 19);

    tree.insert("/sponsors/explore", 100);
    tree.insert("/sponsors/accounts", 101);
    tree.insert("/sponsors/:repo", 102);
    tree.insert("/sponsors/:repo/:user?", 103);
    tree.insert("/sponsors/:repo/+", 104);
    tree.insert("/sponsors/:repo/:user", 105);
    tree.insert("/sponsors/:repo/issues/*", 106);
    tree.insert("/sponsors/:repo/+/:file", 107);
    tree.insert("/sponsors/:repo/+/:filename.:ext", 108);

    tree.insert("/about/careers", 200);
    tree.insert("/about/press", 201);
    tree.insert("/about/diversity", 202);

    tree.insert("/settings", 20);
    tree.insert("/settings/admin", 2000);
    tree.insert("/settings/appearance", 2001);
    tree.insert("/settings/accessibility", 2002);
    tree.insert("/settings/notifications", 2003);

    tree.insert("/settings/billing", 2004);
    tree.insert("/settings/billing/plans", 2005);
    tree.insert("/settings/security", 2006);
    tree.insert("/settings/keys", 2007);
    tree.insert("/settings/organizations", 2008);

    tree.insert("/settings/blocked_users", 2009);
    tree.insert("/settings/interaction_limits", 2010);
    tree.insert("/settings/code_review_limits", 2011);

    tree.insert("/settings/repositories", 2012);
    tree.insert("/settings/codespaces", 2013);
    tree.insert("/settings/deleted_packages", 2014);
    tree.insert("/settings/copilot", 2015);
    tree.insert("/settings/pages", 2016);
    tree.insert("/settings/replies", 2017);

    tree.insert("/settings/security_analysis", 2018);

    tree.insert("/settings/installations", 2019);
    tree.insert("/settings/reminders", 2020);

    tree.insert("/settings/security-log", 2021);
    tree.insert("/settings/sponsors-log", 2022);

    tree.insert("/settings/apps", 2023);
    tree.insert("/settings/developers", 2024);
    tree.insert("/settings/tokens", 2025);

    tree.insert("/404", 21);
    tree.insert("/500", 22);
    tree.insert("/503", 23);

    tree.insert("/:org", 24);
    tree.insert("/:org/:repo", 2400);
    tree.insert("/:org/:repo/issues", 2410);
    tree.insert("/:org/:repo/issues/:id", 2411);
    tree.insert("/:org/:repo/issues/new", 2412);
    tree.insert("/:org/:repo/pulls", 2420);
    tree.insert("/:org/:repo/pull/:id", 2421);
    tree.insert("/:org/:repo/compare", 2422);
    tree.insert("/:org/:repo/discussions", 2430);
    tree.insert("/:org/:repo/discussions/:id", 2431);
    tree.insert("/:org/:repo/actions", 2440);
    tree.insert("/:org/:repo/actions/workflows/:id", 2441);
    tree.insert("/:org/:repo/actions/runs/:id", 2442);
    tree.insert("/:org/:repo/wiki", 2450);
    tree.insert("/:org/:repo/wiki/:id", 2451);
    tree.insert("/:org/:repo/security", 2460);
    tree.insert("/:org/:repo/security/policy", 2461);
    tree.insert("/:org/:repo/security/advisories", 2462);
    tree.insert("/:org/:repo/pulse", 2470);
    tree.insert("/:org/:repo/graphs/contributors", 2480);
    tree.insert("/:org/:repo/graphs/commit-activity", 2481);
    tree.insert("/:org/:repo/graphs/code-frequency", 2482);
    tree.insert("/:org/:repo/community", 2490);
    tree.insert("/:org/:repo/network", 2491);
    tree.insert("/:org/:repo/network/dependencies", 2492);
    tree.insert("/:org/:repo/network/dependents", 2493);
    tree.insert("/:org/:repo/network/members", 2494);
    tree.insert("/:org/:repo/stargazers", 2495);
    tree.insert("/:org/:repo/stargazers/yoou_know", 2496);
    tree.insert("/:org/:repo/watchers", 2497);
    tree.insert("/:org/:repo/releases", 2498);
    tree.insert("/:org/:repo/releases/tag/:id", 2499);
    tree.insert("/:org/:repo/tags", 2500);
    tree.insert("/:org/:repo/tags/:id", 2501);
    tree.insert("/:org/:repo/tree/:id", 2502);
    tree.insert("/:org/:repo/commit/:id", 2503);

    tree.insert("/new", 2504);
    tree.insert("/new/import", 2505);
    tree.insert("/organizations/new", 2506);
    tree.insert("/organizations/plan", 2507);

    tree.insert("/:org/:repo/*", 3000);
    tree.insert("/:org/:repo/releases/*", 3001);
    let id = tree.insert("/:org/:repo/releases/download/:tag/:filename.:ext", 3002);
    assert_eq!(
        tree.url_for(id, &["viz-rs", "path-tree", "v0.5.0", "v0.5.0", "gz"])
            .unwrap(),
        "/viz-rs/path-tree/releases/download/v0.5.0/v0.5.0.gz"
    );

    assert_eq!(
        format!("{:?}", &tree.node),
        r"
/ •0
├── 404 •67
├── 50
│   ├── 0 •68
│   └── 3 •69
├── a
│   ├── bout •2
│   │   └── /
│   │       ├── careers •37
│   │       ├── diversity •39
│   │       └── press •38
│   └── pi •1
├── c
│   ├── ollections •22
│   └── ustomer-stories •17
├── e
│   ├── nterprise •15
│   └── xplore •27
├── features •6
│   └── /
│       ├── actions •7
│       ├── co
│       │   ├── de
│       │   │   ├── -review •12
│       │   │   └── spaces •10
│       │   └── pilot •11
│       ├── discussions •14
│       ├── issues •13
│       ├── packages •8
│       └── security •9
├── issues •25
├── login •3
├── marketplace •26
├── new •106
│   └── /import •107
├── organizations/
│   ├── new •108
│   └── plan •109
├── p
│   ├── ricing •5
│   └── ulls •24
├── readme •19
├── s
│   ├── e
│   │   ├── arch •23
│   │   └── ttings •40
│   │       └── /
│   │           ├── a
│   │           │   ├── ccessibility •43
│   │           │   ├── dmin •41
│   │           │   └── pp
│   │           │       ├── earance •42
│   │           │       └── s •64
│   │           ├── b
│   │           │   ├── illing •45
│   │           │   │   └── /plans •46
│   │           │   └── locked_users •50
│   │           ├── co
│   │           │   ├── de
│   │           │   │   ├── _review_limits •52
│   │           │   │   └── spaces •54
│   │           │   └── pilot •56
│   │           ├── de
│   │           │   ├── leted_packages •55
│   │           │   └── velopers •65
│   │           ├── in
│   │           │   ├── stallations •60
│   │           │   └── teraction_limits •51
│   │           ├── keys •48
│   │           ├── notifications •44
│   │           ├── organizations •49
│   │           ├── pages •57
│   │           ├── re
│   │           │   ├── minders •61
│   │           │   └── p
│   │           │       ├── lies •58
│   │           │       └── ositories •53
│   │           ├── s
│   │           │   ├── ecurity •47
│   │           │   │   ├── -log •62
│   │           │   │   └── _analysis •59
│   │           │   └── ponsors-log •63
│   │           └── tokens •66
│   ├── ignup •4
│   └── ponsors •18
│       └── /
│           ├── accounts •29
│           ├── explore •28
│           └── : •30
│               └── /
│                   ├── issues/
│                   │   └── ** •34
│                   ├── : •33
│                   ├── ?? •31
│                   └── + •32
│                       └── /
│                           └── : •35
│                               └── .
│                                   └── : •36
├── t
│   ├── eam •16
│   ├── opics •20
│   └── rending •21
└── : •70
    └── /
        └── : •71
            └── /
                ├── actions •80
                │   └── /
                │       ├── runs/
                │       │   └── : •82
                │       └── workflows/
                │           └── : •81
                ├── com
                │   ├── m
                │   │   ├── it/
                │   │   │   └── : •105
                │   │   └── unity •92
                │   └── pare •77
                ├── discussions •78
                │   └── /
                │       └── : •79
                ├── graphs/co
                │   ├── de-frequency •91
                │   ├── mmit-activity •90
                │   └── ntributors •89
                ├── issues •72
                │   └── /
                │       ├── new •74
                │       └── : •73
                ├── network •93
                │   └── /
                │       ├── dependen
                │       │   ├── cies •94
                │       │   └── ts •95
                │       └── members •96
                ├── pul
                │   ├── l
                │   │   ├── s •75
                │   │   └── /
                │   │       └── : •76
                │   └── se •88
                ├── releases •100
                │   └── /
                │       ├── download/
                │       │   └── :
                │       │       └── /
                │       │           └── :
                │       │               └── .
                │       │                   └── : •112
                │       ├── tag/
                │       │   └── : •101
                │       └── ** •111
                ├── s
                │   ├── ecurity •85
                │   │   └── /
                │   │       ├── advisories •87
                │   │       └── policy •86
                │   └── targazers •97
                │       └── /yoou_know •98
                ├── t
                │   ├── ags •102
                │   │   └── /
                │   │       └── : •103
                │   └── ree/
                │       └── : •104
                ├── w
                │   ├── atchers •99
                │   └── iki •83
                │       └── /
                │           └── : •84
                └── ** •110
"
    );

    let (h, p) = tree.find("/rust-lang/rust").unwrap();
    assert_eq!(h, &2400);
    assert_eq!(p.params(), vec![("org", "rust-lang"), ("repo", "rust")]);

    let (h, p) = tree.find("/settings").unwrap();
    assert_eq!(h, &20);
    assert!(p.params().is_empty());

    let (h, p) = tree.find("/rust-lang/rust/actions/runs/1").unwrap();
    assert_eq!(h, &2442);
    assert_eq!(
        p.params(),
        vec![("org", "rust-lang"), ("repo", "rust"), ("id", "1")]
    );

    let (h, p) = tree.find("/rust-lang/rust/").unwrap();
    assert_eq!(h, &3000);
    assert_eq!(
        p.params(),
        vec![("org", "rust-lang"), ("repo", "rust"), ("*1", "")]
    );

    let (h, p) = tree.find("/rust-lang/rust/any").unwrap();
    assert_eq!(h, &3000);
    assert_eq!(
        p.params(),
        vec![("org", "rust-lang"), ("repo", "rust"), ("*1", "any")]
    );

    let (h, p) = tree.find("/rust-lang/rust/releases/").unwrap();
    assert_eq!(h, &3001);
    assert_eq!(
        p.params(),
        vec![("org", "rust-lang"), ("repo", "rust"), ("*1", "")]
    );
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"org".to_vec()), Kind::Normal),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"repo".to_vec()), Kind::Normal),
            Piece::String(b"/releases/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMoreSegment),
        ]
    );

    let (h, p) = tree.find("/rust-lang/rust-analyzer/releases/download/2022-09-12/rust-analyzer-aarch64-apple-darwin.gz").unwrap();
    assert_eq!(h, &3002);
    assert_eq!(
        p.params(),
        vec![
            ("org", "rust-lang"),
            ("repo", "rust-analyzer"),
            ("tag", "2022-09-12"),
            ("filename", "rust-analyzer-aarch64-apple-darwin"),
            ("ext", "gz")
        ]
    );
    assert_eq!(
        p.pieces,
        &vec![
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"org".to_vec()), Kind::Normal),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"repo".to_vec()), Kind::Normal),
            Piece::String(b"/releases/download/".to_vec()),
            Piece::Parameter(Position::Named(b"tag".to_vec()), Kind::Normal),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"filename".to_vec()), Kind::Normal),
            Piece::String(b".".to_vec()),
            Piece::Parameter(Position::Named(b"ext".to_vec()), Kind::Normal),
        ]
    );
    assert_eq!(
        p.pattern(),
        "/:org/:repo/releases/download/:tag/:filename.:ext"
    );
    assert_eq!(
        tree.url_for(*p.id, &["viz-rs", "path-tree", "v0.5.0", "v0.5.0", "gz"])
            .unwrap(),
        "/viz-rs/path-tree/releases/download/v0.5.0/v0.5.0.gz"
    );
}

#[test]
fn cloneable() {
    let tree = PathTree::<usize>::new();
    assert_eq!(
        <dyn std::any::Any>::type_id(&tree),
        <dyn std::any::Any>::type_id(&tree.clone())
    );
}

#[test]
fn test_dots_no_ext() {
    let mut tree = PathTree::new();
    let _ = tree.insert("/:name", 1);

    let result = tree.find("/abc.xyz.123");
    assert!(result.is_some());

    let (value, params) = result.unwrap();
    assert_eq!(value, &1);

    assert_eq!(params.params(), &[("name", "abc.xyz.123")]);
}

#[test]
fn test_dots_ext() {
    let mut tree = PathTree::new();
    let _ = tree.insert("/:name*.123", 2);
    let _ = tree.insert("/:name*.123.456", 1);

    let result = tree.find("/abc.xyz.123");
    assert!(result.is_some());

    let (value, params) = result.unwrap();
    assert_eq!(value, &2);

    assert_eq!(params.params(), &[("name", "abc.xyz")]);

    let result = tree.find("/abc.xyz.123.456");
    assert!(result.is_some());

    let (value, params) = result.unwrap();
    assert_eq!(value, &1);

    assert_eq!(params.params(), &[("name", "abc.xyz")]);
}
