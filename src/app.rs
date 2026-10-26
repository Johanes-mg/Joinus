use crate::chat::Historique;
use crate::i18n::{tr, Cle, Langue};
use crate::network::{self, Evenement, Message, Reseau, TIMEOUT_PRESENCE};
use crate::peers::ListePairs;
use crate::persistence::{self, Session};
use crate::profile::{ErreurAvatar, Profil};
use crate::theme::{
    appliquer_theme, bulle_autre, bulle_moi, carte_pair_fond, carte_pair_fond_actif,
    carte_pair_fond_survol, charger_texture_fichier, dessiner_avatar_arrondi,
    dessiner_avatar_petit, CacheTextures, AVATAR_LISTE_TAILLE, AVATAR_TAILLE,
    BOUTON_ENVOYER_ICONE, BOUTON_ENVOYER_LARGEUR, BOUTON_RAFRAICHIR_ICONE,
    BOUTON_RAFRAICHIR_TAILLE, CARTE_PAIR_HAUTEUR, CARTE_PAIR_ROUNDING,
    CHAMP_SAISIE_HAUTEUR, ERREUR, ESPACE_L, ESPACE_M, ESPACE_S, ESPACE_XL, FOND,
    LIMITE_MESSAGE, SECONDAIRE, TEXTE,
};
use eframe::egui;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DELAI_AVANT_DEMANDE_AVATAR: Duration = Duration::from_secs(30);

pub struct App {
    session: Session,
    profil: Profil,
    cache: CacheTextures,
    theme_courant: bool,
    brouillon: String,
    reseau: Option<Reseau>,
    pairs: ListePairs,
    pair_selectionne: Option<String>,
    historique: Historique,
    envoyer_texture: Option<egui::TextureHandle>,
    envoyer_erreur: bool,
    rafraichir_texture: Option<egui::TextureHandle>,
    rafraichir_erreur: bool,
    derniere_rafraichie: Instant,
    avatars_demandes: HashSet<String>,
    avatars_demandes_le: std::collections::HashMap<String, Instant>,
    derniere_verification_inactifs: Instant,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>, session: Session) -> Self {
        appliquer_theme(&cc.egui_ctx, session.sombre);
        let profil = Profil::charger(&session);
        let theme_courant = session.sombre;

        let chemin_envoyer = PathBuf::from("images").join("envoyer.png");
        let envoyer_texture =
            charger_texture_fichier(&cc.egui_ctx, &chemin_envoyer, "envoyer");
        let envoyer_erreur = envoyer_texture.is_none();

        let chemin_rafraichir = PathBuf::from("images").join("rafraichir.png");
        let rafraichir_texture =
            charger_texture_fichier(&cc.egui_ctx, &chemin_rafraichir, "rafraichir");
        let rafraichir_erreur = rafraichir_texture.is_none();

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
            pair_selectionne: None,
            historique: Historique::new(),
            envoyer_texture,
            envoyer_erreur,
            rafraichir_texture,
            rafraichir_erreur,
            derniere_rafraichie: Instant::now(),
            avatars_demandes: HashSet::new(),
            avatars_demandes_le: std::collections::HashMap::new(),
            derniere_verification_inactifs: Instant::now(),
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

    fn traiter_evenements(&mut self) {
        let mut a_demander: Vec<crate::network::Pair> = Vec::new();

        if let Some(reseau) = &self.reseau {
            while let Ok(evt) = reseau.evenements.try_recv() {
                match evt {
                    Evenement::PairVu(pair) => {
                        let id = pair.id.clone();
                        self.pairs.ajouter_ou_mettre_a_jour(pair.clone());

                        let deja_demande = self.avatars_demandes.contains(&id);
                        let avatar_connu = self.pairs.avatar(&id).is_some();
                        let doit_demander = !deja_demande || {
                            match self.avatars_demandes_le.get(&id) {
                                Some(t) => {
                                    t.elapsed() > DELAI_AVANT_DEMANDE_AVATAR && !avatar_connu
                                }
                                None => !avatar_connu,
                            }
                        };
                        if !avatar_connu && doit_demander && id != self.session.id {
                            a_demander.push(pair);
                        }
                    }
                    Evenement::MessageRecu(msg) => {
                        self.historique.ajouter_recu(&msg.de, &msg);
                    }
                    Evenement::AvatarRecu { id, bytes } => {
                        self.pairs.definir_avatar(&id, bytes);
                        self.avatars_demandes_le.insert(id, Instant::now());
                    }
                }
            }
        }

        for pair in a_demander {
            self.avatars_demandes.insert(pair.id.clone());
            self.avatars_demandes_le.insert(pair.id.clone(), Instant::now());
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
            if self.pair_selectionne.as_deref() == Some(&id) {
                self.pair_selectionne = None;
            }
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

    fn dessiner_carte_pair(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        pair_id: &str,
        nom: &str,
        est_moi: bool,
        selectionne: bool,
    ) -> egui::Response {
        let largeur = ui.available_width();
        let taille = egui::vec2(largeur, CARTE_PAIR_HAUTEUR);
        let (rect, reponse) = ui.allocate_exact_size(taille, egui::Sense::click());

        let sombre = self.session.sombre;
        let fond = if selectionne {
            carte_pair_fond_actif(sombre)
        } else if reponse.hovered() {
            carte_pair_fond_survol(sombre)
        } else {
            carte_pair_fond(sombre)
        };
        ui.painter()
            .rect_filled(rect, egui::Rounding::same(CARTE_PAIR_ROUNDING), fond);

        let marge = ESPACE_S;
        let avatar_rect = egui::Rect::from_center_size(
            egui::pos2(rect.left() + marge + AVATAR_LISTE_TAILLE / 2.0, rect.center().y),
            egui::vec2(AVATAR_LISTE_TAILLE, AVATAR_LISTE_TAILLE),
        );

        let texture = self.texture_avatar_pair(ctx, pair_id);
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

        let couleur_texte = if est_moi {
            SECONDAIRE
        } else if sombre {
            TEXTE
        } else {
            FOND
        };
        let x_texte = avatar_rect.right() + ESPACE_M;
        let zone_texte = egui::Rect::from_min_max(
            egui::pos2(x_texte, rect.top() + ESPACE_S),
            egui::pos2(rect.right() - ESPACE_S, rect.bottom() - ESPACE_S),
        );

        let texte_principal = if est_moi {
            format!("{} ({})", nom, tr(self.session.langue, Cle::MoiMeme))
        } else {
            nom.to_string()
        };

        ui.painter().text(
            egui::pos2(zone_texte.left(), zone_texte.center().y),
            egui::Align2::LEFT_CENTER,
            texte_principal,
            egui::FontId::proportional(14.0),
            couleur_texte,
        );

        reponse
    }

    fn panneau_gauche(&mut self, ui: &mut egui::Ui) {
        let langue = self.session.langue;
        let sombre = self.session.sombre;
        ui.add_space(ESPACE_M);

        ui.horizontal(|ui| {
            ui.heading(tr(langue, Cle::Profil));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let mut sombre_local = sombre;
                if ui
                    .checkbox(&mut sombre_local, tr(langue, Cle::ThemeSombre))
                    .changed()
                {
                    self.session.sombre = sombre_local;
                }
                egui::ComboBox::from_id_source("langue")
                    .selected_text(match langue {
                        Langue::Fr => "FR",
                        Langue::En => "EN",
                    })
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(langue == Langue::Fr, "FR").clicked() {
                            self.session.langue = Langue::Fr;
                        }
                        if ui.selectable_label(langue == Langue::En, "EN").clicked() {
                            self.session.langue = Langue::En;
                        }
                    });
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
            let reponse = reponse.on_hover_cursor(egui::CursorIcon::PointingHand);
            if reponse.hovered() {
                let couleur_texte = if sombre { TEXTE } else { FOND };
                ui.painter().circle_stroke(
                    rect.center(),
                    AVATAR_TAILLE / 2.0,
                    egui::Stroke::new(3.0_f32, couleur_texte),
                );
            }
            if reponse.clicked() {
                self.profil.importer_avatar();
                self.rafraichir_mon_avatar_dans_cache();
            }
            ui.add_space(ESPACE_S);
            if ui.button(tr(langue, Cle::ChangerPhoto)).clicked() {
                self.profil.importer_avatar();
                self.rafraichir_mon_avatar_dans_cache();
            }
            if let Some(texte) = self.texte_erreur_avatar(langue) {
                ui.colored_label(ERREUR, texte);
            }
        });

        ui.add_space(ESPACE_L);
        ui.label(tr(langue, Cle::Nom));
        if ui.text_edit_singleline(&mut self.profil.nom).lost_focus() {
            self.sauver();
        }

        ui.add_space(ESPACE_L);
        ui.separator();
        ui.add_space(ESPACE_M);

        let nombre_pairs = self.pairs.len();
        ui.horizontal(|ui| {
            ui.heading(tr(langue, Cle::PersonnesConnectees));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let taille = egui::vec2(BOUTON_RAFRAICHIR_TAILLE, BOUTON_RAFRAICHIR_TAILLE);
                let (rect, reponse) = ui.allocate_exact_size(taille, egui::Sense::click());
                let style = ui.style().interact(&reponse);
                ui.painter().rect_filled(rect, style.rounding, style.weak_bg_fill);
                let couleur_icone = if sombre { TEXTE } else { FOND };
                if let Some(tex) = &self.rafraichir_texture {
                    let zone = egui::Rect::from_center_size(
                        rect.center(),
                        egui::vec2(BOUTON_RAFRAICHIR_ICONE, BOUTON_RAFRAICHIR_ICONE),
                    );
                    ui.painter().image(
                        tex.id(),
                        zone,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        couleur_icone,
                    );
                } else {
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "R",
                        egui::FontId::proportional(14.0),
                        couleur_icone,
                    );
                }
                let reponse = reponse
                    .on_hover_text(tr(langue, Cle::RafraichirTooltip))
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                if reponse.clicked() {
                    self.rafraichir();
                }
                if self.rafraichir_erreur {
                    ui.colored_label(ERREUR, "!");
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

        let pairs_tries: Vec<(String, String, bool)> = {
            let mut v: Vec<(String, String, bool)> = self
                .pairs
                .iter()
                .map(|p| (p.id.clone(), p.nom.clone(), false))
                .collect();
            v.push((self.session.id.clone(), self.profil.nom.clone(), true));
            v.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
            v
        };

        let ctx = ui.ctx().clone();
        for (id, nom, est_moi) in pairs_tries {
            let selectionne = self.pair_selectionne.as_deref() == Some(&id);
            let reponse = self.dessiner_carte_pair(ui, &ctx, &id, &nom, est_moi, selectionne);
            let reponse = reponse.on_hover_cursor(egui::CursorIcon::PointingHand);
            if reponse.clicked() {
                self.pair_selectionne = Some(id.clone());
                self.historique.selectionner(&id);
            }
            ui.add_space(ESPACE_S);
        }

        if nombre_pairs == 0 {
            ui.add_space(ESPACE_M);
            let ecoule = self.derniere_rafraichie.elapsed();
            if ecoule < Duration::from_secs(3) {
                ui.label(
                    egui::RichText::new(tr(langue, Cle::RechercheEnCours))
                        .color(SECONDAIRE)
                        .size(11.0),
                );
            } else {
                ui.label(
                    egui::RichText::new(tr(langue, Cle::AidePareFeu))
                        .color(SECONDAIRE)
                        .size(11.0),
                );
            }
        }
    }

    fn panneau_droite(&mut self, ui: &mut egui::Ui) {
        let langue = self.session.langue;
        let sombre = self.session.sombre;
        ui.add_space(ESPACE_M);
        ui.horizontal(|ui| {
            let titre = self
                .pair_selectionne
                .as_ref()
                .and_then(|id| {
                    if id == &self.session.id {
                        Some(format!(
                            "{} ({})",
                            self.profil.nom,
                            tr(langue, Cle::MoiMeme)
                        ))
                    } else {
                        self.pairs.get(id).map(|p| p.nom.clone())
                    }
                })
                .unwrap_or_else(|| tr(langue, Cle::Conversation).to_string());
            ui.heading(titre);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(tr(langue, Cle::NonChiffre))
                        .color(SECONDAIRE)
                        .size(10.0),
                );
            });
        });
        ui.separator();

        let hauteur_saisie = CHAMP_SAISIE_HAUTEUR + ESPACE_XL + ESPACE_S;
        let hauteur_dispo = ui.available_height();
        let hauteur_messages = (hauteur_dispo - hauteur_saisie - ESPACE_M).max(0.0);

        let est_moi = self
            .pair_selectionne
            .as_ref()
            .map(|id| id == &self.session.id)
            .unwrap_or(false);

        ui.allocate_ui(egui::vec2(ui.available_width(), hauteur_messages), |ui| {
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .auto_shrink([false, false])
                .show(ui, |ui| match &self.pair_selectionne {
                    None => {
                        ui.vertical_centered(|ui| {
                            ui.add_space(40.0);
                            ui.label(
                                egui::RichText::new(tr(langue, Cle::SelectionnerPersonne))
                                    .color(SECONDAIRE)
                                    .size(15.0),
                            );
                        });
                    }
                    Some(_) => {
                        let messages = self
                            .historique
                            .messages_actifs()
                            .map(|v| v.to_vec())
                            .unwrap_or_default();
                        if messages.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.add_space(40.0);
                                ui.label(
                                    egui::RichText::new(tr(langue, Cle::HistoriqueVide))
                                        .color(SECONDAIRE)
                                        .size(14.0),
                                );
                                if est_moi {
                                    ui.add_space(ESPACE_S);
                                    ui.label(
                                        egui::RichText::new(tr(langue, Cle::MessageDeMoi))
                                            .color(SECONDAIRE)
                                            .size(11.0),
                                    );
                                }
                            });
                        } else {
                            for msg in messages {
                                dessiner_bulle(ui, &msg, sombre);
                                ui.add_space(ESPACE_S);
                            }
                        }
                    }
                });
        });

        ui.add_space(ESPACE_S);
        egui::Frame::none()
            .fill(if sombre { FOND } else { TEXTE })
            .inner_margin(egui::Margin::same(ESPACE_S))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let largeur_champ =
                        (ui.available_width() - BOUTON_ENVOYER_LARGEUR - ESPACE_S).max(120.0);
                    let hauteur_champ = CHAMP_SAISIE_HAUTEUR;
                    let mut envoyer_par_entree = false;
                    ui.allocate_ui(egui::vec2(largeur_champ, hauteur_champ), |ui| {
                        let reponse = ui.add_sized(
                            [largeur_champ, hauteur_champ],
                            egui::TextEdit::multiline(&mut self.brouillon)
                                .hint_text(tr(langue, Cle::ZoneSaisie))
                                .desired_rows(4)
                                .char_limit(LIMITE_MESSAGE),
                        );
                        if reponse.has_focus() {
                            let entree = ui.input(|i| i.key_pressed(egui::Key::Enter));
                            let maj = ui.input(|i| i.modifiers.shift);
                            if entree && !maj {
                                envoyer_par_entree = true;
                            }
                        }
                    });
                    ui.add_space(ESPACE_S);
                    let actif =
                        self.pair_selectionne.is_some() && !self.brouillon.trim().is_empty();
                    let taille_bouton = egui::vec2(BOUTON_ENVOYER_LARGEUR, hauteur_champ);
                    let (rect, reponse) =
                        ui.allocate_exact_size(taille_bouton, egui::Sense::click());
                    let style = ui.style().interact(&reponse);
                    let painter = ui.painter();
                    let fond = if actif {
                        style.weak_bg_fill
                    } else {
                        ui.visuals().faint_bg_color
                    };
                    painter.rect_filled(rect, style.rounding, fond);
                    if let Some(tex) = &self.envoyer_texture {
                        let cote = BOUTON_ENVOYER_ICONE;
                        let centre = rect.center();
                        let zone = egui::Rect::from_center_size(centre, egui::vec2(cote, cote));
                        let teinte = if actif {
                            egui::Color32::WHITE
                        } else {
                            SECONDAIRE
                        };
                        painter.image(
                            tex.id(),
                            zone,
                            egui::Rect::from_min_max(
                                egui::pos2(0.0, 0.0),
                                egui::pos2(1.0, 1.0),
                            ),
                            teinte,
                        );
                    } else {
                        let couleur_texte = if sombre { TEXTE } else { FOND };
                        painter.text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            tr(langue, Cle::Envoyer),
                            egui::FontId::proportional(14.0),
                            if actif { couleur_texte } else { SECONDAIRE },
                        );
                    }
                    if self.envoyer_erreur {
                        painter.text(
                            rect.left_bottom() + egui::vec2(0.0, 4.0),
                            egui::Align2::LEFT_TOP,
                            tr(langue, Cle::ErreurImageEnvoyer),
                            egui::FontId::proportional(10.0),
                            ERREUR,
                        );
                    }
                    let tooltip = if actif {
                        tr(langue, Cle::EnvoyerTooltip)
                    } else if self.pair_selectionne.is_none() {
                        tr(langue, Cle::AucunDestinataire)
                    } else {
                        tr(langue, Cle::ZoneSaisie)
                    };
                    let reponse = reponse.on_hover_text(tooltip);
                    if actif {
                        let reponse = reponse.on_hover_cursor(egui::CursorIcon::PointingHand);
                        let clic = reponse.clicked();
                        if clic || envoyer_par_entree {
                            self.envoyer_message_actif();
                        }
                    }
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
                        .size(11.0),
                    );
                });
            });
    }

    fn envoyer_message_actif(&mut self) {
        let Some(pair_id) = self.pair_selectionne.clone() else {
            return;
        };
        let texte = self.brouillon.trim().to_string();
        if texte.is_empty() {
            return;
        }
        if pair_id == self.session.id {
            self.historique.ajouter_local(&pair_id, texte);
            self.brouillon.clear();
            return;
        }
        let Some(pair) = self.pairs.get(&pair_id).cloned() else {
            return;
        };
        let message = Message {
            de: self.session.id.clone(),
            vers: pair.id.clone(),
            texte: texte.clone(),
            ts: maintenant(),
        };
        if network::envoyer_message_tcp(&pair, &message) {
            self.historique.ajouter_local(&pair.id, texte);
            self.brouillon.clear();
        }
    }
}

fn maintenant() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn dessiner_bulle(ui: &mut egui::Ui, msg: &crate::chat::MessageLocal, sombre: bool) {
    let (fond, align) = if msg.de_moi {
        (bulle_moi(sombre), egui::Align::Max)
    } else {
        (bulle_autre(sombre), egui::Align::Min)
    };
    let couleur_texte = if sombre { TEXTE } else { FOND };
    ui.with_layout(egui::Layout::top_down(align), |ui| {
        egui::Frame::none()
            .fill(fond)
            .rounding(egui::Rounding::same(8.0))
            .inner_margin(egui::Margin::same(8.0))
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width() * 0.7);
                ui.label(egui::RichText::new(&msg.texte).color(couleur_texte));
            });
    });
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.traiter_evenements();
        self.verifier_inactifs();

        let nouveau = match &self.reseau {
            Some(r) => r.signal_nouveau.swap(false, std::sync::atomic::Ordering::Relaxed),
            None => false,
        };

        if self.theme_courant != self.session.sombre {
            appliquer_theme(ctx, self.session.sombre);
            self.theme_courant = self.session.sombre;
            self.sauver();
        }

        let largeur_totale = ctx.available_rect().width();
        let largeur_gauche = (largeur_totale * 0.30).max(240.0).min(largeur_totale * 0.45);

        egui::SidePanel::left("panneau_gauche")
            .resizable(true)
            .default_width(largeur_gauche)
            .width_range(240.0..=largeur_totale * 0.5)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.panneau_gauche(ui);
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            self.panneau_droite(ui);
        });

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
