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
    let candidates = open_session_files(&[pid], is_codex_rollout).ok()?;
    most_recently_written(candidates.get(&pid)?)
}

/// Rollout Codex : le seul type de fichier que `open_session_file` classe.
pub(crate) fn is_codex_rollout(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == "jsonl")
}

/// Inventaire complet en une collecte. L'identité ne doit JAMAIS utiliser le
/// classement mtime de `open_session_file` : un sous-agent peut écrire après son parent.
/// Noms avec espaces conservés grâce au format machine lsof ; refus si tronqué.
///
/// `accept` borne la collecte aux fichiers de session reconnus. Il est fourni
/// par l'appelant parce qu'il dépend du fournisseur : un rollout Codex est un
/// `.jsonl`, une session Cursor est un dossier `acp-sessions/<id>`. Élargir ce
/// filtre pour tout le monde ferait classer à `open_session_file` un fichier
/// qui n'est pas un rollout.
pub(crate) fn open_session_files(
    pids: &[u32],
    accept: impl Fn(&Path) -> bool,
) -> std::io::Result<std::collections::HashMap<u32, Vec<std::path::PathBuf>>> {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};
    const MAX_OUTPUT: usize = 2 * 1024 * 1024;
    if pids.is_empty() {
        return Ok(Default::default());
    }
    if pids.len() > 256 {
        return Err(std::io::Error::other("inventaire PID au-delà de la borne"));
    }
    let selection = pids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let mut child = Command::new("lsof")
        .args(["-n", "-P", "-b", "-w", "-F0pn", "-p", &selection])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    // Le lecteur doit vider le tube pendant que lsof écrit. Un polling
    // séquentiel du descripteur peut laisser lsof bloqué dans write(2) sous
    // forte activité T3, puis faire expirer tout l'inventaire d'identité.
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| std::io::Error::other("sortie absente"))?;
    let mut reader = Some(std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.take(MAX_OUTPUT as u64 + 1).read_to_end(&mut bytes)?;
        Ok::<_, std::io::Error>(bytes)
    }));
    let result = (|| {
        let deadline = Instant::now() + Duration::from_secs(2);
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= deadline {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "lsof expiré",
                ));
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        let bytes = reader
            .take()
            .expect("lecteur présent")
            .join()
            .map_err(|_| std::io::Error::other("lecteur lsof interrompu"))??;
        if bytes.len() > MAX_OUTPUT {
            return Err(std::io::Error::other("inventaire fichiers tronqué"));
        }
        let text = std::str::from_utf8(&bytes)
            .map_err(|_| std::io::Error::other("inventaire non UTF8"))?;
        let mut found: std::collections::HashMap<u32, Vec<std::path::PathBuf>> = Default::default();
        let mut listed = std::collections::HashSet::new();
        let mut current = None;
        for field in text.split('\0') {
            let line = field.trim_start_matches('\n');
            if let Some(pid) = line.strip_prefix('p') {
                current = pid.parse::<u32>().ok().filter(|pid| pids.contains(pid));
                listed.extend(current);
            } else if let Some(path) = line.strip_prefix('n')
                && let Some(pid) = current
                && Path::new(path).is_absolute()
                && accept(Path::new(path))
            {
                let files = found.entry(pid).or_default();
                let path = std::path::PathBuf::from(path);
                if !files.contains(&path) {
                    files.push(path);
                }
            }
        }
        // Session 116 : lsof sort en erreur dès qu'un seul processus demandé a
        // disparu, alors qu'il a listé tous les autres. Un fournisseur qui
        // s'arrête pendant la collecte est le cycle de vie normal de T3, pas un
        // inventaire incomplet : le traiter comme tel révoquait l'identité de
        // TOUS les fils, une à deux fois par jour. Un processus mort ne porte
        // aucune identité. Toute autre cause d'échec reste un échec.
        if !status.success() && !only_vanished(pids, &listed) {
            return Err(std::io::Error::other("inventaire lsof incomplet"));
        }
        Ok(found)
    })();
    if result.is_err() {
        // Ce PID est notre enfant lsof non récolté, jamais un PID fournisseur.
        // Pas de SIGKILL ni de signal à un groupe ; wait séparé garde le pont borné.
        if matches!(child.try_wait(), Ok(None)) {
            // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
            unsafe {
                libc::kill(child.id() as libc::pid_t, libc::SIGTERM);
            }
            let reader = reader.take();
            std::thread::spawn(move || {
                let _ = child.wait();
                if let Some(reader) = reader {
                    let _ = reader.join();
                }
            });
        }
    }
    result
}

/// Session 116 : vrai si l'échec de lsof s'explique entièrement par des
/// processus disparus — au moins un processus demandé manque à l'inventaire,
/// et tous ceux qui manquent sont morts. O(n), n ≤ 256 processus demandés.
fn only_vanished(requested: &[u32], listed: &std::collections::HashSet<u32>) -> bool {
    let mut missing = requested
        .iter()
        .filter(|pid| !listed.contains(pid))
        .peekable();
    missing.peek().is_some() && missing.all(|pid| process_gone(*pid))
}

/// `kill(pid, 0)` : ESRCH prouve l'absence, EPERM prouve l'existence.
fn process_gone(pid: u32) -> bool {
    // SAFETY: kill(pid, 0) est l'idiome POSIX de sonde d'existence ; aucun signal n'est envoyé.
    let result = unsafe { libc::kill(pid as libc::pid_t, 0) };
    result == -1 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
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
#[path = "runtime_tests.rs"]
mod tests;
