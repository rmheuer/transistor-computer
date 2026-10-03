use bit_vec::BitVec;

pub mod builder;

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
