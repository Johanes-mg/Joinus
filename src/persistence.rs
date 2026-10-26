use crate::i18n::Langue;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Session {
    pub nom_affiche: String,
    pub langue: Langue,
    pub sombre: bool,
    #[serde(default = "taille_fenetre_defaut")]
    pub window_size: [f32; 2],
    #[serde(default = "id_defaut")]
    pub id: String,
}

fn taille_fenetre_defaut() -> [f32; 2] {
    [1100.0, 700.0]
}

fn id_defaut() -> String {
    Uuid::new_v4().to_string()
}

impl Default for Session {
    fn default() -> Self {
        Self {
            nom_affiche: "Utilisateur".to_string(),
            langue: Langue::Fr,
            sombre: true,
            window_size: taille_fenetre_defaut(),
            id: id_defaut(),
        }
    }
}

pub fn dossier_app() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        let p = PathBuf::from(appdata).join("Joinus");
        if std::fs::create_dir_all(&p).is_ok() {
            return p;
        }
    }
    let p = std::env::temp_dir().join("Joinus");
    let _ = std::fs::create_dir_all(&p);
    p
}

pub fn dossier_historique() -> PathBuf {
    let p = dossier_app().join("historique");
    let _ = std::fs::create_dir_all(&p);
    p
}

fn chemin_session() -> PathBuf {
    dossier_app().join("session.json")
}

pub fn charger_session() -> Session {
    let chemin = chemin_session();
    if let Ok(texte) = std::fs::read_to_string(&chemin) {
        if let Ok(s) = serde_json::from_str::<Session>(&texte) {
            return s;
        }
    }
    Session::default()
}

pub fn sauver_session(session: &Session) {
    let chemin = chemin_session();
    if let Ok(texte) = serde_json::to_string_pretty(session) {
        let _ = std::fs::write(chemin, texte);
    }
}

pub fn chemin_avatar() -> PathBuf {
    dossier_app().join("avatar.dat")
}

pub fn charger_avatar() -> Option<Vec<u8>> {
    let bytes = std::fs::read(chemin_avatar()).ok()?;
    if image::load_from_memory(&bytes).is_err() {
        return None;
    }
    Some(bytes)
}

pub fn sauver_avatar(bytes: &[u8]) -> bool {
    std::fs::write(chemin_avatar(), bytes).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_defaut_valeurs() {
        let s = Session::default();
        assert_eq!(s.nom_affiche, "Utilisateur");
        assert_eq!(s.langue, Langue::Fr);
        assert!(s.sombre);
        assert_eq!(s.window_size, [1100.0, 700.0]);
        assert!(!s.id.is_empty());
    }

    #[test]
    fn session_roundtrip_json() {
        let s = Session {
            nom_affiche: "Test".to_string(),
            langue: Langue::En,
            sombre: false,
            window_size: [1000.0, 600.0],
            id: "abc-123".to_string(),
        };
        let json = serde_json::to_string(&s).unwrap();
        let s2: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(s.nom_affiche, s2.nom_affiche);
        assert_eq!(s.id, s2.id);
    }

    #[test]
    fn session_ancienne_version_genere_id() {
        let json = r#"{"nom_affiche":"Ancien","langue":"Fr","sombre":true}"#;
        let s: Session = serde_json::from_str(json).unwrap();
        assert!(!s.id.is_empty());
        assert_eq!(s.window_size, [1100.0, 700.0]);
    }
}
