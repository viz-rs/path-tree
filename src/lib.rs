use radix_tree::Node;
use std::iter::FromIterator;

#[derive(Clone, Debug)]
pub enum NodeKind {
    Root = 0,
    Static,
    Parameter,
    CatchAll,
}

#[derive(Clone, Debug)]
pub struct NodeMetadata<R> {
    pub key: bool,
    pub kind: NodeKind,
    pub data: Option<R>,
    pub params: Option<Vec<&'static str>>,
}

impl<R> NodeMetadata<R> {
    pub fn new() -> Self {
        NodeMetadata {
            key: false,
            data: None,
            params: None,
            kind: NodeKind::Root,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PathTree<R> {
    pub tree: Node<char, NodeMetadata<R>>,
}

impl<R> PathTree<R>
where
    R: Clone + Copy,
{
    pub fn new(path: &'static str, data: NodeMetadata<R>) -> Self {
        PathTree {
            tree: Node::new(path, data),
        }
    }

    pub fn insert(&mut self, path: &'static str, data: R) -> &mut Self {
        let mut node = &mut self.tree;
        let mut params: Option<Vec<Vec<char>>> = None;
        let mut buf: Vec<char> = path.trim_start_matches('/').chars().collect();

        // Root "/"
        if 0 == buf.len() {
            if let Some(d) = node.data.as_mut() {
                d.key = true;
                d.data = Some(data);
            }
            return self;
        }

        while 0 < buf.len() {
            let mut i: usize = 0;
            let mut next: Vec<char>;
            let mut meta = NodeMetadata::new();

            match buf[i] {
                '*' => {
                    next = buf.split_off(buf.len());
                    match params.as_mut() {
                        Some(p) => {
                            p.push(buf.split_off(1));
                        }
                        None => {
                            params.replace(vec![buf.split_off(1)]);
                        }
                    }
                    meta.kind = NodeKind::CatchAll;
                }
                ':' => {
                    next = buf.split_off(loop {
                        if i == buf.len() {
                            break i;
                        }
                        if '*' == buf[i] || '/' == buf[i] {
                            break i;
                        }
                        i += 1;
                    });
                    match params.as_mut() {
                        Some(p) => {
                            p.push(buf.split_off(1));
                        }
                        None => {
                            params.replace(vec![buf.split_off(1)]);
                        }
                    }
                    meta.kind = NodeKind::Parameter;
                }
                _ => {
                    next = buf.split_off(loop {
                        if i == buf.len() {
                            break i;
                        }
                        if ':' == buf[i] || '*' == buf[i] {
                            break i;
                        }
                        i += 1;
                    });
                    meta.kind = NodeKind::Static;
                }
            }

            let ended = 0 == next.len();

            // end
            if ended {
                if let Some(ref p) = params {
                    meta.params = Some(
                        p.iter()
                            .map(|x| {
                                &*(Box::leak(String::from_iter(x.into_iter()).into_boxed_str()))
                            })
                            .collect(),
                    );
                }
                meta.key = true;
                meta.data = Some(data);
            }

            // Add '/' ':' '*' to last
            node = node.add_node_with(&mut buf, Some(meta), 0, ended, |&l, &c, indices| {
                let mut j = l;
                if 0 == j {
                    return j;
                }

                if '*' == c {
                    return j;
                }
                if '*' == indices[j - 1] {
                    j -= 1;
                }

                if ':' == c {
                    return j;
                }
                if 0 < j && ':' == indices[j - 1] {
                    j -= 1;
                }

                if '/' == c {
                    return j;
                }
                if 0 < j && '/' == indices[j - 1] {
                    j -= 1;
                }

                j
            });

            buf = next;
        }

        self
    }

    pub fn find_with(
        &mut self,
        path: &'static str,
    ) -> Option<(&Node<char, NodeMetadata<R>>, Option<Vec<Vec<char>>>)> {
        recognize(&path.chars().collect(), &self.tree)
    }

    pub fn find(
        &mut self,
        path: &'static str,
    ) -> Option<(
        &Node<char, NodeMetadata<R>>,
        Option<Vec<(&'static str, &'static str)>>,
    )> {
        let mut params: Option<Vec<(&'static str, &'static str)>> = None;
        // Too many if and deep
        if let Some((node, values)) = &self.find_with(path) {
            if let Some(data) = &node.data {
                if !data.key {
                    return None;
                }

                if let Some(ps) = &data.params {
                    if let Some(vs) = &values {
                        params = Some(
                            vs.iter()
                                .enumerate()
                                .map(|(i, v)| {
                                    (
                                        &*ps[i],
                                        &*(Box::leak(
                                            String::from_iter(v.into_iter()).into_boxed_str(),
                                        )),
                                    )
                                })
                                .collect(),
                        );
                    }
                }
            }
            return Some((node, params));
        }

        None
    }
}

pub fn recognize<'a, R>(
    path: &Vec<char>,
    node: &'a Node<char, NodeMetadata<R>>,
) -> Option<(&'a Node<char, NodeMetadata<R>>, Option<Vec<Vec<char>>>)> {
    if 0 == path.len() {
        return None;
    }

    let mut buf: Vec<char> = path.clone();
    let mut values: Option<Vec<Vec<char>>> = None;

    match node.path[0] {
        '*' => {
            match values.as_mut() {
                Some(v) => {
                    v.push(buf);
                }
                None => {
                    values.replace(vec![buf]);
                }
            }
            return Some((&node, values));
        }
        ':' => {
            let mut i = 0;
            let next = buf.split_off(loop {
                if i == buf.len() {
                    break i;
                }
                if '/' == buf[i] {
                    break i;
                }
                i += 1;
            });

            match values.as_mut() {
                Some(v) => {
                    v.push(buf);
                }
                None => {
                    values.replace(vec![buf]);
                }
            }

            if 0 == next.len() {
                return Some((&node, values));
            }

            if 0 == node.indices.len() {
                return None;
            }

            if let Some((n, v)) = recognize(&next, &node.nodes[0]).as_mut() {
                if let Some(d) = v.as_mut() {
                    values.as_mut().unwrap().append(d);
                }
                return Some((&n, values));
            }

            return None;
        }
        _ => {
            let mut m = buf.len();
            let mut n = m;
            let mut o = node.path.len();

            if m >= o {
                m = 0;
                while m < o && buf[m] == node.path[m] {
                    m += 1;
                }
            }

            if m < o {
                return None;
            }

            if m == o && m == n {
                return Some((&node, values));
            }

            let mut l = node.indices.len();
            if 0 == l {
                return None;
            }

            buf = buf.split_off(m);

            o = 0;
            let mut has_star = false;
            if '*' == node.indices[l - 1] {
                l -= 1;
                o = l;
                has_star = true;
            }

            n = 0;
            let mut has_colon = false;
            if l > 0 && ':' == node.indices[l - 1] {
                l -= 1;
                n = l;
                has_colon = true;
            }

            m = 0;
            let c = buf[m];
            let mut has_node = false;
            while m < l {
                if c == node.indices[m] {
                    has_node = true;
                    break;
                }
                m += 1;
            }

            // Static Node
            if has_node {
                if let Some((n, v)) = recognize(&buf, &node.nodes[m]) {
                    if let Some(mut d) = v {
                        match values.as_mut() {
                            Some(v) => {
                                v.append(&mut d);
                            }
                            None => {
                                values.replace(d);
                            }
                        }
                    }

                    // '/'
                    if '/' == n.path[n.path.len() - 1] {
                        if let Some(data) = &n.data {
                            if data.key {
                                // '/' is key node, ended
                                return Some((&n, values));
                            } else if 0 < n.indices.len() && '*' == n.indices[n.indices.len() - 1] {
                                // CatchAll '*'
                                return Some((&n.nodes[n.indices.len() - 1], values));
                            } else {
                                return None;
                            }
                        }
                    }

                    return Some((&n, values));
                }
            }

            // Parameter ':'
            if has_colon {
                if let Some((n, v)) = recognize(&buf, &node.nodes[n]) {
                    if let Some(mut d) = v {
                        match values.as_mut() {
                            Some(v) => {
                                v.append(&mut d);
                            }
                            None => {
                                values.replace(d);
                            }
                        }
                    }
                    return Some((&n, values));
                }
            }

            // CatchAll '*'
            if has_star {
                if let Some((n, v)) = recognize(&buf, &node.nodes[o]) {
                    if let Some(mut d) = v {
                        match values.as_mut() {
                            Some(v) => {
                                v.append(&mut d);
                            }
                            None => {
                                values.replace(d);
                            }
                        }
                    }
                    return Some((&n, values));
                }
            }

            // dbg!(buf);
        }
    }

    None
}
