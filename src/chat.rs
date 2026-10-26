use crate::network::Message;
use crate::persistence;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageLocal {
    pub de_moi: bool,
    pub texte: String,
    pub ts: i64,
}

pub struct Historique {
    par_pair: HashMap<String, Vec<MessageLocal>>,
    pair_actif: Option<String>,
}

impl Historique {
    pub fn new() -> Self {
        Self {
            par_pair: HashMap::new(),
            pair_actif: None,
        }
    }

    pub fn selectionner(&mut self, pair_id: &str) {
        if self.pair_actif.as_deref() != Some(pair_id) {
            self.charger_depuis_disque(pair_id);
            self.pair_actif = Some(pair_id.to_string());
        }
    }

    pub fn messages_actifs(&self) -> Option<&[MessageLocal]> {
        let id = self.pair_actif.as_ref()?;
        self.par_pair.get(id).map(|v| v.as_slice())
    }

    pub fn ajouter_local(&mut self, pair_id: &str, texte: String) {
        let ts = chrono_now();
        let msg = MessageLocal {
            de_moi: true,
            texte,
            ts,
        };
        self.par_pair.entry(pair_id.to_string()).or_default().push(msg);
        self.sauver_disque(pair_id);
    }

    pub fn ajouter_recu(&mut self, pair_id: &str, message: &Message) {
        let local = MessageLocal {
            de_moi: false,
            texte: message.texte.clone(),
            ts: message.ts,
        };
        self.par_pair.entry(pair_id.to_string()).or_default().push(local);
        self.sauver_disque(pair_id);
    }

    fn chemin(pair_id: &str) -> PathBuf {
        persistence::dossier_historique().join(format!("{}.json", pair_id))
    }

    fn charger_depuis_disque(&mut self, pair_id: &str) {
        if self.par_pair.contains_key(pair_id) {
            return;
        }
        let chemin = Self::chemin(pair_id);
        if let Ok(texte) = std::fs::read_to_string(&chemin) {
            if let Ok(v) = serde_json::from_str::<Vec<MessageLocal>>(&texte) {
                self.par_pair.insert(pair_id.to_string(), v);
            }
        }
    }

    fn sauver_disque(&self, pair_id: &str) {
        if let Some(v) = self.par_pair.get(pair_id) {
            if let Ok(texte) = serde_json::to_string_pretty(v) {
                let _ = std::fs::write(Self::chemin(pair_id), texte);
            }
        }
    }
}

fn chrono_now() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ajout_local_puis_lecture() {
        let mut h = Historique::new();
        h.selectionner("test_pair_unique");
        h.ajouter_local("test_pair_unique", "salut".to_string());
        let msgs = h.messages_actifs().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].texte, "salut");
        assert!(msgs[0].de_moi);
        let _ = std::fs::remove_file(Historique::chemin("test_pair_unique"));
    }

    #[test]
    fn ajout_recu() {
        let mut h = Historique::new();
        h.selectionner("test_pair_recu");
        let m = Message {
            de: "autre".to_string(),
            vers: "moi".to_string(),
            texte: "hello".to_string(),
            ts: 12345,
        };
        h.ajouter_recu("test_pair_recu", &m);
        let msgs = h.messages_actifs().unwrap();
        assert_eq!(msgs[0].texte, "hello");
        assert!(!msgs[0].de_moi);
        let _ = std::fs::remove_file(Historique::chemin("test_pair_recu"));
    }
}
