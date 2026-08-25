//! Références de fermeture / réfutation — preuve vérifiable, pas « non vide ».
//!
//! # Formes
//! - `sha:<hex>` — objet Git présent **à l'écriture seulement**
//! - `mesure:<texte avec N/M>` — compte `\d+/\d+`, `0/0` exclu
//!
//! # Arbitrage rebase
//! Validation `git cat-file -e` **à l'écriture seulement**. Pas de ref de
//! branche (cible mouvante). Jamais revalidé à la relecture du journal.
//!
//! # Réfutation
//! Plus grave qu'une fermeture : exige `mesure:` (un SHA seul refuse).

use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferencePreuve {
    Sha(String),
    Mesure {
        texte: String,
        numerateur: u64,
        denominateur: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreuveError {
    Vide,
    FormeInconnue(String),
    ShaInvalide(String),
    MesureSansCompte(String),
    MesureZeroSurZero,
    ShaSeulRefusePourRefutation,
    ShaAbsent(String),
    GitIndisponible(String),
}

impl fmt::Display for PreuveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vide => write!(f, "référence vide refusée"),
            Self::FormeInconnue(raw) => write!(
                f,
                "référence '{raw}' refusée : formes admises sha:<hex> ou mesure:<texte avec N/M>"
            ),
            Self::ShaInvalide(hex) => write!(
                f,
                "sha '{hex}' invalide : hex Git de 7 à 40 caractères attendu"
            ),
            Self::MesureSansCompte(texte) => write!(
                f,
                "mesure '{texte}' sans compte \\d+/\\d+ : annoncer le compte"
            ),
            Self::MesureZeroSurZero => write!(
                f,
                "mesure 0/0 refusée : un compte qui ne prouve rien n'est pas une preuve"
            ),
            Self::ShaSeulRefusePourRefutation => write!(
                f,
                "réfutation refusée : sha: seul insuffisant — mesure:<N/M> obligatoire"
            ),
            Self::ShaAbsent(hex) => write!(
                f,
                "sha:{hex} introuvable dans le dépôt (git cat-file -e) — validation à l'écriture seulement"
            ),
            Self::GitIndisponible(detail) => {
                write!(f, "git indisponible pour valider la référence : {detail}")
            }
        }
    }
}

impl std::error::Error for PreuveError {}

pub fn parse_reference(raw: &str) -> Result<ReferencePreuve, PreuveError> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err(PreuveError::Vide);
    }
    if raw.eq_ignore_ascii_case("ok") || raw.eq_ignore_ascii_case("oui") {
        return Err(PreuveError::FormeInconnue(raw.to_string()));
    }
    if let Some(hex) = raw.strip_prefix("sha:") {
        let hex = hex.trim();
        if !is_git_hex(hex) {
            return Err(PreuveError::ShaInvalide(hex.to_string()));
        }
        return Ok(ReferencePreuve::Sha(hex.to_ascii_lowercase()));
    }
    if let Some(texte) = raw.strip_prefix("mesure:") {
        let texte = texte.trim();
        if texte.is_empty() {
            return Err(PreuveError::MesureSansCompte(String::new()));
        }
        let Some((numerateur, denominateur)) = extract_compte(texte) else {
            return Err(PreuveError::MesureSansCompte(texte.to_string()));
        };
        if numerateur == 0 && denominateur == 0 {
            return Err(PreuveError::MesureZeroSurZero);
        }
        return Ok(ReferencePreuve::Mesure {
            texte: texte.to_string(),
            numerateur,
            denominateur,
        });
    }
    Err(PreuveError::FormeInconnue(raw.to_string()))
}

/// Fermeture / requalification : sha: ou mesure:.
pub fn parse_reference_fermeture(raw: &str) -> Result<ReferencePreuve, PreuveError> {
    parse_reference(raw)
}

/// Réfutation : mesure: uniquement (plus strict).
pub fn parse_reference_refutation(raw: &str) -> Result<ReferencePreuve, PreuveError> {
    match parse_reference(raw)? {
        ReferencePreuve::Sha(_) => Err(PreuveError::ShaSeulRefusePourRefutation),
        other => Ok(other),
    }
}

pub fn assert_sha_exists_in_repo(hex: &str, repo: &Path) -> Result<(), PreuveError> {
    let status = Command::new("git")
        .args(["cat-file", "-e", &format!("{hex}^{{object}}")])
        .current_dir(repo)
        .status()
        .map_err(|error| PreuveError::GitIndisponible(error.to_string()))?;
    if status.success() {
        Ok(())
    } else {
        Err(PreuveError::ShaAbsent(hex.to_string()))
    }
}

pub fn git_toplevel() -> Result<PathBuf, PreuveError> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|error| PreuveError::GitIndisponible(error.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(PreuveError::GitIndisponible(if stderr.is_empty() {
            "rev-parse a échoué".into()
        } else {
            stderr
        }));
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        return Err(PreuveError::GitIndisponible(
            "rev-parse a renvoyé un chemin vide".into(),
        ));
    }
    Ok(PathBuf::from(path))
}

pub fn ensure_reference_fermeture_at_write(raw: &str) -> Result<ReferencePreuve, PreuveError> {
    let parsed = parse_reference_fermeture(raw)?;
    if let ReferencePreuve::Sha(ref hex) = parsed {
        let repo = git_toplevel()?;
        assert_sha_exists_in_repo(hex, &repo)?;
    }
    Ok(parsed)
}

pub fn ensure_reference_refutation_at_write(raw: &str) -> Result<ReferencePreuve, PreuveError> {
    parse_reference_refutation(raw)
}

fn is_git_hex(value: &str) -> bool {
    let len = value.len();
    (7..=40).contains(&len) && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn extract_compte(texte: &str) -> Option<(u64, u64)> {
    let bytes = texte.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i < bytes.len() && bytes[i] == b'/' {
                i += 1;
                let denom_start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i > denom_start {
                    let num: u64 = texte[start..denom_start - 1].parse().ok()?;
                    let den: u64 = texte[denom_start..i].parse().ok()?;
                    return Some((num, den));
                }
            }
            continue;
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuse_ok_et_zero_sur_zero() {
        assert!(parse_reference("ok").is_err());
        assert!(matches!(
            parse_reference("mesure:0/0"),
            Err(PreuveError::MesureZeroSurZero)
        ));
    }

    #[test]
    fn refutation_refuse_sha_seul() {
        assert!(matches!(
            parse_reference_refutation("sha:aca9fbb"),
            Err(PreuveError::ShaSeulRefusePourRefutation)
        ));
        assert!(parse_reference_refutation("mesure:3/3 mutants").is_ok());
    }

    #[test]
    fn fermeture_accepte_sha_ou_mesure() {
        assert!(parse_reference_fermeture("sha:aca9fbb").is_ok());
        assert!(parse_reference_fermeture("mesure:12/12").is_ok());
    }
}
