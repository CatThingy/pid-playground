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
        let mut all_dirty = false;

        if let Some(_) = frame.info().prefer_dark_mode {
            ctx.set_visuals(egui::Visuals::dark());
        }

        let screen = ctx.input().screen_rect();


        if screen.width() > screen.height() {
            egui::SidePanel::right("Settings")
                .min_width(215.0)
                .max_width(215.0)
                .resizable(false)
                .show(ctx, |ui| {
                    self.info_panel(ui, &mut all_dirty, true);
                });
        } else {
            egui::TopBottomPanel::bottom("Settings")
                .min_height(200.0)
                .max_height(200.0)
                .resizable(false)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            self.info_panel(ui, &mut all_dirty, false);
                        });
                });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading(format!("{screen:?}"));
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
                    Some(v) => update_sim(model, v),
                    None => {
                        let mut v: Vec<Value> = vec![];
                        update_sim(model, &mut v);
                        self.values.insert(model.id, v);
                    }
                }
            }
            // Continue simulation on the next frame
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

impl Application {
    fn info_panel(&mut self, ui: &mut Ui, all_dirty: &mut bool, is_vertical: bool) {
        ui.heading("Tuning");
        ui.separator();

        let mut duplicated_models = Vec::<Model>::new();
        let mut deleted_models = Vec::<u64>::new();

        let scroll = match is_vertical {
            true => egui::ScrollArea::vertical().max_height(300.0),
            false => egui::ScrollArea::horizontal().max_height(200.0),
        };

        // The same stuff will be displayed, just spaced in different directions
        let mut scroll_item = |ui: &mut Ui, app: &mut Application| {
            for model in app.models.iter_mut() {
                ui.separator();
                ui.vertical(|ui| {
                    tuning_ui(model, ui);
                    if ui.button("Duplicate").clicked() {
                        let mut new_model = model.clone();
                        new_model.dirty = true;
                        new_model.id = app.last_model_id;
                        app.last_model_id += 1;
                        app.values.insert(new_model.id, vec![]);
                        duplicated_models.push(new_model);
                    }

                    if ui.button("Delete").clicked() {
                        deleted_models.push(model.id);
                    }
                });
            }
        };

        scroll.show(ui, |ui| match is_vertical {
            true => {
                scroll_item(ui, self);
            }
            false => {
                ui.horizontal(|ui| scroll_item(ui, self));
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

        ui.separator();

        egui::Grid::new("Simulation")
            .num_columns(2)
            .striped(true)
            .max_col_width(100.0)
            .min_col_width(100.0)
            .show(ui, |ui| {
                ui.heading("Simulation");
                ui.end_row();

                let size = Vec2::new(80.0, ui.available_height());

                ui.label("Damping");
                let damp_res = ui.add_sized(
                    size,
                    egui::widgets::DragValue::new(&mut self.env.damping)
                        .speed(0.05)
                        .clamp_range(0.0..=100.0),
                );
                ui.end_row();
                ui.label("Setpoint");
                let setpoint_res = ui.add_sized(
                    size,
                    egui::widgets::DragValue::new(&mut self.env.setpoint)
                        .speed(1)
                        .clamp_range(0.0..=150.0),
                );
                ui.end_row();

                ui.label("Applied force");
                let force_res = ui.add_sized(
                    size,
                    egui::widgets::DragValue::new(&mut self.env.applied_force)
                        .speed(0.05)
                        .clamp_range(-10.0..=10.0),
                );
                ui.end_row();

                ui.label("Time step");
                let timestep_res = ui.add_sized(
                    size,
                    egui::widgets::DragValue::new(&mut self.env.timestep)
                        .speed(0.005)
                        .clamp_range(0.001..=1.0),
                );
                ui.end_row();

                ui.end_row();

                ui.label("Realtime sim.");
                let checkbox_res = ui.checkbox(&mut self.realtime, "");

                if !*all_dirty
                    && (damp_res.changed()
                        || force_res.changed()
                        || setpoint_res.changed()
                        || timestep_res.changed()
                        || (checkbox_res.changed() && !self.realtime))
                {
                    *all_dirty = true;
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
fn update_sim(m: &mut Model, v: &mut Vec<Value>) {
    v.push(Value {
        x: m.elapsed_time,
        y: m.value,
    });
    if m.elapsed_time > 20.0 {
        m.elapsed_time = 20.0;
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
