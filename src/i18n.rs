#[derive(Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Langue {
    Fr,
    En,
}

impl Default for Langue {
    fn default() -> Self {
        Langue::Fr
    }
}

#[derive(Clone, Copy)]
pub enum Cle {
    Profil,
    ChangerPhoto,
    Nom,
    PersonnesConnectees,
    AucunePersonne,
    Conversation,
    SelectionnerPersonne,
    ZoneSaisie,
    Envoyer,
    EnvoyerTooltip,
    AucunDestinataire,
    ThemeSombre,
    ErreurLecture,
    ErreurTaille,
    ErreurFormat,
    ErreurEcriture,
    Caracteres,
    ErreurImageEnvoyer,
    ErreurImageRafraichir,
    MoiMeme,
    ConnecteDepuis,
    MessageDeMoi,
    HistoriqueVide,
    NonChiffre,
    Rafraichir,
    RafraichirTooltip,
    AidePareFeu,
    PairsDetectes,
    RechercheEnCours,
    AvatarEnAttente,
}

#[allow(dead_code)]
pub const TOUTES_CLES: &[Cle] = &[
    Cle::Profil,
    Cle::ChangerPhoto,
    Cle::Nom,
    Cle::PersonnesConnectees,
    Cle::AucunePersonne,
    Cle::Conversation,
    Cle::SelectionnerPersonne,
    Cle::ZoneSaisie,
    Cle::Envoyer,
    Cle::EnvoyerTooltip,
    Cle::AucunDestinataire,
    Cle::ThemeSombre,
    Cle::ErreurLecture,
    Cle::ErreurTaille,
    Cle::ErreurFormat,
    Cle::ErreurEcriture,
    Cle::Caracteres,
    Cle::ErreurImageEnvoyer,
    Cle::ErreurImageRafraichir,
    Cle::MoiMeme,
    Cle::ConnecteDepuis,
    Cle::MessageDeMoi,
    Cle::HistoriqueVide,
    Cle::NonChiffre,
    Cle::Rafraichir,
    Cle::RafraichirTooltip,
    Cle::AidePareFeu,
    Cle::PairsDetectes,
    Cle::RechercheEnCours,
    Cle::AvatarEnAttente,
];

pub fn tr(langue: Langue, cle: Cle) -> &'static str {
    match (langue, cle) {
        (Langue::Fr, Cle::Profil) => "Profil",
        (Langue::Fr, Cle::ChangerPhoto) => "Changer la photo",
        (Langue::Fr, Cle::Nom) => "Nom",
        (Langue::Fr, Cle::PersonnesConnectees) => "Personnes connectées",
        (Langue::Fr, Cle::AucunePersonne) => "Aucune personne détectée",
        (Langue::Fr, Cle::Conversation) => "Conversation",
        (Langue::Fr, Cle::SelectionnerPersonne) => "Sélectionnez une personne",
        (Langue::Fr, Cle::ZoneSaisie) => "Écrivez votre message...",
        (Langue::Fr, Cle::Envoyer) => "Envoyer",
        (Langue::Fr, Cle::EnvoyerTooltip) => "Envoyer le message",
        (Langue::Fr, Cle::AucunDestinataire) => "Aucun destinataire connecté",
        (Langue::Fr, Cle::ThemeSombre) => "Thème sombre",
        (Langue::Fr, Cle::ErreurLecture) => "Impossible de lire le fichier",
        (Langue::Fr, Cle::ErreurTaille) => "Image trop volumineuse (5 Mo max)",
        (Langue::Fr, Cle::ErreurFormat) => "Format d'image non reconnu",
        (Langue::Fr, Cle::ErreurEcriture) => "Impossible d'enregistrer l'image",
        (Langue::Fr, Cle::Caracteres) => "caractères",
        (Langue::Fr, Cle::ErreurImageEnvoyer) => "Image du bouton Envoyer introuvable",
        (Langue::Fr, Cle::ErreurImageRafraichir) => "Image du bouton Rafraîchir introuvable",
        (Langue::Fr, Cle::MoiMeme) => "Moi-même",
        (Langue::Fr, Cle::ConnecteDepuis) => "Connecté",
        (Langue::Fr, Cle::MessageDeMoi) => "Message envoyé à moi-même (test local)",
        (Langue::Fr, Cle::HistoriqueVide) => "Aucun message",
        (Langue::Fr, Cle::NonChiffre) => "Messages non chiffrés - réseau de confiance uniquement",
        (Langue::Fr, Cle::Rafraichir) => "Rafraîchir",
        (Langue::Fr, Cle::RafraichirTooltip) => "Chercher des personnes sur le réseau",
        (Langue::Fr, Cle::AidePareFeu) => "Personne d'autre ? Vérifiez que Joinus est autorisé dans le pare-feu Windows, et que l'autre personne est sur le même réseau Wi-Fi.",
        (Langue::Fr, Cle::PairsDetectes) => "pairs détectés",
        (Langue::Fr, Cle::RechercheEnCours) => "Recherche en cours...",
        (Langue::Fr, Cle::AvatarEnAttente) => "Photo en cours de réception",
        (Langue::En, Cle::Profil) => "Profile",
        (Langue::En, Cle::ChangerPhoto) => "Change picture",
        (Langue::En, Cle::Nom) => "Name",
        (Langue::En, Cle::PersonnesConnectees) => "Connected people",
        (Langue::En, Cle::AucunePersonne) => "No one detected",
        (Langue::En, Cle::Conversation) => "Conversation",
        (Langue::En, Cle::SelectionnerPersonne) => "Select a person",
        (Langue::En, Cle::ZoneSaisie) => "Write your message...",
        (Langue::En, Cle::Envoyer) => "Send",
        (Langue::En, Cle::EnvoyerTooltip) => "Send message",
        (Langue::En, Cle::AucunDestinataire) => "No recipient connected",
        (Langue::En, Cle::ThemeSombre) => "Dark theme",
        (Langue::En, Cle::ErreurLecture) => "Cannot read file",
        (Langue::En, Cle::ErreurTaille) => "Image too large (5 MB max)",
        (Langue::En, Cle::ErreurFormat) => "Unrecognized image format",
        (Langue::En, Cle::ErreurEcriture) => "Cannot save image",
        (Langue::En, Cle::Caracteres) => "characters",
        (Langue::En, Cle::ErreurImageEnvoyer) => "Send button image not found",
        (Langue::En, Cle::ErreurImageRafraichir) => "Refresh button image not found",
        (Langue::En, Cle::MoiMeme) => "Myself",
        (Langue::En, Cle::ConnecteDepuis) => "Online",
        (Langue::En, Cle::MessageDeMoi) => "Message sent to myself (local test)",
        (Langue::En, Cle::HistoriqueVide) => "No messages",
        (Langue::En, Cle::NonChiffre) => "Messages are not encrypted - trusted network only",
        (Langue::En, Cle::Rafraichir) => "Refresh",
        (Langue::En, Cle::RafraichirTooltip) => "Look for people on the network",
        (Langue::En, Cle::AidePareFeu) => "No one else? Check that Joinus is allowed in Windows Firewall, and that the other person is on the same Wi-Fi network.",
        (Langue::En, Cle::PairsDetectes) => "peers detected",
        (Langue::En, Cle::RechercheEnCours) => "Searching...",
        (Langue::En, Cle::AvatarEnAttente) => "Picture being received",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toutes_cles_ont_traduction_fr() {
        for cle in TOUTES_CLES {
            let s = tr(Langue::Fr, *cle);
            assert!(!s.is_empty());
        }
    }

    #[test]
    fn toutes_cles_ont_traduction_en() {
        for cle in TOUTES_CLES {
            let s = tr(Langue::En, *cle);
            assert!(!s.is_empty());
        }
    }
}
