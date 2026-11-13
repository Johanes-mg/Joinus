use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const LIMITE_PIECE_OCTETS: usize = 5 * 1024 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PieceJointe {
    pub nom: String,
    pub mime: String,
    pub donnees_b64: String,
}

impl PieceJointe {
    pub fn depuis_fichier(chemin: &Path) -> Result<Self, ErreurPiece> {
        let bytes = std::fs::read(chemin).map_err(|_| ErreurPiece::Lecture)?;
        if bytes.len() > LIMITE_PIECE_OCTETS {
            return Err(ErreurPiece::Taille);
        }
        let nom = chemin
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("fichier")
            .to_string();
        let mime = deviner_mime(&nom);
        Ok(Self {
            nom,
            mime,
            donnees_b64: BASE64.encode(&bytes),
        })
    }

    pub fn decoder(&self) -> Option<Vec<u8>> {
        BASE64.decode(self.donnees_b64.as_bytes()).ok()
    }

    pub fn est_image(&self) -> bool {
        self.mime.starts_with("image/")
    }

    pub fn taille_octets(&self) -> usize {
        self.donnees_b64.len() * 3 / 4
    }
}

pub enum ErreurPiece {
    Lecture,
    Taille,
}

fn deviner_mime(nom: &str) -> String {
    let bas = nom.to_lowercase();
    if bas.ends_with(".png") {
        "image/png".to_string()
    } else if bas.ends_with(".jpg") || bas.ends_with(".jpeg") {
        "image/jpeg".to_string()
    } else if bas.ends_with(".gif") {
        "image/gif".to_string()
    } else if bas.ends_with(".webp") {
        "image/webp".to_string()
    } else if bas.ends_with(".pdf") {
        "application/pdf".to_string()
    } else if bas.ends_with(".txt") {
        "text/plain".to_string()
    } else if bas.ends_with(".zip") {
        "application/zip".to_string()
    } else {
        "application/octet-stream".to_string()
    }
}
