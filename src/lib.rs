use std::mem;

#[derive(Clone, Debug)]
pub enum NodeKind {
    Static(String),
    Parameter,
    CatchAll,
}

#[derive(Clone, Debug)]
pub struct Node<'a, T> {
    kind: NodeKind,
    params: Option<Vec<&'a str>>,
    data: Option<T>,
    indices: Option<String>,
    nodes: Option<Vec<Self>>,
}

impl<'a, T> Default for Node<'a, T> {
    fn default() -> Self {
        Node::new(NodeKind::Static(String::new()))
    }
}

impl<'a, T> Node<'a, T> {
    pub fn new(kind: NodeKind) -> Self {
        Node {
            kind,
            data: None,
            params: None,
            indices: None,
            nodes: None,
        }
    }

    #[inline]
    fn add_node(&mut self, c: char, kind: NodeKind) -> &mut Self {
        let indices: &mut String = self.indices.get_or_insert_with(|| String::new());
        let nodes: &mut Vec<Node<T>> = self.nodes.get_or_insert_with(|| Vec::new());

        match position(indices, c) {
            Some(i) => match kind {
                NodeKind::Static(s) => nodes[i].insert(&s),
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
            NodeKind::Static(ref mut s) => {
                if s.len() == 0 {
                    *s = p.to_owned();
                    return self;
                }

                let np = loc(s, p);

                if s.len() > np.len() {
                    let new_path = &s[np.len()..];
                    let new_node = Node {
                        data: mem::replace(&mut self.data, None),
                        nodes: mem::replace(&mut self.nodes, None),
                        params: mem::replace(&mut self.params, None),
                        indices: mem::replace(&mut self.indices, None),
                        kind: NodeKind::Static(new_path.to_owned()),
                    };
                    self.indices
                        .get_or_insert_with(|| String::new())
                        .push(new_path.chars().next().unwrap());
                    self.nodes.get_or_insert_with(|| Vec::new()).push(new_node);
                    *s = np.to_owned();
                }

                if p.len() == np.len() {
                    self
                } else {
                    self.add_node_static(&p[np.len()..])
                }
            }
            NodeKind::Parameter => self.add_node_static(p),
            NodeKind::CatchAll => self,
        }
    }

    #[inline]
    pub fn find(&self, mut p: &'a str) -> Option<(&Self, Vec<&'a str>)> {
        let mut params = Vec::new();

        match self.kind {
            NodeKind::Static(ref s) => {
                let np = loc(s, p);

                if np.len() == 0 {
                    None
                } else if np.len() < s.len() {
                    None
                } else if np.len() == s.len() && np.len() == p.len() {
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

                    p = &p[np.len()..];

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
pub struct PathTree<'a, T> {
    tree: Node<'a, T>,
}

impl<'a, T> Default for PathTree<'a, T> {
    fn default() -> Self {
        PathTree::new()
    }
}

impl<'a, T> PathTree<'a, T> {
    pub fn new() -> Self {
        PathTree {
            tree: Node::new(NodeKind::Static("/".to_owned())),
        }
    }

    pub fn insert(&mut self, mut path: &'a str, data: T) -> &mut Self {
        let mut next = true;
        let mut node = &mut self.tree;
        let mut params: Option<Vec<&'a str>> = None;

        path = path.trim_start_matches('/');

        if path.len() == 0 {
            node.data = Some(data);
            return self;
        }

        while next {
            match path.chars().position(|c| c == ':' || c == '*') {
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
                        match suffix.chars().position(|c| c == '*' || c == '/') {
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
                    params.get_or_insert_with(|| Vec::new()).push(suffix);
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

    pub fn find(&self, path: &'a str) -> Option<(&T, Vec<(&'a str, &'a str)>)> {
        match self.tree.find(path) {
            Some((node, ref values)) => match (node.data.as_ref(), node.params.as_ref()) {
                (Some(data), Some(params)) => Some((data, make_params(values, params))),
                (Some(data), None) => Some((data, Vec::new())),
                _ => None,
            },
            None => None,
        }
    }
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

fn make_params<'a>(values: &Vec<&'a str>, params: &Vec<&'a str>) -> Vec<(&'a str, &'a str)> {
    params
        .iter()
        .zip(values.iter())
        .map(|(a, b)| (*a, *b))
        .collect()
}
