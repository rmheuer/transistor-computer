use super::{Circuit, NodeId, Transistor};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    Low,
    High,
    HighZ,
}

impl std::fmt::Display for NodeState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            Self::Low => f.write_str("0"),
            Self::High => f.write_str("1"),
            Self::HighZ => f.write_str("-"),
        }
    }
}

pub struct CircuitSim<'a> {
    transistors: &'a Vec<Transistor>,

    default_node_states: Vec<NodeState>,
    curr_node_states: Vec<NodeState>,
}

impl<'a> CircuitSim<'a> {
    pub fn new(circuit: &'a Circuit) -> Self {
        let mut node_states: Vec<NodeState> = (0..circuit.node_count())
            .map(|pullup| {
                if circuit.pullups[pullup] {
                    NodeState::High
                } else {
                    NodeState::HighZ
                }
            })
            .collect();
        node_states[circuit.gnd_node.0] = NodeState::Low;

        Self {
            transistors: &circuit.transistors,
            default_node_states: node_states.clone(),
            curr_node_states: node_states,
        }
    }

    pub fn get_node_state(&self, node: NodeId) -> NodeState {
        self.curr_node_states[node.0]
    }

    pub fn set_node_state(&mut self, node: NodeId, value: NodeState) {
        self.curr_node_states[node.0] = value;
    }

    pub fn step(&mut self) {
        let mut new_states = self.default_node_states.clone();

        for transistor in self.transistors {
            let emitter_state = self.get_node_state(transistor.emitter);
            let base_state = self.get_node_state(transistor.base);

            if base_state == NodeState::High && emitter_state == NodeState::Low {
                new_states[transistor.collector.0] = NodeState::Low;
            }
        }

        self.curr_node_states = new_states;
    }
}
