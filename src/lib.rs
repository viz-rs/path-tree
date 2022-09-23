//! path-tree is a lightweight high performance HTTP request router for Rust.
//!
//! # Example
//!
//! ```
//! use path_tree::PathTree;
//!
//! /*
//! / •0
//! ├── api/
//! │   └── + •13
//! ├── login •1
//! ├── public/
//! │   └── ** •7
//! ├── s
//! │   ├── ettings •3
//! │   │   └── /
//! │   │       └── : •4
//! │   └── ignup •2
//! └── : •5
//!     └── /
//!         └── : •6
//!             └── /
//!                 ├── actions/
//!                 │   └── :
//!                 │       └── \:
//!                 │           └── : •10
//!                 ├── releases/download/
//!                 │   └── :
//!                 │       └── /
//!                 │           └── :
//!                 │               └── .
//!                 │                   └── : •8
//!                 ├── tags/
//!                 │   └── :
//!                 │       └── -
//!                 │           └── :
//!                 │               └── -
//!                 │                   └── : •9
//!                 ├── : •11
//!                 └── ** •12
//! */
//! let mut tree = PathTree::new();
//!
//! tree.insert("/", 0);
//! tree.insert("/login", 1);
//! tree.insert("/signup", 2);
//! tree.insert("/settings", 3);
//! tree.insert("/settings/:page", 4);
//! tree.insert("/:user", 5);
//! tree.insert("/:user/:repo", 6);
//! tree.insert("/public/:any*", 7);
//! tree.insert("/:org/:repo/releases/download/:tag/:filename.:ext", 8);
//! tree.insert("/:org/:repo/tags/:day-:month-:year", 9);
//! tree.insert("/:org/:repo/actions/:name\\::verb", 10);
//! tree.insert("/:org/:repo/:page", 11);
//! tree.insert("/:org/:repo/*", 12);
//! tree.insert("/api/+", 13);
//!
//! let r = tree.find("/").unwrap();
//! assert_eq!(r.value, &0);
//! assert_eq!(r.params(), vec![]);
//!
//! let r = tree.find("/login").unwrap();
//! assert_eq!(r.value, &1);
//! assert_eq!(r.params(), vec![]);
//!
//! let r = tree.find("/settings/admin").unwrap();
//! assert_eq!(r.value, &4);
//! assert_eq!(r.params(), vec![("page", "admin")]);
//!
//! let r = tree.find("/viz-rs").unwrap();
//! assert_eq!(r.value, &5);
//! assert_eq!(r.params(), vec![("user", "viz-rs")]);
//!
//! let r = tree.find("/viz-rs/path-tree").unwrap();
//! assert_eq!(r.value, &6);
//! assert_eq!(r.params(), vec![("user", "viz-rs"), ("repo", "path-tree")]);
//!
//! let r = tree.find("/rust-lang/rust-analyzer/releases/download/2022-09-12/rust-analyzer-aarch64-apple-darwin.gz").unwrap();
//! assert_eq!(r.value, &8);
//! assert_eq!(
//!     r.params(),
//!     vec![
//!         ("org", "rust-lang"),
//!         ("repo", "rust-analyzer"),
//!         ("tag", "2022-09-12"),
//!         ("filename", "rust-analyzer-aarch64-apple-darwin"),
//!         ("ext", "gz")
//!     ]
//! );
//!
//! let r = tree.find("/rust-lang/rust-analyzer/tags/2022-09-12").unwrap();
//! assert_eq!(r.value, &9);
//! assert_eq!(
//!     r.params(),
//!     vec![
//!         ("org", "rust-lang"),
//!         ("repo", "rust-analyzer"),
//!         ("day", "2022"),
//!         ("month", "09"),
//!         ("year", "12")
//!     ]
//! );
//!
//! let r = tree.find("/rust-lang/rust-analyzer/actions/ci:bench").unwrap();
//! assert_eq!(r.value, &10);
//! assert_eq!(
//!     r.params(),
//!     vec![
//!         ("org", "rust-lang"),
//!         ("repo", "rust-analyzer"),
//!         ("name", "ci"),
//!         ("verb", "bench"),
//!     ]
//! );
//!
//! let r = tree.find("/rust-lang/rust-analyzer/stargazers").unwrap();
//! assert_eq!(r.value, &11);
//! assert_eq!(r.params(), vec![("org", "rust-lang"), ("repo", "rust-analyzer"), ("page", "stargazers")]);
//!
//! let r = tree.find("/rust-lang/rust-analyzer/stargazers/404").unwrap();
//! assert_eq!(r.value, &12);
//! assert_eq!(r.params(), vec![("org", "rust-lang"), ("repo", "rust-analyzer"), ("*1", "stargazers/404")]);
//!
//! let r = tree.find("/public/js/main.js").unwrap();
//! assert_eq!(r.value, &7);
//! assert_eq!(r.params(), vec![("any", "js/main.js")]);
//!
//! let r = tree.find("/api/v1").unwrap();
//! assert_eq!(r.value, &13);
//! assert_eq!(r.params(), vec![("+1", "v1")]);
//! ```

#![forbid(unsafe_code)]
#![warn(rust_2018_idioms, unreachable_pub)]

use std::{iter, marker::PhantomData, slice, str::from_utf8};

use smallvec::SmallVec;

mod node;
mod parser;

pub use node::{Node, NodeKind};
pub use parser::{Kind, Parser, Piece, Position};

#[derive(Debug)]
pub struct PathTree<T> {
    id: usize,
    routes: Vec<(T, Vec<Piece>)>,
    pub node: Node<usize>,
}

impl<T> Default for PathTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PathTree<T> {
    pub fn new() -> Self {
        Self {
            id: 0,
            routes: Vec::new(),
            node: Node::new(NodeKind::String("".as_bytes().to_vec()), None),
        }
    }

    pub fn insert(&mut self, path: &str, value: T) -> usize {
        if path.is_empty() {
            return self.id;
        }

        let mut node = &mut self.node;
        let pieces = Parser::new(path).collect::<Vec<_>>();

        for piece in &pieces {
            match piece {
                Piece::String(s) => {
                    node = node.insert_bytes(&s[..]);
                }
                Piece::Parameter(_, k) => {
                    node = node.insert_parameter(*k);
                }
            }
        }

        self.routes.push((value, pieces));
        if let Some(id) = node.value {
            id
        } else {
            let id = self.id;
            node.value = Some(id);
            self.id += 1;
            id
        }
    }

    pub fn find<'b>(&self, path: &'b str) -> Option<Path<'_, 'b, T>> {
        let bytes = path.as_bytes();
        self.node.find(bytes).and_then(|(id, ranges)| {
            self.get_route(*id).map(|(value, pieces)| {
                Path {
                    id,
                    value,
                    pieces,
                    // opt!
                    raws: ranges
                        .into_iter()
                        .map(|r| from_utf8(&bytes[r]).unwrap())
                        .rev()
                        .collect(),
                }
            })
        })
    }

    #[inline]
    pub fn get_route(&self, index: usize) -> Option<&(T, Vec<Piece>)> {
        self.routes.get(index)
    }

    /// Generates URL
    pub fn url_for(&self, index: usize, params: &[&str]) -> Option<String> {
        self.get_route(index).map(|(_, pieces)| {
            let mut bytes = Vec::new();
            let mut iter = params.iter();

            pieces.iter().for_each(|piece| match piece {
                Piece::String(s) => {
                    bytes.extend_from_slice(s);
                }
                Piece::Parameter(_, _) => {
                    if let Some(s) = iter.next() {
                        bytes.extend_from_slice(s.as_bytes());
                    }
                }
            });

            String::from_utf8_lossy(&bytes).to_string()
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Path<'a, 'b, T> {
    pub id: &'a usize,
    pub value: &'a T,
    pub pieces: &'a [Piece],
    pub raws: SmallVec<[&'b str; 4]>,
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
                    Position::Index(_, _) => {
                        if *k == Kind::OneOrMore {
                            bytes.push(b'+');
                        } else if *k == Kind::ZeroOrMore || *k == Kind::ZeroOrMoreSegment {
                            bytes.push(b'*');
                        }
                    }
                    Position::Named(n) => match k {
                        Kind::Normal | Kind::Optional | Kind::OptionalSegment => {
                            bytes.push(b':');
                            bytes.extend_from_slice(n);
                            if *k == Kind::Optional || *k == Kind::OptionalSegment {
                                bytes.push(b'?');
                            }
                        }
                        Kind::OneOrMore => {
                            bytes.push(b'+');
                            bytes.extend_from_slice(n);
                        }
                        Kind::ZeroOrMore | Kind::ZeroOrMoreSegment => {
                            bytes.push(b'*');
                            bytes.extend_from_slice(n);
                        }
                    },
                },
            }
        }

        String::from_utf8_lossy(&bytes).to_string()
    }

    pub fn params(&self) -> Vec<(&'a str, &'b str)> {
        self.params_iter().collect()
    }

    pub fn params_iter<'p>(&'p self) -> ParamsIter<'p, 'a, 'b, T> {
        #[inline]
        fn piece_filter(piece: &Piece) -> Option<&'_ str> {
            match piece {
                Piece::String(_) => None,
                Piece::Parameter(p, _) => from_utf8(match p {
                    Position::Index(_, n) => n,
                    Position::Named(n) => n,
                })
                .ok(),
            }
        }

        ParamsIter {
            iter: self
                .pieces
                .iter()
                .filter_map(piece_filter as fn(piece: &'a Piece) -> Option<&'a str>)
                .zip(self.raws.iter().copied()),
            _t: PhantomData,
        }
    }
}

type FilterIter<'a> =
    iter::FilterMap<slice::Iter<'a, Piece>, fn(piece: &'a Piece) -> Option<&'a str>>;

pub struct ParamsIter<'p, 'a, 'b, T> {
    iter: iter::Zip<FilterIter<'a>, std::iter::Copied<slice::Iter<'p, &'b str>>>,
    _t: PhantomData<T>,
}

impl<'p, 'a, 'b, T> Iterator for ParamsIter<'p, 'a, 'b, T> {
    type Item = (&'a str, &'b str);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
