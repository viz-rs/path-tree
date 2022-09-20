use path_tree::*;
use smallvec::smallvec;

#[test]
fn match_params() {
    // /
    // └── api/v1/
    //     └── :
    //         └── /
    //             └── ** •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/api/v1/:param/*", 1);

    assert_eq!(tree.find("/api/v1/entity"), None);
    assert_eq!(
        tree.find("/api/v1/entity/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["entity", ""]
        ))
    );
    assert_eq!(
        tree.find("/api/v1/entity/1"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["entity", "1"]
        ))
    );
    assert_eq!(tree.find("/api/v"), None);
    assert_eq!(tree.find("/api/v2"), None);
    assert_eq!(tree.find("/api/v1/"), None);
    assert_eq!(
        tree.find("/api/v1/entity/1/foo/bar"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["entity", "1/foo/bar"]
        ))
    );

    // /
    // └── api/v1/
    //     └── :
    //         └── /
    //             └── + •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/api/v1/:param/+", 1);

    assert_eq!(tree.find("/api/v1/entity"), None);
    assert_eq!(tree.find("/api/v1/entity/"), None);
    assert_eq!(
        tree.find("/api/v1/entity/1"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::OneOrMore),
            ],
            smallvec!["entity", "1"]
        ))
    );
    assert_eq!(tree.find("/api/v"), None);
    assert_eq!(tree.find("/api/v2"), None);
    assert_eq!(tree.find("/api/v1/"), None);
    assert_eq!(
        tree.find("/api/v1/entity/1/foo/bar"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::OneOrMore),
            ],
            smallvec!["entity", "1/foo/bar"]
        ))
    );

    // /
    // └── api/v1/
    //     └── ?? •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/api/v1/:param?", 1);

    assert_eq!(
        tree.find("/api/v1/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::OptionalSegment)
            ],
            smallvec![""]
        ))
    );
    assert_eq!(tree.find("/api/v1/entity/1/foo/bar"), None);
    assert_eq!(tree.find("/api/v"), None);
    assert_eq!(tree.find("/api/v2"), None);
    assert_eq!(tree.find("/api/xyz"), None);

    // /
    // └── v1/some/resource/name
    //     └── \:
    //         └── customVerb •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/v1/some/resource/name\\:customVerb", 1);

    assert_eq!(
        tree.find("/v1/some/resource/name:customVerb"),
        Some((
            &1,
            &vec![
                Piece::String(b"/v1/some/resource/name"),
                Piece::String(b":"),
                Piece::String(b"customVerb"),
            ],
            smallvec![]
        ))
    );
    assert_eq!(tree.find("/v1/some/resource/name:test"), None);

    // /
    // └── v1/some/resource/
    //     └── :
    //         └── \:
    //             └── customVerb •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert(r"/v1/some/resource/:name\:customVerb", 1);

    assert_eq!(
        tree.find("/v1/some/resource/test:customVerb"),
        Some((
            &1,
            &vec![
                Piece::String(b"/v1/some/resource/"),
                Piece::Parameter(Position::Named("name"), Kind::Normal),
                Piece::String(b":"),
                Piece::String(b"customVerb"),
            ],
            smallvec!["test"]
        ))
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
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert(r"/v1/some/resource/name\\\\:customVerb?\?/:param/*", 1);

    assert_eq!(
        tree.find("/v1/some/resource/name:customVerb??/test/optionalWildCard/character"),
        Some((
            &1,
            &vec![
                Piece::String(b"/v1/some/resource/name"),
                Piece::String(b":"),
                Piece::String(b"customVerb?"),
                Piece::String(b"?"),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["test", "optionalWildCard/character"]
        ))
    );
    assert_eq!(
        tree.find("/v1/some/resource/name:customVerb??/test/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/v1/some/resource/name"),
                Piece::String(b":"),
                Piece::String(b"customVerb?"),
                Piece::String(b"?"),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["test", ""]
        ))
    );
    assert_eq!(tree.find("/v1/some/resource/name:customVerb??/test"), None);

    // /
    // └── api/v1/
    //     └── ** •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/api/v1/*", 1);

    assert_eq!(tree.find("/api/v1"), None);
    assert_eq!(
        tree.find("/api/v1/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec![""]
        ))
    );
    assert_eq!(
        tree.find("/api/v1/entity"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["entity"]
        ))
    );
    assert_eq!(
        tree.find("/api/v1/entity/1/2"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["entity/1/2"]
        ))
    );
    assert_eq!(
        tree.find("/api/v1/Entity/1/2"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["Entity/1/2"]
        ))
    );

    // /
    // └── api/v1/
    //     └── : •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/api/v1/:param", 1);

    assert_eq!(tree.find("/api/v1"), None);
    assert_eq!(tree.find("/api/v1/"), None);
    assert_eq!(
        tree.find("/api/v1/entity"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
            ],
            smallvec!["entity"]
        ))
    );
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
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/api/v1/:param/:param2", 3);
    tree.insert("/api/v1/:param-:param2", 1);
    tree.insert("/api/v1/:param~:param2", 2);
    tree.insert("/api/v1/:param.:param2", 4);
    tree.insert("/api/v1/:param_:param2", 5);
    tree.insert("/api/v1/:param\\::param2", 6);

    assert_eq!(
        tree.find("/api/v1/entity-entity2"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"-"),
                Piece::Parameter(Position::Named("param2"), Kind::Normal),
            ],
            smallvec!["entity", "entity2"]
        )),
    );
    assert_eq!(
        tree.find("/api/v1/entity~entity2"),
        Some((
            &2,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"~"),
                Piece::Parameter(Position::Named("param2"), Kind::Normal),
            ],
            smallvec!["entity", "entity2"]
        )),
    );
    assert_eq!(
        tree.find("/api/v1/entity.entity2"),
        Some((
            &4,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"."),
                Piece::Parameter(Position::Named("param2"), Kind::Normal),
            ],
            smallvec!["entity", "entity2"]
        )),
    );
    assert_eq!(
        tree.find("/api/v1/entity_entity2"),
        Some((
            &5,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"_"),
                Piece::Parameter(Position::Named("param2"), Kind::Normal),
            ],
            smallvec!["entity", "entity2"]
        )),
    );
    assert_eq!(
        tree.find("/api/v1/entity:entity2"),
        Some((
            &6,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b":"),
                Piece::Parameter(Position::Named("param2"), Kind::Normal),
            ],
            smallvec!["entity", "entity2"]
        )),
    );
    assert_eq!(
        tree.find("/api/v1/entity/entity2"),
        Some((
            &3,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("param2"), Kind::Normal),
            ],
            smallvec!["entity", "entity2"]
        ))
    );
    assert_eq!(tree.find("/api/v1"), None);
    assert_eq!(tree.find("/api/v1/"), None);
    assert_eq!(
        tree.find("/api/v1/test.pdf"),
        Some((
            &4,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"."),
                Piece::Parameter(Position::Named("param2"), Kind::Normal),
            ],
            smallvec!["test", "pdf"]
        )),
    );

    // /
    // └── api/v1/const •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/api/v1/const", 1);

    assert_eq!(
        tree.find("/api/v1/const"),
        Some((&1, &vec![Piece::String(b"/api/v1/const")], smallvec![])),
    );
    assert_eq!(tree.find("/api/v1/cons"), None);
    assert_eq!(tree.find("/api/v1/conststatic"), None);
    assert_eq!(tree.find("/api/v1/let"), None);
    assert_eq!(tree.find("/api/v1/"), None);
    assert_eq!(tree.find("/api/v1"), None);

    // /
    // └── api/
    //     └── :
    //         └── /fixedEnd •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/api/:param/fixedEnd", 1);

    assert_eq!(
        tree.find("/api/abc/fixedEnd"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"/fixedEnd"),
            ],
            smallvec!["abc"]
        )),
    );

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
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert(r"/shop/product/\::filter/color\::color/size\\::size", 1);

    assert_eq!(
        tree.find("/shop/product/:test/color:blue/size:xs"),
        Some((
            &1,
            &vec![
                Piece::String(b"/shop/product/"),
                Piece::String(b":"),
                Piece::Parameter(Position::Named("filter"), Kind::Normal),
                Piece::String(b"/color"),
                Piece::String(b":"),
                Piece::Parameter(Position::Named("color"), Kind::Normal),
                Piece::String(b"/size"),
                Piece::String(b":"),
                Piece::Parameter(Position::Named("size"), Kind::Normal),
            ],
            smallvec!["test", "blue", "xs"]
        ))
    );
    assert_eq!(tree.find("/shop/product/test/color:blue/size:xs"), None);

    // /
    // └── \:
    //     └── ? •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/\\::param?", 1);

    assert_eq!(
        tree.find("/:hello"),
        Some((
            &1,
            &vec![
                Piece::String(b"/"),
                Piece::String(b":"),
                Piece::Parameter(Position::Named("param"), Kind::Optional),
            ],
            smallvec!["hello"]
        ))
    );
    assert_eq!(
        tree.find("/:"),
        Some((
            &1,
            &vec![
                Piece::String(b"/"),
                Piece::String(b":"),
                Piece::Parameter(Position::Named("param"), Kind::Optional),
            ],
            smallvec![""]
        ))
    );
    assert_eq!(tree.find("/"), None);

    // /
    // └── test
    //     └── :
    //         └── : •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/test:sign:param", 1);

    assert_eq!(
        tree.find("/test-abc"),
        Some((
            &1,
            &vec![
                Piece::String(b"/test"),
                Piece::Parameter(Position::Named("sign"), Kind::Normal),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
            ],
            smallvec!["-", "abc"]
        ))
    );
    assert_eq!(
        tree.find("/test-_"),
        Some((
            &1,
            &vec![
                Piece::String(b"/test"),
                Piece::Parameter(Position::Named("sign"), Kind::Normal),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
            ],
            smallvec!["-", "_"]
        ))
    );
    assert_eq!(tree.find("/test-"), None);
    assert_eq!(tree.find("/test"), None);

    // /
    // └── :
    //     └── ?
    //         └── : •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/:param1:param2?:param3", 1);

    assert_eq!(
        tree.find("/abbbc"),
        Some((
            &1,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("param1"), Kind::Normal),
                Piece::Parameter(Position::Named("param2"), Kind::Optional),
                Piece::Parameter(Position::Named("param3"), Kind::Normal),
            ],
            smallvec!["a", "b", "bbc"]
        ))
    );
    assert_eq!(
        tree.find("/ab"),
        Some((
            &1,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("param1"), Kind::Normal),
                Piece::Parameter(Position::Named("param2"), Kind::Optional),
                Piece::Parameter(Position::Named("param3"), Kind::Normal),
            ],
            smallvec!["a", "", "b"]
        ))
    );
    assert_eq!(tree.find("/a"), None);

    // /
    // └── test
    //     └── ?
    //         └── : •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/test:optional?:mandatory", 1);

    assert_eq!(
        tree.find("/testo"),
        Some((
            &1,
            &vec![
                Piece::String(b"/test"),
                Piece::Parameter(Position::Named("optional"), Kind::Optional),
                Piece::Parameter(Position::Named("mandatory"), Kind::Normal),
            ],
            smallvec!["", "o"]
        ))
    );
    assert_eq!(
        tree.find("/testoaaa"),
        Some((
            &1,
            &vec![
                Piece::String(b"/test"),
                Piece::Parameter(Position::Named("optional"), Kind::Optional),
                Piece::Parameter(Position::Named("mandatory"), Kind::Normal),
            ],
            smallvec!["o", "aaa"]
        ))
    );
    assert_eq!(tree.find("/test"), None);
    assert_eq!(tree.find("/tes"), None);

    // /
    // └── test
    //     └── ?
    //         └── ? •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/test:optional?:optional2?", 1);

    assert_eq!(
        tree.find("/testo"),
        Some((
            &1,
            &vec![
                Piece::String(b"/test"),
                Piece::Parameter(Position::Named("optional"), Kind::Optional),
                Piece::Parameter(Position::Named("optional2"), Kind::Optional),
            ],
            smallvec!["o", ""]
        ))
    );
    assert_eq!(
        tree.find("/testoaaa"),
        Some((
            &1,
            &vec![
                Piece::String(b"/test"),
                Piece::Parameter(Position::Named("optional"), Kind::Optional),
                Piece::Parameter(Position::Named("optional2"), Kind::Optional),
            ],
            smallvec!["o", "aaa"]
        ))
    );
    assert_eq!(
        tree.find("/test"),
        Some((
            &1,
            &vec![
                Piece::String(b"/test"),
                Piece::Parameter(Position::Named("optional"), Kind::Optional),
                Piece::Parameter(Position::Named("optional2"), Kind::Optional),
            ],
            smallvec!["", ""]
        ))
    );
    assert_eq!(tree.find("/tes"), None);

    // /
    // └── foo
    //     └── ?
    //         └── bar •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/foo:param?bar", 1);

    assert_eq!(
        tree.find("/foofalsebar"),
        Some((
            &1,
            &vec![
                Piece::String(b"/foo"),
                Piece::Parameter(Position::Named("param"), Kind::Optional),
                Piece::String(b"bar"),
            ],
            smallvec!["false"]
        ))
    );
    assert_eq!(
        tree.find("/foobar"),
        Some((
            &1,
            &vec![
                Piece::String(b"/foo"),
                Piece::Parameter(Position::Named("param"), Kind::Optional),
                Piece::String(b"bar"),
            ],
            smallvec![""]
        ))
    );
    assert_eq!(tree.find("/fooba"), None);
    assert_eq!(tree.find("/foo"), None);

    // /
    // └── foo
    //     └── *
    //         └── bar •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/foo*bar", 1);

    assert_eq!(
        tree.find("/foofalsebar"),
        Some((
            &1,
            &vec![
                Piece::String(b"/foo"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"bar"),
            ],
            smallvec!["false"]
        ))
    );
    assert_eq!(
        tree.find("/foobar"),
        Some((
            &1,
            &vec![
                Piece::String(b"/foo"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"bar"),
            ],
            smallvec![""]
        ))
    );
    assert_eq!(
        tree.find("/foo/bar"),
        Some((
            &1,
            &vec![
                Piece::String(b"/foo"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"bar"),
            ],
            smallvec!["/"]
        ))
    );
    assert_eq!(
        tree.find("/foo/baz/bar"),
        Some((
            &1,
            &vec![
                Piece::String(b"/foo"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"bar"),
            ],
            smallvec!["/baz/"]
        ))
    );
    assert_eq!(tree.find("/fooba"), None);
    assert_eq!(tree.find("/foo"), None);

    // /
    // └── foo
    //     └── +
    //         └── bar •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/foo+bar", 1);

    assert_eq!(
        tree.find("/foofalsebar"),
        Some((
            &1,
            &vec![
                Piece::String(b"/foo"),
                Piece::Parameter(Position::Index(1), Kind::OneOrMore),
                Piece::String(b"bar"),
            ],
            smallvec!["false"]
        ))
    );
    assert_eq!(tree.find("/foobar"), None);
    assert_eq!(
        tree.find("/foo/bar"),
        Some((
            &1,
            &vec![
                Piece::String(b"/foo"),
                Piece::Parameter(Position::Index(1), Kind::OneOrMore),
                Piece::String(b"bar"),
            ],
            smallvec!["/"]
        ))
    );
    assert_eq!(
        tree.find("/foo/baz/bar"),
        Some((
            &1,
            &vec![
                Piece::String(b"/foo"),
                Piece::Parameter(Position::Index(1), Kind::OneOrMore),
                Piece::String(b"bar"),
            ],
            smallvec!["/baz/"]
        ))
    );
    assert_eq!(tree.find("/fooba"), None);
    assert_eq!(tree.find("/foo"), None);

    // /
    // └── a
    //     └── *
    //         └── cde
    //             └── *
    //                 └── g/ •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/a*cde*g/", 1);

    assert_eq!(tree.find("/abbbcdefffg"), None);
    assert_eq!(
        tree.find("/abbbcdefffg/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/a"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"cde"),
                Piece::Parameter(Position::Index(2), Kind::ZeroOrMore),
                Piece::String(b"g/"),
            ],
            smallvec!["bbb", "fff"]
        ))
    );
    assert_eq!(
        tree.find("/acdeg/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/a"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"cde"),
                Piece::Parameter(Position::Index(2), Kind::ZeroOrMore),
                Piece::String(b"g/"),
            ],
            smallvec!["", ""]
        ))
    );
    assert_eq!(
        tree.find("/abcdeg/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/a"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"cde"),
                Piece::Parameter(Position::Index(2), Kind::ZeroOrMore),
                Piece::String(b"g/"),
            ],
            smallvec!["b", ""]
        ))
    );
    assert_eq!(
        tree.find("/acdefg/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/a"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"cde"),
                Piece::Parameter(Position::Index(2), Kind::ZeroOrMore),
                Piece::String(b"g/"),
            ],
            smallvec!["", "f"]
        ))
    );
    assert_eq!(
        tree.find("/abcdefg/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/a"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"cde"),
                Piece::Parameter(Position::Index(2), Kind::ZeroOrMore),
                Piece::String(b"g/"),
            ],
            smallvec!["b", "f"]
        ))
    );
    assert_eq!(
        tree.find("/a/cde/g/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/a"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"cde"),
                Piece::Parameter(Position::Index(2), Kind::ZeroOrMore),
                Piece::String(b"g/"),
            ],
            smallvec!["/", "/"]
        ))
    );
    assert_eq!(
        tree.find("/a/b/cde/f/g/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/a"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"cde"),
                Piece::Parameter(Position::Index(2), Kind::ZeroOrMore),
                Piece::String(b"g/"),
            ],
            smallvec!["/b/", "/f/"]
        ))
    );

    // /
    // └── *
    //     └── v1
    //         └── *
    //             └── proxy/ •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/*v1*/proxy", 1);

    assert_eq!(
        tree.find("/customer/v1/cart/proxy"),
        Some((
            &1,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"v1"),
                Piece::Parameter(Position::Index(2), Kind::ZeroOrMore),
                Piece::String(b"/proxy"),
            ],
            smallvec!["customer/", "/cart"]
        ))
    );
    assert_eq!(
        tree.find("/v1/proxy"),
        Some((
            &1,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
                Piece::String(b"v1"),
                Piece::Parameter(Position::Index(2), Kind::ZeroOrMore),
                Piece::String(b"/proxy"),
            ],
            smallvec!["", ""]
        ))
    );
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
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/name\\::name", 1);
    tree.insert("/@:name", 2);
    tree.insert("/-:name", 3);
    tree.insert("/.:name", 4);
    tree.insert("/~:name", 5);
    tree.insert("/_:name", 6);
    tree.insert("/:name", 7);

    assert_eq!(
        tree.find("/name:john"),
        Some((
            &1,
            &vec![
                Piece::String(b"/name"),
                Piece::String(b":"),
                Piece::Parameter(Position::Named("name"), Kind::Normal),
            ],
            smallvec!["john"]
        ))
    );
    assert_eq!(
        tree.find("/@john"),
        Some((
            &2,
            &vec![
                Piece::String(b"/@"),
                Piece::Parameter(Position::Named("name"), Kind::Normal),
            ],
            smallvec!["john"]
        ))
    );
    assert_eq!(
        tree.find("/-john"),
        Some((
            &3,
            &vec![
                Piece::String(b"/-"),
                Piece::Parameter(Position::Named("name"), Kind::Normal),
            ],
            smallvec!["john"]
        ))
    );
    assert_eq!(
        tree.find("/.john"),
        Some((
            &4,
            &vec![
                Piece::String(b"/."),
                Piece::Parameter(Position::Named("name"), Kind::Normal),
            ],
            smallvec!["john"]
        ))
    );
    assert_eq!(
        tree.find("/~john"),
        Some((
            &5,
            &vec![
                Piece::String(b"/~"),
                Piece::Parameter(Position::Named("name"), Kind::Normal),
            ],
            smallvec!["john"]
        ))
    );
    assert_eq!(
        tree.find("/_john"),
        Some((
            &6,
            &vec![
                Piece::String(b"/_"),
                Piece::Parameter(Position::Named("name"), Kind::Normal),
            ],
            smallvec!["john"]
        ))
    );
    assert_eq!(
        tree.find("/john"),
        Some((
            &7,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("name"), Kind::Normal),
            ],
            smallvec!["john"]
        ))
    );

    // /
    // └── api/v1/
    //     └── :
    //         └── /abc/
    //             └── ** •0
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/api/v1/:param/abc/*", 1);

    dbg!(&tree.node);
    assert_eq!(
        tree.find("/api/v1/well/abc/wildcard"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"/abc/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["well", "wildcard"]
        ))
    );
    assert_eq!(
        tree.find("/api/v1/well/abc/"),
        Some((
            &1,
            &vec![
                Piece::String(b"/api/v1/"),
                Piece::Parameter(Position::Named("param"), Kind::Normal),
                Piece::String(b"/abc/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["well", ""]
        ))
    );
    assert_eq!(tree.find("/api/v1/well/abc"), None);
    assert_eq!(tree.find("/api/v1/well/ttt"), None);
}

#[test]
fn basic() {
    let mut tree = PathTree::<'static, usize>::new("/");

    tree.insert("/", 0);
    tree.insert("/users", 1);
    tree.insert("/users/:id", 2);
    tree.insert("/users/:id/:org", 3);
    tree.insert("/users/:userId/repos", 4);
    tree.insert("/users/:userId/repos/:id", 5);
    tree.insert("/users/:userId/repos/:id/:any*", 6);
    tree.insert(r"/\\::username", 7);
    tree.insert("/*", 8);
    tree.insert("/about", 9);
    tree.insert("/about/", 10);
    tree.insert("/about/us", 11);
    tree.insert("/users/repos/*", 12);
    tree.insert("/:action", 13);
    tree.insert("", 14);

    assert_eq!(
        format!("{:?}", &tree.node),
        r#"
/ •0
├── \:
│   └── : •7
├── about •9
│   └── / •10
│       └── us •11
├── users •1
│   └── /
│       ├── repos/
│       │   └── ** •12
│       └── : •2
│           └── /
│               ├── repos •4
│               │   └── /
│               │       └── : •5
│               │           └── /
│               │               └── ** •6
│               └── : •3
├── : •13
└── ** •8
"#
    );

    assert_eq!(
        tree.find("/"),
        Some((&0, &vec![Piece::String(b"/")], smallvec![]))
    );
    assert_eq!(
        tree.find("/users"),
        Some((&1, &vec![Piece::String(b"/users")], smallvec![]))
    );
    assert_eq!(
        tree.find("/users/foo"),
        Some((
            &2,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("id"), Kind::Normal)
            ],
            smallvec!["foo"]
        ))
    );
    assert_eq!(
        tree.find("/users/foo/bar"),
        Some((
            &3,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("id"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("org"), Kind::Normal),
            ],
            smallvec!["foo", "bar"]
        ))
    );
    assert_eq!(
        tree.find("/users/foo/repos"),
        Some((
            &4,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("userId"), Kind::Normal),
                Piece::String(b"/repos"),
            ],
            smallvec!["foo"]
        ))
    );
    assert_eq!(
        tree.find("/users/foo/repos/bar"),
        Some((
            &5,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("userId"), Kind::Normal),
                Piece::String(b"/repos/"),
                Piece::Parameter(Position::Named("id"), Kind::Normal),
            ],
            smallvec!["foo", "bar"]
        ))
    );
    assert_eq!(
        tree.find("/users/foo/repos/bar/"),
        Some((
            &6,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("userId"), Kind::Normal),
                Piece::String(b"/repos/"),
                Piece::Parameter(Position::Named("id"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("any"), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["foo", "bar", ""]
        ))
    );
    assert_eq!(
        tree.find("/users/foo/repos/bar/baz"),
        Some((
            &6,
            &vec![
                Piece::String(b"/users/"),
                Piece::Parameter(Position::Named("userId"), Kind::Normal),
                Piece::String(b"/repos/"),
                Piece::Parameter(Position::Named("id"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("any"), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["foo", "bar", "baz"]
        ))
    );
    assert_eq!(
        tree.find("/:foo"),
        Some((
            &7,
            &vec![
                Piece::String(b"/"),
                Piece::String(b":"),
                Piece::Parameter(Position::Named("username"), Kind::Normal),
            ],
            smallvec!["foo"]
        ))
    );
    assert_eq!(
        tree.find("/foo/bar/baz/404"),
        Some((
            &8,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["foo/bar/baz/404"]
        ))
    );
    assert_eq!(
        tree.find("/about"),
        Some((&9, &vec![Piece::String(b"/about")], smallvec![]))
    );
    assert_eq!(
        tree.find("/about/"),
        Some((&10, &vec![Piece::String(b"/about/")], smallvec![]))
    );
    assert_eq!(
        tree.find("/about/us"),
        Some((&11, &vec![Piece::String(b"/about/us")], smallvec![]))
    );
    assert_eq!(
        tree.find("/users/repos/foo"),
        Some((
            &12,
            &vec![
                Piece::String(b"/users/repos/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["foo"]
        ))
    );
    assert_eq!(
        tree.find("/users/repos/foo/bar"),
        Some((
            &12,
            &vec![
                Piece::String(b"/users/repos/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["foo/bar"]
        ))
    );
    assert_eq!(
        tree.find("/-foo"),
        Some((
            &13,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("action"), Kind::Normal),
            ],
            smallvec!["-foo"]
        ))
    );
}

#[test]
fn github_tree() {
    let mut tree = PathTree::<'static, usize>::new("/");

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
    tree.insert("/:org/:repo/releases/download/:tag/:filename.:ext", 3002);

    assert_eq!(
        format!("{:?}", dbg!(&tree.node)),
        r#"
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
"#
    );

    assert_eq!(
        tree.find("/rust-lang/rust"),
        Some((
            &2400,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("org"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("repo"), Kind::Normal),
            ],
            smallvec!["rust-lang", "rust"]
        ))
    );

    assert_eq!(
        tree.find("/settings"),
        Some((&20, &vec![Piece::String(b"/settings")], smallvec![]))
    );

    assert_eq!(
        tree.find("/rust-lang/rust/actions/runs/1"),
        Some((
            &2442,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("org"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("repo"), Kind::Normal),
                Piece::String(b"/actions/runs/"),
                Piece::Parameter(Position::Named("id"), Kind::Normal),
            ],
            smallvec!["rust-lang", "rust", "1"]
        ))
    );

    assert_eq!(
        tree.find("/rust-lang/rust/"),
        Some((
            &3000,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("org"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("repo"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["rust-lang", "rust", ""]
        ))
    );

    assert_eq!(
        tree.find("/rust-lang/rust/any"),
        Some((
            &3000,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("org"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("repo"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["rust-lang", "rust", "any"]
        ))
    );

    assert_eq!(
        tree.find("/rust-lang/rust/releases/"),
        Some((
            &3001,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("org"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("repo"), Kind::Normal),
                Piece::String(b"/releases/"),
                Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            ],
            smallvec!["rust-lang", "rust", ""]
        ))
    );

    assert_eq!(
        tree.find("/rust-lang/rust-analyzer/releases/download/2022-09-12/rust-analyzer-aarch64-apple-darwin.gz"),
        Some((
            &3002,
            &vec![
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("org"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("repo"), Kind::Normal),
                Piece::String(b"/releases/download/"),
                Piece::Parameter(Position::Named("tag"), Kind::Normal),
                Piece::String(b"/"),
                Piece::Parameter(Position::Named("filename"), Kind::Normal),
                Piece::String(b"."),
                Piece::Parameter(Position::Named("ext"), Kind::Normal),
            ],
            smallvec!["rust-lang", "rust-analyzer", "2022-09-12", "rust-analyzer-aarch64-apple-darwin", "gz"]
        ))
    );
}
