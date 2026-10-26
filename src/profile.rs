use crate::persistence::{self, Session};
use crate::theme::CacheTextures;
use eframe::egui;

pub enum ErreurAvatar {
    Lecture,
    Taille,
    Format,
    Ecriture,
}

pub struct Profil {
    pub nom: String,
    pub avatar_bytes: Option<Vec<u8>>,
    pub avatar_hash: u64,
    pub erreur: Option<ErreurAvatar>,
}

impl Profil {
    pub fn charger(session: &Session) -> Self {
        let avatar_bytes = persistence::charger_avatar();
        let avatar_hash = avatar_bytes
            .as_ref()
            .map(|b| crate::theme::hash_bytes(b))
            .unwrap_or(0);
        Self {
            nom: session.nom_affiche.clone(),
            avatar_bytes,
            avatar_hash,
            erreur: None,
        }
    }

    pub fn importer_avatar(&mut self) {
        let choix = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg"])
            .pick_file();
        let Some(chemin) = choix else {
            return;
        };
        let Ok(bytes) = std::fs::read(&chemin) else {
            self.erreur = Some(ErreurAvatar::Lecture);
            return;
        };
        if bytes.len() > 5 * 1024 * 1024 {
            self.erreur = Some(ErreurAvatar::Taille);
            return;
        }
        let Ok(img) = image::load_from_memory(&bytes) else {
            self.erreur = Some(ErreurAvatar::Format);
            return;
        };
        let carre = recadrer_carre(img);
        let mut buffer = std::io::Cursor::new(Vec::new());
        let encodeur = image::codecs::png::PngEncoder::new(&mut buffer);
        use image::ImageEncoder;
        let (w, h) = carre.dimensions();
        if encodeur
            .write_image(carre.as_raw(), w, h, image::ExtendedColorType::Rgba8)
            .is_err()
        {
            self.erreur = Some(ErreurAvatar::Ecriture);
            return;
        }
        let bytes_carres = buffer.into_inner();
        if persistence::sauver_avatar(&bytes_carres) {
            self.avatar_hash = crate::theme::hash_bytes(&bytes_carres);
            self.avatar_bytes = Some(bytes_carres);
            self.erreur = None;
        } else {
            self.erreur = Some(ErreurAvatar::Ecriture);
        }
    }

    pub fn texture(&self, ctx: &egui::Context, cache: &mut CacheTextures) -> Option<egui::TextureHandle> {
        let bytes = self.avatar_bytes.as_ref()?;
        cache.obtenir_avec_hash(ctx, bytes, self.avatar_hash, "avatar")
    }

    pub fn initiale(&self) -> Option<char> {
        self.nom.chars().find(|c| !c.is_whitespace())
    }
}

fn recadrer_carre(img: image::DynamicImage) -> image::RgbaImage {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let cote = w.min(h);
    let x = (w - cote) / 2;
    let y = (h - cote) / 2;
    image::imageops::crop_imm(&rgba, x, y, cote, cote).to_image()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recadrer_carre_image_carree() {
        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 100));
        let carre = recadrer_carre(img);
        assert_eq!(carre.dimensions(), (100, 100));
    }

    #[test]
    fn recadrer_carre_image_paysage() {
        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(200, 100));
        let carre = recadrer_carre(img);
        assert_eq!(carre.dimensions(), (100, 100));
    }

    #[test]
    fn recadrer_carre_image_portrait() {
        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::new(100, 200));
        let carre = recadrer_carre(img);
        assert_eq!(carre.dimensions(), (100, 100));
    }
}
