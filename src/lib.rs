//! path-tree is a lightweight high performance HTTP request router for Rust.
//!
//! # Examples
//!
//! ```
//! use path_tree::PathTree;
//!
//! let mut tree = PathTree::new();
//! tree.insert("/", 0);
//!
//! tree.insert("/users/:id", 1)
//!     .insert("/users/:user_id/*", 2)
//!     .insert("/users/:user_id/repos/:id", 3)
//!     .insert("/*any", 4);
//!
//! let r = tree.find("/").unwrap();
//! assert_eq!(r.0, &0);
//! assert_eq!(r.1, vec![]);
//!
//! let r = tree.find("/users/31415926").unwrap();
//! assert_eq!(r.0, &1);
//! assert_eq!(r.1, vec![("id", "31415926")]);
//!
//! let r = tree.find("/users/31415926/settings").unwrap();
//! assert_eq!(r.0, &2);
//! assert_eq!(r.1, vec![("user_id", "31415926"), ("", "settings")]);
//!
//! let r = tree.find("/users/31415926/repos/53589793").unwrap();
//! assert_eq!(r.0, &3);
//! assert_eq!(r.1, vec![("user_id", "31415926"), ("id", "53589793")]);
//!
//! let r = tree.find("/about").unwrap();
//! assert_eq!(r.0, &4);
//! assert_eq!(r.1, vec![("any", "about")]);
//!
//! let r = tree.find("/users/31415926/repos/53589793/branches").unwrap();
//! assert_eq!(r.0, &2);
//! assert_eq!(r.1, vec![("user_id", "31415926"), ("", "repos/53589793/branches")]);
//! ```

#![deny(unsafe_code)]
#![warn(
    nonstandard_style,
    rust_2018_idioms,
    future_incompatible,
    missing_debug_implementations
)]

/// The Kind of a node.
#[derive(Clone, Debug)]
pub enum NodeKind {
    /// A static node with a path
    Static(String),

    /// A named node
    Parameter,

    /// A catch-all node
    CatchAll,
}

/// A node stores kind data params indices and children nodes.
#[derive(Clone, Debug)]
pub struct Node<T> {
    kind: NodeKind,
    data: Option<T>,
    indices: Option<Vec<char>>,
    nodes: Option<Vec<Self>>,
    params: Option<Vec<String>>,
}

impl<T> Default for Node<T> {
    #[inline]
    fn default() -> Self {
        Self::new(NodeKind::Static(String::new()))
    }
}

impl<T> Node<T> {
    /// Creates a new node with a special kind.
    #[inline]
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            data: None,
            nodes: None,
            params: None,
            indices: None,
        }
    }

    fn add_node(&mut self, c: char, kind: NodeKind) -> &mut Self {
        let indices: &mut Vec<char> = self.indices.get_or_insert_with(Vec::new);
        let nodes: &mut Vec<Node<T>> = self.nodes.get_or_insert_with(Vec::new);

        match position(indices, c) {
            Some(i) => match kind {
                NodeKind::Static(ref s) => nodes[i].insert(s),
                _ => &mut nodes[i],
            },
            None => {
                indices.push(c);
                nodes.push(Node::new(kind));
                nodes.last_mut().unwrap()
            }
        }
    }

    /// Adds a child node witch a static path.
    pub fn add_node_static(&mut self, p: &str) -> &mut Self {
        if let Some(c) = p.chars().next() {
            self.add_node(c, NodeKind::Static(p.to_owned()))
        } else {
            self
        }
    }

    /// Adds a child node witch a dynamic path.
    pub fn add_node_dynamic(&mut self, c: char, kind: NodeKind) -> &mut Self {
        self.add_node(c, kind)
    }

    /// Inserts a path into node.
    pub fn insert(&mut self, p: &str) -> &mut Self {
        match self.kind {
            NodeKind::Static(ref mut s) if s.is_empty() => {
                *s += p;
                self
            }
            NodeKind::Static(ref mut s) => {
                let l = loc(s, p);

                // Split node
                if l < s.len() {
                    *s = s[l..].to_owned();
                    let mut node = Node {
                        data: None,
                        params: None,
                        nodes: Some(Vec::new()),
                        indices: s.chars().next().map(|c| vec![c]),
                        kind: NodeKind::Static(String::from(&p[0..l])),
                    };
                    ::std::mem::swap(self, &mut node);
                    self.nodes.as_mut().unwrap().push(node);
                }

                if l == p.len() {
                    self
                } else {
                    self.add_node_static(&p[l..])
                }
            }
            NodeKind::Parameter => self.add_node_static(p),
            NodeKind::CatchAll => self,
        }
    }

    /// Returns a reference to the node corresponding to the path.
    #[inline]
    pub fn find<'a>(&'a self, p: &'a str) -> Option<(&'a Self, Vec<(&'a str, &'a str)>)> {
        self.find_with_capacity(p, 10)
    }

    /// Returns a reference to the node corresponding to the path.
    #[inline]
    pub fn find_with_capacity<'a>(
        &'a self,
        p: &'a str,
        capacity: usize,
    ) -> Option<(&'a Self, Vec<(&'a str, &'a str)>)> {
        let mut params = Vec::with_capacity(capacity);

        self.find_inner(p, &mut params).map(|node| {
            (
                node,
                node.params.as_ref().map_or_else(Vec::new, |node_params| {
                    for (value, (key, _)) in node_params.iter().zip(params.iter_mut()) {
                        *key = value;
                    }

                    params
                }),
            )
        })
    }

    fn find_inner<'a>(
        &'a self,
        mut p: &'a str,
        params: &mut Vec<(&'a str, &'a str)>,
    ) -> Option<&'a Self> {
        match self.kind {
            NodeKind::Static(ref s) => {
                let l = loc(s, p);

                if l == 0 || l < s.len() {
                    None
                } else if l == s.len() && l == p.len() {
                    Some(
                        // Fixed: has only route `/*`
                        // Ended `/` `/*any`
                        if self.data.is_none() && self.indices.is_some() && s.ends_with('/') {
                            &self.nodes.as_ref().unwrap()
                                [position(self.indices.as_ref().unwrap(), '*')?]
                        } else {
                            self
                        },
                    )
                } else {
                    let indices = self.indices.as_ref()?;
                    let nodes = self.nodes.as_ref()?;

                    p = &p[l..];

                    // Static
                    if let Some(i) = position(indices, p.chars().next().unwrap()) {
                        if let Some(n) = nodes[i].find_inner(p, params).as_mut() {
                            return Some(
                                // Ended `/` `/*any`
                                match &n.kind {
                                    NodeKind::Static(s)
                                        if n.data.is_none()
                                            && n.indices.is_some()
                                            && s.ends_with('/') =>
                                    {
                                        &n.nodes.as_ref().unwrap()
                                            [position(n.indices.as_ref().unwrap(), '*')?]
                                    }
                                    _ => n,
                                },
                            );
                        }
                    }

                    // Named Parameter
                    if let Some(i) = position(indices, ':') {
                        if let Some(n) = nodes[i].find_inner(p, params).as_mut() {
                            return Some(n);
                        }
                    }

                    // Catch-All Parameter
                    if let Some(i) = position(indices, '*') {
                        if let Some(n) = nodes[i].find_inner(p, params).as_mut() {
                            return Some(n);
                        }
                    }

                    None
                }
            }
            NodeKind::Parameter => match p.find('/') {
                Some(i) => {
                    let indices = self.indices.as_ref()?;

                    params.push(("", &p[..i]));
                    p = &p[i..];

                    let n = self.nodes.as_ref().unwrap()
                        [position(indices, p.chars().next().unwrap())?]
                    .find_inner(p, params)?;

                    Some(n)
                }
                None if self.params.is_some() => {
                    params.push(("", p));
                    Some(self)
                }
                None => None,
            },
            NodeKind::CatchAll => {
                params.push(("", p));
                Some(self)
            }
        }
    }
}

/// A path tree.
#[derive(Clone, Debug)]
pub struct PathTree<T> {
    root: Node<T>,
    params: usize,
}

impl<T> Default for PathTree<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PathTree<T> {
    /// Creates a new tree with a root node.
    ///
    /// The root node is a static node with `/`.
    #[inline]
    pub fn new() -> Self {
        Self {
            root: Node::new(NodeKind::Static("/".to_owned())),
            params: 0,
        }
    }

    /// Inserts a path and data into tree.
    pub fn insert(&mut self, mut path: &str, data: T) -> &mut Self {
        let mut next = true;
        let mut node = &mut self.root;
        let mut params: Option<Vec<String>> = None;

        let mut most = 0;

        path = path.trim_start_matches('/');

        if path.is_empty() {
            node.data.replace(data);
            return self;
        }

        while next {
            match path.chars().position(has_colon_or_star) {
                Some(i) => {
                    let kind: NodeKind;
                    let mut prefix = &path[..i];
                    let mut suffix = &path[i..];

                    if !prefix.is_empty() {
                        node = node.add_node_static(prefix);
                    }

                    prefix = &suffix[..1];
                    suffix = &suffix[1..];

                    let c = prefix.chars().next().unwrap();
                    if c == ':' {
                        match suffix.chars().position(has_star_or_slash) {
                            Some(i) => {
                                path = &suffix[i..];
                                suffix = &suffix[..i];
                            }
                            None => {
                                next = false;
                            }
                        }
                        kind = NodeKind::Parameter;
                    } else {
                        next = false;
                        kind = NodeKind::CatchAll;
                    }
                    most += 1;
                    params.get_or_insert_with(Vec::new).push(suffix.to_owned());
                    node = node.add_node_dynamic(c, kind);
                }
                None => {
                    next = false;
                    node = node.add_node_static(path);
                }
            }
        }

        if most > self.params {
            self.params = most;
        }

        node.data = Some(data);
        node.params = params;

        self
    }

    /// Returns a reference to the node data and params corresponding to the path.
    pub fn find<'a>(&'a self, path: &'a str) -> Option<(&'a T, Vec<(&'a str, &'a str)>)> {
        self.root
            .find_with_capacity(path, self.params)
            .and_then(|(node, params)| node.data.as_ref().map(|data| (data, params)))
    }
}

#[inline]
const fn has_colon_or_star(c: char) -> bool {
    (c == ':') | (c == '*')
}

#[inline]
const fn has_star_or_slash(c: char) -> bool {
    (c == '*') | (c == '/')
}

#[inline]
fn position(p: &[char], c: char) -> Option<usize> {
    p.iter().position(|x| *x == c)
}

#[inline]
fn loc(s: &str, p: &str) -> usize {
    s.chars()
        .zip(p.chars())
        .take_while(|(a, b)| a == b)
        .map(|(c, _)| c.len_utf8())
        .sum()
}
