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
