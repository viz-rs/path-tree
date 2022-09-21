use smallvec::SmallVec;
use std::{
    cmp::Ordering,
    fmt::{self, Write},
};

use crate::Kind;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeKind<'a> {
    String(&'a [u8]),
    Parameter(Kind),
}

pub struct Node<'a, T> {
    pub value: Option<T>,
    pub kind: NodeKind<'a>,
    /// Stores string node
    pub nodes0: Option<Vec<Self>>,
    /// Stores parameter node
    pub nodes1: Option<Vec<Self>>,
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
                if p.is_empty() {
                    *p = bytes;
                    return self;
                }

                let cursor = p
                    .iter()
                    .zip(bytes.iter())
                    .take_while(|(a, b)| a == b)
                    .count();

                (
                    cursor,
                    if cursor == 0 {
                        true
                    } else {
                        // split node
                        if cursor < p.len() {
                            let (prefix, suffix) = p.split_at(cursor);
                            let mut node = Node::new(NodeKind::String(prefix), None);
                            *p = suffix;
                            ::std::mem::swap(self, &mut node);
                            self.nodes0.get_or_insert_with(Vec::new).push(node);
                        }
                        cursor != bytes.len()
                    },
                )
            }
            NodeKind::Parameter(_) => (0, true),
        };

        // insert node
        if diff {
            bytes = &bytes[cursor..];
            let nodes = self.nodes0.get_or_insert_with(Vec::new);
            return match nodes.binary_search_by(|node| match node.kind {
                NodeKind::String(s) => {
                    // s[0].cmp(&bytes[0])
                    // opt!
                    // lets `/` at end
                    compare(s[0], bytes[0])
                }
                _ => unreachable!(),
            }) {
                Ok(i) => nodes[i].insert_bytes(bytes),
                Err(i) => {
                    nodes.insert(i, Node::new(NodeKind::String(bytes), None));
                    &mut nodes[i]
                }
            };
        }

        self
    }

    pub fn insert_parameter(&mut self, kind: Kind) -> &mut Self {
        let nodes = self.nodes1.get_or_insert_with(Vec::new);
        let i = nodes
            .binary_search_by(|node| match node.kind {
                NodeKind::Parameter(pk) => pk.cmp(&kind),
                _ => unreachable!(),
            })
            .unwrap_or_else(|i| {
                nodes.insert(i, Node::new(NodeKind::Parameter(kind), None));
                i
            });
        &mut nodes[i]
    }

    #[inline]
    fn _find(
        &self,
        mut start: usize,
        mut bytes: &[u8],
        ranges: &mut SmallVec<[usize; 8]>,
    ) -> Option<&T> {
        let mut m = bytes.len();
        match self.kind {
            NodeKind::String(s) => {
                let n = s.len();
                let mut flag = m >= n;

                // opt!
                if flag {
                    if n == 1 {
                        flag = s[0] == bytes[0];
                    } else {
                        flag = s == &bytes[..n];
                    }
                }

                // starts with prefix
                if flag {
                    m -= n;
                    start += n;
                    bytes = &bytes[n..];

                    if m == 0 {
                        if let Some(id) = &self.value {
                            return Some(id);
                        }
                    } else {
                        // static
                        if let Some(id) = self.nodes0.as_ref().and_then(|nodes| {
                            nodes
                                .binary_search_by(|node| match node.kind {
                                    NodeKind::String(s) => {
                                        // s[0].cmp(&bytes[0])
                                        // opt!
                                        // lets `/` at end
                                        compare(s[0], bytes[0])
                                    }
                                    _ => unreachable!(),
                                })
                                .ok()
                                .and_then(|i| nodes[i]._find(start, bytes, ranges))
                        }) {
                            return Some(id);
                        }
                    }

                    // parameter
                    if let Some(id) = self.nodes1.as_ref().and_then(|nodes| {
                        let b = m > 0;
                        nodes
                            .iter()
                            .filter(|node| match node.kind {
                                NodeKind::Parameter(pk)
                                    if pk == Kind::Normal || pk == Kind::OneOrMore =>
                                {
                                    b
                                }
                                _ => true,
                            })
                            .find_map(|node| node._find(start, bytes, ranges))
                    }) {
                        return Some(id);
                    }
                } else if n == 1 && s[0] == b'/' {
                    if let Some(id) = self.nodes1.as_ref().and_then(|nodes| {
                        nodes
                            .iter()
                            .filter(|node| {
                                matches!(node.kind,
                                    NodeKind::Parameter(pk)
                                        if pk == Kind::OptionalSegment
                                            || pk == Kind::ZeroOrMoreSegment
                                )
                            })
                            .find_map(|node| node._find(start, bytes, ranges))
                    }) {
                        return Some(id);
                    }
                }
            }
            NodeKind::Parameter(k) => match k {
                // Kind::Normal => {
                //     if m == 0 {
                //         return None;
                //     }
                //
                //     // slash node
                //     let mut slash = None;
                //     // slash position at path
                //     let sp = bytes.iter().position(|b| *b == b'/');
                //
                //     // static
                //     if let Some(id) = self.nodes0.as_ref().and_then(|nodes| {
                //         let tmp = &bytes[0..sp.unwrap_or(m)];
                //         nodes.iter().find_map(|node| match node.kind {
                //             NodeKind::String(s) => {
                //                 if s[0] == b'/' {
                //                     slash.replace(node);
                //                     return None;
                //                 }
                //
                //                 tmp.iter().position(|b| s[0] == *b).and_then(|n| {
                //                     node._find(start + n, &bytes[n..], ranges).map(|id| {
                //                         ranges.push(start);
                //                         ranges.push(start + n);
                //                         id
                //                     })
                //                 })
                //             }
                //             _ => unreachable!(),
                //         })
                //     }) {
                //         return Some(id);
                //     }
                //
                //     // parameter => `:a:b:c`
                //     if let Some(id) = self.nodes1.as_ref().and_then(|nodes| {
                //         let b = m - 1 > 0;
                //         nodes
                //             .iter()
                //             .filter(|node| match node.kind {
                //                 NodeKind::Parameter(pk)
                //                     if pk == Kind::Normal || pk == Kind::OneOrMore =>
                //                 {
                //                     b
                //                 }
                //                 _ => true,
                //             })
                //             .find_map(|node| node._find(start + 1, &bytes[1..], ranges))
                //     }) {
                //         ranges.push(start);
                //         ranges.push(start + 1);
                //         return Some(id);
                //     }
                //
                //     if let Some(n) = sp {
                //         return slash
                //             .and_then(|node| node._find(start + n, &bytes[n..], ranges))
                //             .map(|id| {
                //                 ranges.push(start);
                //                 ranges.push(start + n);
                //                 id
                //             });
                //     } else {
                //         return self.value.as_ref().map(|id| {
                //             ranges.push(start);
                //             ranges.push(start + m);
                //             id
                //         });
                //     }
                // }
                Kind::Normal | Kind::Optional | Kind::OptionalSegment => {
                    if m == 0 {
                        if k == Kind::Normal {
                            return None;
                        } else {
                            // last
                            if self.nodes0.is_none() && self.nodes1.is_none() {
                                return self.value.as_ref().map(|id| {
                                    ranges.push(start);
                                    ranges.push(start);
                                    id
                                });
                            }
                        }
                    } else {
                        // static
                        if let Some(id) = self.nodes0.as_ref().and_then(|nodes| {
                            nodes.iter().find_map(|node| match node.kind {
                                NodeKind::String(s) => {
                                    bytes.iter().position(|b| s[0] == *b).and_then(|n| {
                                        node._find(start + n, &bytes[n..], ranges).map(|id| {
                                            ranges.push(start);
                                            ranges.push(start + n);
                                            id
                                        })
                                    })
                                }
                                _ => unreachable!(),
                            })
                        }) {
                            return Some(id);
                        }

                        // parameter => `:a:b:c`
                        if let Some(id) = self.nodes1.as_ref().and_then(|nodes| {
                            let b = m - 1 > 0;
                            nodes
                                .iter()
                                .filter(|node| match node.kind {
                                    NodeKind::Parameter(pk)
                                        if pk == Kind::Normal || pk == Kind::OneOrMore =>
                                    {
                                        b
                                    }
                                    _ => true,
                                })
                                .find_map(|node| node._find(start + 1, &bytes[1..], ranges))
                        }) {
                            ranges.push(start);
                            ranges.push(start + 1);
                            return Some(id);
                        }
                    }

                    // parameter => `:a:b?:c?`
                    if k == Kind::Optional || k == Kind::OptionalSegment {
                        if let Some(id) = self.nodes1.as_ref().and_then(|nodes| {
                            let b = m > 0;
                            nodes
                                .iter()
                                .filter(|node| match node.kind {
                                    NodeKind::Parameter(pk)
                                        if pk == Kind::Normal || pk == Kind::OneOrMore =>
                                    {
                                        b
                                    }
                                    _ => true,
                                })
                                .find_map(|node| node._find(start, bytes, ranges))
                        }) {
                            // param should be empty
                            ranges.push(start + m);
                            ranges.push(start + m);
                            return Some(id);
                        }
                    }

                    if let Some(n) = bytes.iter().position(|b| *b == b'/') {
                        bytes = &bytes[n..];
                    } else {
                        if let Some(id) = &self.value {
                            ranges.push(start);
                            ranges.push(start + m);
                            return Some(id);
                        }
                        bytes = &bytes[m..];
                    }

                    if k == Kind::OptionalSegment {
                        if let Some(id) = self.nodes0.as_ref().and_then(|nodes| {
                            nodes
                                .last()
                                .filter(|node| match node.kind {
                                    NodeKind::String(s) => s[0] == b'/',
                                    _ => unreachable!(),
                                })
                                .and_then(|node| node._find(start, bytes, ranges))
                        }) {
                            ranges.push(start);
                            ranges.push(start + m);
                            return Some(id);
                        }
                    }
                }
                Kind::OneOrMore | Kind::ZeroOrMore | Kind::ZeroOrMoreSegment => {
                    let is_one_or_more = k == Kind::OneOrMore;
                    if m == 0 {
                        if is_one_or_more {
                            return None;
                        } else {
                            // last
                            if self.nodes0.is_none() && self.nodes1.is_none() {
                                return self.value.as_ref().map(|id| {
                                    ranges.push(start);
                                    ranges.push(start);
                                    id
                                });
                            }
                        }
                    } else {
                        if self.nodes0.is_none() && self.nodes1.is_none() {
                            if let Some(id) = &self.value {
                                ranges.push(start);
                                ranges.push(start + m);
                                return Some(id);
                            }
                        }

                        // static
                        if let Some(id) = self.nodes0.as_ref().and_then(|nodes| {
                            nodes.iter().find_map(|node| {
                                if let NodeKind::String(s) = node.kind {
                                    let right_length = if is_one_or_more {
                                        m > s.len()
                                    } else {
                                        m >= s.len()
                                    };
                                    if right_length {
                                        return memchr::memmem::find_iter(bytes, s)
                                            .into_iter()
                                            .find_map(|n| {
                                                node._find(start + n, &bytes[n..], ranges).map(
                                                    |id| {
                                                        ranges.push(start);
                                                        ranges.push(start + n);
                                                        id
                                                    },
                                                )
                                            });
                                    }
                                }
                                None
                            })
                        }) {
                            return Some(id);
                        }
                    }

                    if k == Kind::ZeroOrMoreSegment {
                        if let Some(id) = self.nodes0.as_ref().and_then(|nodes| {
                            nodes
                                .last()
                                .filter(|node| match node.kind {
                                    NodeKind::String(s) => s[0] == b'/',
                                    _ => unreachable!(),
                                })
                                .and_then(|node| node._find(start, bytes, ranges))
                        }) {
                            // param should be empty
                            ranges.push(start + m);
                            ranges.push(start + m);
                            return Some(id);
                        }
                    }
                }
            },
        }
        None
    }

    pub fn find<'b>(&self, bytes: &'b [u8]) -> Option<(&T, SmallVec<[usize; 8]>)> {
        let mut ranges = SmallVec::<[usize; 8]>::new(); // opt!
        return self._find(0, bytes, &mut ranges).map(|t| (t, ranges));
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for Node<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const EDGE: &str = "├──";
        const LINE: &str = "│  ";
        const CORNER: &str = "└──";
        const BLANK: &str = "   ";

        fn print_nodes<'a, T: fmt::Debug>(
            f: &mut fmt::Formatter<'_>,
            nodes: &Vec<Node<'a, T>>,
            check: bool,
            pad: &str,
            space: &str,
        ) -> fmt::Result {
            for (index, node) in nodes.iter().enumerate() {
                let (left, right) = if check && index == nodes.len() - 1 {
                    (BLANK, CORNER)
                } else {
                    (LINE, EDGE)
                };
                f.write_str(pad)?;
                f.write_str(space)?;
                f.write_str(right)?;
                print_tree(f, node, false, &format!("{}{}{}", pad, space, left))?;
            }
            Ok(())
        }

        fn print_tree<'a, T: fmt::Debug>(
            f: &mut fmt::Formatter<'_>,
            node: &Node<'a, T>,
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
                    f.write_str(
                        &String::from_utf8_lossy(path)
                            .replace(':', "\\:")
                            .replace('?', "\\?")
                            .replace('+', "\\+"),
                    )?;
                }
                NodeKind::Parameter(kind) => {
                    let c = match kind {
                        Kind::Normal => ':',
                        Kind::Optional => '?',
                        Kind::OptionalSegment => {
                            f.write_char('?')?;
                            '?'
                        }
                        Kind::OneOrMore => '+',
                        Kind::ZeroOrMore => '*',
                        Kind::ZeroOrMoreSegment => {
                            f.write_char('*')?;
                            '*'
                        }
                    };
                    f.write_char(c)?;
                }
            }
            if let Some(value) = &node.value {
                f.write_str(" •")?;
                value.fmt(f)?;
            }
            f.write_char('\n')?;

            // nodes0
            if let Some(nodes) = &node.nodes0 {
                print_nodes(f, nodes, node.nodes1.is_none(), pad, space)?;
            }

            // nodes1
            if let Some(nodes) = &node.nodes1 {
                print_nodes(f, nodes, true, pad, space)?;
            }

            Ok(())
        }

        print_tree(f, self, true, "")
    }
}

#[inline]
fn compare(a: u8, b: u8) -> Ordering {
    if a == b {
        Ordering::Equal
    } else if a == b'/' {
        Ordering::Greater
    } else if b == b'/' {
        Ordering::Less
    } else {
        a.cmp(&b)
    }
}
