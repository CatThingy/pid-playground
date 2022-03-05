#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod pid;

use eframe::{
    egui::{
        self,
        plot::{Value, Values},
    },
    epaint::Color32,
    epi,
};
use pid::PidController;

struct Application {
    controller: PidController,
    values: Vec<egui::plot::Value>,
}

impl epi::App for Application {
    fn name(&self) -> &str {
        "PID Playground"
    }
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        ctx.set_visuals(egui::Visuals::dark());
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("PID Playground");
            egui::plot::Plot::new("PID Graph")
                .width(ui.available_width())
                .height(ui.available_height())
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
                            .style(egui::plot::LineStyle::dashed_loose())
                    );
                });
        });

        egui::Window::new("Settings")
            .resizable(false)
            .default_pos([3000.0, 0.0])
            .show(ctx, |ui| {
                egui::Grid::new("Tuning")
                    .num_columns(2)
                    .striped(true)
                    .max_col_width(100.0)
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        ui.heading("Tuning");
                        ui.end_row();

                        ui.label("kP");
                        ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.k_p)
                                .speed(0.01)
                        );
                        ui.end_row();

                        ui.label("kI");
                        ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.k_i).speed(0.0001),
                        );
                        ui.end_row();

                        ui.label("kD");
                        ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.k_d).speed(0.01),
                        );
                        ui.end_row();
                    });

                egui::Grid::new("Simulation")
                    .num_columns(2)
                    .striped(true)
                    .max_col_width(100.0)
                    .min_col_width(100.0)
                    .show(ui, |ui| {
                        ui.heading("Simulation");
                        ui.end_row();

                        ui.label("Damping");
                        ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.env.damping)
                                .speed(0.05)
                                .clamp_range(0.0..=100.0),
                        );
                        ui.end_row();

                        ui.label("Applied force");
                        ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.env.applied_force)
                                .speed(0.05)
                                .clamp_range(-10.0..=10.0),
                        );
                        ui.end_row();

                        ui.label("Time step");
                        ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.env.timestep)
                                .speed(0.005)
                                .clamp_range(0.001..=0.2),
                        );
                        ui.end_row();

                        ui.label("Setpoint");
                        ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.setpoint)
                                .speed(1)
                                .clamp_range(0.0..=125.0),
                        );
                        ui.end_row();

                        ui.label("Max. acceleration");
                        ui.add_sized(
                            ui.available_size(),
                            egui::widgets::DragValue::new(&mut self.controller.env.max_accel)
                                .speed(0.1)
                                .clamp_range(0.1..=50.0),
                        );
                        ui.end_row();
                    });
            });

        // Resize the native window to be just the size we need it to be:
        frame.set_window_size(ctx.used_size());
        self.controller.reset();
        self.values = self.controller.evaluate(20.0);
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        Box::new(Application {
            controller: PidController::default(),
            values: Vec::<Value>::new(),
        }),
        options,
    );
}
