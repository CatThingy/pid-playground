#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

mod app;
mod pid;
use crate::pid::Model;
use crate::app::Application;

#[cfg(not(target_family = "wasm"))]
fn main() {
    let options = eframe::NativeOptions::default();
    let mut app = Application::default();

    let mut model = Model::new("Model 1", app.last_model_id);
    app.last_model_id += 1;
    app.values.insert(model.id, model.evaluate(20.0, &app.env).to_vec());
    app.models.push(model);

    eframe::run_native(Box::new(app), options);
}
