use crate::pieces_jointes::PieceJointe;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub const PORT_UDP: u16 = 42001;
pub const INTERVALLE_ANNONCE: Duration = Duration::from_secs(5);
pub const TIMEOUT_PRESENCE: Duration = Duration::from_secs(20);
pub const LIMITE_MESSAGE_OCTETS: usize = 32 * 1024 * 1024;
pub const LIMITE_AVATAR_OCTETS: usize = 5 * 1024 * 1024;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Annonce {
    pub id: String,
    pub nom: String,
    pub tcp_port: u16,
    pub version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    #[serde(default)]
    pub id: String,
    pub de: String,
    pub vers: String,
    pub texte: String,
    pub ts: i64,
    #[serde(default)]
    pub reponse_a: Option<String>,
    #[serde(default)]
    pub piece_jointe: Option<PieceJointe>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Enveloppe {
    Chat {
        id: String,
        de: String,
        vers: String,
        texte: String,
        ts: i64,
        #[serde(default)]
        reponse_a: Option<String>,
        #[serde(default)]
        piece_jointe: Option<PieceJointe>,
    },
    DemandeAvatar {
        de: String,
    },
    ReponseAvatar {
        de: String,
        avatar_b64: String,
    },
    SupprimerMessage {
        de: String,
        vers: String,
        id: String,
    },
}

#[derive(Clone, Debug)]
pub struct Pair {
    pub id: String,
    pub nom: String,
    pub adresse: SocketAddr,
    pub tcp_port: u16,
    pub derniere_annonce: Instant,
}

pub enum Evenement {
    PairVu(Pair),
    MessageRecu(Message),
    AvatarRecu { id: String, bytes: Vec<u8> },
    MessageSupprime { de: String, id: String },
}

pub struct Reseau {
    pub evenements: Receiver<Evenement>,
    pub signal_nouveau: Arc<AtomicBool>,
    tx_evenements: Sender<Evenement>,
    signal_rafraichir: Arc<AtomicBool>,
    _gardes: Vec<thread::JoinHandle<()>>,
}

impl Reseau {
    pub fn demarrer(id_local: String, nom_local: String) -> Option<Self> {
        let bind_udp: SocketAddr = format!("0.0.0.0:{}", PORT_UDP).parse().ok()?;
        let socket_udp = UdpSocket::bind(bind_udp).ok()?;
        socket_udp.set_broadcast(true).ok()?;

        let listener_tcp = TcpListener::bind("0.0.0.0:0").ok()?;
        let tcp_port = listener_tcp.local_addr().ok()?.port();

        let (tx, rx) = channel();
        let pairs_arc: Arc<Mutex<Vec<Pair>>> = Arc::new(Mutex::new(Vec::new()));
        let signal_rafraichir = Arc::new(AtomicBool::new(false));
        let signal_nouveau = Arc::new(AtomicBool::new(false));

        let annonce = Annonce {
            id: id_local.clone(),
            nom: nom_local.clone(),
            tcp_port,
            version: env!("CARGO_PKG_VERSION").to_string(),
        };

        let mut gardes = Vec::new();

        gardes.push(spawn_emetteur_udp(
            socket_udp.try_clone().ok()?,
            annonce,
            signal_rafraichir.clone(),
        ));
        gardes.push(spawn_recepteur_udp(
            socket_udp,
            id_local.clone(),
            tx.clone(),
            pairs_arc.clone(),
            signal_nouveau.clone(),
        ));
        gardes.push(spawn_serveur_tcp(
            listener_tcp,
            id_local,
            tx.clone(),
            signal_nouveau.clone(),
        ));

        Some(Self {
            evenements: rx,
            signal_nouveau,
            tx_evenements: tx,
            signal_rafraichir,
            _gardes: gardes,
        })
    }

    pub fn rafraichir(&self) {
        self.signal_rafraichir.store(true, Ordering::Relaxed);
    }

    pub fn demander_avatar_async(&self, pair: Pair, mon_id: String) {
        let tx = self.tx_evenements.clone();
        let signal = self.signal_nouveau.clone();
        thread::spawn(move || {
            if let Some(bytes) = demander_avatar(&pair, &mon_id) {
                let _ = tx.send(Evenement::AvatarRecu {
                    id: pair.id.clone(),
                    bytes,
                });
                signal.store(true, Ordering::Relaxed);
            }
        });
    }
}

fn adresses_broadcast() -> Vec<SocketAddr> {
    let mut cibles = Vec::new();
    let global: SocketAddr = format!("255.255.255.255:{}", PORT_UDP).parse().unwrap();
    cibles.push(global);
    if let Ok(interfaces) = if_addrs_via_udp() {
        for ip in interfaces {
            if let Some(addr) = broadcast_dirige(ip) {
                cibles.push(SocketAddr::new(IpAddr::V4(addr), PORT_UDP));
            }
        }
    }
    cibles
}

fn if_addrs_via_udp() -> Result<Vec<Ipv4Addr>, std::io::Error> {
    let mut adresses = Vec::new();
    let sonde = UdpSocket::bind("0.0.0.0:0")?;
    if sonde.connect("8.8.8.8:80").is_ok() {
        if let Ok(SocketAddr::V4(addr)) = sonde.local_addr() {
            adresses.push(*addr.ip());
        }
    }
    Ok(adresses)
}

fn broadcast_dirige(ip: Ipv4Addr) -> Option<Ipv4Addr> {
    let octets = ip.octets();
    if octets[0] == 127 || octets[0] == 0 {
        return None;
    }
    Some(Ipv4Addr::new(octets[0], octets[1], octets[2], 255))
}

fn spawn_emetteur_udp(
    socket: UdpSocket,
    annonce: Annonce,
    signal: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let cibles = adresses_broadcast();
        loop {
            if let Ok(texte) = serde_json::to_string(&annonce) {
                for cible in &cibles {
                    let _ = socket.send_to(texte.as_bytes(), cible);
                }
            }
            let mut attendu = Duration::from_millis(200);
            let mut ecoule = Duration::ZERO;
            while ecoule < INTERVALLE_ANNONCE {
                if signal.swap(false, Ordering::Relaxed) {
                    break;
                }
                thread::sleep(attendu);
                ecoule += attendu;
                attendu = Duration::from_millis(500);
            }
        }
    })
}

fn spawn_recepteur_udp(
    socket: UdpSocket,
    id_local: String,
    tx: Sender<Evenement>,
    pairs: Arc<Mutex<Vec<Pair>>>,
    signal_nouveau: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = [0u8; 2048];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((n, addr)) => {
                    if let Ok(texte) = std::str::from_utf8(&buf[..n]) {
                        if let Ok(annonce) = serde_json::from_str::<Annonce>(texte) {
                            if annonce.id == id_local {
                                continue;
                            }
                            let pair = Pair {
                                id: annonce.id.clone(),
                                nom: annonce.nom.clone(),
                                adresse: addr,
                                tcp_port: annonce.tcp_port,
                                derniere_annonce: Instant::now(),
                            };
                            {
                                let mut liste = pairs.lock().unwrap();
                                if let Some(existant) = liste.iter_mut().find(|p| p.id == pair.id) {
                                    existant.derniere_annonce = Instant::now();
                                    existant.adresse = addr;
                                    existant.tcp_port = annonce.tcp_port;
                                    existant.nom = annonce.nom;
                                } else {
                                    liste.push(pair.clone());
                                }
                            }
                            let _ = tx.send(Evenement::PairVu(pair));
                            signal_nouveau.store(true, Ordering::Relaxed);
                        }
                    }
                }
                Err(_) => break,
            }
        }
    })
}

fn spawn_serveur_tcp(
    listener: TcpListener,
    id_local: String,
    tx: Sender<Evenement>,
    signal_nouveau: Arc<AtomicBool>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        for flux in listener.incoming() {
            let Ok(flux) = flux else { continue };
            let tx = tx.clone();
            let id_local = id_local.clone();
            let signal = signal_nouveau.clone();
            thread::spawn(move || {
                traiter_connexion_entrante(flux, &id_local, &tx, &signal);
            });
        }
    })
}

fn traiter_connexion_entrante(
    mut flux: TcpStream,
    id_local: &str,
    tx: &Sender<Evenement>,
    signal: &Arc<AtomicBool>,
) {
    let _ = flux.set_read_timeout(Some(Duration::from_secs(15)));
    let mut buf = Vec::new();
    let mut temp = [0u8; 65536];
    loop {
        match flux.read(&mut temp) {
            Ok(0) => break,
            Ok(n) => {
                buf.extend_from_slice(&temp[..n]);
                if buf.len() > LIMITE_MESSAGE_OCTETS {
                    return;
                }
                if n < temp.len() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    if buf.is_empty() {
        return;
    }
    let Ok(texte) = std::str::from_utf8(&buf) else {
        return;
    };
    let Ok(enveloppe) = serde_json::from_str::<Enveloppe>(texte) else {
        return;
    };
    match enveloppe {
        Enveloppe::Chat {
            id,
            de,
            vers,
            texte,
            ts,
            reponse_a,
            piece_jointe,
        } => {
            if vers == id_local {
                let _ = tx.send(Evenement::MessageRecu(Message {
                    id,
                    de,
                    vers,
                    texte,
                    ts,
                    reponse_a,
                    piece_jointe,
                }));
                signal.store(true, Ordering::Relaxed);
            }
        }
        Enveloppe::DemandeAvatar { de: _ } => {
            if let Some(bytes) = lire_avatar_local() {
                if bytes.len() <= LIMITE_AVATAR_OCTETS {
                    let reponse = Enveloppe::ReponseAvatar {
                        de: id_local.to_string(),
                        avatar_b64: BASE64.encode(&bytes),
                    };
                    if let Ok(txt) = serde_json::to_string(&reponse) {
                        let _ = flux.write_all(txt.as_bytes());
                    }
                }
            }
        }
        Enveloppe::ReponseAvatar { de, avatar_b64 } => {
            if let Ok(bytes) = BASE64.decode(avatar_b64.as_bytes()) {
                if bytes.len() <= LIMITE_AVATAR_OCTETS {
                    let _ = tx.send(Evenement::AvatarRecu { id: de, bytes });
                    signal.store(true, Ordering::Relaxed);
                }
            }
        }
        Enveloppe::SupprimerMessage { de, vers, id } => {
            if vers == id_local {
                let _ = tx.send(Evenement::MessageSupprime { de, id });
                signal.store(true, Ordering::Relaxed);
            }
        }
    }
}

fn lire_avatar_local() -> Option<Vec<u8>> {
    std::fs::read(crate::persistence::chemin_avatar()).ok()
}

pub fn envoyer_message_tcp(pair: &Pair, message: &Message) -> bool {
    let enveloppe = Enveloppe::Chat {
        id: message.id.clone(),
        de: message.de.clone(),
        vers: message.vers.clone(),
        texte: message.texte.clone(),
        ts: message.ts,
        reponse_a: message.reponse_a.clone(),
        piece_jointe: message.piece_jointe.clone(),
    };
    envoyer_enveloppe(pair, &enveloppe)
}

pub fn envoyer_suppression(pair: &Pair, de: &str, id: &str) -> bool {
    let enveloppe = Enveloppe::SupprimerMessage {
        de: de.to_string(),
        vers: pair.id.clone(),
        id: id.to_string(),
    };
    envoyer_enveloppe(pair, &enveloppe)
}

pub fn demander_avatar(pair: &Pair, de: &str) -> Option<Vec<u8>> {
    let addr = format!("{}:{}", pair.adresse.ip(), pair.tcp_port);
    let mut flux = TcpStream::connect_timeout(&addr.parse().ok()?, Duration::from_secs(2)).ok()?;
    let _ = flux.set_read_timeout(Some(Duration::from_secs(5)));
    let enveloppe = Enveloppe::DemandeAvatar { de: de.to_string() };
    let texte = serde_json::to_string(&enveloppe).ok()?;
    flux.write_all(texte.as_bytes()).ok()?;
    flux.shutdown(std::net::Shutdown::Write).ok()?;
    let mut buf = vec![0u8; LIMITE_AVATAR_OCTETS + 1024];
    let mut total = 0usize;
    loop {
        match flux.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => {
                total += n;
                if total >= buf.len() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
    if total == 0 {
        return None;
    }
    let texte = std::str::from_utf8(&buf[..total]).ok()?;
    let enveloppe = serde_json::from_str::<Enveloppe>(texte).ok()?;
    match enveloppe {
        Enveloppe::ReponseAvatar { avatar_b64, .. } => BASE64.decode(avatar_b64.as_bytes()).ok(),
        _ => None,
    }
}

fn envoyer_enveloppe(pair: &Pair, enveloppe: &Enveloppe) -> bool {
    let addr = format!("{}:{}", pair.adresse.ip(), pair.tcp_port);
    let Ok(mut flux) = TcpStream::connect_timeout(&addr.parse().unwrap(), Duration::from_secs(5))
    else {
        return false;
    };
    let Ok(texte) = serde_json::to_string(enveloppe) else {
        return false;
    };
    if flux.write_all(texte.as_bytes()).is_err() {
        return false;
    }
    let _ = flux.shutdown(std::net::Shutdown::Write);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn annonce_roundtrip_json() {
        let a = Annonce {
            id: "x".to_string(),
            nom: "Test".to_string(),
            tcp_port: 50000,
            version: "3.4.1".to_string(),
        };
        let json = serde_json::to_string(&a).unwrap();
        let a2: Annonce = serde_json::from_str(&json).unwrap();
        assert_eq!(a.id, a2.id);
    }

    #[test]
    fn broadcast_dirige_local() {
        let ip = Ipv4Addr::new(192, 168, 1, 42);
        assert_eq!(broadcast_dirige(ip), Some(Ipv4Addr::new(192, 168, 1, 255)));
    }
}
