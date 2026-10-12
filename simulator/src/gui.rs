use crate::circuit::{
    logic_sim::{CircuitSim, NodeState},
    NodeId,
};
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
                    self.config.sim.set_node_state(
                        input.node_id,
                        if self.input_states[index] {
                            NodeState::High
                        } else {
                            NodeState::Low
                        },
                    );
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
