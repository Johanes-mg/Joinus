use eframe::egui;
use std::collections::HashMap;
use std::path::Path;

pub const FOND: egui::Color32 = egui::Color32::from_rgb(0x13, 0x15, 0x14);
pub const TEXTE: egui::Color32 = egui::Color32::from_rgb(0xe9, 0xea, 0xe5);
pub const SURFACE: egui::Color32 = egui::Color32::from_rgb(0xf7, 0xf7, 0xf5);
pub const SECONDAIRE: egui::Color32 = egui::Color32::from_rgb(0x69, 0x6a, 0x65);
pub const ERREUR: egui::Color32 = egui::Color32::from_rgb(0xc0, 0x50, 0x50);

pub const ESPACE_S: f32 = 6.0;
pub const ESPACE_M: f32 = 12.0;
pub const ESPACE_L: f32 = 18.0;
pub const ESPACE_XL: f32 = 24.0;

pub const AVATAR_TAILLE: f32 = 140.0;
pub const AVATAR_LISTE_TAILLE: f32 = 36.0;
pub const CARTE_PAIR_HAUTEUR: f32 = 52.0;
pub const CARTE_PAIR_ROUNDING: f32 = 8.0;
pub const CHAMP_SAISIE_HAUTEUR: f32 = 100.0;
pub const LIMITE_MESSAGE: usize = 4096;
pub const BOUTON_ENVOYER_LARGEUR: f32 = 110.0;
pub const BOUTON_ENVOYER_ICONE: f32 = 48.0;
pub const BOUTON_RAFRAICHIR_TAILLE: f32 = 28.0;
pub const BOUTON_RAFRAICHIR_ICONE: f32 = 20.0;

pub fn carte_pair_fond(sombre: bool) -> egui::Color32 {
    if sombre {
        egui::Color32::from_rgb(0x1c, 0x1e, 0x1c)
    } else {
        egui::Color32::from_rgb(0xea, 0xea, 0xe6)
    }
}

pub fn carte_pair_fond_survol(sombre: bool) -> egui::Color32 {
    if sombre {
        egui::Color32::from_rgb(0x2a, 0x2d, 0x2a)
    } else {
        egui::Color32::from_rgb(0xdd, 0xde, 0xd9)
    }
}

pub fn carte_pair_fond_actif(sombre: bool) -> egui::Color32 {
    if sombre {
        egui::Color32::from_rgb(0x3a, 0x3d, 0x3a)
    } else {
        egui::Color32::from_rgb(0xcf, 0xd0, 0xcb)
    }
}

pub fn bulle_moi(sombre: bool) -> egui::Color32 {
    if sombre {
        egui::Color32::from_rgb(0x2a, 0x2d, 0x2a)
    } else {
        egui::Color32::from_rgb(0xd8, 0xd9, 0xd4)
    }
}

pub fn bulle_autre(sombre: bool) -> egui::Color32 {
    if sombre {
        egui::Color32::from_rgb(0x1c, 0x1e, 0x1c)
    } else {
        egui::Color32::from_rgb(0xe9, 0xea, 0xe5)
    }
}

pub fn fond_avatar(sombre: bool) -> egui::Color32 {
    if sombre {
        egui::Color32::from_rgb(0x3a, 0x3d, 0x3a)
    } else {
        egui::Color32::from_rgb(0xcf, 0xd0, 0xcb)
    }
}

pub fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

pub struct CacheTextures {
    entrees: HashMap<u64, egui::TextureHandle>,
}

impl CacheTextures {
    pub fn new() -> Self {
        Self {
            entrees: HashMap::new(),
        }
    }

    pub fn obtenir_avec_hash(
        &mut self,
        ctx: &egui::Context,
        bytes: &[u8],
        hash: u64,
        nom: &str,
    ) -> Option<egui::TextureHandle> {
        if let Some(t) = self.entrees.get(&hash) {
            return Some(t.clone());
        }
        let img = image::load_from_memory(bytes).ok()?.to_rgba8();
        let (w, h) = img.dimensions();
        let couleur =
            egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw());
        let handle = ctx.load_texture(
            nom,
            couleur,
            egui::TextureOptions {
                magnification: egui::TextureFilter::Linear,
                minification: egui::TextureFilter::Linear,
                wrap_mode: egui::TextureWrapMode::ClampToEdge,
            },
        );
        self.entrees.insert(hash, handle.clone());
        Some(handle)
    }
}

pub fn charger_texture_fichier(
    ctx: &egui::Context,
    chemin: &Path,
    nom: &str,
) -> Option<egui::TextureHandle> {
    let bytes = std::fs::read(chemin).ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let img = image::DynamicImage::ImageRgba8(img.to_rgba8());
    let (w, h) = (img.width(), img.height());
    let cote_min = w.min(h);
    let img = if cote_min < 64 {
        let cible = 128u32;
        let ratio = cible as f32 / cote_min as f32;
        let nw = (w as f32 * ratio).round() as u32;
        let nh = (h as f32 * ratio).round() as u32;
        img.resize(nw, nh, image::imageops::FilterType::Lanczos3)
    } else {
        img
    };
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let couleur = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
    Some(ctx.load_texture(
        nom,
        couleur,
        egui::TextureOptions {
            magnification: egui::TextureFilter::Linear,
            minification: egui::TextureFilter::Linear,
            wrap_mode: egui::TextureWrapMode::ClampToEdge,
        },
    ))
}

pub fn appliquer_theme(ctx: &egui::Context, sombre: bool) {
    let mut visuels = if sombre {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };

    if sombre {
        visuels.panel_fill = FOND;
        visuels.window_fill = FOND;
        visuels.extreme_bg_color = egui::Color32::from_rgb(0x0a, 0x0b, 0x0a);
        visuels.faint_bg_color = egui::Color32::from_rgb(0x1c, 0x1e, 0x1c);
        visuels.override_text_color = Some(TEXTE);
        visuels.widgets.noninteractive.bg_stroke.color = SECONDAIRE;
        visuels.widgets.inactive.bg_fill = egui::Color32::from_rgb(0x1c, 0x1e, 0x1c);
        visuels.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(0x1c, 0x1e, 0x1c);
        visuels.widgets.inactive.fg_stroke.color = TEXTE;
        visuels.widgets.hovered.bg_fill = egui::Color32::from_rgb(0x2a, 0x2d, 0x2a);
        visuels.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(0x2a, 0x2d, 0x2a);
        visuels.widgets.hovered.fg_stroke.color = TEXTE;
        visuels.widgets.active.bg_fill = egui::Color32::from_rgb(0x3a, 0x3d, 0x3a);
        visuels.widgets.active.weak_bg_fill = egui::Color32::from_rgb(0x3a, 0x3d, 0x3a);
        visuels.widgets.active.fg_stroke.color = TEXTE;
        visuels.selection.bg_fill = SECONDAIRE;
        visuels.selection.stroke.color = TEXTE;
        visuels.hyperlink_color = SURFACE;
    } else {
        visuels.panel_fill = SURFACE;
        visuels.window_fill = SURFACE;
        visuels.extreme_bg_color = egui::Color32::WHITE;
        visuels.faint_bg_color = egui::Color32::from_rgb(0xe9, 0xea, 0xe5);
        visuels.override_text_color = Some(FOND);
        visuels.widgets.noninteractive.bg_stroke.color = SECONDAIRE;
        visuels.widgets.inactive.bg_fill = egui::Color32::from_rgb(0xea, 0xea, 0xe6);
        visuels.widgets.inactive.weak_bg_fill = egui::Color32::from_rgb(0xea, 0xea, 0xe6);
        visuels.widgets.inactive.fg_stroke.color = FOND;
        visuels.widgets.hovered.bg_fill = egui::Color32::from_rgb(0xdd, 0xde, 0xd9);
        visuels.widgets.hovered.weak_bg_fill = egui::Color32::from_rgb(0xdd, 0xde, 0xd9);
        visuels.widgets.hovered.fg_stroke.color = FOND;
        visuels.widgets.active.bg_fill = egui::Color32::from_rgb(0xcf, 0xd0, 0xcb);
        visuels.widgets.active.weak_bg_fill = egui::Color32::from_rgb(0xcf, 0xd0, 0xcb);
        visuels.widgets.active.fg_stroke.color = FOND;
        visuels.selection.bg_fill = SECONDAIRE;
        visuels.selection.stroke.color = SURFACE;
        visuels.hyperlink_color = SECONDAIRE;
    }

    ctx.set_visuals(visuels);
}

fn mesh_disque(
    texture: &egui::TextureHandle,
    centre: egui::Pos2,
    rayon: f32,
    segments: usize,
) -> egui::Mesh {
    let mut mesh = egui::Mesh::with_texture(texture.id());
    let centre_idx = mesh.vertices.len() as u32;
    mesh.vertices.push(egui::epaint::Vertex {
        pos: centre,
        uv: egui::pos2(0.5, 0.5),
        color: egui::Color32::WHITE,
    });
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        mesh.vertices.push(egui::epaint::Vertex {
            pos: centre + egui::vec2(angle.cos(), angle.sin()) * rayon,
            uv: egui::pos2(0.5 + 0.5 * angle.cos(), 0.5 + 0.5 * angle.sin()),
            color: egui::Color32::WHITE,
        });
    }
    for i in 0..segments {
        let a = centre_idx + 1 + i as u32;
        let b = centre_idx + 1 + ((i + 1) % segments) as u32;
        mesh.indices.push(centre_idx);
        mesh.indices.push(a);
        mesh.indices.push(b);
    }
    mesh
}

pub fn dessiner_avatar_arrondi(
    ui: &egui::Ui,
    rect: egui::Rect,
    texture: Option<&egui::TextureHandle>,
    initiale: Option<char>,
    sombre: bool,
) {
    let painter = ui.painter();
    let centre = rect.center();
    let rayon = rect.width().min(rect.height()) / 2.0;
    let couleur_texte = if sombre { TEXTE } else { FOND };

    painter.circle_filled(centre, rayon, fond_avatar(sombre));

    if let Some(tex) = texture {
        painter.add(egui::Shape::mesh(mesh_disque(tex, centre, rayon, 64)));
    } else {
        let texte = initiale
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string());
        painter.text(
            centre,
            egui::Align2::CENTER_CENTER,
            texte,
            egui::FontId::proportional(rayon * 0.9),
            couleur_texte,
        );
    }

    painter.circle_stroke(centre, rayon, egui::Stroke::new(2.0_f32, couleur_texte));
}

pub fn dessiner_avatar_petit(
    ui: &egui::Ui,
    rect: egui::Rect,
    texture: Option<&egui::TextureHandle>,
    initiale: Option<char>,
    couleur_texte: egui::Color32,
    sombre: bool,
) {
    let painter = ui.painter();
    let centre = rect.center();
    let rayon = rect.width().min(rect.height()) / 2.0;

    painter.circle_filled(centre, rayon, fond_avatar(sombre));

    if let Some(tex) = texture {
        painter.add(egui::Shape::mesh(mesh_disque(tex, centre, rayon, 32)));
    } else {
        let texte = initiale
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "?".to_string());
        painter.text(
            centre,
            egui::Align2::CENTER_CENTER,
            texte,
            egui::FontId::proportional(rayon * 1.1),
            couleur_texte,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_bytes_deterministe() {
        assert_eq!(hash_bytes(b"abc"), hash_bytes(b"abc"));
    }

    #[test]
    fn hash_bytes_different() {
        assert_ne!(hash_bytes(b"abc"), hash_bytes(b"abd"));
    }

    #[test]
    fn hash_bytes_vide() {
        assert_eq!(hash_bytes(b""), 0xcbf29ce484222325);
    }

    #[test]
    fn couleurs_sombres_differentes_de_claires() {
        assert_ne!(carte_pair_fond(true), carte_pair_fond(false));
        assert_ne!(bulle_moi(true), bulle_moi(false));
        assert_ne!(bulle_autre(true), bulle_autre(false));
    }
}
