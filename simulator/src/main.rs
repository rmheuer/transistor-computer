mod adder;
mod circuit;
mod gates;
mod gui;
mod logic_unit;

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

fn main() -> eframe::Result {
    let mut builder = circuit::builder::CircuitBuilder::new();

    /*let adder = adder::make_adder(&mut builder);
    let inputs = [
        &[adder.carry_in_inv][..],
        &adder.input_a[..],
        &adder.input_b[..],
    ]
    .concat();
    let outputs = [&adder.sum_out_inv[..], &[adder.carry_out][..]].concat();
     */

    let lu = logic_unit::make_logic_unit(&mut builder);
    let inputs = [
        &lu.bits_in_inv[..],
        &lu.reverser_1_ctrl.pass_inv[..],
        &lu.reverser_1_ctrl.reverse_inv[..],
        &lu.shift_4_ctrl.pass_inv[..],
        &[lu.shift_4_ctrl.shift_inv][..],
        &[lu.shift_4_ctrl.rotate_inv][..],
        &[lu.shift_4_ctrl.sign_ext_inv][..],
        &lu.shift_2_ctrl.pass_inv[..],
        &lu.shift_2_ctrl.shift_inv[..],
        &[lu.shift_2_ctrl.sign_ext_inv][..],
        &[lu.shift_2_ctrl.rotate_inv][..],
        &[lu.shift_2_ctrl.zero_6_7][..],
        &lu.shift_1_ctrl.pass_1to6_inv[..],
        &[lu.shift_1_ctrl.pass_7_inv][..],
        &lu.shift_1_ctrl.shift_inv[..],
        &[lu.shift_1_ctrl.rotate_inv][..],
        &lu.reverser_2_ctrl.pass_inv[..],
        &lu.reverser_2_ctrl.reverse_inv[..],
    ]
    .concat();
    let outputs = [&lu.bits_out[..]].concat();

    let (circuit, node_map, node_names) = builder.build();
    let sim = circuit::logic_sim::CircuitSim::new(&circuit);
    gui::run_ui(gui::SimConfig {
        sim,
        inputs: &make_handles(inputs, &node_map, &node_names),
        outputs: &make_handles(outputs, &node_map, &node_names),
    })
}
