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

impl<'a, T: fmt::Debug> PathTree<'a, T> {
    pub fn new(root: &'a str) -> Self {
        Self {
            id: 0,
            routes: Vec::new(),
            node: Node::new(NodeKind::String(root.as_bytes()), None),
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

    pub fn find<'b>(
        &'b self,
        path: &'b str,
    ) -> Option<(&T, &Vec<Piece<'a>>, SmallVec<[&'b str; 4]>)> {
        let bytes = path.as_bytes();
        self.node.find(bytes).and_then(|(id, ranges)| {
            self.get_route(*id).map(|(t, p)| {
                (
                    t,
                    p,
                    // opt!
                    ranges
                        .chunks(2)
                        .map(|r| from_utf8(&bytes[r[0]..r[1]]).unwrap())
                        .rev()
                        .collect(),
                )
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
