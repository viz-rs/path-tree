use std::mem;

#[derive(Clone, Debug)]
pub enum NodeKind {
    Static(String),
    Parameter,
    CatchAll,
}

#[derive(Clone, Debug)]
pub struct Node<T> {
    kind: NodeKind,
    data: Option<T>,
    indices: Option<String>,
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
    #[inline]
    pub fn new(kind: NodeKind) -> Self {
        Self {
            kind,
            data: None,
            params: None,
            indices: None,
            nodes: None,
        }
    }

    #[inline]
    fn add_node(&mut self, c: char, kind: NodeKind) -> &mut Self {
        let indices: &mut String = self.indices.get_or_insert_with(String::new);
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

    #[inline]
    pub fn add_node_static(&mut self, p: &str) -> &mut Self {
        self.add_node(p.chars().next().unwrap(), NodeKind::Static(p.to_owned()))
    }

    #[inline]
    pub fn add_node_dynamic(&mut self, c: char, kind: NodeKind) -> &mut Self {
        self.add_node(c, kind)
    }

    #[inline]
    pub fn insert(&mut self, p: &str) -> &mut Self {
        match self.kind {
            NodeKind::Static(ref mut s) if s.len() == 0 => {
                *s += p;
                self
            }
            NodeKind::Static(ref mut s) => {
                let np = loc(s, p);
                let l = np.len();

                // Split node
                if l < s.len() {
                    *s = s[l..].to_owned();
                    let mut node = Node {
                        data: None,
                        params: None,
                        nodes: Some(Vec::new()),
                        indices: Some(s.chars().next().unwrap().to_string()),
                        kind: NodeKind::Static(np.to_owned()),
                    };
                    mem::swap(self, &mut node);
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

    #[inline]
    pub fn find<'a>(&self, mut p: &'a str) -> Option<(&Self, Vec<&'a str>)> {
        let mut params = Vec::new();

        match self.kind {
            NodeKind::Static(ref s) => {
                let np = loc(s, p);
                let l = np.len();

                if l == 0 {
                    None
                } else if l < s.len() {
                    None
                } else if l == s.len() && l == p.len() {
                    Some((
                        // Fixed: has only route `/*`
                        // Ended `/` `/*any`
                        if self.data.is_none()
                            && self.indices.is_some()
                            && '/' == s.chars().last().unwrap()
                        {
                            &self.nodes.as_ref().unwrap()
                                [position(self.indices.as_ref().unwrap(), '*')?]
                        } else {
                            self
                        },
                        params,
                    ))
                } else {
                    let indices = self.indices.as_ref()?;
                    let nodes = self.nodes.as_ref().unwrap();

                    p = &p[l..];

                    // Static
                    if let Some(i) = position(indices, p.chars().next().unwrap()) {
                        if let Some((n, ps)) = nodes[i].find(p).as_mut() {
                            params.append(ps);

                            return Some((
                                // Ended `/` `/*any`
                                match &n.kind {
                                    NodeKind::Static(s)
                                        if n.data.is_none()
                                            && n.indices.is_some()
                                            && '/' == s.chars().last().unwrap() =>
                                    {
                                        &n.nodes.as_ref().unwrap()
                                            [position(n.indices.as_ref().unwrap(), '*')?]
                                    }
                                    _ => n,
                                },
                                params,
                            ));
                        }
                    }

                    // Named Parameter
                    if let Some(i) = position(indices, ':') {
                        if let Some((n, ps)) = nodes[i].find(p).as_mut() {
                            params.append(ps);
                            return Some((n, params));
                        }
                    }

                    // Catch-All Parameter
                    if let Some(i) = position(indices, '*') {
                        if let Some((n, ps)) = nodes[i].find(p).as_mut() {
                            params.append(ps);
                            return Some((n, params));
                        }
                    }

                    None
                }
            }
            NodeKind::Parameter => match position(p, '/') {
                Some(i) => {
                    let indices = self.indices.as_ref()?;

                    params.push(&p[..i]);
                    p = &p[i..];

                    let (n, ref mut ps) = self.nodes.as_ref().unwrap()
                        [position(indices, p.chars().next().unwrap())?]
                    .find(p)?;

                    params.append(ps);
                    Some((n, params))
                }
                None => {
                    params.push(p);
                    Some((self, params))
                }
            },
            NodeKind::CatchAll => {
                params.push(p);
                Some((self, params))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PathTree<T>(Node<T>);

impl<T> Default for PathTree<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PathTree<T> {
    #[inline]
    pub fn new() -> Self {
        Self(Node::new(NodeKind::Static("/".to_owned())))
    }

    pub fn insert(&mut self, mut path: &str, data: T) -> &mut Self {
        let mut next = true;
        let mut node = &mut self.0;
        let mut params: Option<Vec<String>> = None;

        path = path.trim_start_matches('/');

        if path.len() == 0 {
            node.data = Some(data);
            return self;
        }

        while next {
            match path.chars().position(has_colon_or_star) {
                Some(i) => {
                    let mut prefix = &path[..i];
                    let mut suffix = &path[i..];

                    if prefix.len() > 0 {
                        node = node.add_node_static(prefix);
                    }

                    prefix = &suffix[..1];
                    suffix = &suffix[1..];

                    let mut kind: NodeKind;

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
                    params.get_or_insert_with(Vec::new).push(suffix.to_owned());
                    node = node.add_node_dynamic(c, kind);
                }
                None => {
                    next = false;
                    node = node.add_node_static(path);
                }
            }
        }

        node.data = Some(data);
        node.params = params;

        self
    }

    pub fn find<'a>(&'a self, path: &'a str) -> Option<(&'a T, Vec<(&'a str, &'a str)>)> {
        match self.0.find(path) {
            Some((node, ref values)) => match (node.data.as_ref(), node.params.as_ref()) {
                (Some(data), Some(params)) => Some((data, make_params(values, params))),
                (Some(data), None) => Some((data, Vec::new())),
                _ => None,
            },
            None => None,
        }
    }
}

const fn has_colon_or_star(c: char) -> bool {
    (c == ':') | (c == '*')
}

const fn has_star_or_slash(c: char) -> bool {
    (c == '*') | (c == '/')
}

fn position(p: &str, c: char) -> Option<usize> {
    p.chars().position(|x| x == c)
}

fn loc(s: &str, p: &str) -> String {
    s.chars()
        .zip(p.chars())
        .take_while(|(a, b)| a == b)
        .map(|v| v.0)
        .collect()
}

fn make_params<'a>(values: &Vec<&'a str>, params: &'a Vec<String>) -> Vec<(&'a str, &'a str)> {
    params
        .iter()
        .zip(values.iter())
        .map(|(a, b)| (a.as_ref(), *b))
        .collect()
}
