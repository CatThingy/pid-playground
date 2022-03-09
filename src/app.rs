use std::collections::HashMap;

use crate::pid::{Environment, Model};

use eframe::{
    egui::{
        self,
        plot::{Value, Values},
        Response, Ui,
    },
    epaint::Vec2,
    epi,
};

#[derive(Default)]
pub struct Application {
    pub models: Vec<Model>,
    pub env: Environment,
    pub values: HashMap<u64, Vec<Value>>,
    pub realtime: bool,
    pub last_model_id: u64,
}

impl epi::App for Application {
    fn name(&self) -> &str {
        "PID Playground"
    }

    fn max_size_points(&self) -> egui::Vec2 {
        egui::vec2(1920.0, 1080.0)
    }

    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        let mut all_dirty = false;

        egui::SidePanel::right("Settings")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Tuning");
                ui.separator();

                let mut duplicated_models = Vec::<Model>::new();
                let mut deleted_models = Vec::<u64>::new();

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for model in self.models.iter_mut() {
                            tuning_ui(model, ui);
                            if ui.button("Duplicate").clicked() {
                                let mut new_model = model.clone();
                                new_model.dirty = true;
                                new_model.id = self.last_model_id;
                                self.last_model_id += 1;
                                self.values.insert(new_model.id, vec![]);
                                duplicated_models.push(new_model);
                            }

                            if ui.button("Delete").clicked() {
                                deleted_models.push(model.id);
                            }
                            ui.separator();
                        }
                        if ui.button("Add new model").clicked() {
                            let mut model = Model::new(
                                &("Model ".to_owned() + &self.last_model_id.to_string()),
                                self.last_model_id,
                            );

                            model.dirty = true;
                            self.last_model_id += 1;
                            self.values.insert(model.id, vec![]);
                            self.models.push(model);
                        }
                    });

                for model in duplicated_models {
                    self.models.push(model);
                }

                for id in deleted_models {
                    self.models = self
                        .models
                        .to_vec()
                        .into_iter()
                        .filter(|v| v.id != id)
                        .collect::<Vec<Model>>();
                }

                ui.separator();

                egui::Grid::new("Simulation")
                    .num_columns(2)
                    .striped(true)
                    .max_col_width(100.0)
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        ui.heading("Simulation");
                        ui.end_row();

                        ui.label("Damping");
                        let damp_res = ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.env.damping)
                                .speed(0.05)
                                .clamp_range(0.0..=100.0),
                        );
                        ui.end_row();
                        ui.label("Setpoint");
                        let setpoint_res = ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.env.setpoint)
                                .speed(1)
                                .clamp_range(0.0..=150.0),
                        );
                        ui.end_row();

                        ui.label("Applied force");
                        let force_res = ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.env.applied_force)
                                .speed(0.05)
                                .clamp_range(-10.0..=10.0),
                        );
                        ui.end_row();

                        ui.label("Time step");
                        let timestep_res = ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.env.timestep)
                                .speed(0.005)
                                .clamp_range(0.001..=1.0),
                        );
                        ui.end_row();

                        ui.end_row();

                        ui.label("Realtime sim.");
                        let checkbox_res = ui.checkbox(&mut self.realtime, "");

                        if !all_dirty
                            && (damp_res.changed()
                                || force_res.changed()
                                || setpoint_res.changed()
                                || timestep_res.changed()
                                || (checkbox_res.changed() && !self.realtime))
                        {
                            all_dirty = true;
                        }
                    });
                if ui
                    .add_enabled(
                        self.realtime,
                        egui::widgets::Button::new("Reset simulation"),
                    )
                    .clicked()
                {
                    for model in self.models.iter_mut() {
                        model.reset();
                        if let Some(v) = self.values.get_mut(&model.id) {
                            v.clear();
                        };
                    }
                };

                egui::TopBottomPanel::bottom("TEST")
                    .frame(egui::Frame::none())
                    .show_inside(ui, |ui| {
                        ui.separator();
                        ui.label("Hello!");
                    })
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PID Playground");
            egui::plot::Plot::new("PID Graph")
                .allow_zoom(false)
                .show_x(false)
                .show_y(false)
                .allow_drag(false)
                .allow_boxed_zoom(false)
                .include_x(0.0)
                .include_x(20.0)
                .include_y(175.0)
                .include_y(0.0)
                .legend(egui::plot::Legend::default())
                .show(ui, |ui| {
                    ui.hline(
                        egui::plot::HLine::new(self.env.setpoint)
                            .style(egui::plot::LineStyle::dashed_loose())
                            .name("Setpoint"),
                    );
                    for model in self.models.iter() {
                        if let Some(v) = self.values.get(&model.id) {
                            // Add ZWSPs to ensure proper ordering
                            let mut name: String = "\u{200b}".to_string();
                            for _ in 0..model.id {
                                name.push('\u{200b}');
                            }
                            name.push_str(&model.name);
                            ui.line(
                                egui::plot::Line::new(Values::from_values(v.to_vec())).name(name),
                            );
                        }
                    }
                });
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());

        if !self.realtime && all_dirty {
            for model in self.models.iter_mut() {
                model.reset();
                match self.values.get_mut(&model.id) {
                    Some(v) => {
                        *v = model.evaluate(20.0, &self.env);
                    }
                    None => {
                        self.values
                            .insert(model.id, model.evaluate(20.0, &self.env));
                    }
                }
            }
        } else if self.realtime {
            for model in self.models.iter_mut() {
                model.update(&self.env, 0.016);
                match self.values.get_mut(&model.id) {
                    Some(v) => {
                        v.push(Value {
                            x: model.elapsed_time,
                            y: model.value,
                        });
                        if model.elapsed_time > 20.0 {
                            model.elapsed_time = 20.0;
                            *v = v
                                .iter()
                                .map(|v| Value {
                                    x: v.x - 0.016,
                                    y: v.y,
                                })
                                .filter(|v| v.x > 0.0)
                                .collect::<Vec<Value>>();
                        }
                    }

                    None => {
                        let mut v: Vec<Value> = vec![];
                        v.push(Value {
                            x: model.elapsed_time,
                            y: model.value,
                        });

                        if model.elapsed_time > 20.0 {
                            model.elapsed_time = 20.0;
                            v = v
                                .iter()
                                .map(|v| Value {
                                    x: v.x - 0.016,
                                    y: v.y,
                                })
                                .filter(|v| v.x > 0.0)
                                .collect::<Vec<Value>>();
                        }
                        self.values.insert(model.id, v);
                    }
                }
            }
            ctx.request_repaint();
        } else {
            for model in self.models.iter_mut() {
                if model.dirty {
                    model.reset();
                    match self.values.get_mut(&model.id) {
                        Some(v) => {
                            *v = model.evaluate(20.0, &self.env);
                        }
                        None => {
                            self.values
                                .insert(model.id, model.evaluate(20.0, &self.env));
                        }
                    }
                }
                model.dirty = false;
            }
        }
    }
}

fn tuning_ui(model: &mut Model, ui: &mut Ui) -> Response {
    egui::Grid::new(model.id)
        .num_columns(2)
        .striped(true)
        .max_col_width(100.0)
        .min_col_width(100.0)
        .show(ui, |ui| {
            let size = Vec2::new(80.0, ui.available_height());

            ui.label("Name");
            ui.add_sized(size, egui::widgets::TextEdit::singleline(&mut model.name));
            ui.end_row();

            ui.label("kP");
            let k_p_res = ui.add_sized(
                size,
                egui::widgets::DragValue::new(&mut model.controller.k_p).speed(0.01),
            );
            ui.end_row();

            ui.label("kI");
            let k_i_res = ui.add_sized(
                size,
                egui::widgets::DragValue::new(&mut model.controller.k_i).speed(0.0001),
            );
            ui.end_row();

            ui.label("kD");
            let k_d_res = ui.add_sized(
                size,
                egui::widgets::DragValue::new(&mut model.controller.k_d).speed(0.01),
            );
            ui.end_row();

            ui.label("Max. acceleration");
            let max_accel_res = ui.add_sized(
                size,
                egui::widgets::DragValue::new(&mut model.max_accel)
                    .speed(0.1)
                    .clamp_range(0.1..=50.0),
            );

            if !model.dirty
                && (k_p_res.changed()
                    || max_accel_res.changed()
                    || k_i_res.changed()
                    || k_d_res.changed())
            {
                model.dirty = true;
            }
        })
        .response
}
