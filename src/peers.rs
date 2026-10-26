use crate::network::Pair;
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct ListePairs {
    pairs: HashMap<String, Pair>,
    avatars: HashMap<String, Vec<u8>>,
}

impl ListePairs {
    pub fn new() -> Self {
        Self {
            pairs: HashMap::new(),
            avatars: HashMap::new(),
        }
    }

    pub fn ajouter_ou_mettre_a_jour(&mut self, pair: Pair) {
        if let Some(existant) = self.pairs.get_mut(&pair.id) {
            existant.nom = pair.nom;
            existant.adresse = pair.adresse;
            existant.tcp_port = pair.tcp_port;
            existant.derniere_annonce = Instant::now();
        } else {
            self.pairs.insert(pair.id.clone(), pair);
        }
    }

    #[allow(dead_code)]
    pub fn retirer(&mut self, id: &str) {
        self.pairs.remove(id);
        self.avatars.remove(id);
    }

    pub fn retirer_inactifs(&mut self, duree: Duration) -> Vec<String> {
        let maintenant = Instant::now();
        let perdus: Vec<String> = self
            .pairs
            .iter()
            .filter(|(_, p)| maintenant.duration_since(p.derniere_annonce) > duree)
            .map(|(id, _)| id.clone())
            .collect();
        for id in &perdus {
            self.pairs.remove(id);
            self.avatars.remove(id);
        }
        perdus
    }

    pub fn iter(&self) -> impl Iterator<Item = &Pair> {
        self.pairs.values()
    }

    pub fn get(&self, id: &str) -> Option<&Pair> {
        self.pairs.get(id)
    }

    pub fn avatar(&self, id: &str) -> Option<&Vec<u8>> {
        self.avatars.get(id)
    }

    pub fn definir_avatar(&mut self, id: &str, bytes: Vec<u8>) {
        self.avatars.insert(id.to_string(), bytes);
    }

    pub fn len(&self) -> usize {
        self.pairs.len()
    }

    #[allow(dead_code)]
    pub fn trier_par_nom(&self) -> Vec<&Pair> {
        let mut v: Vec<&Pair> = self.pairs.values().collect();
        v.sort_by(|a, b| a.nom.to_lowercase().cmp(&b.nom.to_lowercase()));
        v
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;

    fn pair_test(id: &str, nom: &str) -> Pair {
        Pair {
            id: id.to_string(),
            nom: nom.to_string(),
            adresse: "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
            tcp_port: 0,
            derniere_annonce: Instant::now(),
        }
    }

    #[test]
    fn ajouter_mettre_a_jour() {
        let mut lp = ListePairs::new();
        lp.ajouter_ou_mettre_a_jour(pair_test("a", "Alice"));
        lp.ajouter_ou_mettre_a_jour(pair_test("a", "Alicia"));
        assert_eq!(lp.len(), 1);
        assert_eq!(lp.get("a").unwrap().nom, "Alicia");
    }

    #[test]
    fn retirer_supprime_avatar() {
        let mut lp = ListePairs::new();
        lp.definir_avatar("a", vec![1, 2, 3]);
        lp.retirer("a");
        assert!(lp.avatar("a").is_none());
    }

    #[test]
    fn retirer_inactifs() {
        let mut lp = ListePairs::new();
        lp.ajouter_ou_mettre_a_jour(pair_test("a", "Alice"));
        let perdus = lp.retirer_inactifs(Duration::from_secs(0));
        assert_eq!(perdus, vec!["a".to_string()]);
        assert_eq!(lp.len(), 0);
    }
}
