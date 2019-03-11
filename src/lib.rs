use std::mem;

#[derive(Debug)]
pub enum NodeKind {
    Static(String),
    Parameter,
    CatchAll,
}

#[inline]
fn position(p: &str, c: char) -> Option<usize> {
    p.chars().position(|x| x == c)
}

#[derive(Debug)]
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

                let np = s
                    .chars()
                    .zip(p.chars())
                    .take_while(|(a, b)| a == b)
                    .map(|v| v.0)
                    .collect::<String>();

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
                let np = s
                    .chars()
                    .zip(p.chars())
                    .take_while(|(a, b)| a == b)
                    .map(|v| v.0)
                    .collect::<String>();

                if np.len() == 0 {
                    None
                } else if np.len() < s.len() {
                    None
                } else if np.len() == s.len() && np.len() == p.len() {
                    Some((self, params))
                } else {
                    if self.indices.is_none() {
                        return None;
                    }

                    let indices = self.indices.as_ref().unwrap();
                    let nodes = self.nodes.as_ref().unwrap();
                    p = &p[np.len()..];

                    if let Some(i) = position(indices, p.chars().next().unwrap()) {
                        if let Some((n, ps)) = nodes[i].find(p).as_mut() {
                            params.append(ps);

                            // end `/` `/*any`
                            if let NodeKind::Static(s) = &n.kind {
                                if '/' == s.chars().last().unwrap() {
                                    if n.data.is_some() {
                                        return Some((n, params));
                                    } else if n.indices.is_some() {
                                        let indices = n.indices.as_ref().unwrap();
                                        let nodes = n.nodes.as_ref().unwrap();
                                        return match position(indices, '*') {
                                            Some(i) => Some((&nodes[i], params)),
                                            None => None,
                                        };
                                    }
                                }
                            }

                            return Some((n, params));
                        }
                    }

                    if let Some(i) = position(indices, ':') {
                        if let Some((n, ps)) = nodes[i].find(p).as_mut() {
                            params.append(ps);
                            return Some((n, params));
                        }
                    }

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
                    if self.indices.is_none() {
                        return None;
                    }

                    let indices = self.indices.as_ref().unwrap();
                    let nodes = self.nodes.as_ref().unwrap();

                    params.push(&p[..i]);

                    p = &p[i..];

                    if let Some(i) = position(indices, p.chars().next().unwrap()) {
                        if let Some((n, ps)) = nodes[i].find(p).as_mut() {
                            params.append(ps);
                            return Some((n, params));
                        }
                    }
                    None
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

#[derive(Debug)]
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
        let mut params = Vec::new();

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
                    params.push(suffix);
                    node = node.add_node_dynamic(c, kind);
                }
                None => {
                    next = false;
                    node = node.add_node_static(path);
                }
            }
        }

        if params.len() > 0 {
            node.params = Some(params);
        }
        node.data = Some(data);

        self
    }

    pub fn find(&self, path: &'a str) -> Option<(&T, Vec<(&'a str, &'a str)>)> {
        match self.tree.find(path) {
            Some((node, values)) => match (node.data.as_ref(), node.params.as_ref()) {
                (Some(data), Some(params)) => Some((
                    data,
                    params
                        .iter()
                        .zip(values.iter())
                        .map(|(a, b)| (*a, *b))
                        .collect(),
                )),
                (Some(data), None) => Some((data, Vec::new())),
                _ => None,
            },
            None => None,
        }
    }
}
