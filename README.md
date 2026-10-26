# Joinus

Messagerie locale multi-utilisateurs pour réseaux partagés (hotspot, LAN).

## Version

0.1.0

## État

Étape 1 terminée : interface, profil (photo + nom), thème sombre/clair, persistance session.
Étape 2 à venir : découverte réseau UDP, messagerie TCP, liste des personnes connectées.

## Environnement

- Windows 10 ou 11, x86_64
- Rust edition 2021, toolchain stable, cible x86_64-pc-windows-gnullvm
- egui / eframe 0.28, backend wgpu (feature dx12)

## Compilation

    cargo build --release

## Pipeline complet de release

    cargo build --release

    rcedit "D:\Creation\Rust\Joinus\target\release\joinus.exe" `
        --set-icon "D:\Creation\Rust\Joinus\images\logo.ico" `
        --set-file-version "0.1.0" `
        --set-product-version "0.1.0" `
        --set-version-string "ProductName" "Joinus" `
        --set-version-string "FileDescription" "Messagerie locale multi-utilisateurs" `
        --set-version-string "CompanyName" "Johanes-mg" `
        --set-version-string "LegalCopyright" "Johanes-mg" `
        --set-version-string "OriginalFilename" "joinus.exe"

    & "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" "D:\Creation\Rust\Joinus\installer.iss"

Sortie : D:\Creation\Rust\Joinus\Installeur\JoinusSetup.exe

## Données utilisateur

Session et avatar dans %APPDATA%\Joinus\ :
- session.json : nom, langue, thème
- avatar.dat : photo de profil

## Icône

images/logo.png : icône fenêtre chargée au runtime.
images/logo.ico : icône multi-tailles pour l'exe et l'installeur.

Régénérer le .ico depuis le .png :

    magick "images\logo.png" -define icon:auto-resize=256,128,64,48,32,16 "images\logo.ico"
