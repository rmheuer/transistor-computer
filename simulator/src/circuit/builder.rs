use bit_vec::BitVec;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Node(usize);

#[derive(Debug)]
pub struct NodeMap {
    map: Vec<super::NodeId>,
}

impl NodeMap {
    pub fn get_node_id(&self, node: Node) -> super::NodeId {
        self.map[node.0]
    }
}

#[derive(Debug)]
pub struct NodeNames {
    map: Vec<String>,
}

impl NodeNames {
    pub fn get_node_name(&self, node_id: super::NodeId) -> &str {
        &self.map[node_id.0]
    }
}

struct NodeInfo {
    depth: usize,
    name: String,
}

struct Transistor {
    emitter: Node,
    base: Node,
    collector: Node,
}

struct Join {
    input: Node,
    output: Node,
}

pub struct CircuitBuilder {
    nodes: Vec<NodeInfo>,
    transistors: Vec<Transistor>,
    joins: Vec<Join>,
    pullups: Vec<Node>,
    gnd_node: Node,

    path_prefix: String,
    current_depth: usize,
}

impl CircuitBuilder {
    pub fn new() -> Self {
        Self {
            nodes: vec![NodeInfo {
                depth: 0,
                name: "gnd".into(),
            }],
            transistors: Vec::new(),
            joins: Vec::new(),
            pullups: Vec::new(),
            gnd_node: Node(0),

            path_prefix: "".into(),
            current_depth: 1,
        }
    }

    pub fn gnd(&self) -> Node {
        self.gnd_node
    }

    pub fn node(&mut self, name: &str) -> Node {
        let node = Node(self.nodes.len());
        self.nodes.push(NodeInfo {
            depth: self.current_depth,
            name: format!("{}{name}", self.path_prefix),
        });
        node
    }

    pub fn nodes<const N: usize>(&mut self, prefix: &str) -> [Node; N] {
        std::array::from_fn(|n| self.node(&format!("{prefix}[{n}]")))
    }

    pub fn transistor(&mut self, emitter: Node, base: Node, collector: Node) {
        self.transistors.push(Transistor {
            emitter,
            base,
            collector,
        });
    }

    pub fn join(&mut self, input: Node, output: Node) {
        self.joins.push(Join { input, output });
    }

    pub fn join_bus<const N: usize>(&mut self, inputs: [Node; N], outputs: [Node; N]) {
        for i in 0..N {
            self.join(inputs[i], outputs[i]);
        }
    }

    pub fn pullup(&mut self, node: Node) {
        if self.pullups.contains(&node) {
            let name = &self.nodes[node.0].name;
            eprintln!("Duplicate pullup on node: {name}");
        }
        self.pullups.push(node);
    }

    pub fn subcircuit<T>(&mut self, name: &str, f: &impl Fn(&mut Self) -> T) -> T {
        let initial_path_len = self.path_prefix.len();
        self.path_prefix.push_str(name);
        self.path_prefix.push('.');
        self.current_depth += 1;

        let sub = f(self);

        self.path_prefix.truncate(initial_path_len);
        self.current_depth -= 1;

        sub
    }

    pub fn subcircuits<T, const N: usize>(
        &mut self,
        prefix: &str,
        f: &impl Fn(&mut Self) -> T,
    ) -> [T; N] {
        std::array::from_fn(move |n| self.subcircuit(&format!("{prefix}[{n}]"), f))
    }

    pub fn build(self) -> (super::Circuit, NodeMap, NodeNames) {
        enum InfoOrRedirect {
            Info(NodeInfo),
            Redirect(usize),
        }

        let mut infos: Vec<_> = self.nodes.into_iter().map(InfoOrRedirect::Info).collect();

        fn find_info(infos: &Vec<InfoOrRedirect>, mut index: usize) -> (usize, &NodeInfo) {
            loop {
                match &infos[index] {
                    InfoOrRedirect::Info(info) => return (index, info),
                    InfoOrRedirect::Redirect(target) => index = *target,
                }
            }
        }

        for Join { input, output } in self.joins {
            let (input_index, input_info) = find_info(&infos, input.0);
            let (output_index, output_info) = find_info(&infos, output.0);
            if input_index == output_index {
                // Prevent loops from breaking things
                continue;
            }

            if input_info.depth <= output_info.depth {
                infos[output_index] = InfoOrRedirect::Redirect(input_index);
            } else {
                infos[input_index] = InfoOrRedirect::Redirect(output_index);
            }
        }

        let mut node_map = vec![super::NodeId(999); infos.len()];
        let mut node_names = Vec::new();
        for (index, info_or_redirect) in infos.iter().enumerate() {
            if let InfoOrRedirect::Info(info) = info_or_redirect {
                let id = super::NodeId(node_names.len());
                node_names.push(info.name.clone());
                node_map[index] = id;
            }
        }
        for (index, info_or_redirect) in infos.iter().enumerate() {
            if let InfoOrRedirect::Redirect(target) = info_or_redirect {
                let (target, _) = find_info(&infos, *target);
                let target_id = node_map[target];
                node_map[index] = target_id;
            }
        }
        let node_map = NodeMap { map: node_map };

        let transistors: Vec<_> = self
            .transistors
            .into_iter()
            .map(|t| super::Transistor {
                emitter: node_map.get_node_id(t.emitter),
                base: node_map.get_node_id(t.base),
                collector: node_map.get_node_id(t.collector),
            })
            .collect();

        let mut pullups = BitVec::with_capacity(infos.len());
        pullups.grow(infos.len(), false);
        for pullup in self.pullups {
            let id = node_map.get_node_id(pullup);
            if pullups[id.0] {
                let name = &node_names[id.0];
                eprintln!("Duplicate pullup on joined node: {name}");
            }
            pullups.set(id.0, true);
        }

        (
            super::Circuit {
                pullups,
                gnd_node: node_map.get_node_id(self.gnd_node),
                transistors,
            },
            node_map,
            NodeNames { map: node_names },
        )
    }
}
