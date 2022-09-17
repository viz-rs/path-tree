use std::{
    cmp::Ordering,
    fmt::{self, Write},
};

use crate::Kind;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum NodeKind<'a> {
    String(&'a [u8]),
    Parameter(Kind),
}

pub struct Node<'a, T> {
    pub value: Option<T>,
    pub kind: NodeKind<'a>,
    /// Stores  string node
    pub nodes0: Option<Vec<Node<'a, T>>>,
    /// Stores parameter node
    pub nodes1: Option<Vec<Node<'a, T>>>,
}

impl<'a, T: fmt::Debug> Node<'a, T> {
    pub fn new(kind: NodeKind<'a>, value: Option<T>) -> Self {
        Self {
            kind,
            value,
            nodes0: None,
            nodes1: None,
        }
    }

    pub fn insert_bytes(&mut self, mut bytes: &'a [u8]) -> &mut Self {
        let (cursor, diff) = match &mut self.kind {
            NodeKind::String(p) => {
                let cursor = p
                    .iter()
                    .zip(bytes.iter())
                    .take_while(|(a, b)| a == b)
                    .count();

                let pl = p.len();

                // split node
                if cursor < pl {
                    let (prefix, suffix) = p.split_at(cursor);
                    let mut node = Node {
                        kind: NodeKind::String(prefix),
                        value: None,
                        nodes0: None,
                        nodes1: None,
                    };
                    *p = suffix;
                    ::std::mem::swap(self, &mut node);
                    self.nodes0
                        .get_or_insert_with(|| Vec::with_capacity(1))
                        .push(node);
                }
                (cursor, cursor != bytes.len())
            }
            NodeKind::Parameter(_) => (0, true),
        };

        // insert node
        if diff {
            let nodes = self.nodes0.get_or_insert_with(Vec::new);
            bytes = &bytes[cursor..];

            match nodes.binary_search_by(|node| match &node.kind {
                NodeKind::String(s) => s[0].cmp(&bytes[0]),
                _ => Ordering::Greater,
            }) {
                Ok(i) => nodes[i].insert_bytes(bytes),
                Err(i) => {
                    nodes.insert(
                        i,
                        Node {
                            kind: NodeKind::String(bytes),
                            value: None,
                            nodes0: None,
                            nodes1: None,
                        },
                    );
                    &mut nodes[i]
                }
            }
        } else {
            self
        }
    }

    pub fn insert_parameter(&mut self, kind: Kind) -> &mut Self {
        let nodes = self.nodes1.get_or_insert_with(Vec::new);
        let i = nodes
            .binary_search_by(|node| match &node.kind {
                NodeKind::Parameter(pk) => pk.cmp(&kind),
                _ => Ordering::Less,
            })
            .unwrap_or_else(|i| {
                nodes.insert(
                    i,
                    Node {
                        kind: NodeKind::Parameter(kind),
                        value: None,
                        nodes0: None,
                        nodes1: None,
                    },
                );
                i
            });

        &mut nodes[i]
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for Node<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const EDGE: &str = "├──";
        const LINE: &str = "│  ";
        const CORNER: &str = "└──";
        const BLANK: &str = "   ";

        fn print_tree<'a, T: fmt::Debug>(
            node: &Node<'a, T>,
            f: &mut fmt::Formatter<'_>,
            root: bool,
            pad: &str,
        ) -> fmt::Result {
            let space = if root {
                f.write_char('\n')?;
                ""
            } else {
                f.write_char(' ')?;
                " "
            };
            match &node.kind {
                NodeKind::String(path) => {
                    f.write_str(&String::from_utf8_lossy(path))?;
                }
                NodeKind::Parameter(kind) => match kind {
                    Kind::Normal => f.write_char(':')?,
                    Kind::Optional => f.write_char('?')?,
                    Kind::OptionalSegment => f.write_str("??")?,
                    Kind::OneOrMore => f.write_char('+')?,
                    Kind::ZeroOrMore => f.write_char('*')?,
                    Kind::ZeroOrMoreSegment => f.write_str("**")?,
                },
            }
            if let Some(value) = &node.value {
                f.write_str(" •")?;
                value.fmt(f)?;
            }
            f.write_char('\n')?;

            let check = node.nodes1.is_none();

            // nodes0
            if let Some(nodes) = &node.nodes0 {
                for (index, node) in nodes.iter().enumerate() {
                    let (left, right) = if check && index == nodes.len() - 1 {
                        (BLANK, CORNER)
                    } else {
                        (LINE, EDGE)
                    };
                    f.write_str(pad)?;
                    f.write_str(space)?;
                    f.write_str(right)?;
                    print_tree(node, f, false, &format!("{}{}{}", pad, space, left))?;
                }
            }

            // nodes1
            if let Some(nodes) = &node.nodes1 {
                for (index, node) in nodes.iter().enumerate() {
                    let (left, right) = if index == nodes.len() - 1 {
                        (BLANK, CORNER)
                    } else {
                        (LINE, EDGE)
                    };
                    f.write_str(pad)?;
                    f.write_str(space)?;
                    f.write_str(right)?;
                    print_tree(node, f, false, &format!("{}{}{}", pad, space, left))?;
                }
            }

            Ok(())
        }

        print_tree(self, f, true, "")
    }
}
