#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
mod app;
mod pid;
use crate::pid::PidController;
use crate::app::Application;



#[cfg(not(target_family = "wasm"))]
fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        Box::new(Application {
            controller: PidController::default(),
            values: Vec::<eframe::egui::plot::Value>::new(),
        }),
        options,
    );
}
