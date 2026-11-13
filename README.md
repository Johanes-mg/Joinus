# Joinus

<div align="center">

<img src="images/logo.png" alt="Joinus logo" width="120">

**Local multi-user messaging for shared networks (hotspot, LAN).**

[![GitHub release](https://img.shields.io/github/v/release/Johanes-mg/Joinus?style=flat-square)](https://github.com/Johanes-mg/Joinus/releases)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg?style=flat-square)](https://www.rust-lang.org)

</div>

## Overview

Joinus is a lightweight desktop messaging application for local networks. It auto-discovers peers on the same LAN through UDP broadcast, exchanges messages over TCP, and keeps every conversation stored locally. No server, no account, no cloud. Built in Rust with egui / eframe and packaged as a single static Windows executable.

## Key features

- **Automatic local network discovery**: every instance broadcasts an announcement every 5 seconds, using both global broadcast and per-interface directed broadcast (works on home Wi-Fi, hotspot, and LAN).
- **Real-time messaging**: TCP message exchange with instant delivery indicators, no perceptible latency, no manual refresh required.
- **Shared peer avatars**: each user can import a profile picture, automatically cropped to a square and transmitted to other participants over TCP. Remote avatars appear in the connected people list.
- **Persistent conversation history**: messages exchanged with each peer are saved locally and restored at startup, one JSON file per conversation.
- **Customizable profile**: display name and rounded profile picture, persisted in `%APPDATA%\Joinus\`.
- **Local test mode**: send a message to yourself ("Myself") to test the interface without needing a second machine.
- **Multilingual interface**: available in **French** and **English**, with instant switching at runtime.
- **Dark / Light themes**: quick toggle between a dark theme (`#131514`) and a light theme (`#f7f7f5`), with full adaptive color support.
- **Manual refresh**: a "Refresh" button forces an immediate network announcement if discovery seems slow.
- **Groups**: create local groups from the home screen and broadcast a message to all connected members at once.
- **Message actions**: click any message to reveal inline reply, forward, and delete actions, shown between the selected message and the next one.
- **File and image attachments**: attach any file up to 5 MB via the "+" button. Images are previewed inline above the message text, other files are listed with their name and size.
- **Attachment download**: any received or sent attachment can be saved back to disk with the download button on the message bubble.
- **Zero-configuration install**: Windows Firewall rules (UDP 42001 and inbound program) are added automatically by the installer and removed on uninstall.

## Privacy and security

- **No external connection**: Joinus runs exclusively on the local network. No data transits through the Internet, no remote server is contacted.
- **No account**: a unique identifier (UUID v4) is generated on first launch and stored locally in `session.json`. No sign-up, no password.
- **Messages are not encrypted**: messages travel in cleartext on the local network. Joinus is designed for trusted networks (home Wi-Fi, personal hotspot), not for public networks. This limitation is deliberate and documented in the interface.
- **Local data**: all data (profile, avatar, history) stays on your machine, in `%APPDATA%\Joinus\`. The uninstaller offers to keep or delete it.
- **Not intended for enterprise compliance**: Joinus does not provide transport encryption, authentication, audit trails, or guaranteed remote deletion. For regulated or enterprise messaging, use a dedicated solution (Mattermost, Rocket.Chat, Matrix/Synapse).

## Project structure

```text
Joinus/
│
├── images/                          # Graphical assets (logo, icons, buttons)
│   ├── illustration.png             # Home screen illustration
│   ├── logo.ico                     # Multi-size icon (256, 128, 64, 48, 32, 16)
│   ├── logo.png                     # Window icon source
│   ├── envoyer.png                  # Send button
│   ├── plus-light.png               # Attach button (dark theme)
│   ├── plus-dark.png                # Attach button (light theme)
│   ├── refresh-light.png            # Refresh button (dark theme)
│   ├── refresh-dark.png             # Refresh button (light theme)
│   ├── dark.png                     # Theme toggle, switch to dark (light theme)
│   ├── light.png                    # Theme toggle, switch to light (dark theme)
│   ├── 3pointslight.png             # Options menu button (dark theme)
│   ├── 3pointsdark.png              # Options menu button (light theme)
│   ├── group-light.png              # Create group icon (dark theme)
│   ├── group-dark.png               # Create group icon (light theme)
│   ├── reply-light.png              # Reply action (dark theme)
│   ├── reply-dark.png               # Reply action (light theme)
│   ├── share-light.png              # Forward action (dark theme)
│   ├── share-dark.png               # Forward action (light theme)
│   ├── delete-light.png             # Delete action (dark theme)
│   └── delete-dark.png              # Delete action (light theme)
│
├── Installeur/                      # Inno Setup output (not versioned)
├── src/                             # Rust source code
│   ├── app.rs                       # Global UI logic, panels, dialogs, message actions
│   ├── chat.rs                      # Conversation history and message persistence
│   ├── groupes.rs                   # Local groups (creation, membership, persistence)
│   ├── i18n.rs                      # Internationalization (FR, EN)
│   ├── main.rs                      # Entry point, ViewportBuilder, font and window icon loading
│   ├── network.rs                   # UDP discovery, TCP server, protocol, avatar transfer
│   ├── peers.rs                     # Detected peers list and local avatar cache
│   ├── persistence.rs               # Session, UUID, avatar, alias, user data directory
│   ├── pieces_jointes.rs            # File and image attachments (base64, 5 MB limit)
│   ├── profile.rs                   # Local profile (name, picture, square crop, errors)
│   └── theme.rs                     # Light / dark themes, texture cache, image buttons
│
├── .cargo/
│   └── config.toml                  # Target and static runtime configuration
├── .gitignore                       # Files and folders ignored by Git
├── Cargo.lock                       # Exact dependency versions
├── Cargo.toml                       # Rust project configuration and metadata
├── installer.iss                    # Inno Setup script with Windows Firewall rules
├── LICENSE                          # MIT License
├── README.md                        # Project documentation
└── rust-toolchain.toml              # Rust toolchain configuration
```

## Environment

- **OS**: Windows 10 or 11, x86_64
- **Language**: Rust edition 2021
- **Toolchain**: stable, target `x86_64-pc-windows-gnullvm`
  The GNU toolchain crashes at startup on some configurations, so the gnullvm target is enforced and `crt-static` + `link-self-contained` are enabled to produce a fully statically linked executable (no `libunwind.dll` or MinGW runtime dependency).
- **GUI**: egui / eframe 0.28, wgpu backend with DX12 feature
- **Working directory**: `D:\Creation\Rust\Joinus\`

## Build

```powershell
cargo build --release
```

The resulting binary is written to `target\x86_64-pc-windows-gnullvm\release\joinus.exe`. It is a single static executable with no external runtime dependency.

## User data

Stored in `%APPDATA%\Joinus\` (falls back to `%TEMP%\Joinus\` if `APPDATA` is unavailable):

- `session.json`: display name, language, theme, window size, UUID
- `avatar.dat`: profile picture (square-cropped PNG)
- `alias.json`: local nickname overrides for remote peers
- `groupes.json`: local group definitions
- `historique\<peer_uuid>.json`: one JSON file per conversation

## Attachments

- **Maximum size**: 5 MB per file (enforced before encoding)
- **Encoding**: base64 inside the JSON message payload over TCP
- **Supported MIME detection**: PNG, JPEG, GIF, WebP, PDF, TXT, ZIP, generic binary fallback
- **Preview**: images are previewed inline above the message text
- **Download**: any attachment (image or file) can be saved back to disk via the download button on the message bubble

## Groups

Groups are local by design. Each member must create the same group on their own machine to see it. Messages sent to a group are delivered individually to each member as a 1-to-1 message on the recipient side — this is a limitation of a purely peer-to-peer architecture without a central server.

## Message deletion

Deleting a message removes it from your local history and sends a best-effort deletion notice to the peer if they are currently online. There is no guarantee: the recipient may ignore it, be offline at that moment, or have modified their client. The local copy is always deleted, the remote copy is a courtesy request only.

## Author

**Johanes-mg**

- GitHub: [@Johanes-mg](https://github.com/Johanes-mg)
- Project: [Joinus](https://github.com/Johanes-mg/Joinus)

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for details.
