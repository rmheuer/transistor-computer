use crate::circuit::builder::{CircuitBuilder, Node};

pub struct ReverserCtrl {
    pub pass_inv: [Node; 2],
    pub reverse_inv: [Node; 2],
}

// Buffered
struct BitReverser {
    pub bits_in_inv: [Node; 8],
    pub bits_out: [Node; 8],
    pub ctrl: ReverserCtrl,
}

fn make_bit_reverser(builder: &mut CircuitBuilder) -> BitReverser {
    let reverser = BitReverser {
        bits_in_inv: builder.nodes("bits_in_inv"),
        bits_out: builder.nodes("bits_out"),
        ctrl: ReverserCtrl {
            pass_inv: builder.nodes("pass_inv"),
            reverse_inv: builder.nodes("reverse_inv"),
        },
    };
    let ctrl = &reverser.ctrl;

    for bit in 0..8 {
        let (pass_inv, reverse_inv) = if bit < 4 {
            (ctrl.pass_inv[0], ctrl.reverse_inv[0])
        } else {
            (ctrl.pass_inv[1], ctrl.reverse_inv[1])
        };

        builder.transistor(pass_inv, reverser.bits_in_inv[bit], reverser.bits_out[bit]);
        builder.transistor(
            reverse_inv,
            reverser.bits_in_inv[7 - bit],
            reverser.bits_out[bit],
        );
        builder.pullup(reverser.bits_out[bit]);
    }

    reverser
}

pub struct RightShift1Ctrl {
    // pass:             pass_1to6 = 1, pass_7 = 1, shift = 0, rotate = 0
    // logical shift:    pass_1to6 = 0, pass_7 = 0, shift = 1, rotate = 0
    // arithmetic shift: pass_1to6 = 0, pass_7 = 1, shift = 1, rotate = 0
    // rotate:           pass_1to6 = 0, pass_7 = 0, shift = 1, rotate = 1
    // cancel (all 1):   pass_1to6 = 0, pass_7 = 0, shift = 0, rotate = 0
    pub pass_1to6_inv: [Node; 2],
    pub pass_7_inv: Node,
    pub shift_inv: [Node; 2],
    pub rotate_inv: Node,
}

// Passive
struct RightShift1 {
    pub bits_in: [Node; 8],
    pub bits_out_inv: [Node; 8],
    pub ctrl: RightShift1Ctrl,
}

fn make_right_shift_1(builder: &mut CircuitBuilder) -> RightShift1 {
    let shifter = RightShift1 {
        bits_in: builder.nodes("bits_in"),
        bits_out_inv: builder.nodes("bits_out_inv"),
        ctrl: RightShift1Ctrl {
            pass_1to6_inv: builder.nodes("pass_1to6_inv"),
            pass_7_inv: builder.node("pass_7_inv"),
            shift_inv: builder.nodes("shift_inv"),
            rotate_inv: builder.node("rotate_inv"),
        },
    };
    let ctrl = &shifter.ctrl;

    for bit in 0..7 {
        let (pass_inv, shift_inv) = if bit < 4 {
            (ctrl.pass_1to6_inv[0], ctrl.shift_inv[0])
        } else {
            (ctrl.pass_1to6_inv[1], ctrl.shift_inv[1])
        };

        builder.transistor(pass_inv, shifter.bits_in[bit], shifter.bits_out_inv[bit]);
        builder.transistor(
            shift_inv,
            shifter.bits_in[bit + 1],
            shifter.bits_out_inv[bit],
        );
        builder.pullup(shifter.bits_out_inv[bit]);
    }

    builder.transistor(ctrl.pass_7_inv, shifter.bits_in[7], shifter.bits_out_inv[7]);
    builder.transistor(ctrl.rotate_inv, shifter.bits_in[0], shifter.bits_out_inv[7]);
    builder.pullup(shifter.bits_out_inv[7]);

    shifter
}

pub struct RightShift2Ctrl {
    // pass:             pass = 1, shift = 0, sign_ext = 0, rotate = 0, zero_6_7 = 0
    // logical shift:    pass = 0, shift = 1, sign_ext = 0, rotate = 0, zero_6_7 = 1
    // arithmetic shift: pass = 0, shift = 1, sign_ext = 1, rotate = 0, zero_6_7 = 0
    // rotate:           pass = 0, shift = 1, sign_ext = 0, rotate = 1, zero_6_7 = 0
    pub pass_inv: [Node; 2],
    pub shift_inv: [Node; 2],
    pub sign_ext_inv: Node,
    pub rotate_inv: Node,
    pub zero_6_7: Node,
}

// Buffered
struct RightShift2 {
    pub bits_in_inv: [Node; 8],
    pub bits_out: [Node; 8],
    pub ctrl: RightShift2Ctrl,
}

fn make_right_shift_2(builder: &mut CircuitBuilder) -> RightShift2 {
    let shifter = RightShift2 {
        bits_in_inv: builder.nodes("bits_in_inv"),
        bits_out: builder.nodes("bits_out"),
        ctrl: RightShift2Ctrl {
            pass_inv: builder.nodes("pass_inv"),
            shift_inv: builder.nodes("shift_inv"),
            sign_ext_inv: builder.node("sign_ext_inv"),
            rotate_inv: builder.node("rotate_inv"),
            zero_6_7: builder.node("zero_6_7"),
        },
    };
    let ctrl = &shifter.ctrl;

    for bit in 0..6 {
        let pass_inv = if bit < 4 {
            ctrl.pass_inv[0]
        } else {
            ctrl.pass_inv[1]
        };
        let shift_inv = if bit < 3 {
            ctrl.shift_inv[0]
        } else {
            ctrl.shift_inv[1]
        };

        builder.transistor(pass_inv, shifter.bits_in_inv[bit], shifter.bits_out[bit]);
        builder.transistor(
            shift_inv,
            shifter.bits_in_inv[bit + 2],
            shifter.bits_out[bit],
        );
        builder.pullup(shifter.bits_out[bit]);
    }

    // Bit 6
    builder.transistor(
        ctrl.pass_inv[1],
        shifter.bits_in_inv[6],
        shifter.bits_out[6],
    );
    builder.transistor(ctrl.rotate_inv, shifter.bits_in_inv[0], shifter.bits_out[6]);
    builder.transistor(
        ctrl.sign_ext_inv,
        shifter.bits_in_inv[7],
        shifter.bits_out[6],
    );
    builder.transistor(builder.gnd(), ctrl.zero_6_7, shifter.bits_out[6]);
    builder.pullup(shifter.bits_out[6]);

    // Bit 7
    builder.transistor(
        ctrl.pass_inv[1],
        shifter.bits_in_inv[7],
        shifter.bits_out[7],
    );
    builder.transistor(ctrl.rotate_inv, shifter.bits_in_inv[1], shifter.bits_out[7]);
    builder.transistor(
        ctrl.sign_ext_inv,
        shifter.bits_in_inv[7],
        shifter.bits_out[7],
    );
    builder.transistor(builder.gnd(), ctrl.zero_6_7, shifter.bits_out[7]);
    builder.pullup(shifter.bits_out[7]);

    shifter
}

pub struct RightShift4Ctrl {
    pub pass_inv: [Node; 2],
    pub shift_inv: Node,
    pub rotate_inv: Node,
    pub sign_ext_inv: Node,
}

struct RightShift4 {
    pub bits_in: [Node; 8],
    pub bits_out_inv: [Node; 8],
    pub ctrl: RightShift4Ctrl,
}

fn make_right_shift_4(builder: &mut CircuitBuilder) -> RightShift4 {
    let shifter = RightShift4 {
        bits_in: builder.nodes("bits_in"),
        bits_out_inv: builder.nodes("bits_out_inv"),
        ctrl: RightShift4Ctrl {
            pass_inv: builder.nodes("pass_inv"),
            shift_inv: builder.node("shift_inv"),
            rotate_inv: builder.node("rotate_inv"),
            sign_ext_inv: builder.node("sign_ext_inv"),
        },
    };
    let ctrl = &shifter.ctrl;

    for bit in 0..4 {
        builder.transistor(
            ctrl.pass_inv[0],
            shifter.bits_in[bit],
            shifter.bits_out_inv[bit],
        );
        builder.transistor(
            ctrl.shift_inv,
            shifter.bits_in[bit + 4],
            shifter.bits_out_inv[bit],
        );
        builder.pullup(shifter.bits_out_inv[bit]);
    }
    for bit in 4..8 {
        builder.transistor(
            ctrl.pass_inv[1],
            shifter.bits_in[bit],
            shifter.bits_out_inv[bit],
        );
        builder.transistor(
            ctrl.rotate_inv,
            shifter.bits_in[bit - 4],
            shifter.bits_out_inv[bit],
        );
        builder.transistor(
            ctrl.sign_ext_inv,
            shifter.bits_in[7],
            shifter.bits_out_inv[bit],
        );
        builder.pullup(shifter.bits_out_inv[bit]);
    }

    shifter
}

pub struct LogicUnit {
    pub bits_in_inv: [Node; 8],
    pub bits_out: [Node; 8],

    pub reverser_1_ctrl: ReverserCtrl,
    pub shift_4_ctrl: RightShift4Ctrl,
    pub shift_2_ctrl: RightShift2Ctrl,
    pub shift_1_ctrl: RightShift1Ctrl,
    pub reverser_2_ctrl: ReverserCtrl,
}

pub fn make_logic_unit(builder: &mut CircuitBuilder) -> LogicUnit {
    // (-) -> (-)reverser_1(+) -> (+)shift4(-) -> (-)shift2(+) -> (+)shift1(-) -> (-)reverser_2(+) -> (+)

    let reverser_1 = builder.subcircuit("reverser_1", &make_bit_reverser);
    let shift_4 = builder.subcircuit("shift_4", &make_right_shift_4);
    let shift_2 = builder.subcircuit("shift_2", &make_right_shift_2);
    let shift_1 = builder.subcircuit("shift_1", &make_right_shift_1);
    let reverser_2 = builder.subcircuit("reverser_2", &make_bit_reverser);

    let logic_unit = LogicUnit {
        bits_in_inv: builder.nodes("bits_in_inv"),
        bits_out: builder.nodes("bits_out"),
        reverser_1_ctrl: reverser_1.ctrl,
        shift_4_ctrl: shift_4.ctrl,
        shift_2_ctrl: shift_2.ctrl,
        shift_1_ctrl: shift_1.ctrl,
        reverser_2_ctrl: reverser_2.ctrl,
    };

    builder.join_bus(logic_unit.bits_in_inv, reverser_1.bits_in_inv);
    builder.join_bus(reverser_1.bits_out, shift_4.bits_in);
    builder.join_bus(shift_4.bits_out_inv, shift_2.bits_in_inv);
    builder.join_bus(shift_2.bits_out, shift_1.bits_in);
    builder.join_bus(shift_1.bits_out_inv, reverser_2.bits_in_inv);
    builder.join_bus(reverser_2.bits_out, logic_unit.bits_out);

    logic_unit
}
