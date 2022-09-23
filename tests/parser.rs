use path_tree::*;

#[test]
fn parses() {
    assert_eq!(
        Parser::new(r"/shop/product/\::filter/color\::color/size\::size").collect::<Vec<_>>(),
        [
            Piece::String(b"/shop/product/".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::Parameter(Position::Named(b"filter".to_vec()), Kind::Normal),
            Piece::String(b"/color".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::Parameter(Position::Named(b"color".to_vec()), Kind::Normal),
            Piece::String(b"/size".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::Parameter(Position::Named(b"size".to_vec()), Kind::Normal),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:param/abc/*").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
            Piece::String(b"/abc/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMoreSegment),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:param/+").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Index(1, b"+1".to_vec()), Kind::OneOrMore),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:param?").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::OptionalSegment),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:param?").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::OptionalSegment),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:param").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/*").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMoreSegment),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:param-:param2").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
            Piece::String(b"-".to_vec()),
            Piece::Parameter(Position::Named(b"param2".to_vec()), Kind::Normal),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/:filename.:extension").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/v1/".to_vec()),
            Piece::Parameter(Position::Named(b"filename".to_vec()), Kind::Normal),
            Piece::String(b".".to_vec()),
            Piece::Parameter(Position::Named(b"extension".to_vec()), Kind::Normal),
        ],
    );

    assert_eq!(
        Parser::new("/api/v1/string").collect::<Vec<_>>(),
        [Piece::String(b"/api/v1/string".to_vec()),],
    );

    assert_eq!(
        Parser::new(r"/\::param?").collect::<Vec<_>>(),
        [
            Piece::String(b"/".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Optional),
        ],
    );

    assert_eq!(
        Parser::new("/:param1:param2?:param3").collect::<Vec<_>>(),
        [
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"param1".to_vec()), Kind::Normal),
            Piece::Parameter(Position::Named(b"param2".to_vec()), Kind::Optional),
            Piece::Parameter(Position::Named(b"param3".to_vec()), Kind::Normal),
        ],
    );

    assert_eq!(
        Parser::new("/test:sign:param").collect::<Vec<_>>(),
        [
            Piece::String(b"/test".to_vec()),
            Piece::Parameter(Position::Named(b"sign".to_vec()), Kind::Normal),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
        ],
    );

    assert_eq!(
        Parser::new("/foo:param?bar").collect::<Vec<_>>(),
        [
            Piece::String(b"/foo".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Optional),
            Piece::String(b"bar".to_vec()),
        ],
    );

    assert_eq!(
        Parser::new("/foo*bar").collect::<Vec<_>>(),
        [
            Piece::String(b"/foo".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMore),
            Piece::String(b"bar".to_vec()),
        ],
    );

    assert_eq!(
        Parser::new("/foo+bar").collect::<Vec<_>>(),
        [
            Piece::String(b"/foo".to_vec()),
            Piece::Parameter(Position::Index(1, b"+1".to_vec()), Kind::OneOrMore),
            Piece::String(b"bar".to_vec()),
        ],
    );

    assert_eq!(
        Parser::new("/a*cde*g/").collect::<Vec<_>>(),
        [
            Piece::String(b"/a".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMore),
            Piece::String(b"cde".to_vec()),
            Piece::Parameter(Position::Index(2, b"*2".to_vec()), Kind::ZeroOrMore),
            Piece::String(b"g/".to_vec()),
        ],
    );

    assert_eq!(
        Parser::new(r"/name\::name").collect::<Vec<_>>(),
        [
            Piece::String(b"/name".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/@:name").collect::<Vec<_>>(),
        [
            Piece::String(b"/@".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/-:name").collect::<Vec<_>>(),
        [
            Piece::String(b"/-".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/.:name").collect::<Vec<_>>(),
        [
            Piece::String(b"/.".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/_:name").collect::<Vec<_>>(),
        [
            Piece::String(b"/_".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/~:name").collect::<Vec<_>>(),
        [
            Piece::String(b"/~".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
        ]
    );

    assert_eq!(
        Parser::new("/v1/some/resource/name\\:customVerb").collect::<Vec<_>>(),
        [
            Piece::String(b"/v1/some/resource/name".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::String(b"customVerb".to_vec()),
        ],
    );

    assert_eq!(
        Parser::new("/v1/some/resource/:name\\:customVerb").collect::<Vec<_>>(),
        [
            Piece::String(b"/v1/some/resource/".to_vec()),
            Piece::Parameter(Position::Named(b"name".to_vec()), Kind::Normal),
            Piece::String(b":".to_vec()),
            Piece::String(b"customVerb".to_vec()),
        ],
    );

    assert_eq!(
        Parser::new("/v1/some/resource/name\\:customVerb??/:param/*").collect::<Vec<_>>(),
        [
            Piece::String(b"/v1/some/resource/name".to_vec()),
            Piece::String(b":".to_vec()),
            Piece::String(b"customVerb??/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMoreSegment)
        ],
    );

    assert_eq!(
        Parser::new("/api/*/:param/:param2").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMoreSegment),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"param".to_vec()), Kind::Normal),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"param2".to_vec()), Kind::Normal)
        ],
    );

    assert_eq!(
        Parser::new("/test:optional?:optional2?").collect::<Vec<_>>(),
        [
            Piece::String(b"/test".to_vec()),
            Piece::Parameter(Position::Named(b"optional".to_vec()), Kind::Optional),
            Piece::Parameter(Position::Named(b"optional2".to_vec()), Kind::Optional)
        ],
    );

    assert_eq!(
        Parser::new("/config/+.json").collect::<Vec<_>>(),
        [
            Piece::String(b"/config/".to_vec()),
            Piece::Parameter(Position::Index(1, b"+1".to_vec()), Kind::OneOrMore),
            Piece::String(b".json".to_vec()),
        ]
    );

    assert_eq!(
        Parser::new("/config/*.json").collect::<Vec<_>>(),
        [
            Piece::String(b"/config/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMore),
            Piece::String(b".json".to_vec()),
        ]
    );

    assert_eq!(
        Parser::new("/api/:day.:month?.:year?").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/".to_vec()),
            Piece::Parameter(Position::Named(b"day".to_vec()), Kind::Normal),
            Piece::String(b".".to_vec()),
            Piece::Parameter(Position::Named(b"month".to_vec()), Kind::Optional),
            Piece::String(b".".to_vec()),
            Piece::Parameter(Position::Named(b"year".to_vec()), Kind::Optional),
        ]
    );

    assert_eq!(
        Parser::new("/api/:day/:month?/:year?").collect::<Vec<_>>(),
        [
            Piece::String(b"/api/".to_vec()),
            Piece::Parameter(Position::Named(b"day".to_vec()), Kind::Normal),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"month".to_vec()), Kind::OptionalSegment),
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"year".to_vec()), Kind::OptionalSegment),
        ]
    );

    assert_eq!(
        Parser::new("/*v1*/proxy").collect::<Vec<_>>(),
        [
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Index(1, b"*1".to_vec()), Kind::ZeroOrMore),
            Piece::String(b"v1".to_vec()),
            Piece::Parameter(Position::Index(2, b"*2".to_vec()), Kind::ZeroOrMore),
            Piece::String(b"/proxy".to_vec()),
        ]
    );

    assert_eq!(
        Parser::new("/:a*v1:b+/proxy").collect::<Vec<_>>(),
        [
            Piece::String(b"/".to_vec()),
            Piece::Parameter(Position::Named(b"a".to_vec()), Kind::ZeroOrMore),
            Piece::String(b"v1".to_vec()),
            Piece::Parameter(Position::Named(b"b".to_vec()), Kind::OneOrMore),
            Piece::String(b"/proxy".to_vec()),
        ]
    );
}
