use crate::i18n::Langue;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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
    [1150.0, 720.0]
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

fn chemin_alias() -> PathBuf {
    dossier_app().join("alias.json")
}

pub fn charger_alias() -> HashMap<String, String> {
    if let Ok(texte) = std::fs::read_to_string(chemin_alias()) {
        if let Ok(m) = serde_json::from_str::<HashMap<String, String>>(&texte) {
            return m;
        }
    }
    HashMap::new()
}

pub fn sauver_alias(alias: &HashMap<String, String>) {
    if let Ok(texte) = serde_json::to_string_pretty(alias) {
        let _ = std::fs::write(chemin_alias(), texte);
    }
}

pub fn supprimer_historique(pair_id: &str) {
    let chemin = dossier_historique().join(format!("{}.json", pair_id));
    let _ = std::fs::remove_file(chemin);
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
}
