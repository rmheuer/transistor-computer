mod adder;
mod circuit;
mod gates;
mod gui;

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

    let adder = adder::make_adder(&mut builder);
    let inputs = [
        &[adder.carry_in_inv][..],
        &adder.input_a[..],
        &adder.input_b[..],
    ]
    .concat();
    let outputs = [&adder.sum_out_inv[..], &[adder.carry_out][..]].concat();

    let (circuit, node_map, node_names) = builder.build();
    let sim = circuit::logic_sim::CircuitSim::new(&circuit);
    gui::run_ui(gui::SimConfig {
        sim,
        inputs: &make_handles(inputs, &node_map, &node_names),
        outputs: &make_handles(outputs, &node_map, &node_names),
    })
}
