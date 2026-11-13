use crate::chat::Historique;
use crate::groupes::{self, Groupe};
use crate::i18n::{tr, Cle, Langue};
use crate::network::{self, Evenement, Message, Reseau, TIMEOUT_PRESENCE};
use crate::peers::ListePairs;
use crate::persistence::{self, Session};
use crate::pieces_jointes::{ErreurPiece, PieceJointe};
use crate::profile::{ErreurAvatar, Profil};
use crate::theme::{
    appliquer_theme, bordure_champ, bouton_action, bouton_icone, bouton_tricolore, bulle_autre,
    bulle_moi, carte_pair_bordure, carte_pair_fond, carte_pair_fond_actif,
    carte_pair_fond_survol, charger_texture_fichier, dessiner_avatar_arrondi,
    dessiner_avatar_petit, popup_fond, CacheTextures, AVATAR_LISTE_TAILLE, AVATAR_TAILLE,
    BOUTON_ENVOYER_HAUTEUR, BOUTON_ENVOYER_ICONE, BOUTON_ENVOYER_LARGEUR,
    BOUTON_RAFRAICHIR_ICONE, BOUTON_RAFRAICHIR_TAILLE, BOUTON_THEME_ICONE, BOUTON_THEME_TAILLE,
    CARTE_PAIR_HAUTEUR, CARTE_PAIR_ROUNDING, CHAMP_SAISIE_HAUTEUR, ERREUR, ESPACE_L, ESPACE_M,
    ESPACE_S, ESPACE_XL, FOND, LIMITE_MESSAGE, NOTIFICATION, ONLINE, SECONDAIRE, SURFACE, TEXTE,
};
use eframe::egui;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DELAI_AVANT_DEMANDE_AVATAR: Duration = Duration::from_secs(30);
const COULEUR_REPLY: egui::Color32 = egui::Color32::from_rgb(0x3A, 0x7B, 0xE0);
const COULEUR_SHARE: egui::Color32 = egui::Color32::from_rgb(0x2E, 0xA0, 0x43);
const COULEUR_DELETE: egui::Color32 = egui::Color32::from_rgb(0xE0, 0x50, 0x50);
const TAILLE_BTN_POPUP: f32 = 36.0;

#[derive(Clone)]
enum Dialogue {
    Aucun,
    MenuOptionsConversation,
    ChangerPseudo { pair_id: String, saisie: String },
    ConfirmerVider { pair_id: String, nom: String },
    Transferer { msg_id: String, source_pair: String },
    CreerGroupe { nom: String, membres: Vec<String> },
}

pub struct App {
    session: Session,
    profil: Profil,
    cache: CacheTextures,
    theme_courant: bool,
    brouillon: String,
    reseau: Option<Reseau>,
    pairs: ListePairs,
    groupes: Vec<Groupe>,
    alias: HashMap<String, String>,
    pair_selectionne: Option<String>,
    historique: Historique,
    envoyer_texture: Option<egui::TextureHandle>,
    rafraichir_texture_sombre: Option<egui::TextureHandle>,
    rafraichir_texture_clair: Option<egui::TextureHandle>,
    theme_sombre_texture: Option<egui::TextureHandle>,
    theme_clair_texture: Option<egui::TextureHandle>,
    points_sombre: Option<egui::TextureHandle>,
    points_clair: Option<egui::TextureHandle>,
    plus_sombre: Option<egui::TextureHandle>,
    plus_clair: Option<egui::TextureHandle>,
    groupe_sombre: Option<egui::TextureHandle>,
    groupe_clair: Option<egui::TextureHandle>,
    illustration_texture: Option<egui::TextureHandle>,
    reply_sombre: Option<egui::TextureHandle>,
    reply_clair: Option<egui::TextureHandle>,
    share_sombre: Option<egui::TextureHandle>,
    share_clair: Option<egui::TextureHandle>,
    delete_sombre: Option<egui::TextureHandle>,
    delete_clair: Option<egui::TextureHandle>,
    derniere_rafraichie: Instant,
    avatars_demandes: HashSet<String>,
    avatars_demandes_le: HashMap<String, Instant>,
    derniere_verification_inactifs: Instant,
    messages_non_lus: HashSet<String>,
    piece_jointe_attente: Option<PieceJointe>,
    piece_jointe_erreur: Option<String>,
    reponse_a: Option<String>,
    dialogue: Dialogue,
    dialogue_erreur: Option<String>,
    message_popup: Option<(String, String)>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, session: Session) -> Self {
        appliquer_theme(&cc.egui_ctx, session.sombre);
        let profil = Profil::charger(&session);
        let theme_courant = session.sombre;

        let dossier = PathBuf::from("images");
        let charge = |nom: &str, id: &str| {
            charger_texture_fichier(&cc.egui_ctx, &dossier.join(nom), id)
        };

        let envoyer_texture = charge("envoyer.png", "envoyer");
        let rafraichir_texture_sombre = charge("refresh-light.png", "refresh_light");
        let rafraichir_texture_clair = charge("refresh-dark.png", "refresh_dark");
        let theme_sombre_texture = charge("dark.png", "theme_dark");
        let theme_clair_texture = charge("light.png", "theme_light");
        let points_sombre = charge("3pointslight.png", "points_light");
        let points_clair = charge("3pointsdark.png", "points_dark");
        let plus_sombre = charge("plus-light.png", "plus_light");
        let plus_clair = charge("plus-dark.png", "plus_dark");
        let groupe_sombre = charge("group-light.png", "group_light");
        let groupe_clair = charge("group-dark.png", "group_dark");
        let illustration_texture = charge("illustration.png", "illustration");
        let reply_sombre = charge("reply-light.png", "reply_light");
        let reply_clair = charge("reply-dark.png", "reply_dark");
        let share_sombre = charge("share-light.png", "share_light");
        let share_clair = charge("share-dark.png", "share_dark");
        let delete_sombre = charge("delete-light.png", "delete_light");
        let delete_clair = charge("delete-dark.png", "delete_dark");

        let reseau = Reseau::demarrer(session.id.clone(), session.nom_affiche.clone());
        let mut pairs = ListePairs::new();
        if let Some(bytes) = &profil.avatar_bytes {
            pairs.definir_avatar(&session.id, bytes.clone());
        }

        Self {
            session,
            profil,
            cache: CacheTextures::new(),
            theme_courant,
            brouillon: String::new(),
            reseau,
            pairs,
            groupes: groupes::charger(),
            alias: persistence::charger_alias(),
            pair_selectionne: None,
            historique: Historique::new(),
            envoyer_texture,
            rafraichir_texture_sombre,
            rafraichir_texture_clair,
            theme_sombre_texture,
            theme_clair_texture,
            points_sombre,
            points_clair,
            plus_sombre,
            plus_clair,
            groupe_sombre,
            groupe_clair,
            illustration_texture,
            reply_sombre,
            reply_clair,
            share_sombre,
            share_clair,
            delete_sombre,
            delete_clair,
            derniere_rafraichie: Instant::now(),
            avatars_demandes: HashSet::new(),
            avatars_demandes_le: HashMap::new(),
            derniere_verification_inactifs: Instant::now(),
            messages_non_lus: HashSet::new(),
            piece_jointe_attente: None,
            piece_jointe_erreur: None,
            reponse_a: None,
            dialogue: Dialogue::Aucun,
            dialogue_erreur: None,
            message_popup: None,
        }
    }

    fn sauver(&self) {
        let s = Session {
            nom_affiche: self.profil.nom.clone(),
            langue: self.session.langue,
            sombre: self.session.sombre,
            window_size: self.session.window_size,
            id: self.session.id.clone(),
        };
        persistence::sauver_session(&s);
    }

    fn rafraichir_mon_avatar_dans_cache(&mut self) {
        if let Some(bytes) = &self.profil.avatar_bytes {
            self.pairs.definir_avatar(&self.session.id, bytes.clone());
        }
    }

    fn texte_erreur_avatar(&self, langue: Langue) -> Option<&'static str> {
        match &self.profil.erreur {
            Some(ErreurAvatar::Lecture) => Some(tr(langue, Cle::ErreurLecture)),
            Some(ErreurAvatar::Taille) => Some(tr(langue, Cle::ErreurTaille)),
            Some(ErreurAvatar::Format) => Some(tr(langue, Cle::ErreurFormat)),
            Some(ErreurAvatar::Ecriture) => Some(tr(langue, Cle::ErreurEcriture)),
            None => None,
        }
    }

    fn nom_affiche(&self, id: &str) -> String {
        if let Some(alias) = self.alias.get(id) {
            return alias.clone();
        }
        if id == self.session.id {
            return self.profil.nom.clone();
        }
        if let Some(p) = self.pairs.get(id) {
            return p.nom.clone();
        }
        if let Some(g) = self.groupes.iter().find(|g| g.id == id) {
            return g.nom.clone();
        }
        "Inconnu".to_string()
    }

    fn est_groupe(&self, id: &str) -> bool {
        self.groupes.iter().any(|g| g.id == id)
    }

    fn traiter_evenements(&mut self) {
        let mut a_demander: Vec<network::Pair> = Vec::new();
        if let Some(reseau) = &self.reseau {
            while let Ok(evt) = reseau.evenements.try_recv() {
                match evt {
                    Evenement::PairVu(pair) => {
                        let id = pair.id.clone();
                        self.pairs.ajouter_ou_mettre_a_jour(pair.clone());
                        let deja = self.avatars_demandes.contains(&id);
                        let connu = self.pairs.avatar(&id).is_some();
                        let doit = !deja
                            || match self.avatars_demandes_le.get(&id) {
                                Some(t) => t.elapsed() > DELAI_AVANT_DEMANDE_AVATAR && !connu,
                                None => !connu,
                            };
                        if !connu && doit && id != self.session.id {
                            a_demander.push(pair);
                        }
                    }
                    Evenement::MessageRecu(msg) => {
                        if self.pair_selectionne.as_deref() != Some(&msg.de) {
                            self.messages_non_lus.insert(msg.de.clone());
                        }
                        self.historique.ajouter_recu(&msg.de, &msg);
                    }
                    Evenement::AvatarRecu { id, bytes } => {
                        self.pairs.definir_avatar(&id, bytes);
                        self.avatars_demandes_le.insert(id, Instant::now());
                    }
                    Evenement::MessageSupprime { de, id } => {
                        self.historique.supprimer_message(&de, &id);
                    }
                }
            }
        }
        for pair in a_demander {
            self.avatars_demandes.insert(pair.id.clone());
            self.avatars_demandes_le
                .insert(pair.id.clone(), Instant::now());
            if let Some(reseau) = &self.reseau {
                reseau.demander_avatar_async(pair, self.session.id.clone());
            }
        }
    }

    fn verifier_inactifs(&mut self) {
        if self.derniere_verification_inactifs.elapsed() < Duration::from_secs(1) {
            return;
        }
        self.derniere_verification_inactifs = Instant::now();
        let perdus = self.pairs.retirer_inactifs(TIMEOUT_PRESENCE);
        for id in perdus {
            self.avatars_demandes.remove(&id);
            self.avatars_demandes_le.remove(&id);
            self.messages_non_lus.remove(&id);
        }
    }

    fn rafraichir(&mut self) {
        if let Some(reseau) = &self.reseau {
            reseau.rafraichir();
        }
        self.derniere_rafraichie = Instant::now();
    }

    fn texture_avatar_pair(
        &mut self,
        ctx: &egui::Context,
        pair_id: &str,
    ) -> Option<egui::TextureHandle> {
        if pair_id == self.session.id {
            return self.profil.texture(ctx, &mut self.cache);
        }
        let bytes = self.pairs.avatar(pair_id)?.clone();
        let hash = crate::theme::hash_bytes(&bytes);
        self.cache.obtenir_avec_hash(ctx, &bytes, hash, pair_id)
    }

    fn initiale_pair(&self, nom: &str) -> Option<char> {
        nom.chars().find(|c| !c.is_whitespace())
    }

    fn sauvegarder_piece_jointe(&self, pair_id: &str, msg_id: &str) {
        let Some(msg) = self.historique.obtenir(pair_id, msg_id) else {
            return;
        };
        let Some(pj) = &msg.piece_jointe else {
            return;
        };
        let Some(bytes) = pj.decoder() else {
            return;
        };
        if let Some(chemin) = rfd::FileDialog::new()
            .set_file_name(&pj.nom)
            .save_file()
        {
            let _ = std::fs::write(chemin, bytes);
        }
    }

    fn dessiner_carte(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        id: &str,
        nom: &str,
        est_moi: bool,
        est_groupe: bool,
        selectionne: bool,
    ) -> egui::Response {
        let largeur = ui.available_width();
        let (rect, reponse) = ui.allocate_exact_size(
            egui::vec2(largeur, CARTE_PAIR_HAUTEUR),
            egui::Sense::click(),
        );
        let sombre = self.session.sombre;
        let survol = reponse.hovered();
        let fond = if selectionne {
            carte_pair_fond_actif(sombre)
        } else if survol {
            carte_pair_fond_survol(sombre)
        } else {
            carte_pair_fond(sombre)
        };
        let painter = ui.painter();
        painter.rect_filled(rect, egui::Rounding::same(CARTE_PAIR_ROUNDING), fond);
        let bordure = carte_pair_bordure(sombre, selectionne, survol);
        let epaisseur = if selectionne { 2.0_f32 } else { 1.0_f32 };
        painter.rect_stroke(
            rect,
            egui::Rounding::same(CARTE_PAIR_ROUNDING),
            egui::Stroke::new(epaisseur, bordure),
        );

        let marge = ESPACE_S;
        let avatar_rect = egui::Rect::from_center_size(
            egui::pos2(
                rect.left() + marge + AVATAR_LISTE_TAILLE / 2.0,
                rect.center().y,
            ),
            egui::vec2(AVATAR_LISTE_TAILLE, AVATAR_LISTE_TAILLE),
        );

        let texture = if est_groupe {
            None
        } else {
            self.texture_avatar_pair(ctx, id)
        };
        let initiale = self.initiale_pair(nom);
        let couleur_initiale = if sombre { TEXTE } else { FOND };
        dessiner_avatar_petit(
            ui,
            avatar_rect,
            texture.as_ref(),
            initiale,
            couleur_initiale,
            sombre,
        );

        if !est_groupe {
            let dot = egui::pos2(avatar_rect.right() - 3.0, avatar_rect.bottom() - 3.0);
            ui.painter().circle_filled(dot, 5.0_f32, ONLINE);
            ui.painter()
                .circle_stroke(dot, 5.0_f32, egui::Stroke::new(1.5_f32, fond));
        }

        let couleur_texte = if est_moi {
            SECONDAIRE
        } else if sombre {
            TEXTE
        } else {
            FOND
        };
        let x_texte = avatar_rect.right() + ESPACE_M;
        let texte_principal = if est_moi {
            format!("{} ({})", nom, tr(self.session.langue, Cle::MoiMeme))
        } else {
            nom.to_string()
        };
        ui.painter().text(
            egui::pos2(x_texte, rect.center().y),
            egui::Align2::LEFT_CENTER,
            texte_principal,
            egui::FontId::proportional(14.0),
            couleur_texte,
        );

        if self.messages_non_lus.contains(id) {
            let dot = egui::pos2(rect.right() - ESPACE_M, rect.center().y);
            ui.painter().circle_filled(dot, 5.0_f32, NOTIFICATION);
        }
        reponse
    }

    fn dessiner_accueil(&mut self, ui: &mut egui::Ui) {
        let langue = self.session.langue;
        let sombre = self.session.sombre;

        ui.add_space(ESPACE_XL * 2.0);

        ui.vertical_centered(|ui| {
            ui.label(
                egui::RichText::new(tr(langue, Cle::Bienvenue))
                    .size(34.0)
                    .color(if sombre { TEXTE } else { FOND }),
            );
            ui.add_space(ESPACE_S);
            ui.label(
                egui::RichText::new(tr(langue, Cle::SousTitreAccueil))
                    .size(15.0)
                    .color(SECONDAIRE),
            );
        });

        let restant = ui.available_height();
        let marge_bas = ESPACE_L;
        if restant <= 40.0 {
            return;
        }
        let hauteur_illu = (restant - marge_bas).max(0.0);
        ui.add_space(ESPACE_L);

        let illustration = self.illustration_texture.clone();
        ui.vertical_centered(|ui| {
            if let Some(tex) = &illustration {
                let taille = tex.size_vec2();
                let ratio = if taille.y > 0.0 {
                    taille.x / taille.y
                } else {
                    1.0
                };
                let largeur_max = (ui.available_width() - ESPACE_L * 2.0).max(60.0);
                let largeur = (hauteur_illu * ratio).min(largeur_max);
                let hauteur = if ratio > 0.0 {
                    largeur / ratio
                } else {
                    hauteur_illu
                };
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(largeur, hauteur),
                    egui::Sense::hover(),
                );
                ui.painter().image(
                    tex.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );
            }
        });
    }

    fn panneau_gauche(&mut self, ui: &mut egui::Ui) {
        let langue = self.session.langue;
        let sombre = self.session.sombre;
        ui.add_space(ESPACE_M);

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let tex = if sombre {
                    self.theme_clair_texture.as_ref()
                } else {
                    self.theme_sombre_texture.as_ref()
                };
                let tooltip = if sombre {
                    tr(langue, Cle::ThemeVersClair)
                } else {
                    tr(langue, Cle::ThemeVersSombre)
                };
                let rep = bouton_icone(
                    ui,
                    BOUTON_THEME_TAILLE,
                    BOUTON_THEME_ICONE,
                    tex,
                    sombre,
                    "T",
                );
                let rep = rep
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(tooltip);
                if rep.clicked() {
                    self.session.sombre = !self.session.sombre;
                    self.sauver();
                }
            });
        });

        ui.add_space(ESPACE_L);

        ui.vertical_centered(|ui| {
            let (rect, reponse) = ui.allocate_exact_size(
                egui::vec2(AVATAR_TAILLE, AVATAR_TAILLE),
                egui::Sense::click(),
            );
            let texture = self.profil.texture(ui.ctx(), &mut self.cache);
            let initiale = self.profil.initiale();
            dessiner_avatar_arrondi(ui, rect, texture.as_ref(), initiale, sombre);
            let dot = egui::pos2(rect.right() - 6.0, rect.bottom() - 6.0);
            ui.painter().circle_filled(dot, 7.0_f32, ONLINE);
            let bord = if sombre { FOND } else { SURFACE };
            ui.painter()
                .circle_stroke(dot, 7.0_f32, egui::Stroke::new(2.0_f32, bord));
            let reponse = reponse.on_hover_cursor(egui::CursorIcon::PointingHand);
            if reponse.clicked() {
                self.profil.importer_avatar();
                self.rafraichir_mon_avatar_dans_cache();
            }
        });

        ui.add_space(ESPACE_M);

        ui.label(
            egui::RichText::new(tr(langue, Cle::Nom))
                .color(SECONDAIRE)
                .size(11.0),
        );

        let couleur_bord = bordure_champ(sombre);
        let marge_champ = egui::Margin::symmetric(8.0, 6.0);
        egui::Frame::none()
            .stroke(egui::Stroke::new(1.0_f32, couleur_bord))
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(marge_champ)
            .show(ui, |ui| {
                ui.add_sized(
                    [ui.available_width(), 20.0],
                    egui::TextEdit::singleline(&mut self.profil.nom)
                        .frame(false)
                        .desired_width(f32::INFINITY),
                );
            });
        if ui.input(|i| i.pointer.any_click()) && ui.memory(|m| m.focused().is_none()) {
            self.sauver();
        }

        ui.add_space(ESPACE_S);
        ui.horizontal(|ui| {
            if ui
                .add(egui::Button::new(tr(langue, Cle::ChangerPhoto)).rounding(8.0))
                .clicked()
            {
                self.profil.importer_avatar();
                self.rafraichir_mon_avatar_dans_cache();
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                egui::ComboBox::from_id_source("langue")
                    .width(56.0)
                    .selected_text(match langue {
                        Langue::Fr => "FR",
                        Langue::En => "EN",
                    })
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(langue == Langue::Fr, "FR").clicked() {
                            self.session.langue = Langue::Fr;
                            self.sauver();
                        }
                        if ui.selectable_label(langue == Langue::En, "EN").clicked() {
                            self.session.langue = Langue::En;
                            self.sauver();
                        }
                    });
            });
        });
        if let Some(texte) = self.texte_erreur_avatar(langue) {
            ui.colored_label(ERREUR, texte);
        }

        ui.add_space(ESPACE_L);
        ui.separator();
        ui.add_space(ESPACE_M);

        let nombre_pairs = self.pairs.len();
        ui.horizontal(|ui| {
            ui.heading(
                egui::RichText::new(tr(langue, Cle::PersonnesConnectees)).size(14.0),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let tex = if sombre {
                    self.rafraichir_texture_sombre.as_ref()
                } else {
                    self.rafraichir_texture_clair.as_ref()
                };
                let rep = bouton_icone(
                    ui,
                    BOUTON_RAFRAICHIR_TAILLE,
                    BOUTON_RAFRAICHIR_ICONE,
                    tex,
                    sombre,
                    "R",
                );
                let rep = rep
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(tr(langue, Cle::RafraichirTooltip));
                if rep.clicked() {
                    self.rafraichir();
                }
            });
        });
        ui.add_space(ESPACE_S);
        ui.label(
            egui::RichText::new(format!(
                "{} {}",
                nombre_pairs,
                tr(langue, Cle::PairsDetectes)
            ))
            .color(SECONDAIRE)
            .size(11.0),
        );
        ui.add_space(ESPACE_M);

        let icone_groupe = if sombre {
            self.groupe_sombre.as_ref()
        } else {
            self.groupe_clair.as_ref()
        };
        if bouton_action(ui, icone_groupe, tr(langue, Cle::CreerGroupe), sombre).clicked() {
            self.dialogue = Dialogue::CreerGroupe {
                nom: String::new(),
                membres: Vec::new(),
            };
        }
        ui.add_space(ESPACE_M);

        let mut entrees: Vec<(String, String, bool, bool)> = Vec::new();
        for p in self.pairs.iter() {
            entrees.push((p.id.clone(), self.nom_affiche(&p.id), false, false));
        }
        for g in &self.groupes {
            entrees.push((g.id.clone(), g.nom.clone(), false, true));
        }
        entrees.push((
            self.session.id.clone(),
            self.profil.nom.clone(),
            true,
            false,
        ));
        entrees.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));

        let ctx = ui.ctx().clone();
        for (id, nom, est_moi, est_groupe) in entrees {
            let selectionne = self.pair_selectionne.as_deref() == Some(&id);
            let reponse =
                self.dessiner_carte(ui, &ctx, &id, &nom, est_moi, est_groupe, selectionne);
            let reponse = reponse.on_hover_cursor(egui::CursorIcon::PointingHand);
            if reponse.clicked() {
                if selectionne {
                    self.pair_selectionne = None;
                } else {
                    self.pair_selectionne = Some(id.clone());
                    self.historique.selectionner(&id);
                    self.messages_non_lus.remove(&id);
                    self.reponse_a = None;
                }
                self.message_popup = None;
            }
            ui.add_space(ESPACE_S);
        }
    }

    fn panneau_droite(&mut self, ui: &mut egui::Ui) {
        let langue = self.session.langue;
        let sombre = self.session.sombre;

        if self.pair_selectionne.is_none() {
            self.dessiner_accueil(ui);
            return;
        }

        let pair_id = self.pair_selectionne.clone().unwrap();
        let nom_affiche = self.nom_affiche(&pair_id);
        let est_groupe = self.est_groupe(&pair_id);
        let est_moi = pair_id == self.session.id;

        ui.add_space(ESPACE_M);
        ui.horizontal(|ui| {
            let (avatar_rect, _) =
                ui.allocate_exact_size(egui::vec2(36.0, 36.0), egui::Sense::hover());
            if !est_groupe {
                let texture = self.texture_avatar_pair(ui.ctx(), &pair_id);
                let init = self.initiale_pair(&nom_affiche);
                let coul = if sombre { TEXTE } else { FOND };
                dessiner_avatar_petit(ui, avatar_rect, texture.as_ref(), init, coul, sombre);
            } else {
                ui.painter().circle_filled(
                    avatar_rect.center(),
                    18.0_f32,
                    crate::theme::fond_avatar(sombre),
                );
            }
            ui.add_space(ESPACE_S);
            ui.vertical(|ui| {
                ui.label(egui::RichText::new(&nom_affiche).size(16.0).strong());
                if est_groupe {
                    let n = self
                        .groupes
                        .iter()
                        .find(|g| g.id == pair_id)
                        .map(|g| g.membres.len())
                        .unwrap_or(0);
                    ui.label(
                        egui::RichText::new(format!("{} {}", n, tr(langue, Cle::Membres)))
                            .size(10.0)
                            .color(SECONDAIRE),
                    );
                } else if est_moi {
                    ui.label(
                        egui::RichText::new(tr(langue, Cle::MoiMeme))
                            .size(10.0)
                            .color(SECONDAIRE),
                    );
                }
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let tex = if sombre {
                    self.points_sombre.as_ref()
                } else {
                    self.points_clair.as_ref()
                };
                let rep = bouton_icone(
                    ui,
                    BOUTON_THEME_TAILLE,
                    BOUTON_THEME_ICONE,
                    tex,
                    sombre,
                    "...",
                );
                let rep = rep
                    .on_hover_cursor(egui::CursorIcon::PointingHand)
                    .on_hover_text(tr(langue, Cle::OptionsConversation));
                if rep.clicked() {
                    self.dialogue = if matches!(
                        self.dialogue,
                        Dialogue::MenuOptionsConversation
                    ) {
                        Dialogue::Aucun
                    } else {
                        Dialogue::MenuOptionsConversation
                    };
                }
            });
        });
        ui.separator();

        if matches!(self.dialogue, Dialogue::MenuOptionsConversation) {
            self.afficher_menu_options(ui, &pair_id, &nom_affiche);
        }

        let hauteur_saisie = CHAMP_SAISIE_HAUTEUR + ESPACE_XL + ESPACE_S + 30.0;
        let hauteur_dispo = ui.available_height();
        let hauteur_messages = (hauteur_dispo - hauteur_saisie - ESPACE_M).max(0.0);

        let messages = self
            .historique
            .messages_actifs()
            .map(|v| v.to_vec())
            .unwrap_or_default();
        let popup_target = self.message_popup.as_ref().map(|(_, id)| id.clone());
        let reply_tex = if sombre {
            self.reply_sombre.clone()
        } else {
            self.reply_clair.clone()
        };
        let share_tex = if sombre {
            self.share_sombre.clone()
        } else {
            self.share_clair.clone()
        };
        let delete_tex = if sombre {
            self.delete_sombre.clone()
        } else {
            self.delete_clair.clone()
        };
        let ctx_clone = ui.ctx().clone();
        let cache = &mut self.cache;

        let mut clic_id: Option<String> = None;
        let mut download_id: Option<String> = None;
        let mut popup_action: Option<(String, String)> = None;

        ui.allocate_ui(egui::vec2(ui.available_width(), hauteur_messages), |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if messages.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label(
                                egui::RichText::new(tr(langue, Cle::HistoriqueVide))
                                    .color(SECONDAIRE)
                                    .size(14.0),
                            );
                        });
                    } else {
                        for msg in &messages {
                            let (clic, download) =
                                dessiner_bulle(ui, &ctx_clone, cache, msg, sombre, langue);
                            if clic.is_some() {
                                clic_id = clic;
                            }
                            if download.is_some() {
                                download_id = download;
                            }

                            if popup_target.as_deref() == Some(msg.id.as_str()) {
                                ui.add_space(2.0);
                                ui.horizontal(|ui| {
                                    let group_w =
                                        TAILLE_BTN_POPUP * 3.0 + ESPACE_S * 2.0;
                                    let dispo = ui.available_width();
                                    let pad = ((dispo - group_w) / 2.0).max(0.0);
                                    if pad > 0.0 {
                                        ui.add_space(pad);
                                    }
                                    let r = bouton_tricolore(
                                        ui,
                                        reply_tex.as_ref(),
                                        COULEUR_REPLY,
                                        "R",
                                        TAILLE_BTN_POPUP,
                                    )
                                    .on_hover_text(tr(langue, Cle::Repondre));
                                    if r.clicked() {
                                        popup_action =
                                            Some((msg.id.clone(), "reply".to_string()));
                                    }
                                    ui.add_space(ESPACE_S);
                                    let s = bouton_tricolore(
                                        ui,
                                        share_tex.as_ref(),
                                        COULEUR_SHARE,
                                        "S",
                                        TAILLE_BTN_POPUP,
                                    )
                                    .on_hover_text(tr(langue, Cle::Transferer));
                                    if s.clicked() {
                                        popup_action =
                                            Some((msg.id.clone(), "share".to_string()));
                                    }
                                    ui.add_space(ESPACE_S);
                                    let d = bouton_tricolore(
                                        ui,
                                        delete_tex.as_ref(),
                                        COULEUR_DELETE,
                                        "D",
                                        TAILLE_BTN_POPUP,
                                    )
                                    .on_hover_text(tr(langue, Cle::Effacer));
                                    if d.clicked() {
                                        popup_action =
                                            Some((msg.id.clone(), "delete".to_string()));
                                    }
                                });
                            }

                            ui.add_space(ESPACE_S);
                        }
                    }
                });
        });

        if let Some(id) = download_id {
            self.sauvegarder_piece_jointe(&pair_id, &id);
        }

        if let Some((id, action)) = popup_action {
            match action.as_str() {
                "reply" => {
                    self.reponse_a = Some(id);
                }
                "share" => {
                    self.dialogue = Dialogue::Transferer {
                        msg_id: id,
                        source_pair: pair_id.clone(),
                    };
                }
                "delete" => {
                    self.historique.supprimer_message(&pair_id, &id);
                    if !est_moi && !est_groupe {
                        if let Some(pair) = self.pairs.get(&pair_id).cloned() {
                            let _ =
                                network::envoyer_suppression(&pair, &self.session.id, &id);
                        }
                    }
                }
                _ => {}
            }
            self.message_popup = None;
        } else if let Some(id) = clic_id {
            let deja_ouvert =
                matches!(&self.message_popup, Some((_, mid)) if mid == &id);
            if deja_ouvert {
                self.message_popup = None;
            } else {
                self.message_popup = Some((pair_id.clone(), id));
            }
        }

        ui.add_space(ESPACE_S);

        if let Some(rid) = self.reponse_a.clone() {
            if let Some(msg) = self.historique.obtenir(&pair_id, &rid).cloned() {
                egui::Frame::none()
                    .fill(if sombre {
                        egui::Color32::from_rgb(0x24, 0x27, 0x24)
                    } else {
                        egui::Color32::from_rgb(0xee, 0xef, 0xea)
                    })
                    .rounding(egui::Rounding::same(8.0))
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}: {}",
                                    tr(langue, Cle::EnReponseA),
                                    tronquer(&msg.texte, 60)
                                ))
                                .size(11.0)
                                .color(SECONDAIRE),
                            );
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.button("x").clicked() {
                                        self.reponse_a = None;
                                    }
                                },
                            );
                        });
                    });
            } else {
                self.reponse_a = None;
            }
        }

        if let Some(pj) = self.piece_jointe_attente.clone() {
            let mut retirer = false;
            egui::Frame::none()
                .fill(if sombre {
                    egui::Color32::from_rgb(0x24, 0x27, 0x24)
                } else {
                    egui::Color32::from_rgb(0xee, 0xef, 0xea)
                })
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::same(6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{}: {} ({} o)",
                                tr(langue, Cle::PieceJointe),
                                pj.nom,
                                pj.taille_octets()
                            ))
                            .size(11.0)
                            .color(SECONDAIRE),
                        );
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.button("x").clicked() {
                                    retirer = true;
                                }
                            },
                        );
                    });
                });
            if retirer {
                self.piece_jointe_attente = None;
            }
        }

        if let Some(err) = &self.piece_jointe_erreur {
            ui.colored_label(ERREUR, err.clone());
        }

        egui::Frame::none()
            .fill(if sombre { FOND } else { TEXTE })
            .inner_margin(egui::Margin::same(ESPACE_S))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let largeur_totale = ui.available_width();
                    let largeur_boutons = BOUTON_ENVOYER_LARGEUR.min(48.0) + ESPACE_S;
                    let largeur_champ =
                        (largeur_totale - largeur_boutons - ESPACE_S).max(140.0);
                    let hauteur_champ = CHAMP_SAISIE_HAUTEUR;
                    let mut envoyer_par_entree = false;
                    ui.allocate_ui(egui::vec2(largeur_champ, hauteur_champ), |ui| {
                        let rep = ui.add_sized(
                            [largeur_champ, hauteur_champ],
                            egui::TextEdit::multiline(&mut self.brouillon)
                                .hint_text(tr(langue, Cle::ZoneSaisie))
                                .desired_rows(3)
                                .char_limit(LIMITE_MESSAGE),
                        );
                        if rep.has_focus() {
                            let entree = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            let maj = ui.input(|i| i.modifiers.shift);
                            if entree && !maj {
                                envoyer_par_entree = true;
                            }
                        }
                    });
                    ui.add_space(ESPACE_S);
                    ui.vertical(|ui| {
                        let plus_tex = if sombre {
                            self.plus_sombre.as_ref()
                        } else {
                            self.plus_clair.as_ref()
                        };
                        let plus_rep = bouton_icone(
                            ui,
                            BOUTON_ENVOYER_LARGEUR.min(48.0),
                            BOUTON_ENVOYER_ICONE,
                            plus_tex,
                            sombre,
                            "+",
                        );
                        let plus_rep = plus_rep
                            .on_hover_cursor(egui::CursorIcon::PointingHand)
                            .on_hover_text(tr(langue, Cle::JoindreFichier));
                        if plus_rep.clicked() {
                            if let Some(chemin) = rfd::FileDialog::new().pick_file() {
                                match PieceJointe::depuis_fichier(&chemin) {
                                    Ok(pj) => {
                                        self.piece_jointe_attente = Some(pj);
                                        self.piece_jointe_erreur = None;
                                    }
                                    Err(ErreurPiece::Taille) => {
                                        self.piece_jointe_erreur =
                                            Some(tr(langue, Cle::PieceJointeTrop).to_string());
                                    }
                                    Err(ErreurPiece::Lecture) => {
                                        self.piece_jointe_erreur =
                                            Some(tr(langue, Cle::ErreurLecture).to_string());
                                    }
                                }
                            }
                        }
                        ui.add_space(ESPACE_S);
                        let actif = !self.brouillon.trim().is_empty()
                            || self.piece_jointe_attente.is_some();
                        let (rect, rep) = ui.allocate_exact_size(
                            egui::vec2(
                                BOUTON_ENVOYER_LARGEUR.min(48.0),
                                BOUTON_ENVOYER_HAUTEUR,
                            ),
                            egui::Sense::click(),
                        );
                        let style = ui.style().interact(&rep);
                        let fond = if actif {
                            if sombre {
                                egui::Color32::from_rgb(0x3a, 0x3d, 0x3a)
                            } else {
                                egui::Color32::from_rgb(0xcf, 0xd0, 0xcb)
                            }
                        } else {
                            style.weak_bg_fill
                        };
                        ui.painter()
                            .rect_filled(rect, egui::Rounding::same(8.0), fond);
                        if let Some(tex) = &self.envoyer_texture {
                            let zone = egui::Rect::from_center_size(
                                rect.center(),
                                egui::vec2(BOUTON_ENVOYER_ICONE, BOUTON_ENVOYER_ICONE),
                            );
                            let teinte = if actif {
                                egui::Color32::WHITE
                            } else {
                                SECONDAIRE
                            };
                            ui.painter().image(
                                tex.id(),
                                zone,
                                egui::Rect::from_min_max(
                                    egui::pos2(0.0, 0.0),
                                    egui::pos2(1.0, 1.0),
                                ),
                                teinte,
                            );
                        } else {
                            let coul = if sombre { TEXTE } else { FOND };
                            ui.painter().text(
                                rect.center(),
                                egui::Align2::CENTER_CENTER,
                                tr(langue, Cle::Envoyer),
                                egui::FontId::proportional(11.0),
                                if actif { coul } else { SECONDAIRE },
                            );
                        }
                        let rep = rep.on_hover_text(tr(langue, Cle::EnvoyerTooltip));
                        if actif {
                            let rep = rep.on_hover_cursor(egui::CursorIcon::PointingHand);
                            if rep.clicked() || envoyer_par_entree {
                                self.envoyer_message_actif(&pair_id);
                            }
                        }
                    });
                });
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{} / {} {}",
                            self.brouillon.chars().count(),
                            LIMITE_MESSAGE,
                            tr(langue, Cle::Caracteres)
                        ))
                        .color(SECONDAIRE)
                        .size(10.0),
                    );
                });
            });
    }

    fn afficher_menu_options(&mut self, ui: &mut egui::Ui, pair_id: &str, nom: &str) {
        let langue = self.session.langue;
        let sombre = self.session.sombre;
        let fond = popup_fond(sombre);
        let mut nouvelle_action: Option<Dialogue> = None;
        egui::Frame::none()
            .fill(fond)
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(egui::Margin::same(6.0))
            .show(ui, |ui| {
                if bouton_action(ui, None, tr(langue, Cle::ChangerPseudo), sombre).clicked() {
                    nouvelle_action = Some(Dialogue::ChangerPseudo {
                        pair_id: pair_id.to_string(),
                        saisie: self.nom_affiche(pair_id),
                    });
                }
                if bouton_action(ui, None, tr(langue, Cle::ViderConversation), sombre).clicked() {
                    nouvelle_action = Some(Dialogue::ConfirmerVider {
                        pair_id: pair_id.to_string(),
                        nom: nom.to_string(),
                    });
                }
            });
        if let Some(d) = nouvelle_action {
            self.dialogue = d;
        }
    }

    fn afficher_dialogues(&mut self, ctx: &egui::Context) {
        let langue = self.session.langue;
        match self.dialogue.clone() {
            Dialogue::ChangerPseudo { pair_id, mut saisie } => {
                let mut ouvert = true;
                let mut valider = false;
                let mut annuler = false;
                egui::Window::new(tr(langue, Cle::ChangerPseudo))
                    .open(&mut ouvert)
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(tr(langue, Cle::Nom));
                        ui.text_edit_singleline(&mut saisie);
                        ui.add_space(ESPACE_S);
                        ui.horizontal(|ui| {
                            if ui.button(tr(langue, Cle::Confirmer)).clicked() {
                                valider = true;
                            }
                            if ui.button(tr(langue, Cle::Annuler)).clicked() {
                                annuler = true;
                            }
                        });
                    });
                if valider {
                    if saisie.trim().is_empty() {
                        self.alias.remove(&pair_id);
                    } else {
                        self.alias.insert(pair_id.clone(), saisie.trim().to_string());
                    }
                    persistence::sauver_alias(&self.alias);
                    self.dialogue = Dialogue::Aucun;
                } else if annuler || !ouvert {
                    self.dialogue = Dialogue::Aucun;
                } else {
                    self.dialogue = Dialogue::ChangerPseudo { pair_id, saisie };
                }
            }
            Dialogue::ConfirmerVider { pair_id, nom } => {
                let mut ouvert = true;
                let mut valider = false;
                let mut annuler = false;
                egui::Window::new(tr(langue, Cle::ViderConversation))
                    .open(&mut ouvert)
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label(format!(
                            "{} ({})",
                            tr(langue, Cle::ConfirmerVider),
                            nom
                        ));
                        ui.add_space(ESPACE_S);
                        ui.horizontal(|ui| {
                            if ui.button(tr(langue, Cle::Confirmer)).clicked() {
                                valider = true;
                            }
                            if ui.button(tr(langue, Cle::Annuler)).clicked() {
                                annuler = true;
                            }
                        });
                    });
                if valider {
                    self.historique.vider_conversation(&pair_id);
                    persistence::supprimer_historique(&pair_id);
                    self.messages_non_lus.remove(&pair_id);
                    self.dialogue = Dialogue::Aucun;
                } else if annuler || !ouvert {
                    self.dialogue = Dialogue::Aucun;
                }
            }
            Dialogue::Transferer { msg_id, source_pair } => {
                let mut ouvert = true;
                let mut choix: Option<String> = None;
                egui::Window::new(tr(langue, Cle::ChoixDestinataire))
                    .open(&mut ouvert)
                    .collapsible(false)
                    .resizable(true)
                    .show(ctx, |ui| {
                        let mut cibles: Vec<(String, String)> = Vec::new();
                        for p in self.pairs.iter() {
                            cibles.push((p.id.clone(), p.nom.clone()));
                        }
                        for g in &self.groupes {
                            cibles.push((g.id.clone(), g.nom.clone()));
                        }
                        cibles.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
                        for (id, nom) in cibles {
                            if ui.button(nom).clicked() {
                                choix = Some(id);
                            }
                        }
                    });
                if let Some(dest) = choix {
                    if let Some(msg) = self.historique.obtenir(&source_pair, &msg_id).cloned() {
                        let texte = msg.texte.clone();
                        let pj = msg.piece_jointe.clone();
                        let reponse = msg.reponse_a.clone();
                        self.envoyer_vers(&dest, texte, reponse, pj);
                    }
                    self.dialogue = Dialogue::Aucun;
                } else if !ouvert {
                    self.dialogue = Dialogue::Aucun;
                }
            }
            Dialogue::CreerGroupe { mut nom, mut membres } => {
                let mut ouvert = true;
                let mut valider = false;
                let mut annuler = false;
                let mut candidats: Vec<(String, String)> = Vec::new();
                for p in self.pairs.iter() {
                    candidats.push((p.id.clone(), p.nom.clone()));
                }
                candidats.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
                egui::Window::new(tr(langue, Cle::NouveauGroupe))
                    .open(&mut ouvert)
                    .collapsible(false)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.label(tr(langue, Cle::NomGroupe));
                        ui.text_edit_singleline(&mut nom);
                        ui.add_space(ESPACE_S);
                        ui.label(tr(langue, Cle::Membres));
                        for (id, n) in &candidats {
                            let mut coche = membres.contains(id);
                            if ui.checkbox(&mut coche, n).changed() {
                                if coche {
                                    membres.push(id.clone());
                                } else {
                                    membres.retain(|m| m != id);
                                }
                            }
                        }
                        if membres.is_empty() {
                            ui.colored_label(SECONDAIRE, tr(langue, Cle::AucunMembre));
                        }
                        if let Some(err) = &self.dialogue_erreur {
                            ui.colored_label(ERREUR, err);
                        }
                        ui.add_space(ESPACE_S);
                        ui.horizontal(|ui| {
                            if ui.button(tr(langue, Cle::CreerGroupe)).clicked() {
                                valider = true;
                            }
                            if ui.button(tr(langue, Cle::Annuler)).clicked() {
                                annuler = true;
                            }
                        });
                    });
                if valider {
                    if !nom.trim().is_empty() && !membres.is_empty() {
                        let g = groupes::creer(nom.trim(), membres.clone());
                        self.groupes.push(g);
                        groupes::sauver(&self.groupes);
                        self.dialogue_erreur = None;
                        self.dialogue = Dialogue::Aucun;
                    } else {
                        self.dialogue_erreur =
                            Some(tr(langue, Cle::AucunMembre).to_string());
                        self.dialogue = Dialogue::CreerGroupe { nom, membres };
                    }
                } else if annuler || !ouvert {
                    self.dialogue_erreur = None;
                    self.dialogue = Dialogue::Aucun;
                } else {
                    self.dialogue = Dialogue::CreerGroupe { nom, membres };
                }
            }
            _ => {}
        }
    }

    fn envoyer_vers(
        &mut self,
        dest: &str,
        texte: String,
        reponse_a: Option<String>,
        piece_jointe: Option<PieceJointe>,
    ) {
        if dest == self.session.id {
            self.historique
                .ajouter_local(dest, texte, reponse_a, piece_jointe);
            return;
        }
        if let Some(g) = self.groupes.iter().find(|g| g.id == dest).cloned() {
            for membre in &g.membres {
                if let Some(pair) = self.pairs.get(membre).cloned() {
                    let msg = Message {
                        id: uuid::Uuid::new_v4().to_string(),
                        de: self.session.id.clone(),
                        vers: membre.clone(),
                        texte: texte.clone(),
                        ts: maintenant(),
                        reponse_a: reponse_a.clone(),
                        piece_jointe: piece_jointe.clone(),
                    };
                    let _ = network::envoyer_message_tcp(&pair, &msg);
                }
            }
            self.historique
                .ajouter_local(dest, texte, reponse_a, piece_jointe);
            return;
        }
        if let Some(pair) = self.pairs.get(dest).cloned() {
            let msg = Message {
                id: uuid::Uuid::new_v4().to_string(),
                de: self.session.id.clone(),
                vers: pair.id.clone(),
                texte: texte.clone(),
                ts: maintenant(),
                reponse_a: reponse_a.clone(),
                piece_jointe: piece_jointe.clone(),
            };
            if network::envoyer_message_tcp(&pair, &msg) {
                self.historique
                    .ajouter_local(dest, texte, reponse_a, piece_jointe);
            }
        }
    }

    fn envoyer_message_actif(&mut self, pair_id: &str) {
        let texte = self.brouillon.trim().to_string();
        if texte.is_empty() && self.piece_jointe_attente.is_none() {
            return;
        }
        let pj = self.piece_jointe_attente.take();
        let reponse = self.reponse_a.take();
        self.envoyer_vers(pair_id, texte, reponse, pj);
        self.brouillon.clear();
    }
}

fn maintenant() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn tronquer(s: &str, max: usize) -> String {
    let chars: Vec<char> = s.chars().take(max).collect();
    let mut r: String = chars.into_iter().collect();
    if s.chars().count() > max {
        r.push_str("...");
    }
    r
}

fn dessiner_bulle(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    cache: &mut CacheTextures,
    msg: &crate::chat::MessageLocal,
    sombre: bool,
    langue: Langue,
) -> (Option<String>, Option<String>) {
    let (fond, align) = if msg.de_moi {
        (bulle_moi(sombre), egui::Align::Max)
    } else {
        (bulle_autre(sombre), egui::Align::Min)
    };
    let couleur_texte = if sombre { TEXTE } else { FOND };
    let mut clic: Option<String> = None;
    let mut download: Option<String> = None;

    ui.with_layout(egui::Layout::top_down(align), |ui| {
        let frame_resp = egui::Frame::none()
            .fill(fond)
            .rounding(egui::Rounding::same(10.0))
            .inner_margin(egui::Margin::symmetric(10.0, 7.0))
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width() * 0.72);

                if msg.supprime {
                    ui.label(
                        egui::RichText::new(tr(langue, Cle::MessageSupprime))
                            .italics()
                            .color(SECONDAIRE)
                            .size(12.0),
                    );
                } else {
                    if msg.reponse_a.is_some() {
                        ui.label(
                            egui::RichText::new(format!("| {}", tr(langue, Cle::EnReponseA)))
                                .size(10.0)
                                .color(SECONDAIRE),
                        );
                    }

                    if let Some(pj) = &msg.piece_jointe {
                        if pj.est_image() {
                            if let Some(bytes) = pj.decoder() {
                                let hash = crate::theme::hash_bytes(&bytes);
                                let nom_tex = format!("img_{}", msg.id);
                                if let Some(tex) =
                                    cache.obtenir_avec_hash(ctx, &bytes, hash, &nom_tex)
                                {
                                    let max_w = ui.available_width().min(240.0);
                                    let tex_size = tex.size_vec2();
                                    let ratio = if tex_size.y > 0.0 {
                                        tex_size.x / tex_size.y
                                    } else {
                                        1.0
                                    };
                                    let (w, h) = if ratio >= 1.0 {
                                        (max_w, max_w / ratio)
                                    } else {
                                        (max_w * ratio, max_w)
                                    };
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(w, h),
                                        egui::Sense::hover(),
                                    );
                                    ui.painter().image(
                                        tex.id(),
                                        rect,
                                        egui::Rect::from_min_max(
                                            egui::pos2(0.0, 0.0),
                                            egui::pos2(1.0, 1.0),
                                        ),
                                        egui::Color32::WHITE,
                                    );
                                    ui.add_space(4.0);
                                }
                            }
                        }
                    }

                    if !msg.texte.is_empty() {
                        ui.label(egui::RichText::new(&msg.texte).color(couleur_texte));
                    }

                    if let Some(pj) = &msg.piece_jointe {
                        ui.horizontal(|ui| {
                            let etiquette = if pj.est_image() {
                                "[IMG]"
                            } else {
                                "[FICHIER]"
                            };
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} {} ({} o)",
                                    etiquette,
                                    pj.nom,
                                    pj.taille_octets()
                                ))
                                .color(couleur_texte)
                                .size(11.0),
                            );
                            let btn = ui
                                .add(
                                    egui::Button::new(
                                        egui::RichText::new("↓").size(12.0),
                                    )
                                    .small(),
                                )
                                .on_hover_text(tr(langue, Cle::Enregistrer));
                            if btn.clicked() {
                                download = Some(msg.id.clone());
                            }
                        });
                    }
                }
            });

        let click_resp = ui.interact(
            frame_resp.response.rect,
            ui.id().with(("clic_msg", &msg.id)),
            egui::Sense::click(),
        );
        if click_resp.clicked() && download.is_none() {
            clic = Some(msg.id.clone());
        }
    });
    ui.add_space(ESPACE_S);
    (clic, download)
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.traiter_evenements();
        self.verifier_inactifs();

        let nouveau = match &self.reseau {
            Some(r) => r
                .signal_nouveau
                .swap(false, std::sync::atomic::Ordering::Relaxed),
            None => false,
        };

        if self.theme_courant != self.session.sombre {
            appliquer_theme(ctx, self.session.sombre);
            self.theme_courant = self.session.sombre;
            self.sauver();
        }

        let largeur_totale = ctx.available_rect().width();
        let largeur_gauche = (largeur_totale * 0.30)
            .max(280.0)
            .min(largeur_totale * 0.45);

        egui::SidePanel::left("panneau_gauche")
            .resizable(true)
            .default_width(largeur_gauche)
            .width_range(280.0..=largeur_totale * 0.5)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.panneau_gauche(ui);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.panneau_droite(ui);
        });

        self.afficher_dialogues(ctx);

        if ctx.input(|i| i.viewport().close_requested()) {
            let taille = ctx
                .input(|i| i.viewport().inner_rect)
                .map(|r| [r.width(), r.height()]);
            if let Some(t) = taille {
                self.session.window_size = t;
            }
            self.sauver();
        }

        if nouveau {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
    }
}
