use std::fmt;

mod node;
mod parser;

pub use node::{Node, NodeKind};
pub use parser::{Kind, Parser, Piece, Position};

#[derive(Debug)]
pub struct PathTree<'a, T> {
    id: usize,
    node: Node<'a, usize>,
    routes: Vec<(T, Vec<Piece<'a>>)>,
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
                    node = node.insert_bytes(&s);
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

    pub fn find(&self) {}

    pub fn get_route(&self, index: usize) -> Option<&(T, Vec<Piece<'a>>)> {
        self.routes.get(index)
    }
}
