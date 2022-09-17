
use path_tree::*;

#[test]
fn parses() {
    assert_eq!(
        Parser::new(r"/shop/product/\::filter/color\::color/size\::size").collect::<Vec<_>>(),
        [
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
    );

    assert_eq!(
        Parser::new("/api/v1/:param/abc/*").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/"),
            Piece::Parameter(Position::Named("param"), Kind::Normal),
            Piece::String(b"/abc/"),
            Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:param/+").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/"),
            Piece::Parameter(Position::Named("param"), Kind::Normal),
            Piece::String(b"/"),
            Piece::Parameter(Position::Index(1), Kind::OneOrMore),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:param?").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/"),
            Piece::Parameter(Position::Named("param"), Kind::OptionalSegment),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:param?").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/"),
            Piece::Parameter(Position::Named("param"), Kind::OptionalSegment),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:param").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/"),
            Piece::Parameter(Position::Named("param"), Kind::Normal),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/*").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/"),
            Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:param-:param2").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/"),
            Piece::Parameter(Position::Named("param"), Kind::Normal),
            Piece::String(b"-"),
            Piece::Parameter(Position::Named("param2"), Kind::Normal),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:filename.:extension").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/"),
            Piece::Parameter(Position::Named("filename"), Kind::Normal),
            Piece::String(b"."),
            Piece::Parameter(Position::Named("extension"), Kind::Normal),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/string").collect::<Vec<_>>(),
        [Piece::String(b"/api/v1/string"),],
    );

    assert_eq!(
        Parser::new(r"/\::param?").collect::<Vec<_>>(),
        [
            Piece::String(b"/"),
            Piece::String(b":"),
            Piece::Parameter(Position::Named("param"), Kind::Optional),
        ],
    );

    assert_eq!(
        Parser::new("/:param1:param2?:param3").collect::<Vec<_>>(),
        [
            Piece::String(b"/"),
            Piece::Parameter(Position::Named("param1"), Kind::Normal),
            Piece::Parameter(Position::Named("param2"), Kind::Optional),
            Piece::Parameter(Position::Named("param3"), Kind::Normal),
        ],
    );

    assert_eq!(
        Parser::new("/test:sign:param").collect::<Vec<_>>(),
        [
            Piece::String(b"/test"),
            Piece::Parameter(Position::Named("sign"), Kind::Normal),
            Piece::Parameter(Position::Named("param"), Kind::Normal),
        ],
    );

    assert_eq!(
        Parser::new("/foo:param?bar").collect::<Vec<_>>(),
        [
            Piece::String(b"/foo"),
            Piece::Parameter(Position::Named("param"), Kind::Optional),
            Piece::String(b"bar"),
        ],
    );

    assert_eq!(
        Parser::new("/foo*bar").collect::<Vec<_>>(),
        [
            Piece::String(b"/foo"),
            Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
            Piece::String(b"bar"),
        ],
    );

    assert_eq!(
        Parser::new("/foo+bar").collect::<Vec<_>>(),
        [
            Piece::String(b"/foo"),
            Piece::Parameter(Position::Index(1), Kind::OneOrMore),
            Piece::String(b"bar"),
        ],
    );

    assert_eq!(
        Parser::new("/a*cde*g/").collect::<Vec<_>>(),
        [
            Piece::String(b"/a"),
            Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
            Piece::String(b"cde"),
            Piece::Parameter(Position::Index(2), Kind::ZeroOrMore),
            Piece::String(b"g/"),
        ],
    );

    assert_eq!(
        Parser::new(r"/name\::name").collect::<Vec<_>>(),
        [
            Piece::String(b"/name"),
            Piece::String(b":"),
            Piece::Parameter(Position::Named("name"), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/@:name").collect::<Vec<_>>(),
        [
            Piece::String(b"/@"),
            Piece::Parameter(Position::Named("name"), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/-:name").collect::<Vec<_>>(),
        [
            Piece::String(b"/-"),
            Piece::Parameter(Position::Named("name"), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/.:name").collect::<Vec<_>>(),
        [
            Piece::String(b"/."),
            Piece::Parameter(Position::Named("name"), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/_:name").collect::<Vec<_>>(),
        [
            Piece::String(b"/_"),
            Piece::Parameter(Position::Named("name"), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/~:name").collect::<Vec<_>>(),
        [
            Piece::String(b"/~"),
            Piece::Parameter(Position::Named("name"), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/v1/some/resource/name\\:customVerb").collect::<Vec<_>>(),
        [
            Piece::String(b"/v1/some/resource/name"),
            Piece::String(b":"),
            Piece::String(b"customVerb"),
        ],
    );

    assert_eq!(
        Parser::new("/v1/some/resource/:name\\:customVerb").collect::<Vec<_>>(),
        [
            Piece::String(b"/v1/some/resource/"),
            Piece::Parameter(Position::Named("name"), Kind::Normal),
            Piece::String(b":"),
            Piece::String(b"customVerb"),
        ],
    );

    assert_eq!(
        Parser::new("/v1/some/resource/name\\:customVerb??/:param/*").collect::<Vec<_>>(),
        [
            Piece::String(b"/v1/some/resource/name"),
            Piece::String(b":"),
            Piece::String(b"customVerb??/"),
            Piece::Parameter(Position::Named("param"), Kind::Normal),
            Piece::String(b"/"),
            Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment)
        ],
    );

    assert_eq!(
        Parser::new("/api/*/:param/:param2").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/"),
            Piece::Parameter(Position::Index(1), Kind::ZeroOrMoreSegment),
            Piece::String(b"/"),
            Piece::Parameter(Position::Named("param"), Kind::Normal),
            Piece::String(b"/"),
            Piece::Parameter(Position::Named("param2"), Kind::Normal)
        ],
    );

    assert_eq!(
        Parser::new("/test:optional?:optional2?").collect::<Vec<_>>(),
        [
            Piece::String(b"/test"),
            Piece::Parameter(Position::Named("optional"), Kind::Optional),
            Piece::Parameter(Position::Named("optional2"), Kind::Optional)
        ],
    );

    assert_eq!(
        Parser::new("/config/+.json").collect::<Vec<_>>(),
        [
            Piece::String(b"/config/"),
            Piece::Parameter(Position::Index(1), Kind::OneOrMore),
            Piece::String(b".json")
        ]
    );

    assert_eq!(
        Parser::new("/config/*.json").collect::<Vec<_>>(),
        [
            Piece::String(b"/config/"),
            Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
            Piece::String(b".json")
        ]
    );

    assert_eq!(
        Parser::new("/api/:day.:month?.:year?").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/"),
            Piece::Parameter(Position::Named("day"), Kind::Normal),
            Piece::String(b"."),
            Piece::Parameter(Position::Named("month"), Kind::Optional),
            Piece::String(b"."),
            Piece::Parameter(Position::Named("year"), Kind::Optional),
        ]
    );

    assert_eq!(
        Parser::new("/api/:day/:month?/:year?").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/"),
            Piece::Parameter(Position::Named("day"), Kind::Normal),
            Piece::String(b"/"),
            Piece::Parameter(Position::Named("month"), Kind::OptionalSegment),
            Piece::String(b"/"),
            Piece::Parameter(Position::Named("year"), Kind::OptionalSegment),
        ]
    );

    assert_eq!(
        Parser::new("/*v1*/proxy").collect::<Vec<_>>(),
        [
            Piece::String(b"/"),
            Piece::Parameter(Position::Index(1), Kind::ZeroOrMore),
            Piece::String(b"v1"),
            Piece::Parameter(Position::Index(2), Kind::ZeroOrMore),
            Piece::String(b"/proxy")
        ]
    );

    assert_eq!(
        Parser::new("/:a*v1:b+/proxy").collect::<Vec<_>>(),
        [
            Piece::String(b"/"),
            Piece::Parameter(Position::Named("a"), Kind::ZeroOrMore),
            Piece::String(b"v1"),
            Piece::Parameter(Position::Named("b"), Kind::OneOrMore),
            Piece::String(b"/proxy")
        ]
    );
}
