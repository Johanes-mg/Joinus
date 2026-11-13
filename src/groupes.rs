use crate::persistence;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Groupe {
    pub id: String,
    pub nom: String,
    pub membres: Vec<String>,
}

fn chemin() -> PathBuf {
    persistence::dossier_app().join("groupes.json")
}

pub fn charger() -> Vec<Groupe> {
    if let Ok(texte) = std::fs::read_to_string(chemin()) {
        if let Ok(v) = serde_json::from_str::<Vec<Groupe>>(&texte) {
            return v;
        }
    }
    Vec::new()
}

pub fn sauver(groupes: &[Groupe]) {
    if let Ok(texte) = serde_json::to_string_pretty(groupes) {
        let _ = std::fs::write(chemin(), texte);
    }
}

pub fn creer(nom: &str, membres: Vec<String>) -> Groupe {
    Groupe {
        id: format!("groupe-{}", Uuid::new_v4()),
        nom: nom.to_string(),
        membres,
    }
}
