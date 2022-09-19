use smallvec::SmallVec;
use std::fmt::{self, Write};

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
            match nodes.binary_search_by(|node| match node.kind {
                NodeKind::String(s) => s[0].cmp(&bytes[0]),
                _ => unreachable!(),
            }) {
                Ok(i) => nodes[i].insert_bytes(bytes),
                Err(i) => {
                    nodes.insert(i, Node::new(NodeKind::String(bytes), None));
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
        ranges: &mut SmallVec<[usize; 6]>,
    ) -> Option<&T> {
        let mut m = bytes.len();
        match self.kind {
            NodeKind::String(s) => {
                let n = s.len();
                // starts with prefix
                if m >= n && &bytes[..n] == s {
                    if m == n {
                        if let Some(id) = &self.value {
                            return Some(id);
                        }
                    }

                    bytes = &bytes[n..];
                    start += n;
                    m -= n;

                    // static
                    if m > 0 {
                        if let Some(nodes) = &self.nodes0 {
                            if let Ok(i) = nodes.binary_search_by(|node| match node.kind {
                                NodeKind::String(s) => s[0].cmp(&bytes[0]),
                                _ => unreachable!(),
                            }) {
                                if let Some(id) = nodes[i]._find(start, bytes, ranges) {
                                    return Some(id);
                                }
                            }
                        }
                    }

                    // parameter
                    if let Some(nodes) = &self.nodes1 {
                        for node in nodes {
                            if let Some(id) = node._find(start, bytes, ranges) {
                                return Some(id);
                            }
                        }
                    }
                } else if s == b"/" {
                    if let Some(nodes) = &self.nodes1 {
                        let mut iter = nodes.iter();

                        if let Some(node) = iter
                            .find(|node| node.kind == NodeKind::Parameter(Kind::OptionalSegment))
                        {
                            if let Some(id) = node._find(start, bytes, ranges) {
                                return Some(id);
                            }
                        }

                        if let Some(node) = iter
                            .find(|node| node.kind == NodeKind::Parameter(Kind::ZeroOrMoreSegment))
                        {
                            if let Some(id) = node._find(start, bytes, ranges) {
                                return Some(id);
                            }
                        }
                    }
                }
            }
            NodeKind::Parameter(k) => match k {
                Kind::Normal | Kind::Optional | Kind::OptionalSegment => {
                    if m > 0 {
                        // static
                        if let Some(nodes) = &self.nodes0 {
                            for node in nodes {
                                if let NodeKind::String(s) = node.kind {
                                    if s[0] == b'/' {
                                        if let Some(n) = bytes.iter().position(|b| *b == b'/') {
                                            if let Some(id) =
                                                node._find(start + n, &bytes[n..], ranges)
                                            {
                                                ranges.push(start);
                                                ranges.push(start + n);
                                                return Some(id);
                                            }
                                        }
                                    } else if let Some(n) = memchr::memmem::find(bytes, s) {
                                        if let Some(id) = node._find(start + n, &bytes[n..], ranges)
                                        {
                                            ranges.push(start);
                                            ranges.push(start + n);
                                            return Some(id);
                                        }
                                    }
                                }
                            }
                        }

                        // parameter => `:a:b:c`
                        if let Some(nodes) = &self.nodes1 {
                            for node in nodes {
                                if let Some(id) = node._find(start + 1, &bytes[1..], ranges) {
                                    ranges.push(start);
                                    ranges.push(start + 1);
                                    return Some(id);
                                }
                            }
                        }
                    }

                    // parameter => `:a:b?:c?`
                    if k == Kind::Optional || k == Kind::OptionalSegment {
                        if let Some(nodes) = &self.nodes1 {
                            for node in nodes {
                                if let Some(id) = node._find(start, bytes, ranges) {
                                    ranges.push(start);
                                    ranges.push(start + m);
                                    return Some(id);
                                }
                            }
                        }
                    }

                    if !bytes.contains(&b'/') {
                        if let Some(id) = &self.value {
                            ranges.push(start);
                            ranges.push(start + m);
                            return Some(id);
                        }
                    }

                    if k == Kind::OptionalSegment {
                        if let Some(nodes) = &self.nodes0 {
                            if let Ok(i) = nodes.binary_search_by(|node| match node.kind {
                                NodeKind::String(s) => s[0].cmp(&b'/'),
                                _ => unreachable!(),
                            }) {
                                if let Some(id) = nodes[i]._find(start, bytes, ranges) {
                                    ranges.push(start);
                                    ranges.push(start + m);
                                    return Some(id);
                                }
                            }
                        }
                    }
                }
                Kind::OneOrMore | Kind::ZeroOrMore | Kind::ZeroOrMoreSegment => {
                    let flag = if k == Kind::OneOrMore { m > 0 } else { true };
                    if flag {
                        if self.nodes0.is_none() && self.nodes1.is_none() {
                            if let Some(id) = &self.value {
                                ranges.push(start);
                                ranges.push(start + m);
                                return Some(id);
                            }
                        }

                        // static
                        if let Some(nodes) = &self.nodes0 {
                            for node in nodes {
                                if let NodeKind::String(s) = &node.kind {
                                    if m > s.len() {
                                        let iter = memchr::memmem::find_iter(bytes, s);
                                        for n in iter {
                                            if let Some(id) =
                                                node._find(start + n, &bytes[n..], ranges)
                                            {
                                                ranges.push(start);
                                                ranges.push(start + n);
                                                return Some(id);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if k == Kind::ZeroOrMoreSegment {
                        if let Some(nodes) = &self.nodes0 {
                            if let Ok(i) = nodes.binary_search_by(|node| match node.kind {
                                NodeKind::String(s) => s[0].cmp(&b'/'),
                                _ => unreachable!(),
                            }) {
                                if let Some(id) = nodes[i]._find(start, bytes, ranges) {
                                    ranges.push(start);
                                    ranges.push(start + m);
                                    return Some(id);
                                }
                            }
                        }
                    }
                }
            },
        }
        None
    }

    pub fn find<'b>(&self, bytes: &'b [u8]) -> Option<(&T, SmallVec<[usize; 6]>)> {
        let mut ranges = SmallVec::<[usize; 6]>::new();
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
                    f.write_str(&String::from_utf8_lossy(path).replace(':', "\\:"))?;
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
