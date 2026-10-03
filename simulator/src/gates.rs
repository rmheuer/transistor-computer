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
