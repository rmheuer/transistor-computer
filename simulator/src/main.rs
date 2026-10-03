mod circuit {
    use bit_vec::BitVec;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct NodeId(usize);

    #[derive(Debug)]
    pub struct Transistor {
        emitter: NodeId,
        base: NodeId,
        collector: NodeId,
    }

    #[derive(Debug)]
    pub struct Circuit {
        node_count: usize,
        gnd_node: NodeId,
        transistors: Vec<Transistor>,
    }

    pub struct CircuitSim<'a> {
        transistors: &'a Vec<Transistor>,
        gnd_node: NodeId,
        curr_node_states: BitVec,
        prev_node_states: BitVec,
    }

    impl<'a> CircuitSim<'a> {
        pub fn new(circuit: &'a Circuit) -> Self {
            let mut node_states = BitVec::with_capacity(circuit.node_count);
            node_states.grow(circuit.node_count, true);
            node_states.set(circuit.gnd_node.0, false);

            Self {
                transistors: &circuit.transistors,
                gnd_node: circuit.gnd_node,
                curr_node_states: node_states.clone(),
                prev_node_states: node_states,
            }
        }

        pub fn get_node_state(&self, node: NodeId) -> bool {
            self.curr_node_states[node.0]
        }

        pub fn get_prev_node_state(&self, node: NodeId) -> bool {
            self.prev_node_states[node.0]
        }

        pub fn set_node_state(&mut self, node: NodeId, value: bool) {
            self.curr_node_states.set(node.0, value);
        }

        pub fn step(&mut self) {
            std::mem::swap(&mut self.curr_node_states, &mut self.prev_node_states);

            self.curr_node_states.fill(true);
            self.curr_node_states.set(self.gnd_node.0, false);

            for transistor in self.transistors {
                let emitter_state = self.get_prev_node_state(transistor.emitter);
                let base_state = self.get_prev_node_state(transistor.base);

                if base_state && !emitter_state {
                    self.curr_node_states.set(transistor.collector.0, false);
                }
            }
        }
    }

    pub mod builder {
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

                (
                    super::Circuit {
                        node_count: node_names.len(),
                        gnd_node: node_map.get_node_id(self.gnd_node),
                        transistors,
                    },
                    node_map,
                    NodeNames { map: node_names },
                )
            }
        }
    }
}

mod gates {
    use crate::circuit::builder::{CircuitBuilder, Node};

    pub struct Gate1 {
        pub input: Node,
        pub output: Node,
    }

    fn make_gate1(builder: &mut CircuitBuilder) -> Gate1 {
        Gate1 {
            input: builder.node("input"),
            output: builder.node("output"),
        }
    }

    pub fn not(builder: &mut CircuitBuilder) -> Gate1 {
        let gate = make_gate1(builder);
        builder.transistor(builder.gnd(), gate.input, gate.output);
        gate
    }

    pub struct Gate2 {
        pub input_a: Node,
        pub input_b: Node,
        pub output: Node,
    }

    fn make_gate2(builder: &mut CircuitBuilder) -> Gate2 {
        Gate2 {
            input_a: builder.node("input_a"),
            input_b: builder.node("input_b"),
            output: builder.node("output"),
        }
    }

    pub fn xnor(builder: &mut CircuitBuilder) -> Gate2 {
        let gate = make_gate2(builder);
        builder.transistor(gate.input_a, gate.input_b, gate.output);
        builder.transistor(gate.input_b, gate.input_a, gate.output);
        gate
    }
}

mod adder {
    use crate::{
        circuit::builder::{CircuitBuilder, Node},
        gates,
    };

    struct Bit {
        carry_in_inv: Node,
        input_a: Node,
        input_b: Node,
        sum_out_inv: Node,
        carry_out: Node,
    }

    fn make_bit(builder: &mut CircuitBuilder) -> Bit {
        let bit = Bit {
            carry_in_inv: builder.node("carry_in_inv"),
            input_a: builder.node("input_a"),
            input_b: builder.node("input_b"),
            sum_out_inv: builder.node("sum_out_inv"),
            carry_out: builder.node("carry_out"),
        };

        let xnor1 = builder.subcircuit("xnor1", &gates::xnor);
        builder.join(bit.carry_in_inv, xnor1.input_a);
        builder.join(bit.input_a, xnor1.input_b);

        let xnor2 = builder.subcircuit("xnor2", &gates::xnor);
        builder.join(xnor1.output, xnor2.input_a);
        builder.join(bit.input_b, xnor2.input_b);

        builder.join(xnor2.output, bit.sum_out_inv);
        builder.transistor(bit.input_a, bit.carry_in_inv, bit.carry_out);
        builder.transistor(bit.input_b, xnor1.output, bit.carry_out);

        bit
    }

    pub struct Adder {
        pub carry_in_inv: Node,
        pub input_a: [Node; 8],
        pub input_b: [Node; 8],
        pub sum_out_inv: [Node; 8],
        pub carry_out: Node,
    }

    pub fn make_adder(builder: &mut CircuitBuilder) -> Adder {
        let adder = Adder {
            carry_in_inv: builder.node("carry_in_inv"),
            input_a: builder.nodes("input_a"),
            input_b: builder.nodes("input_b"),
            sum_out_inv: builder.nodes("sum_out_inv"),
            carry_out: builder.node("carry_out"),
        };

        let bits: [Bit; 8] = builder.subcircuits("bit", &make_bit);
        let buffers: [_; 7] = builder.subcircuits("carry_inv", &gates::not);
        for i in 0..7 {
            builder.join(bits[i].carry_out, buffers[i].input);
            builder.join(buffers[i].output, bits[i + 1].carry_in_inv);
        }

        builder.join(adder.carry_in_inv, bits[0].carry_in_inv);
        for i in 0..8 {
            builder.join(adder.input_a[i], bits[i].input_a);
            builder.join(adder.input_b[i], bits[i].input_b);
            builder.join(bits[i].sum_out_inv, adder.sum_out_inv[i]);
        }
        builder.join(bits[7].carry_out, adder.carry_out);

        adder
    }
}

mod gui {
    use crate::circuit::{CircuitSim, NodeId};
    use eframe::egui;

    pub struct NodeHandle<'a> {
        pub node_id: NodeId,
        pub name: &'a str,
    }

    pub struct SimConfig<'a> {
        pub sim: CircuitSim<'a>,
        pub inputs: &'a [NodeHandle<'a>],
        pub outputs: &'a [NodeHandle<'a>],
    }

    struct SimApplication<'a> {
        config: SimConfig<'a>,
        input_states: Vec<bool>,
    }

    impl<'a> SimApplication<'a> {
        pub fn new(config: SimConfig<'a>) -> Self {
            Self {
                input_states: vec![false; config.inputs.len()],
                config,
            }
        }
    }

    impl eframe::App for SimApplication<'_> {
        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            egui::CentralPanel::default().show(ui, |ui| {
                for (index, input) in self.config.inputs.into_iter().enumerate() {
                    ui.checkbox(&mut self.input_states[index], input.name);
                }

                for _ in 0..1 {
                    for (index, input) in self.config.inputs.into_iter().enumerate() {
                        self.config
                            .sim
                            .set_node_state(input.node_id, self.input_states[index]);
                    }
                    self.config.sim.step();
                }

                for output in self.config.outputs {
                    ui.label(format!(
                        "{} = {}",
                        output.name,
                        self.config.sim.get_node_state(output.node_id)
                    ));
                }
            });

            ui.request_repaint();
        }
    }

    pub fn run_ui(config: SimConfig) -> eframe::Result {
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
            ..Default::default()
        };

        eframe::run_native(
            "Transistor Simulator",
            options,
            Box::new(|_| Ok(Box::new(SimApplication::new(config)))),
        )
    }
}

fn main() -> eframe::Result {
    let mut builder = circuit::builder::CircuitBuilder::new();
    let adder = adder::make_adder(&mut builder);

    //let not_gate = gates::not(&mut builder);

    let (circuit, node_map, node_names) = builder.build();
    println!("circuit = {circuit:#?}");
    println!("node_map = {node_map:?}");
    println!("node_names = {node_names:?}");

    //let inputs = [not_gate.input].to_vec();
    //let outputs = [not_gate.output].to_vec();

    let inputs = [
        &[adder.carry_in_inv][..],
        &adder.input_a[..],
        &adder.input_b[..],
    ]
    .concat();
    let outputs = [&adder.sum_out_inv[..], &[adder.carry_out][..]].concat();

    fn make_handles<'a>(
        nodes: Vec<circuit::builder::Node>,
        node_map: &circuit::builder::NodeMap,
        node_names: &'a circuit::builder::NodeNames,
    ) -> Vec<gui::NodeHandle<'a>> {
        nodes
            .into_iter()
            .map(|node| {
                let node_id = node_map.get_node_id(node);
                gui::NodeHandle {
                    node_id,
                    name: node_names.get_node_name(node_id),
                }
            })
            .collect()
    }

    let sim = circuit::CircuitSim::new(&circuit);
    gui::run_ui(gui::SimConfig {
        sim,
        inputs: &make_handles(inputs, &node_map, &node_names),
        outputs: &make_handles(outputs, &node_map, &node_names),
    })
}
