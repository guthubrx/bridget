//! Transport tmux — injecte les messages dans un pane tmux via send-keys.
//!
//! C'est l'implémentation concrète du trait Transport pour le mode tmux.
//! Le wrapper utilise ce transport pour livrer les messages entrants.

use crate::transport::{Transport, TransportError};
use bridget_core::{BridgetMessage, envelope::wrap_envelope};
use std::process::Command;

/// Transport tmux : livre les messages en collant le texte dans un pane.
pub struct TmuxTransport {
    /// ID du pane tmux cible (ex: "%5")
    pane_id: String,
    /// PID du process de l'agent CLI (pour vérifier qu'il est vivant)
    agent_pid: u32,
}

impl TmuxTransport {
    pub fn new(pane_id: impl Into<String>, agent_pid: u32) -> Self {
        TmuxTransport {
            pane_id: pane_id.into(),
            agent_pid,
        }
    }

    /// Vérifie que tmux est disponible.
    pub fn tmux_available() -> bool {
        Command::new("tmux")
            .arg("display-message")
            .arg("-p")
            .arg("#{session_name}")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

/// Caractères de prompt des CLI agents (Codex ›, Claude ❯, shells >).
const TUI_PROMPT_CHARS: &str = "›❯>";

impl TmuxTransport {
    /// Inspecte la zone composer (du dernier prompt jusqu'au bas du pane)
    /// pour détecter si un message est resté coincé en saisie.
    fn composer_has_content(&self) -> bool {
        let capture = match Command::new("tmux")
            .args(["capture-pane", "-t", &self.pane_id, "-p"])
            .output()
        {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
            _ => return false,
        };

        let lines: Vec<&str> = capture.lines().collect();
        if lines.is_empty() {
            return false;
        }

        // Trouver la dernière ligne de prompt (remonte depuis le bas)
        let mut prompt_idx = lines.len();
        for (i, line) in lines.iter().enumerate().rev() {
            let trimmed = line.trim_start();
            if trimmed.starts_with(|c: char| TUI_PROMPT_CHARS.contains(c)) {
                prompt_idx = i;
                break;
            }
        }

        if prompt_idx >= lines.len() {
            return false;
        }

        // Le composer = tout ce qui est après la dernière ligne de prompt
        let composer_lines: Vec<String> = lines[prompt_idx..]
            .iter()
            .map(|l| {
                l.trim_start_matches(|c: char| TUI_PROMPT_CHARS.contains(c))
                    .trim_start()
                    .to_string()
            })
            .collect();
        let composer = composer_lines.join("\n");
        let composer_clean: String = composer.chars().filter(|c| !c.is_whitespace()).collect();

        // Placeholder de collage replié
        if composer.contains("[Pasted") {
            return true;
        }

        // Plusieurs lignes non-vides OU une ligne longue = contenu réel
        let non_empty_lines: usize = composer_lines
            .iter()
            .filter(|l| !l.trim().is_empty())
            .count();
        non_empty_lines >= 2 || composer_clean.len() > 80
    }
}

/// Valide le contenu destiné à tmux pour prévenir les injections
/// Rejette les séquences de contrôle tmux dangereuses et limite la taille
pub fn validate_tmux_content(content: &str) -> Result<(), TransportError> {
    // Limite de taille pour prévenir les attaques par mémoire
    const MAX_CONTENT_SIZE: usize = 100_000;
    if content.len() > MAX_CONTENT_SIZE {
        return Err(TransportError::DeliveryFailed(
            format!("Contenu trop volumineux: {} octets (max: {})", content.len(), MAX_CONTENT_SIZE)
        ));
    }

    // Séquences de contrôle tmux potentiellement dangereuses
    let dangerous_patterns = [
        "bind-key",
        "unbind-key",
        "send-keys",
        "run-shell",
        "if-shell",
        "display-message",
        "set-option",
        "show-options",
        "command-prompt",
        "confirm-before",
        "new-window",
        "split-window",
        "kill-window",
        "kill-pane",
        "join-pane",
        "move-pane",
        "select-pane",
        "select-window",
        "swap-pane",
        "swap-window",
        "rename-window",
    ];

    let content_lower = content.to_lowercase();
    for pattern in &dangerous_patterns {
        if content_lower.contains(pattern) {
            return Err(TransportError::DeliveryFailed(
                format!("Séquence tmux interdite détectée: {}", pattern)
            ));
        }
    }

    // Vérifier les tentatives d'injection via caractères de contrôle
    if content.contains('\x1b') && (content.contains('[') || content.contains(']')) {
        return Err(TransportError::DeliveryFailed(
            "Séquences ANSI ESC détectées".to_string()
        ));
    }

    Ok(())
}

impl Transport for TmuxTransport {
    fn deliver(&mut self, msg: &BridgetMessage) -> Result<(), TransportError> {
        // Vérifier que l'agent est vivant avant d'injecter
        if !self.is_alive() {
            return Err(TransportError::AgentDead);
        }

        let envelope = wrap_envelope(msg);
        let pane = &self.pane_id;

        // P0a : buffer nommé unique par deliver (anti croisement de payload).
        // Le buffer global anonyme peut être collé STALE par un autre process.
        let buf_name = format!(
            "bridget-{}-{}",
            std::process::id(),
            msg.id.chars().take(8).collect::<String>()
        );

        // VALIDATION DE SÉCURITÉ contre injection tmux
        validate_tmux_content(&envelope)?;

        // 1. set-buffer NOMMÉ avec stdin (vérifier le rc)
        let set_buffer = Command::new("tmux")
            .args(["load-buffer", "-b", &buf_name, "-"])
            .stdin(std::process::Stdio::piped())
            .spawn();

        match set_buffer {
            Ok(mut child) => {
                if let Some(mut stdin) = child.stdin.take() {
                    use std::io::Write;
                    let _ = stdin.write_all(envelope.as_bytes());
                }
                let output = child.wait();
                match output {
                    Ok(status) if !status.success() => {
                        return Err(TransportError::DeliveryFailed(
                            "load-buffer échec — ZÉRO paste".to_string(),
                        ));
                    }
                    Err(e) => {
                        return Err(TransportError::Io(format!("load-buffer: {e}")));
                    }
                    _ => {}
                }
            }
            Err(e) => {
                return Err(TransportError::Io(format!("load-buffer spawn: {e}")));
            }
        }

        // 2. paste-buffer NOMMÉ vers le pane cible, avec -d (supprime après paste)
        let paste = Command::new("tmux")
            .args(["paste-buffer", "-b", &buf_name, "-t", pane, "-d"])
            .output();

        match paste {
            Ok(o) if !o.status.success() => {
                // Nettoyage du buffer même en cas d'échec
                let _ = Command::new("tmux")
                    .args(["delete-buffer", "-b", &buf_name])
                    .output();
                return Err(TransportError::DeliveryFailed(format!(
                    "paste-buffer: {}",
                    String::from_utf8_lossy(&o.stderr)
                )));
            }
            Err(e) => {
                return Err(TransportError::Io(format!("paste-buffer exec: {e}")));
            }
            _ => {}
        }

        // 3. Vérifier que le buffer a bien été supprimé (preuve que le paste a réussi)
        let check = Command::new("tmux")
            .args(["show-buffer", "-b", &buf_name])
            .output();
        if check.map(|o| o.status.success()).unwrap_or(false) {
            // Le buffer existe encore = paste-buffer -d n'a pas marché
            let _ = Command::new("tmux")
                .args(["delete-buffer", "-b", &buf_name])
                .output();
            return Err(TransportError::DeliveryFailed(
                "paste-buffer -d n'a pas consommé le buffer".to_string(),
            ));
        }

        // 4. Attendre que le TUI digère le collage - polling actif au lieu de sleep fixe
        let mut digestion_attempts = 0;
        const MAX_DIGESTION_ATTEMPTS: u32 = 30; // 30 * 50ms = 1.5s max au lieu de 600ms fixe
        while digestion_attempts < MAX_DIGESTION_ATTEMPTS {
            std::thread::sleep(std::time::Duration::from_millis(50));
            // Vérifier si le composer a du contenu = message est en train de digérer
            if !self.composer_has_content() {
                break;
            }
            digestion_attempts += 1;
        }

        // 5. CR littéral (0x0d) pour soumettre
        let cr = Command::new("tmux")
            .args(["send-keys", "-t", pane, "-l", "\r"])
            .output();

        match cr {
            Ok(o) if !o.status.success() => {
                return Err(TransportError::DeliveryFailed(format!(
                    "send-keys CR: {}",
                    String::from_utf8_lossy(&o.stderr)
                )));
            }
            Err(e) => {
                return Err(TransportError::Io(format!("send-keys exec: {e}")));
            }
            _ => {}
        }

        // 6. Vérifier que le message a été soumis (composer vide) - polling actif
        let mut verification_attempts = 0;
        const MAX_VERIFICATION_ATTEMPTS: u32 = 16; // 16 * 50ms = 800ms max
        while verification_attempts < MAX_VERIFICATION_ATTEMPTS {
            std::thread::sleep(std::time::Duration::from_millis(50));
            if !self.composer_has_content() {
                break;
            }
            verification_attempts += 1;
        }

        if self.composer_has_content() {
            // Le composer a encore du contenu : retenter avec Enter nommé
            std::thread::sleep(std::time::Duration::from_millis(200)); // Réduit de 500ms à 200ms
            let _ = Command::new("tmux")
                .args(["send-keys", "-t", pane, "Enter"])
                .output();

            // Vérification finale avec polling
            let mut retry_attempts = 0;
            const MAX_RETRY_ATTEMPTS: u32 = 10;
            while retry_attempts < MAX_RETRY_ATTEMPTS {
                std::thread::sleep(std::time::Duration::from_millis(50));
                if !self.composer_has_content() {
                    break;
                }
                retry_attempts += 1;
            }

            if self.composer_has_content() {
                log::warn!("composer encore plein après 2 CR — message peut-être non soumis");
            }
        }

        // 7. CR de sécurité systématique - délai minimal car on a déjà polling
        std::thread::sleep(std::time::Duration::from_millis(100)); // Réduit de 300ms à 100ms
        let _ = Command::new("tmux")
            .args(["send-keys", "-t", pane, "-l", "\r"])
            .output();

        Ok(())
    }

    fn is_alive(&self) -> bool {
        // Vérifier que le process de l'agent CLI tourne toujours
        // kill(pid, 0) retourne 0 si le process existe
        // Utilisation sûre via libc crate
        unsafe { libc::kill(self.agent_pid as i32, 0) == 0 }
    }

    fn connection_id(&self) -> &str {
        &self.pane_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tmux_transport_creation() {
        let t = TmuxTransport::new("%5", 12345);
        assert_eq!(t.connection_id(), "%5");
    }

    #[test]
    fn test_is_alive_current_process() {
        // Le process actuel est vivant
        let pid = std::process::id();
        let t = TmuxTransport::new("%0", pid);
        assert!(t.is_alive());
    }

    #[test]
    fn test_is_alive_dead_process() {
        // PID 999999 n'existe probablement pas
        let t = TmuxTransport::new("%0", 999999);
        assert!(!t.is_alive());
    }

    // Tests de validation tmux
    #[test]
    fn test_validate_tmux_content_normal() {
        let content = "Message normal sans rien de spécial";
        assert!(validate_tmux_content(content).is_ok());
    }

    #[test]
    fn test_validate_tmux_content_dangerous_bind_key() {
        let content = "Message avec bind-key";
        assert!(validate_tmux_content(content).is_err());
    }

    #[test]
    fn test_validate_tmux_content_dangerous_send_keys() {
        let content = "Message avec send-keys";
        assert!(validate_tmux_content(content).is_err());
    }

    #[test]
    fn test_validate_tmux_content_too_large() {
        let large_content = "a".repeat(100_001);
        assert!(validate_tmux_content(&large_content).is_err());
    }

    #[test]
    fn test_validate_tmux_content_ansi_escape() {
        let content = "Message avec \x1b[";
        assert!(validate_tmux_content(content).is_err());
    }

    #[test]
    fn test_validate_tmux_content_case_insensitive() {
        let content = "Message avec BIND-KEY";
        assert!(validate_tmux_content(content).is_err());
    }

    #[test]
    fn test_validate_tmux_content_at_limit() {
        let content = "a".repeat(100_000);
        assert!(validate_tmux_content(&content).is_ok());
    }
}
