#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod chat;
mod groupes;
mod i18n;
mod network;
mod peers;
mod persistence;
mod pieces_jointes;
mod profile;
mod theme;

use eframe::egui;
use std::path::PathBuf;

fn configurer_police(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let candidats = [
        "C:\\Windows\\Fonts\\trebuc.ttf",
        "C:\\Windows\\Fonts\\Candara.ttf",
        "C:\\Windows\\Fonts\\calibri.ttf",
        "C:\\Windows\\Fonts\\verdana.ttf",
    ];
    for chemin in candidats {
        if let Ok(bytes) = std::fs::read(chemin) {
            fonts.font_data.insert(
                "principale".to_string(),
                egui::FontData::from_owned(bytes),
            );
            fonts
                .families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "principale".to_string());
            break;
        }
    }
    ctx.set_fonts(fonts);
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(19.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(11.0, egui::FontFamily::Proportional),
    );
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.spacing.item_spacing = egui::vec2(8.0, 7.0);
    style.visuals.widgets.inactive.rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.hovered.rounding = egui::Rounding::same(8.0);
    style.visuals.widgets.active.rounding = egui::Rounding::same(8.0);
    ctx.set_style(style);
}

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
        .with_min_inner_size([900.0, 550.0])
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
        Box::new(move |cc| {
            configurer_police(&cc.egui_ctx);
            Ok(Box::new(app::App::new(cc, session)))
        }),
    )
}
