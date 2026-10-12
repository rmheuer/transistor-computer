use bit_vec::BitVec;

pub mod builder;
pub mod logic_sim;

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
    pullups: BitVec,
    gnd_node: NodeId,
    transistors: Vec<Transistor>,
}

impl Circuit {
    pub fn node_count(&self) -> usize {
        self.pullups.len()
    }
}
