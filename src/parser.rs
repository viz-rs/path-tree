use std::{iter::Peekable, str::CharIndices};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Kind {
    /// `:` 58
    Normal,
    /// `?` 63
    /// Optional: `/:name?-`
    Optional,
    /// Optional Segment: `/:name?/` or `/:name?`
    OptionalSegment,
    // Optional,
    /// `+` 43
    OneOrMore,
    /// `*` 42
    /// ZeroOrMore: `/*-`
    ZeroOrMore,
    /// ZeroOrMore Segment: `/*/` or `/*`
    ZeroOrMoreSegment,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Piece<'a> {
    String(&'a [u8]),
    Parameter(Position<'a>, Kind),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Position<'a> {
    Index(usize),
    Named(&'a str),
}

pub struct Parser<'a> {
    pos: usize,
    count: usize,
    input: &'a str,
    cursor: Peekable<CharIndices<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            pos: 0,
            count: 0,
            cursor: input.char_indices().peekable(),
        }
    }

    fn string(&mut self) -> &'a [u8] {
        let mut start = self.pos;
        while let Some(&(i, c)) = self.cursor.peek() {
            match c {
                '\\' => {
                    if start < i {
                        self.pos = i;
                        return self.input[start..i].as_bytes();
                    }

                    self.cursor.next();
                    if let Some(&(j, c)) = self.cursor.peek() {
                        // removes `\`
                        if c == '\\' {
                            start = j;
                        } else {
                            self.cursor.next();
                            self.pos = j + 1;
                            return self.input[j..self.pos].as_bytes();
                        }
                    }
                }
                ':' | '+' | '*' => {
                    self.pos = i + 1;
                    return self.input[start..i].as_bytes();
                }
                _ => {
                    self.cursor.next();
                }
            }
        }

        self.input[start..self.input.len()].as_bytes()
    }

    fn parameter(&mut self) -> (Position<'a>, Kind) {
        let start = self.pos;
        while let Some(&(i, c)) = self.cursor.peek() {
            match c {
                '-' | '.' | '_' | '~' | '/' | '\\' | ':' => {
                    self.pos = i;
                    return (Position::Named(&self.input[start..i]), Kind::Normal);
                }
                '?' | '+' | '*' => {
                    self.cursor.next();
                    self.pos = i + 1;
                    return (
                        Position::Named(&self.input[start..i]),
                        if c == '+' {
                            Kind::OneOrMore
                        } else {
                            let f = {
                                let prefix = self
                                    .input
                                    .get(start - 2..start - 1)
                                    .map(|s| s == "/")
                                    .unwrap_or(false);
                                let suffix =
                                    self.cursor.peek().map(|(_, c)| *c == '/').unwrap_or(true);
                                prefix && suffix
                            };
                            if c == '?' {
                                if f {
                                    Kind::OptionalSegment
                                } else {
                                    Kind::Optional
                                }
                            } else {
                                if f {
                                    Kind::ZeroOrMoreSegment
                                } else {
                                    Kind::ZeroOrMore
                                }
                            }
                        },
                    );
                }
                _ => {
                    self.cursor.next();
                }
            }
        }

        (
            Position::Named(&self.input[start..self.input.len()]),
            Kind::Normal,
        )
    }
}

impl<'a> Iterator for Parser<'a> {
    type Item = Piece<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.cursor.peek() {
            Some(&(i, c)) => match c {
                ':' => {
                    self.cursor.next();
                    self.pos = i + 1;
                    let (position, kind) = self.parameter();
                    Some(Piece::Parameter(position, kind))
                }
                '+' | '*' => {
                    self.cursor.next();
                    self.count += 1;
                    self.pos = i + 1;
                    Some(Piece::Parameter(
                        Position::Index(self.count),
                        if c == '+' {
                            Kind::OneOrMore
                        } else {
                            let f = {
                                let prefix =
                                    self.input.get(i - 1..i).map(|s| s == "/").unwrap_or(false);
                                let suffix =
                                    self.cursor.peek().map(|(_, c)| *c == '/').unwrap_or(true);
                                prefix && suffix
                            };
                            if f {
                                Kind::ZeroOrMoreSegment
                            } else {
                                Kind::ZeroOrMore
                            }
                        },
                    ))
                }
                _ => Some(Piece::String(self.string())),
            },
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
