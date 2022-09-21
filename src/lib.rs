#![forbid(unsafe_code)]
// #![warn(
//     missing_debug_implementations,
//     missing_docs,
//     rust_2018_idioms,
//     unreachable_pub
// )]
// #![doc(test(
//     no_crate_inject,
//     attr(
//         deny(warnings, rust_2018_idioms),
//         allow(dead_code, unused_assignments, unused_variables)
//     )
// ))]
// #![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

use smallvec::SmallVec;
use std::{fmt, str::from_utf8};

mod node;
mod parser;

pub use node::{Node, NodeKind};
pub use parser::{Kind, Parser, Piece, Position};

#[derive(Debug)]
pub struct PathTree<'a, T> {
    id: usize,
    routes: Vec<(T, Vec<Piece<'a>>)>,
    pub node: Node<'a, usize>,
}

impl<'a, T: fmt::Debug> Default for PathTree<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T: fmt::Debug> PathTree<'a, T> {
    pub fn new() -> Self {
        Self {
            id: 0,
            routes: Vec::new(),
            node: Node::new(NodeKind::String("".as_bytes()), None),
        }
    }

    pub fn insert(&mut self, path: &'a str, value: T) -> &mut Self {
        if path.is_empty() {
            return self;
        }

        let mut node = &mut self.node;
        let pieces = Parser::new(path).collect::<Vec<_>>();

        for piece in &pieces {
            match piece {
                Piece::String(s) => {
                    node = node.insert_bytes(s);
                }
                Piece::Parameter(_, k) => {
                    node = node.insert_parameter(*k);
                }
            }
        }

        node.value = Some(self.id);
        self.routes.push((value, pieces));
        self.id += 1;
        self
    }

    pub fn find<'b>(&'a self, path: &'b str) -> Option<Path<'a, 'b, T>> {
        let bytes = path.as_bytes();
        self.node.find(bytes).and_then(|(id, ranges)| {
            self.get_route(*id).map(|(value, pieces)| {
                Path {
                    id,
                    value,
                    pieces,
                    // opt!
                    params: ranges
                        .chunks(2)
                        .map(|r| from_utf8(&bytes[r[0]..r[1]]).unwrap())
                        .rev()
                        .collect(),
                }
            })
        })
    }

    #[inline]
    pub fn get_route(&self, index: usize) -> Option<&(T, Vec<Piece<'a>>)> {
        self.routes.get(index)
    }

    // Generates URL
    // pub fn url_for(&self, index: usize, params: Vec<String>) -> Option<String> {
    //     None
    // }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Path<'a, 'b, T> {
    pub id: &'a usize,
    pub value: &'a T,
    pub pieces: &'a [Piece<'a>],
    pub params: SmallVec<[&'b str; 4]>,
}

impl<'a, 'b, T> Path<'a, 'b, T> {
    pub fn pattern(&self) -> String {
        let mut bytes = Vec::new();

        for piece in self.pieces {
            match piece {
                Piece::String(s) => {
                    if s == b":" || s == b"+" || s == b"?" {
                        bytes.push(b'\\');
                    }
                    bytes.extend_from_slice(s);
                }
                Piece::Parameter(p, k) => match p {
                    Position::Index(_) => {
                        if *k == Kind::OneOrMore {
                            bytes.push(b'+');
                        } else if *k == Kind::ZeroOrMore || *k == Kind::ZeroOrMoreSegment {
                            bytes.push(b'*');
                        }
                    }
                    Position::Named(n) => match k {
                        Kind::Normal | Kind::Optional | Kind::OptionalSegment => {
                            bytes.push(b':');
                            bytes.extend_from_slice(n.as_bytes());
                            if *k == Kind::Optional || *k == Kind::OptionalSegment {
                                bytes.push(b'?');
                            }
                        }
                        Kind::OneOrMore => {
                            bytes.push(b'+');
                            bytes.extend_from_slice(n.as_bytes());
                        }
                        Kind::ZeroOrMore | Kind::ZeroOrMoreSegment => {
                            bytes.push(b'*');
                            bytes.extend_from_slice(n.as_bytes());
                        }
                    },
                },
            }
        }

        String::from_utf8_lossy(&bytes).to_string()
    }
}
