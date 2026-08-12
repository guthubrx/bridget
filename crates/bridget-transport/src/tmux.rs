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

        // 4. Attendre que le TUI digère le collage
        std::thread::sleep(std::time::Duration::from_millis(600));

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

        // 6. Vérifier que le message a été soumis (composer vide)
        std::thread::sleep(std::time::Duration::from_millis(800));
        if self.composer_has_content() {
            // Le composer a encore du contenu : retenter avec Enter nommé
            std::thread::sleep(std::time::Duration::from_millis(500));
            let _ = Command::new("tmux")
                .args(["send-keys", "-t", pane, "Enter"])
                .output();
            std::thread::sleep(std::time::Duration::from_millis(800));

            // Dernière vérification
            if self.composer_has_content() {
                log::warn!("composer encore plein après 2 CR — message peut-être non soumis");
            }
        }

        // 7. CR de sécurité systématique
        std::thread::sleep(std::time::Duration::from_millis(300));
        let _ = Command::new("tmux")
            .args(["send-keys", "-t", pane, "-l", "\r"])
            .output();

        Ok(())
    }

    fn is_alive(&self) -> bool {
        // Vérifier que le process de l'agent CLI tourne toujours
        // kill(pid, 0) retourne 0 si le process existe
        unsafe { libc_kill(self.agent_pid, 0) == 0 }
    }

    fn connection_id(&self) -> &str {
        &self.pane_id
    }
}



unsafe extern "C" {
    #[link_name = "kill"]
    fn libc_kill(pid: u32, sig: i32) -> i32;
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
}
