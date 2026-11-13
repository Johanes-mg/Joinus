use crate::network::Message;
use crate::persistence;
use crate::pieces_jointes::PieceJointe;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MessageLocal {
    #[serde(default)]
    pub id: String,
    pub de_moi: bool,
    pub texte: String,
    pub ts: i64,
    #[serde(default)]
    pub reponse_a: Option<String>,
    #[serde(default)]
    pub piece_jointe: Option<PieceJointe>,
    #[serde(default)]
    pub supprime: bool,
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

    pub fn ajouter(&mut self, pair_id: &str, message: MessageLocal) {
        self.charger_depuis_disque(pair_id);
        self.par_pair
            .entry(pair_id.to_string())
            .or_default()
            .push(message);
        self.sauver_disque(pair_id);
    }

    pub fn ajouter_local(
        &mut self,
        pair_id: &str,
        texte: String,
        reponse_a: Option<String>,
        piece_jointe: Option<PieceJointe>,
    ) -> MessageLocal {
        let msg = MessageLocal {
            id: Uuid::new_v4().to_string(),
            de_moi: true,
            texte,
            ts: chrono_now(),
            reponse_a,
            piece_jointe,
            supprime: false,
        };
        self.ajouter(pair_id, msg.clone());
        msg
    }

    pub fn ajouter_recu(&mut self, pair_id: &str, message: &Message) {
        self.charger_depuis_disque(pair_id);
        let local = MessageLocal {
            id: if message.id.is_empty() {
                Uuid::new_v4().to_string()
            } else {
                message.id.clone()
            },
            de_moi: false,
            texte: message.texte.clone(),
            ts: message.ts,
            reponse_a: message.reponse_a.clone(),
            piece_jointe: message.piece_jointe.clone(),
            supprime: false,
        };
        self.ajouter(pair_id, local);
    }

    pub fn supprimer_message(&mut self, pair_id: &str, msg_id: &str) {
        self.charger_depuis_disque(pair_id);
        if let Some(v) = self.par_pair.get_mut(pair_id) {
            for m in v.iter_mut() {
                if m.id == msg_id {
                    m.supprime = true;
                    m.texte = String::new();
                    m.piece_jointe = None;
                }
            }
        }
        self.sauver_disque(pair_id);
    }

    pub fn vider_conversation(&mut self, pair_id: &str) {
        self.par_pair.remove(pair_id);
        persistence::supprimer_historique(pair_id);
    }

    pub fn obtenir(&self, pair_id: &str, msg_id: &str) -> Option<&MessageLocal> {
        self.par_pair
            .get(pair_id)?
            .iter()
            .find(|m| m.id == msg_id)
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
                return;
            }
        }
        self.par_pair.insert(pair_id.to_string(), Vec::new());
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
    fn ajout_et_lecture() {
        let pair = "test_chat_1";
        let _ = std::fs::remove_file(Historique::chemin(pair));
        let mut h = Historique::new();
        h.selectionner(pair);
        h.ajouter_local(pair, "salut".to_string(), None, None);
        let msgs = h.messages_actifs().unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].texte, "salut");
        assert!(msgs[0].de_moi);
        let _ = std::fs::remove_file(Historique::chemin(pair));
    }

    #[test]
    fn suppression_message() {
        let pair = "test_chat_2";
        let _ = std::fs::remove_file(Historique::chemin(pair));
        let mut h = Historique::new();
        h.selectionner(pair);
        let msg = h.ajouter_local(pair, "a effacer".to_string(), None, None);
        h.supprimer_message(pair, &msg.id);
        let msgs = h.messages_actifs().unwrap();
        assert!(msgs[0].supprime);
        assert!(msgs[0].texte.is_empty());
        let _ = std::fs::remove_file(Historique::chemin(pair));
    }
}
