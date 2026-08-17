//! Lecture du modèle et du niveau d'effort courants dans les fichiers de
//! session des agents CLI.
//!
//! Deux formats, une même contrainte : ces fichiers sont en append continu et
//! peuvent peser plusieurs centaines de mégaoctets (945 Mo observés sur un
//! rollout Codex réel). Ils sont donc lus **depuis la fin, par fenêtre bornée**,
//! jamais parcourus intégralement.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// Taille de la première fenêtre de lecture.
const INITIAL_WINDOW: u64 = 256 * 1024;
/// Plafond au-delà duquel on renonce plutôt que de charger un fichier entier.
const MAX_WINDOW: u64 = 4 * 1024 * 1024;

/// Observation extraite d'un fichier de session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeObservation {
    pub model: String,
    /// `None` signifie « observé absent » : certains modèles n'exposent aucun
    /// niveau d'effort. La valeur ne doit pas être héritée du modèle précédent.
    pub effort: Option<String>,
}

/// Lit la fin d'un fichier et rend les lignes **complètes** qu'elle contient,
/// de la plus récente à la plus ancienne.
///
/// Deux troncatures sont possibles et toutes deux sont écartées :
/// - la fenêtre commence presque toujours au milieu d'une ligne : la première
///   ligne est jetée, sauf si la fenêtre couvre le début du fichier ;
/// - le fichier peut être en cours d'écriture : une dernière ligne sans saut de
///   ligne final est jetée.
///
/// Complexité : O(taille de la fenêtre), indépendante de la taille du fichier.
fn tail_lines(path: &Path, window: u64) -> std::io::Result<Vec<String>> {
    let mut file = File::open(path)?;
    let size = file.metadata()?.len();
    let from_start = window >= size;
    let offset = size.saturating_sub(window);
    file.seek(SeekFrom::Start(offset))?;

    let mut buffer = Vec::with_capacity(window.min(size) as usize);
    file.take(window).read_to_end(&mut buffer)?;
    let text = String::from_utf8_lossy(&buffer);

    let mut lines: Vec<&str> = text.split('\n').collect();

    // Dernier fragment. S'il suit un saut de ligne final, c'est une chaîne
    // vide sans intérêt. Sinon, c'est soit une ligne en cours d'écriture, soit
    // une ligne complète d'un fichier sans saut de ligne final : on la garde,
    // et le parseur JSON écarte de lui-même ce qui est tronqué. La jeter
    // systématiquement perdrait la dernière observation d'un fichier bien
    // formé — défaut soulevé par la contre-revue « agent-1 ».
    if text.ends_with('\n') {
        lines.pop();
    }
    // Première ligne : tronquée par le début de fenêtre, sauf si l'on a lu
    // le fichier depuis son tout début.
    if !from_start && !lines.is_empty() {
        lines.remove(0);
    }

    Ok(lines
        .into_iter()
        .rev()
        .map(str::to_owned)
        .filter(|line| !line.trim().is_empty())
        .collect())
}

/// Applique `extract` aux lignes de la fin du fichier, en agrandissant la
/// fenêtre tant qu'aucune observation n'a été trouvée.
///
/// Complexité : O(fenêtre finale), bornée par `MAX_WINDOW`.
fn scan_from_end<F>(path: &Path, extract: F) -> Option<RuntimeObservation>
where
    F: Fn(&str) -> Option<RuntimeObservation>,
{
    let file_size = std::fs::metadata(path).ok()?.len();
    let mut window = INITIAL_WINDOW;
    loop {
        let lines = match tail_lines(path, window) {
            Ok(lines) => lines,
            Err(error) => {
                log::debug!("lecture de {} impossible: {}", path.display(), error);
                return None;
            }
        };
        for line in &lines {
            if let Some(observation) = extract(line) {
                return Some(observation);
            }
        }
        // La fenêtre couvre déjà tout le fichier, ou le plafond est atteint.
        if window >= file_size || window >= MAX_WINDOW {
            return None;
        }
        window = (window * 4).min(MAX_WINDOW);
    }
}

/// Extrait le modèle et l'effort d'une ligne de transcript Claude Code.
///
/// Retient une ligne `type=assistant` hors sous-agent (`isSidechain`) et
/// **portant réellement** `message.model` : des lignes assistant synthétiques
/// existent, et s'arrêter à la première masquerait le vrai modèle courant.
fn claude_line(line: &str) -> Option<RuntimeObservation> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    if value.get("type")?.as_str()? != "assistant" {
        return None;
    }
    if value
        .get("isSidechain")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return None;
    }
    let model = value.get("message")?.get("model")?.as_str()?;
    if model.is_empty() {
        return None;
    }
    Some(RuntimeObservation {
        model: model.to_string(),
        effort: value
            .get("effort")
            .and_then(serde_json::Value::as_str)
            .filter(|effort| !effort.is_empty())
            .map(str::to_owned),
    })
}

/// Extrait le modèle et l'effort d'une ligne de rollout Codex.
///
/// Le `turn_context` est réécrit à chaque tour et porte les deux valeurs ;
/// `session_meta` ne porte pas l'effort et ne suivrait pas un changement.
fn codex_line(line: &str) -> Option<RuntimeObservation> {
    let value: serde_json::Value = serde_json::from_str(line).ok()?;
    if value.get("type")?.as_str()? != "turn_context" {
        return None;
    }
    let payload = value.get("payload")?;
    let model = payload.get("model")?.as_str()?;
    if model.is_empty() {
        return None;
    }
    Some(RuntimeObservation {
        model: model.to_string(),
        effort: payload
            .get("effort")
            .and_then(serde_json::Value::as_str)
            .filter(|effort| !effort.is_empty())
            .map(str::to_owned),
    })
}

/// Dernier modèle réellement servi dans un transcript Claude Code.
pub fn parse_claude_transcript(path: &Path) -> Option<RuntimeObservation> {
    scan_from_end(path, claude_line)
}

/// Dernier modèle configuré dans un rollout Codex.
pub fn parse_codex_rollout(path: &Path) -> Option<RuntimeObservation> {
    scan_from_end(path, codex_line)
}

/// Retrouve le fichier de session qu'un processus tient ouvert.
///
/// Codex maintient son rollout ouvert en permanence : c'est le seul lien exact
/// entre un processus et sa session. Deviner le chemin serait faux — un rollout
/// observé vivait sur un volume externe, pas sous `~/.codex/sessions/`.
///
/// Plusieurs rollouts peuvent être ouverts après un `resume` ; celui dont la
/// date de modification est la plus récente est celui où l'agent écrit.
///
/// Complexité : O(nombre de descripteurs ouverts par le processus).
pub fn open_session_file(pid: u32) -> Option<std::path::PathBuf> {
    let output = std::process::Command::new("lsof")
        .args(["-p", &pid.to_string()])
        .output()
        .ok()?;
    if !output.status.success() {
        log::debug!("lsof a échoué pour le pid {}", pid);
        return None;
    }

    let candidates: Vec<std::path::PathBuf> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_whitespace().next_back())
        .filter(|path| path.ends_with(".jsonl"))
        .map(std::path::PathBuf::from)
        .collect();
    most_recently_written(&candidates)
}

/// Retient le fichier le plus récemment écrit parmi des candidats.
///
/// Complexité : O(n) sur le nombre de candidats, avec un `stat` par candidat.
fn most_recently_written(candidates: &[std::path::PathBuf]) -> Option<std::path::PathBuf> {
    candidates
        .iter()
        .filter_map(|path| {
            let modified = std::fs::metadata(path).ok()?.modified().ok()?;
            Some((modified, path.clone()))
        })
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn fixture(name: &str, content: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "bridget-runtime-{}-{}.jsonl",
            std::process::id(),
            name
        ));
        let mut file = File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn claude_transcript_nominal() {
        let path = fixture(
            "claude-nominal",
            concat!(
                r#"{"type":"user","message":{"role":"user"}}"#,
                "\n",
                r#"{"type":"assistant","isSidechain":false,"effort":"high","message":{"model":"claude-opus-5"}}"#,
                "\n"
            ),
        );
        let observed = parse_claude_transcript(&path).unwrap();
        assert_eq!(observed.model, "claude-opus-5");
        assert_eq!(observed.effort.as_deref(), Some("high"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn claude_ignore_le_sous_agent_et_les_lignes_sans_modele() {
        // La ligne la plus récente est un sous-agent sur un autre modèle, la
        // précédente une ligne assistant synthétique sans modèle : ni l'une ni
        // l'autre ne décrit le modèle de la session principale.
        let path = fixture(
            "claude-sidechain",
            concat!(
                r#"{"type":"assistant","isSidechain":false,"effort":"high","message":{"model":"claude-opus-5"}}"#,
                "\n",
                r#"{"type":"assistant","isSidechain":false,"message":{"role":"assistant"}}"#,
                "\n",
                r#"{"type":"assistant","isSidechain":true,"message":{"model":"claude-haiku-4-5"}}"#,
                "\n"
            ),
        );
        let observed = parse_claude_transcript(&path).unwrap();
        assert_eq!(observed.model, "claude-opus-5");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn claude_effort_absent_reste_absent() {
        // Cas Haiku observé en Phase 0 : aucun champ `effort`.
        let path = fixture(
            "claude-sans-effort",
            concat!(
                r#"{"type":"assistant","isSidechain":false,"message":{"model":"claude-haiku-4-5-20251001"}}"#,
                "\n"
            ),
        );
        let observed = parse_claude_transcript(&path).unwrap();
        assert_eq!(observed.model, "claude-haiku-4-5-20251001");
        assert_eq!(observed.effort, None);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn codex_rollout_nominal() {
        let path = fixture(
            "codex-nominal",
            concat!(
                r#"{"type":"session_meta","payload":{"cli_version":"0.147.0"}}"#,
                "\n",
                r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex","effort":"xhigh"}}"#,
                "\n"
            ),
        );
        let observed = parse_codex_rollout(&path).unwrap();
        assert_eq!(observed.model, "gpt-5.3-codex");
        assert_eq!(observed.effort.as_deref(), Some("xhigh"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn codex_dernier_turn_context_sans_modele_est_ignore() {
        let path = fixture(
            "codex-sans-modele",
            concat!(
                r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex","effort":"high"}}"#,
                "\n",
                r#"{"type":"turn_context","payload":{"cwd":"/tmp"}}"#,
                "\n"
            ),
        );
        let observed = parse_codex_rollout(&path).unwrap();
        assert_eq!(observed.model, "gpt-5.3-codex");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn fenetre_tronquee_aux_deux_bornes() {
        // Fichier plus grand que la fenêtre initiale : la première ligne lue
        // est coupée, et la dernière est en cours d'écriture (sans \n final).
        // Aucune des deux ne doit produire d'observation ni faire échouer le
        // scan. Cas soulevé par la contre-revue « agent-1 ».
        let mut content = String::new();
        content.push_str(&format!(
            "{{\"type\":\"turn_context\",\"payload\":{{\"model\":\"ancien\",\"filler\":\"{}\"}}}}\n",
            "x".repeat(300 * 1024)
        ));
        content.push_str(
            r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex","effort":"low"}}"#,
        );
        content.push('\n');
        content.push_str(r#"{"type":"turn_context","payload":{"model":"tronq"#);
        let path = fixture("codex-tronque", &content);

        let observed = parse_codex_rollout(&path).unwrap();
        assert_eq!(observed.model, "gpt-5.3-codex");
        assert_eq!(observed.effort.as_deref(), Some("low"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn fenetre_agrandie_quand_l_observation_est_plus_haut() {
        // L'unique turn_context est au début d'un fichier dépassant la fenêtre
        // initiale : l'agrandissement doit le retrouver.
        let mut content =
            String::from(r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex"}}"#);
        content.push('\n');
        for _ in 0..40 {
            content.push_str(&format!(
                "{{\"type\":\"response_item\",\"payload\":{{\"filler\":\"{}\"}}}}\n",
                "y".repeat(10 * 1024)
            ));
        }
        let path = fixture("codex-loin", &content);

        let observed = parse_codex_rollout(&path).unwrap();
        assert_eq!(observed.model, "gpt-5.3-codex");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn fichier_vide_ou_absent_ne_produit_rien() {
        let path = fixture("vide", "");
        assert!(parse_codex_rollout(&path).is_none());
        assert!(parse_claude_transcript(&path).is_none());
        let _ = std::fs::remove_file(&path);

        let absent = std::env::temp_dir().join("bridget-runtime-inexistant.jsonl");
        assert!(parse_codex_rollout(&absent).is_none());
    }

    #[test]
    fn derniere_ligne_complete_sans_saut_final_est_lue() {
        // Un fichier bien formé peut ne pas se terminer par un saut de ligne.
        // Sa dernière ligne porte pourtant l'observation la plus récente.
        let path = fixture(
            "sans-saut-final",
            r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex","effort":"medium"}}"#,
        );
        let observed = parse_codex_rollout(&path).unwrap();
        assert_eq!(observed.model, "gpt-5.3-codex");
        assert_eq!(observed.effort.as_deref(), Some("medium"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn derniere_ligne_incomplete_est_rejetee_au_profit_de_la_precedente() {
        // Écriture en cours : le JSON final est tronqué. Le parseur doit
        // l'écarter et retenir l'observation complète qui précède.
        let path = fixture(
            "ecriture-en-cours",
            concat!(
                r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex","effort":"low"}}"#,
                "\n",
                r#"{"type":"turn_context","payload":{"model":"gpt-5.4-cod"#
            ),
        );
        let observed = parse_codex_rollout(&path).unwrap();
        assert_eq!(observed.model, "gpt-5.3-codex");
        assert_eq!(observed.effort.as_deref(), Some("low"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn rollout_actif_est_le_plus_recemment_ecrit() {
        // Après un `codex resume`, deux rollouts sont ouverts par le même
        // processus. Le plus récemment écrit est celui où l'agent travaille,
        // quel que soit l'ordre de leur nom ou de leur date de création.
        // Fixture demandée par la contre-revue « agent-1 ».
        let ancien = fixture("rollout-ancien", "{}\n");
        let recent = fixture("rollout-recent", "{}\n");
        let hier = std::time::SystemTime::now() - std::time::Duration::from_secs(86_400);
        File::open(&ancien)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(hier))
            .unwrap();

        let choisi = most_recently_written(&[ancien.clone(), recent.clone()]).unwrap();
        assert_eq!(choisi, recent);
        // L'ordre des candidats ne change pas le résultat.
        let choisi = most_recently_written(&[recent.clone(), ancien.clone()]).unwrap();
        assert_eq!(choisi, recent);

        assert!(most_recently_written(&[]).is_none());
        let _ = std::fs::remove_file(ancien);
        let _ = std::fs::remove_file(recent);
    }

    #[test]
    fn json_invalide_est_ignore_sans_erreur() {
        let path = fixture(
            "json-casse",
            concat!(
                r#"{"type":"turn_context","payload":{"model":"gpt-5.3-codex"}}"#,
                "\n",
                "pas du json du tout\n",
                r#"{"type":"turn_context","payload":"#,
                "\n"
            ),
        );
        let observed = parse_codex_rollout(&path).unwrap();
        assert_eq!(observed.model, "gpt-5.3-codex");
        let _ = std::fs::remove_file(path);
    }
}
