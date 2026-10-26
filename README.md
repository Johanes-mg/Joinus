# Joinus

<div align="center">

<div align="center">
  <img src="images/logo.png" alt="Lechapes Logo" width="120">
</div>

**Messagerie locale multi-utilisateurs pour réseaux partagés (hotspot, LAN).**

[![GitHub release](https://img.shields.io/github/v/release/Johanes-mg/Lechapes?style=flat-square)](https://github.com/Johanes-mg/Lechapes/releases)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg?style=flat-square)](https://www.rust-lang.org)

</div>


# Fonctionnalités principales

- **Découverte automatique sur le réseau local** : chaque instance diffuse une annonce toutes les 5 secondes sur le réseau, avec support du broadcast global et du broadcast dirigé par interface (compatible Wi-Fi domestique, hotspot, LAN).
- **Messagerie temps réel** : échange de messages via TCP avec indicateur de transmission instantané, sans latence perceptible, et sans nécessiter de rafraîchir manuellement la conversation.
- **Avatars partagés entre pairs** : chaque utilisateur peut importer une photo de profil, automatiquement recadrée en carré et transmise aux autres participants via TCP. Les avatars distants apparaissent dans la liste des personnes connectées.
- **Historique persistant des conversations** : les messages échangés avec chaque pair sont sauvegardés localement et restaurés au lancement de l'application, un fichier JSON par conversation.
- **Profil personnalisable** : nom d'affichage, photo de profil arrondie, avec persistance dans `%APPDATA%\Joinus\`.
- **Mode test local** : possibilité de s'envoyer un message à soi-même ("Moi-même") pour tester l'interface sans avoir besoin d'un second poste.
- **Interface multilingue** : disponible en **Français** et **Anglais**, avec bascule instantanée sans redémarrage.
- **Mode Sombre / Clair** : bascule facile entre un thème sombre élégant (`#131514`) et un thème clair lumineux (`#f7f7f5`), avec prise en charge complète des couleurs adaptatives.
- **Rafraîchissement manuel** : bouton "Rafraîchir" pour forcer une annonce réseau immédiate si la découverte semble lente.
- **Installation sans configuration** : les règles pare-feu Windows (UDP 42001 et programme entrant) sont ajoutées automatiquement par l'installeur et retirées à la désinstallation.

---

## Confidentialité et sécurité

- **Aucune connexion externe** : Joinus fonctionne exclusivement sur le réseau local. Aucune donnée ne transite par Internet, aucun serveur distant n'est contacté.
- **Aucun compte** : un identifiant unique (UUID v4) est généré au premier lancement et stocké localement dans `session.json`. Aucune inscription, aucun mot de passe.
- **Messages non chiffrés** : les messages circulent en clair sur le réseau local. Joinus est pensé pour un réseau de confiance (Wi-Fi domestique, hotspot personnel), pas pour un réseau public. Cette limite est volontaire et documentée dans l'interface.
- **Données locales** : toutes les données (profil, avatar, historique) restent sur votre machine, dans `%APPDATA%\Joinus\`. La désinstallation propose de les conserver ou de les supprimer.

```
Joinus/
│
├── images/                 # Ressources graphiques (logo, icônes, boutons)
│   ├── logo.ico
│   ├── logo.png
│   ├── envoyer.png
│   └── rafraichir.png
│
├── Installeur/             # Sortie de l'installeur Inno Setup (non versionné)
├── src/                    # Code source principal en Rust
│   ├── app.rs              # Logique globale de l'application UI, gestion des panneaux
│   ├── chat.rs             # Historique des conversations et persistance des messages
│   ├── i18n.rs             # Internationalisation (FR, EN)
│   ├── main.rs             # Point d'entrée, ViewportBuilder, chargement icône fenêtre
│   ├── network.rs          # Découverte UDP, serveur TCP, protocole et transfert d'avatars
│   ├── peers.rs            # Liste des pairs détectés et cache local des avatars
│   ├── persistence.rs      # Session, UUID, avatar et dossier de données utilisateur
│   ├── profile.rs          # Profil local (nom, photo, recadrage carré, erreurs)
│   └── theme.rs            # Thèmes clair / sombre, cache textures, boutons image
│
├── .gitignore              # Fichiers et dossiers ignorés par Git
├── Cargo.lock              # Versions exactes des dépendances Cargo
├── Cargo.toml              # Configuration et métadonnées du projet Rust
├── installer.iss           # Script de création de l'installateur Windows avec règles pare-feu
├── README.md               # Documentation du projet
└── rust-toolchain.toml     # Configuration de la chaîne d'outils Rust

```

## Environnement

- Windows
- Rust edition 2021, toolchain stable, cible x86_64-pc-windows-gnullvm
- egui / eframe 0.28, backend wgpu (feature dx12)

## Compilation

    cargo build --release

## Données utilisateur

Session et avatar dans %APPDATA%\Joinus\ :
- session.json : nom, langue, thème
- avatar.dat : photo de profil

```
## 👤 Auteur

 * **Johanes-mg** 
 
 
## 📄 Licence
Ce projet est sous licence ouverte. Voir le fichier LICENSE pour plus de détails.

```
