#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod chat;
mod i18n;
mod network;
mod peers;
mod persistence;
mod profile;
mod theme;

use eframe::egui;
use std::path::PathBuf;

fn charger_icone_fenetre() -> Option<egui::IconData> {
    let chemin = PathBuf::from("images").join("logo.png");
    let bytes = std::fs::read(chemin).ok()?;
    let img = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let (largeur, hauteur) = img.dimensions();
    Some(egui::IconData {
        rgba: img.into_raw(),
        width: largeur,
        height: hauteur,
    })
}

fn main() -> eframe::Result<()> {
    let session = persistence::charger_session();
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size(session.window_size)
        .with_min_inner_size([800.0, 500.0])
        .with_title("Joinus");
    if let Some(icone) = charger_icone_fenetre() {
        viewport = viewport.with_icon(icone);
    }
    let options = eframe::NativeOptions {
        viewport,
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "Joinus",
        options,
        Box::new(move |cc| Ok(Box::new(app::App::new(cc, session)))),
    )
}
