use crate::pid::PidController;

use eframe::{
    egui::{
        self,
        plot::{Value, Values},
    },
    epaint::Color32,
    epi,
};

pub struct Application {
    pub controller: PidController,
    pub values: Vec<Value>,
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

        let mut dirty = false;

        egui::SidePanel::right("Settings")
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("");
                egui::Grid::new("Tuning")
                    .num_columns(2)
                    .striped(true)
                    .max_col_width(100.0)
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        ui.heading("Tuning");
                        ui.end_row();

                        ui.label("kP");
                        let k_p_res = ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.k_p).speed(0.01),
                        );
                        ui.end_row();

                        ui.label("kI");
                        let k_i_res = ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.k_i).speed(0.0001),
                        );
                        ui.end_row();

                        ui.label("kD");
                        let k_d_res = ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.k_d).speed(0.01),
                        );
                        ui.end_row();

                        if !dirty && (k_p_res.changed() || k_i_res.changed() || k_d_res.changed()) {
                            dirty = true;
                        }
                    });

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
                            egui::widgets::DragValue::new(&mut self.controller.env.damping)
                                .speed(0.05)
                                .clamp_range(0.0..=100.0),
                        );
                        ui.end_row();

                        ui.label("Applied force");
                        let force_res = ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.env.applied_force)
                                .speed(0.05)
                                .clamp_range(-10.0..=10.0),
                        );
                        ui.end_row();

                        ui.label("Time step");
                        let timestep_res = ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.env.timestep)
                                .speed(0.005)
                                .clamp_range(0.001..=1.0),
                        );
                        ui.end_row();

                        ui.label("Setpoint");
                        let setpoint_res = ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.setpoint)
                                .speed(1)
                                .clamp_range(0.0..=150.0),
                        );
                        ui.end_row();

                        ui.label("Max. acceleration");
                        let max_accel_res = ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.env.max_accel)
                                .speed(0.1)
                                .clamp_range(0.1..=50.0),
                        );
                        ui.end_row();

                        if !dirty
                            && (damp_res.changed()
                                || force_res.changed()
                                || timestep_res.changed()
                                || setpoint_res.changed()
                                || max_accel_res.changed())
                        {
                            dirty = true;
                        }
                    });
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
                .include_x(25.0)
                .include_y(150.0)
                .include_y(0.0)
                .show(ui, |ui| {
                    ui.line(egui::plot::Line::new(Values::from_values(
                        self.values.to_vec(),
                    )));
                    ui.hline(
                        egui::plot::HLine::new(self.controller.setpoint)
                            .color(Color32::from_rgb(196, 64, 64))
                            .style(egui::plot::LineStyle::dashed_loose()),
                    );
                });
        });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());

        if dirty {
            self.controller.reset();
            self.values = self.controller.evaluate(20.0);
        }
    }
}
