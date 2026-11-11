//! Module wrapper — lance un agent CLI et le connecte au daemon bridget.
//!
//! Appelé par le CLI quand l'utilisateur tape : bridget codex, bridget claude, etc.

#[path = "wrapper_workspace_repair.rs"]
mod workspace_repair;

use bridget_transport::journal::{
    IncrementalJournalReader, JournalLiveFeed, JournalReadItem, JournalSourceIdentity,
    JournalWindowError, JournalWriter, current_host_date, resolve_window,
};
use bridget_transport::poison::RecoverPoison;
use bridget_transport::protocol::{
    CommunicationProjectSource, DelegatedRuntimeEventFrame, DelegatedRuntimeEventKind,
    DiskSpaceFact, ExecutionControlCommand, ExecutionControlOperation, ExecutionDeliveryContext,
    ExecutionProviderContext, PresenceMode, ProviderOperation, decode, encode,
};
use bridget_transport::{
    AcpOptions, AcpTransport, AttachRefusal, AttachWindow, ChannelReport, ClaudeStreamJsonOptions,
    ClaudeStreamJsonTransport, CodexAppServerOptions, CodexAppServerTransport, DaemonToWrapper,
    MAX_ATTACH_FRAGMENT_BYTES, MAX_ATTACH_SERIALIZED_FRAME_BYTES, ManagedEvent, ManagedEventKind,
    ManagedProviderIdentity, ManagedSession, ManagedSessionDescriptor, ManagedTerminal,
    TmuxTransport, Transport, WrapperToDaemon,
};
use log::{debug, error, info, warn};
use sha2::{Digest, Sha256};

use crate::agent_profile::AttentionEventType;
use crate::agent_profile::{AgentProfileStore, InstructionStatus};
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::ffi::OsString;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::receipt_store::{ReceiptDecision, ReceiptQuota, ReceiptStore};

// Constantes de reconnexion optimisées pour auto-reconnect transparent
const RECONNECT_INITIAL_DELAY: Duration = Duration::from_secs(1);
const RECONNECT_MAX_DELAY: Duration = Duration::from_secs(30);
const RECONNECT_STABLE_RESET: Duration = Duration::from_secs(60);

// HEARTBEAT amélioré : plus fréquent pour une détection rapide (3s au lieu de 15s)
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(3);

// SOCKET_CHECK_INTERVAL : vérification proactive de disponibilité du socket
const SOCKET_CHECK_INTERVAL: Duration = Duration::from_secs(5);
// Grâce du fournisseur distant pendant une panne du maître ; aucun essai sans fin.
const REMOTE_MASTER_RECONNECT_GRACE: Duration = Duration::from_secs(300);

// Sonde de runtime : le fichier de session n'est relu que si sa date de
// modification a changé, et le chemin n'est re-résolu que rarement — `lsof`
// coûte environ 130 ms, contre quelques microsecondes pour un `stat`.
const RUNTIME_PROBE_INTERVAL: Duration = Duration::from_secs(20);
// Un `codex resume` ouvre un second rollout sans fermer le premier : garder le
// chemin trop longtemps rendrait la nouvelle session invisible d'autant.
// 60 s aligne cette latence sur l'exigence de fraîcheur de la spec, pour un
// surcoût de 134 ms par minute.
const RUNTIME_PATH_REFRESH: Duration = Duration::from_secs(60);
const ATTACH_RELAY_COMMAND_CAPACITY: usize = 8;
const ATTACH_RELAY_READ_BYTES: usize = 128 * 1024;
const ATTACH_RELAY_IDLE_WAIT: Duration = Duration::from_millis(10);
/// Plafond de `AttachRelayWorker::shutdown` : un join sans borne laisse le
/// wrapper (et donc tout le binaire de test) coincé si le worker est bloqué
/// dans un hook ou un wait non coopératif.
const ATTACH_RELAY_SHUTDOWN_BOUND: Duration = Duration::from_secs(2);
/// Motifs sensibles volatils, jamais persistés. Le masquage conserve une
/// fenêtre par canal afin de couvrir les secrets coupés entre deux fragments.
pub struct OutputRedactionLease {
    patterns: Vec<Vec<u8>>,
    tails: BTreeMap<String, Vec<u8>>,
}

impl OutputRedactionLease {
    pub fn new(patterns: Vec<Vec<u8>>) -> Self {
        Self {
            patterns,
            tails: BTreeMap::new(),
        }
    }
    pub fn redact(&mut self, channel: &str, bytes: &[u8], final_fragment: bool) -> Vec<u8> {
        let overlap = self
            .patterns
            .iter()
            .map(Vec::len)
            .max()
            .unwrap_or(0)
            .saturating_sub(1);
        let mut combined = self.tails.remove(channel).unwrap_or_default();
        combined.extend_from_slice(bytes);
        for pattern in &self.patterns {
            if !pattern.is_empty() {
                redact_bytes_in_place(&mut combined, pattern);
            }
        }
        let retained = if final_fragment {
            0
        } else {
            overlap.min(combined.len())
        };
        let output_len = combined.len().saturating_sub(retained);
        let output = combined.drain(..output_len).collect();
        if !final_fragment {
            self.tails.insert(channel.to_string(), combined);
        }
        output
    }
}

#[cfg(test)]
mod core_089_runtime_refusal_tests {
    use super::*;
    use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};

    #[test]
    fn ancien_runtime_est_refuse_avant_toute_creation_de_fournisseur() {
        const CHILD: &str = "BRIDGET_CORE_RUNTIME_REFUSAL_CHILD";
        if let Ok(variable) = std::env::var(CHILD) {
            let root = PathBuf::from(std::env::var_os("BRIDGET_HOME").unwrap());
            let home = PathBuf::from(std::env::var_os("HOME").unwrap());
            let script = home.join("provider-fixture.sh");
            let registry = crate::registry::AgentRegistry::from_json(
                &serde_json::json!({"agents":{"fixture":{"command":"/bin/sh","args":[script],
                    "protocol":"acp","forbidden_env":[],"pass_env":[],"notify_timeout_secs":1}}})
                .to_string(),
                home.join("agents.json"),
            )
            .unwrap();
            let identity = "89000000-0000-4000-8000-000000000119";
            let definition = registry.resolved_definition("fixture").unwrap();
            // Trois entrées de production : aucun repli hôte, même si seul
            // SECRET_ENV_FILES ou une variable runtime inconnue est présent.
            for error in [
                launch(
                    "/bin/sh",
                    "fixture",
                    &[script.display().to_string()],
                    Some(identity),
                )
                .unwrap_err(),
                launch_acp_with(
                    "fixture",
                    &[],
                    Some(identity),
                    &registry,
                    &root.join("s"),
                    &home,
                )
                .unwrap_err(),
                launch_managed_acp("fixture", identity, &definition).unwrap_err(),
            ] {
                assert!(
                    error.to_string().contains(&variable),
                    "refus non causal: {error}"
                );
            }
            assert!(
                !home.join("provider-started").exists(),
                "un fournisseur a été créé"
            );
            assert!(
                !root.join("agent-names").exists(),
                "état créé avant le refus"
            );
            return;
        }
        let base = std::env::temp_dir().join(format!(
            "rp{}",
            &uuid::Uuid::new_v4().simple().to_string()[..8]
        ));
        for path in [
            &base,
            &base.join("home"),
            &base.join("state"),
            &base.join("tmp"),
        ] {
            std::fs::DirBuilder::new().mode(0o700).create(path).unwrap();
        }
        let script = base.join("home/provider-fixture.sh");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o700)
            .open(&script)
            .unwrap();
        writeln!(
            file,
            "#!/bin/sh\nprintf unexpected > '{}'/provider-started\n",
            base.join("home").display()
        )
        .unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o700)).unwrap();
        drop(file);
        let executable = std::env::current_exe().unwrap();
        let deadline = Instant::now() + Duration::from_secs(40);
        for variable in [
            "BRIDGET_RUNTIME_SOCKET",
            "BRIDGET_RUNTIME_PROJECT_ID",
            "BRIDGET_RUNTIME_BINDING_GENERATION",
            "BRIDGET_RUNTIME_CONTAINER_ID",
            "BRIDGET_RUNTIME_ENVIRONMENT_EPOCH",
            "BRIDGET_RUNTIME_AGENT_GENERATION",
            "BRIDGET_RUNTIME_INSTANCE_ID",
            "BRIDGET_RUNTIME_SECRET_ENV_FILES",
            "BRIDGET_RUNTIME_FUTUR",
        ] {
            let log_path = base.join(variable);
            let log = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&log_path)
                .unwrap();
            let mut child = Command::new(&executable)
                .args(["--exact", "wrapper::core_089_runtime_refusal_tests::ancien_runtime_est_refuse_avant_toute_creation_de_fournisseur", "--nocapture"])
                .env_clear().env("HOME", base.join("home")).env("TMPDIR", base.join("tmp"))
                .env("BRIDGET_HOME", base.join("state")).env("BRIDGET_SOCKET", base.join("state/s"))
                .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin").env(CHILD, variable).env(variable, "")
                .stdin(Stdio::null()).stdout(log.try_clone().unwrap()).stderr(log).spawn().unwrap();
            let status = loop {
                if let Some(status) = child.try_wait().unwrap() {
                    break status;
                }
                if Instant::now() >= deadline {
                    let observed = Command::new("/bin/ps")
                        .args(["-p", &child.id().to_string(), "-o", "command="])
                        .output()
                        .unwrap();
                    assert!(
                        String::from_utf8_lossy(&observed.stdout)
                            .trim()
                            .starts_with(&executable.display().to_string())
                    );
                    // Enfant isolé identifié, jamais de groupe ni fournisseur.
                    // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
                    let _ = unsafe { libc::kill(child.id() as i32, libc::SIGTERM) };
                    let cleanup_deadline = Instant::now() + Duration::from_secs(3);
                    while matches!(child.try_wait(), Ok(None)) && Instant::now() < cleanup_deadline
                    {
                        thread::sleep(Duration::from_millis(5));
                    }
                    if matches!(child.try_wait(), Ok(None)) {
                        // Autorisation explicite, toujours le même fils non récolté.
                        // SAFETY: kill ne déréférence aucun pointeur ; ses arguments sont des entiers.
                        let _ = unsafe { libc::kill(child.id() as i32, libc::SIGKILL) };
                        let _ = child.wait();
                    }
                    panic!(
                        "budget global atteint pour {variable}; log {}",
                        log_path.display()
                    );
                }
                thread::sleep(Duration::from_millis(5)); // watchdog uniquement
            };
            assert!(
                status.success(),
                "{variable}: {}",
                std::fs::read_to_string(&log_path).unwrap()
            );
            assert!(!base.join("home/provider-started").exists());
        }
        std::fs::remove_dir_all(base).unwrap();
    }
}

fn redact_bytes_in_place(buffer: &mut [u8], pattern: &[u8]) {
    if pattern.len() > buffer.len() {
        return;
    }
    let mut start = 0;
    while let Some(offset) = buffer[start..]
        .windows(pattern.len())
        .position(|window| window == pattern)
    {
        let begin = start + offset;
        for byte in &mut buffer[begin..begin + pattern.len()] {
            *byte = 42;
        }
        start = begin + pattern.len();
    }
}

/// Carte factuelle reconstruite à chaque naissance : identité figée et Git.
/// Aucune lecture de greffe ni décision de coordination implicite.
/// Texte d'origine extérieure intercalé dans la carte de reprise.
///
/// La carte est lue comme une CONSIGNE : un saut de ligne dans une valeur
/// interpolée créerait une ligne autonome (ordre exécuté). Les caractères
/// de contrôle sont rendus visibles (`\n`, `\r`, …) sans jamais ouvrir
/// une nouvelle ligne. Le vecteur « nom » est déjà fermé en amont
/// (`validate_agent_id`) ; celui-ci ferme tout autre champ interpolé.
/// La carte de reprise est un message REMIS à un agent, pas un inventaire.
/// Elle est bornée à l'écriture : une source qui déborde est tronquée ici,
/// avec sa mention, plutôt que de produire un corps que personne ne peut
/// lire. Mesure du 2026-09-04 : un répertoire de build non suivi avait porté
/// une carte à 1 034 232 caractères, contre 9 167 pour le plus gros message
/// légitime du ledger.
pub(crate) const RESUME_CARD_MAX_CHARS: usize = 16_000;

/// Plafond d'un corps que le wrapper fabrique lui-même et injecte dans le
/// fournisseur. Au-delà, la remise est refusée et journalisée : un producteur
/// non borné ne doit pas pouvoir traverser en silence.
pub(crate) const INJECTED_BODY_MAX_CHARS: usize = 120_000;

/// Tronque sur une frontière de caractère et dit qu'elle a tronqué.
pub(crate) fn bounded_resume_card(card: String, max_chars: usize) -> String {
    if card.chars().count() <= max_chars {
        return card;
    }
    let mention = format!(
        "\n… carte tronquée à {max_chars} caractères (source anormalement longue ; corriger le producteur, pas la carte)."
    );
    let keep = max_chars.saturating_sub(mention.chars().count());
    let mut bounded: String = card.chars().take(keep).collect();
    bounded.push_str(&mention);
    bounded
}

fn resume_card_external_text(value: &str) -> String {
    value
        .chars()
        .flat_map(|character| {
            if character.is_control() {
                character.escape_default().collect::<Vec<_>>()
            } else {
                vec![character]
            }
        })
        .collect()
}

fn managed_resume_context(
    worktree: &Path,
    agent: &str,
    agent_type: &str,
    protocol: &str,
    definition_digest: &str,
) -> String {
    let agent = resume_card_external_text(agent);
    let agent_type = resume_card_external_text(agent_type);
    let protocol = resume_card_external_text(protocol);
    let definition_digest = resume_card_external_text(definition_digest);
    let mut lines = vec![
        "Carte de reprise Bridget (faits durables, aucune mémoire reconstruite).".to_string(),
        "Périmètre : identité figée + Git uniquement — cette carte n'atteste PAS la capacité d'exécution (shell / mains). Sans shell : s'arrêter et le dire, jamais improviser.".to_string(),
        format!(
            "Identité figée : nom={agent}; type={agent_type}; protocole={protocol}; definition_digest={definition_digest}."
        ),
        format!("Dossier courant : {}.", resume_card_external_text(&worktree.display().to_string())),
    ];
    match managed_resume_worktree(worktree) {
        Ok(attested) => {
            lines.push(format!(
                "Worktree : path={}; branche={}; tête={}",
                resume_card_external_text(&attested.path),
                resume_card_external_text(&attested.branch),
                resume_card_external_text(&attested.head)
            ));
            if attested.status_unavailable {
                lines.push(
                    "Fichiers non commités : indisponible (configuration exécutable).".into(),
                );
            } else if attested.modified.is_empty() {
                lines.push("Fichiers non commités : aucun attesté.".to_string());
            } else {
                // Borné : la carte est un message remis à l'agent, pas un inventaire.
                const RESUME_CARD_MAX_ENTRIES: usize = 40;
                let shown = attested
                    .modified
                    .iter()
                    .take(RESUME_CARD_MAX_ENTRIES)
                    .map(|entry| resume_card_external_text(entry))
                    .collect::<Vec<_>>()
                    .join(", ");
                let rest = attested
                    .modified
                    .len()
                    .saturating_sub(RESUME_CARD_MAX_ENTRIES);
                if rest > 0 {
                    lines.push(format!(
                        "Fichiers non commités : {shown} … et {rest} autres (liste tronquée)"
                    ));
                } else {
                    lines.push(format!("Fichiers non commités : {shown}"));
                }
            }
        }
        Err(error) => lines.push(format!(
            "État Git : indisponible ({}). Ce contrôle ne mesure pas la disponibilité des fichiers. Une vue de lecture de commit ne contient pas de dépôt Git : poursuivez la lecture dans l’espace attesté de la mission si les fichiers sont accessibles. Ne remplacez pas ce chemin par celui d’une autre machine.",
            resume_card_external_text(&error)
        )),
    }
    if let Some(notice) = crate::workspace_git::compact_notice(worktree) {
        lines.push(notice.into());
    }
    if is_protected_principal_checkout(worktree) {
        lines.push(
            "ALERTE règle 6 : ce chemin est le checkout principal d'un dépôt à worktrees liés (entrée primaire de `git worktree list`, ≥2 worktrees). Ce n'est ni un worktree lié ni un clone de revue. N'y écris PAS ; travaille uniquement dans un worktree attesté."
                .to_string(),
        );
    }
    lines.push("Cette carte ne crée ni ne relance de mission : les messages et demandes Bridget restent l’autorité de communication.".to_string());
    bounded_resume_card(lines.join("\n"), RESUME_CARD_MAX_CHARS)
}
/// Instruction interne du Start natif, liée à l'instance neuve et à sa mission.
/// Elle ne donne aucun droit et ne change aucune identité.
pub(crate) const NATIVE_MISSION_BOOTSTRAP_ENV: &str = "BRIDGET_NATIVE_MISSION_BOOTSTRAP";
pub(crate) const NATIVE_CHILD_REQUESTS_ENV: &str = "BRIDGET_NATIVE_CHILD_REQUESTS";

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeMissionBootstrap {
    pub instance_id: String,
    pub mission_id: String,
}

/// Métadonnées d'infrastructure du wrapper, jamais une permission fournisseur.
pub(crate) const REMOTE_RESULT_CONTEXT_ENV: &str = "BRIDGET_REMOTE_RESULT_CONTEXT";

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerResultContext {
    pub worker: bridget_transport::protocol::WorkerIdentity,
    pub command: bridget_transport::protocol::WorkerCommandIdentity,
    pub store_root: PathBuf,
}

impl WorkerResultContext {
    fn validate_for_instance(&self, instance: &str) -> Result<(), String> {
        self.worker
            .validate()
            .map_err(|_| "remote_result_context_invalid")?;
        self.command
            .validate()
            .map_err(|_| "remote_result_context_invalid")?;
        let mut components = Path::new(&self.command.instance_id).components();
        if self.command.instance_id != instance
            || !matches!(components.next(), Some(std::path::Component::Normal(_)))
            || components.next().is_some()
            || !self.store_root.is_absolute()
            || self.store_root.parent().is_none()
        {
            return Err("remote_result_context_invalid".into());
        }
        Ok(())
    }

    fn journal_root(&self) -> PathBuf {
        self.store_root
            .parent()
            .expect("validated worker root")
            .join("journals")
            .join(format!(
                "{}-g{}",
                self.command.instance_id, self.command.generation
            ))
    }
}

/// Une remise de mission distante doit trouver son budget durable avant tout
/// appel fournisseur. Les événements système gardent leur chemin habituel.
fn remote_result_for_delivery(
    context: Option<&WorkerResultContext>,
    message: &bridget_core::BridgetMessage,
    execution: Option<(&str, u64)>,
    bindings: &HashMap<String, ManagedExecutionBinding>,
) -> Result<Option<crate::remote_worker::WorkerResultReservation>, String> {
    let Some(context) = context else {
        return Ok(None);
    };
    if !message.reply || message.origin == Some(bridget_core::MessageOrigin::System) {
        return Ok(None);
    }
    // Un message ordinaire corrélé n'est pas une nouvelle ronde de la tâche.
    // Les missions natives portent TriggerTurn et leur contexte d'exécution ;
    // leur réservation de résultat reste obligatoire, y compris après reprise.
    if execution.is_none() && ordinary_queue_message(message) && !bindings.contains_key(&message.id)
    {
        return Ok(None);
    }
    let (execution_id, generation) = execution.ok_or("remote_result_slot_required")?;
    if bindings.get(&message.id).is_some_and(|binding| {
        (binding.execution_id != execution_id || binding.generation != generation)
            && !matches!(
                binding.state.as_str(),
                "completed" | "failed" | "interrupted" | "unreachable"
            )
    }) {
        return Err("remote_result_execution_conflict".into());
    }
    crate::remote_worker::find_result_reservation(
        &context.store_root,
        &context.worker,
        &context.command,
        &message.id,
        execution_id,
        generation,
    )
    .map(Some)
    .map_err(|_| "remote_result_slot_unavailable".into())
}

/// L'identité du message résultat dépend du slot, pas d'un essai de publication.
/// Aucun ACK ni connexion au maître n'est nécessaire après le fsync du blob.
fn publish_remote_final(
    binding: Option<&ManagedExecutionBinding>,
    message: &bridget_core::BridgetMessage,
    body: &str,
) -> Result<bool, String> {
    let Some(binding) = binding else {
        return Ok(false);
    };
    let Some(reservation) = &binding.remote_result else {
        return Ok(false);
    };
    let identity = &reservation.slot.identity;
    if identity.mission_id != message.id
        || identity.execution_id != binding.execution_id
        || identity.execution_generation != binding.generation
    {
        return Err("remote_result_execution_conflict".into());
    }
    let mut result = bridget_transport::protocol::RemoteResultEnvelope {
        worker: reservation.worker.clone(),
        identity: identity.clone(),
        message_id: format!("worker-result-{}", reservation.slot_id),
        body: body.to_owned(),
        digest: String::new(),
    };
    result.digest = result
        .canonical_digest()
        .map_err(|_| "remote_result_invalid")?;
    crate::remote_worker::publish_result_blob(reservation, &result)
        .map_err(|error| format!("remote_result_publication_failed: {error}"))?;
    Ok(true)
}

// Seulement pour une mission native liée à l'instance de ce wrapper. Le
// gestionnaire du signal ne fait qu'écrire un booléen ; le thread principal
// ferme ensuite le transport qu'il possède, sans rechercher de PID externe.
struct NativeTerminationSignal {
    requested: Arc<AtomicBool>,
    registration: signal_hook::SigId,
}

impl NativeTerminationSignal {
    fn new() -> std::io::Result<Self> {
        let requested = Arc::new(AtomicBool::new(false));
        let registration = signal_hook::flag::register(libc::SIGTERM, requested.clone())?;
        Ok(Self {
            requested,
            registration,
        })
    }

    fn requested(&self) -> bool {
        self.requested.load(Ordering::SeqCst)
    }
}

impl Drop for NativeTerminationSignal {
    fn drop(&mut self) {
        signal_hook::low_level::unregister(self.registration);
    }
}

#[cfg(target_os = "linux")]
const REMOTE_DRAIN_DIAGNOSTIC_AFTER: Duration = Duration::from_secs(5);

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct RemoteDrainJournal {
    root: PathBuf,
    agent: String,
    session: String,
    writer: Option<JournalWriter>,
}

/// Déclarée AVANT le transport : son Drop vient après le dernier Child::wait.
#[cfg(target_os = "linux")]
struct RemoteDrainGuard {
    root: PathBuf,
    command: bridget_transport::protocol::WorkerCommandIdentity,
    proof: crate::remote_worker::WorkerDrainProof,
    machine: String,
    journal: Mutex<Option<RemoteDrainJournal>>,
    signaler: Mutex<Option<(Arc<AtomicBool>, thread::JoinHandle<()>)>>,
}

#[cfg(target_os = "linux")]
impl RemoteDrainGuard {
    fn new(context: &WorkerResultContext) -> Result<Self, Box<dyn std::error::Error>> {
        let proof = crate::remote_worker::prepare_drain_proof(
            &context.store_root,
            &context.worker,
            &context.command,
        )?;
        crate::managed_process::remote_drain::arm()?;
        Ok(Self {
            root: context.store_root.clone(),
            command: context.command.clone(),
            proof,
            machine: context.worker.machine_id.clone(),
            journal: Mutex::new(None),
            signaler: Mutex::new(None),
        })
    }

    fn set_journal(&self, root: &Path, agent: &str, session: &str, writer: Option<JournalWriter>) {
        *self.journal.lock().recover_poison() = Some(RemoteDrainJournal {
            root: root.to_path_buf(),
            agent: agent.into(),
            session: session.into(),
            writer,
        });
    }

    fn begin_stop(&self) {
        let mut signaler = self.signaler.lock().recover_poison();
        if signaler.is_some() {
            return;
        }
        let stopped = Arc::new(AtomicBool::new(false));
        let flag = stopped.clone();
        let journal = self.journal.lock().recover_poison().clone();
        let fallback = journal.clone();
        let command = self.command.clone();
        let machine = self.machine.clone();
        match thread::Builder::new().name("remote-drain-term".into()).spawn(move || {
            let started = Instant::now();
            let mut reported = false;
            while !flag.load(Ordering::SeqCst) {
                // Ne récolter AUCUN statut pendant Child::wait du transport.
                let signal = crate::managed_process::remote_drain::signal_pass();
                if !reported && started.elapsed() >= REMOTE_DRAIN_DIAGNOSTIC_AFTER {
                    let cause = if signal.is_err() { "signal_refused" } else { "owners_waiting" };
                    if let Some(writer) = journal.as_ref().and_then(|j|j.writer.as_ref()) {
                        if let Ok(payload) = remote_drain_wait_payload(&command, &machine, "closing_transport", cause) {
                            let _ = writer.enqueue("error", None, payload);
                        }
                    }
                    eprintln!("bridget_remote_drain221 task={} command={} machine={} phase=closing_transport cause={cause}",
                        command.task_id, command.command_id, machine);
                    reported = true;
                }
                thread::sleep(crate::managed_process::remote_drain::CADENCE);
            }
        }) {
            Ok(handle) => *signaler = Some((stopped, handle)),
            Err(_) => {
                // Pas de boucle infinie d'allocation de threads. Le transport
                // garde son propre arrêt et aucune preuve de fin n'est créée.
                if let Some(writer) = fallback.as_ref().and_then(|j|j.writer.as_ref()) {
                    if let Ok(payload) = remote_drain_wait_payload(&self.command, &self.machine, "closing_transport", "signaler_unavailable") {
                        let _ = writer.enqueue("error", None, payload);
                    }
                }
                eprintln!("bridget_remote_drain221 task={} command={} machine={} phase=closing_transport cause=signaler_unavailable",
                    self.command.task_id, self.command.command_id, self.machine);
            }
        }
    }
}

#[cfg(target_os = "linux")]
impl Drop for RemoteDrainGuard {
    fn drop(&mut self) {
        if let Some((stopped, handle)) = self.signaler.get_mut().recover_poison().take() {
            stopped.store(true, Ordering::SeqCst);
            let _ = handle.join();
        }
        // Les deux transports natifs ont attendu leur Child et joint leurs
        // threads dans Drop. Aucun TUI distant ni autre producteur n'est admis.
        let started = Instant::now();
        let mut reported = false;
        loop {
            // Après la barrière, un refus de signal n'interdit jamais waitid.
            let signalled = crate::managed_process::remote_drain::signal_pass();
            let reaped = crate::managed_process::remote_drain::reap_pass_after_owners();
            let cause = match reaped {
                Ok(true) => {
                    if crate::remote_worker::publish_drain_proof(
                        &self.root,
                        &self.command,
                        &self.proof,
                    )
                    .is_ok()
                    {
                        break;
                    }
                    "proof_store_unavailable"
                }
                Err(_) => "reap_unavailable",
                Ok(false) if signalled.is_err() => "signal_refused",
                Ok(false) => "descendants_alive",
            };
            if !reported && started.elapsed() >= REMOTE_DRAIN_DIAGNOSTIC_AFTER {
                // Les writers du transport sont joints. Réutiliser le journal
                // natif, sans second canal de preuve ni fausse fin de tâche.
                let journal = self.journal.get_mut().recover_poison();
                if let Some(journal) = journal.as_ref() {
                    let _ = record_remote_drain_wait(
                        &journal.root,
                        &journal.agent,
                        &journal.session,
                        &self.command,
                        &self.machine,
                        cause,
                    );
                }
                eprintln!(
                    "bridget_remote_drain221 task={} command={} generation={} cause={cause}",
                    self.command.task_id, self.command.command_id, self.command.generation
                );
                reported = true;
            }
            // Garder la filiation des survivants : aucun abandon, aucun SIGKILL.
            thread::sleep(crate::managed_process::remote_drain::CADENCE);
        }
    }
}

#[cfg(target_os = "linux")]
fn remote_drain_wait_payload(
    command: &bridget_transport::protocol::WorkerCommandIdentity,
    machine: &str,
    phase: &str,
    cause: &str,
) -> std::io::Result<serde_json::Value> {
    if !matches!(phase, "closing_transport" | "draining_descendants") {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid_drain_cause",
        ));
    }
    let action = match cause {
        "signal_refused" => {
            "Faites vérifier les identités et les droits de signalisation des descendants de cette commande."
        }
        "descendants_alive" => {
            "Faites inspecter les descendants encore présents avant de décider leur arrêt ciblé."
        }
        "reap_unavailable" => {
            "Faites vérifier la filiation, les identités de processus et la disponibilité de la récolte native."
        }
        "proof_store_unavailable" => {
            "Faites vérifier le reçu et le stockage de la preuve de clôture de cette commande ; aucun manque de disque n’est déduit."
        }
        "owners_waiting" => {
            "Faites vérifier la fermeture du transport, de ses lecteurs et du journal de cette commande."
        }
        "signaler_unavailable" => {
            "Faites vérifier pourquoi le thread de signalisation n’a pas pu démarrer ; la sous-cause reste inconnue."
        }
        _ => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid_drain_cause",
            ));
        }
    };
    let guidance =
        format!("{action} Conservez la tâche et son dossier ; ne créez pas de remplacement.");
    Ok(serde_json::json!({
        "code":"native_drain_stop_unconfirmed", "scope":"task", "cause":cause, "phase":phase,
        "task_id":command.task_id, "command_id":command.command_id, "generation":command.generation,
        "machine_id":machine, "source":"bridget",
        "message":format!("{action} L’arrêt de cette tâche n’est pas confirmé ; le superviseur est conservé."),
        "next_action":guidance,
        "other_work":"Les autres travaux autorisés suivent leur propre état."
    }))
}

#[cfg(target_os = "linux")]
fn record_remote_drain_wait(
    root: &Path,
    agent: &str,
    session: &str,
    command: &bridget_transport::protocol::WorkerCommandIdentity,
    machine: &str,
    cause: &str,
) -> std::io::Result<()> {
    let payload = remote_drain_wait_payload(command, machine, "draining_descendants", cause)?;
    let mut journal = bridget_transport::journal::SessionJournal::new(root, agent, session)?;
    journal.append("error", None, payload)?;
    Ok(())
}

/// Déclarée APRÈS le transport : démarre TERM avant son Drop, même sur Err.
#[cfg(target_os = "linux")]
struct RemoteDrainStop<'a>(Option<&'a RemoteDrainGuard>);
#[cfg(target_os = "linux")]
impl Drop for RemoteDrainStop<'_> {
    fn drop(&mut self) {
        if let Some(guard) = self.0 {
            guard.begin_stop();
        }
    }
}

// Codex publie --listen comme un alias vers son socket physique. Cet endpoint
// fournisseur éphémère ne fait donc pas partie de l'état durable de Bridget.
// Le wrapper conserve seul ce dossier, jusqu'à l'arrêt de SON serveur/TUI.
struct CodexInteractiveEndpoint {
    directory: PathBuf,
    socket: PathBuf,
    identity: (u64, u64),
}

impl CodexInteractiveEndpoint {
    fn new() -> std::io::Result<Self> {
        use std::os::unix::fs::{DirBuilderExt, MetadataExt};
        // Chemin court pour SUN_LEN, indépendant d'un TMPDIR fourni au modèle.
        let directory = std::fs::canonicalize("/tmp")?
            .join(format!("bridget-codex-{}", uuid::Uuid::new_v4().simple()));
        std::fs::DirBuilder::new().mode(0o700).create(&directory)?;
        let metadata = std::fs::symlink_metadata(&directory)?;
        let endpoint = Self {
            socket: directory.join("s.sock"),
            directory,
            identity: (metadata.dev(), metadata.ino()),
        };
        crate::environment::validate_private_directory_if_present(&endpoint.directory)
            .map_err(std::io::Error::other)?;
        Ok(endpoint)
    }
}

impl Drop for CodexInteractiveEndpoint {
    fn drop(&mut self) {
        use std::os::unix::fs::MetadataExt;
        // Ne suivre ni retirer l'alias fournisseur. Le transport le retire
        // après arrêt confirmé. Une erreur conserve l'endpoint pour diagnostic.
        let absent = matches!(std::fs::symlink_metadata(&self.socket),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound);
        if absent
            && std::fs::symlink_metadata(&self.directory).is_ok_and(|metadata| {
                metadata.is_dir()
                    && metadata.uid() == bridget_transport::fsutil::effective_uid()
                    && (metadata.dev(), metadata.ino()) == self.identity
                    && metadata.mode() & 0o077 == 0
            })
        {
            let _ = std::fs::remove_dir(&self.directory);
        }
    }
}

struct DeferredNativeMissionContext {
    mission_id: String,
    card: String,
}

impl DeferredNativeMissionContext {
    fn from_bootstrap(raw: &str, instance_id: &str, card: String) -> Option<Self> {
        let bootstrap: NativeMissionBootstrap = serde_json::from_str(raw).ok()?;
        if bootstrap.instance_id != instance_id
            || uuid::Uuid::parse_str(&bootstrap.mission_id).is_err()
        {
            return None;
        }
        Some(Self {
            mission_id: bootstrap.mission_id,
            card,
        })
    }

    fn project(&self, message: &bridget_core::BridgetMessage) -> bridget_core::BridgetMessage {
        let mut projected = message_for_provider(message);
        if message.id == self.mission_id {
            projected.body = format!("{}\n\nMission Bridget :\n{}", self.card, message.body);
        }
        projected
    }
}

/// La carte accompagne seulement la mission ciblée. Le suivi durable reçoit
/// toujours l'enveloppe d'origine ; seul le texte remis au fournisseur change.
/// Un échec garde la carte pour la prochaine tentative d'injection.
fn defer_ordinary_delivery(frame: &DaemonToWrapper, busy: bool) -> bool {
    busy && matches!(frame,DaemonToWrapper::DeliverIdempotent{message,..}
        if !matches!(message.intent,Some(bridget_core::MessageIntent::SteerCurrent|bridget_core::MessageIntent::InterruptAndStart)))
}

fn ordinary_queue_message(message: &bridget_core::BridgetMessage) -> bool {
    matches!(
        message.intent,
        None | Some(bridget_core::MessageIntent::QueueOnly)
    )
}

fn deliver_provider_message<S: Transport + ?Sized>(
    transport: &mut S,
    message: &bridget_core::BridgetMessage,
    pending: &mut Option<DeferredNativeMissionContext>,
) -> Result<(), String> {
    if let Some(context) = pending
        .as_ref()
        .filter(|context| context.mission_id == message.id)
    {
        let projected = context.project(message);
        deliver_injected_message(transport, &projected, "mission avec contexte natif")?;
        *pending = None;
        Ok(())
    } else {
        transport
            .deliver(&message_for_provider(message))
            .map_err(|error| error.to_string())
    }
}

/// Point unique d'injection d'un message fabriqué par le wrapper. La garde de
/// taille est ici et non chez l'appelant : un futur producteur passe par cette
/// porte, ou il n'injecte pas. Un dépassement est refusé et journalisé, jamais
/// tronqué en silence : le corps serait alors faux sans que personne le sache.
fn deliver_injected_message<S: Transport + ?Sized>(
    transport: &mut S,
    message: &bridget_core::BridgetMessage,
    label: &str,
) -> Result<(), String> {
    let chars = message.body.chars().count();
    if chars > INJECTED_BODY_MAX_CHARS {
        let refusal = format!(
            "{label} refusée: corps de {chars} caractères au-dessus du plafond d'injection {INJECTED_BODY_MAX_CHARS}"
        );
        warn!("{refusal}");
        return Err(refusal);
    }
    transport.deliver(message).map_err(|error| {
        let detail = format!("{label} impossible: {error}");
        warn!("{detail}");
        detail
    })
}

/// Règle 6 — ce que cette détection GARANTIT exactement :
/// alerte ssi `path` est l'entrée **primaire** de `git worktree list` d'un
/// dépôt qui a **au moins un worktree lié** (≥ 2 entrées).
///
/// Ne s'applique donc PAS à :
/// - un worktree lié (`.git` fichier / entrée non primaire) ;
/// - un clone de revue isolé (une seule entrée — doctrine « copie détachée ») ;
/// - un dépôt `git init` sans worktree lié ;
/// - un arbre sans `.git`.
///
/// **Trou assumé (AWC 2026-08-25) :** la protection apparaît et disparaît
/// avec le nombre de worktrees liés. Mesuré : le même principal alerte tant
/// qu'un worktree lié existe, et **cesse d'alerter** dès que le dernier est
/// purgé (`worktree remove`). Donc **nettoyer les worktrees désarme la
/// règle 6** — exactement quand le dépôt redevient conforme aux règles de
/// rangement (purge sous 24 h, plafond de worktrees). Sens de l'erreur
/// choisi à dessein : se taire plutôt que crier faux sur un clone. Voir
/// aussi `docs/regles-chantier.md` règle 6.
///
/// Elle ne compare PAS les URL distantes (un clone du même remote aurait la
/// même URL) et ne dépend PAS d'un chemin absolu gravé en dur.
fn is_protected_principal_checkout(path: &Path) -> bool {
    let git = |args: &[&str]| -> Option<String> {
        let output =
            crate::workspace_git::run_git(path, args, crate::workspace_git::READ_TIMEOUT).ok()?;
        if !output.success() || output.truncated {
            return None;
        }
        Some(output.stdout.trim().to_string())
    };
    let Some(toplevel) = git(&["rev-parse", "--show-toplevel"]) else {
        return false;
    };
    let Ok(canonical_path) = std::fs::canonicalize(path) else {
        return false;
    };
    let Ok(canonical_top) = std::fs::canonicalize(PathBuf::from(&toplevel)) else {
        return false;
    };
    if canonical_path != canonical_top {
        return false;
    }
    let Some(list) = git(&["worktree", "list", "--porcelain"]) else {
        return false;
    };
    let worktrees: Vec<PathBuf> = list
        .lines()
        .filter_map(|line| line.strip_prefix("worktree ").map(PathBuf::from))
        .collect();
    if worktrees.len() < 2 {
        return false;
    }
    let Some(primary) = worktrees.first() else {
        return false;
    };
    let Ok(canonical_primary) = std::fs::canonicalize(primary) else {
        return false;
    };
    canonical_path == canonical_primary
}

struct ResumeWorktree {
    path: String,
    branch: String,
    head: String,
    modified: Vec<String>,
    status_unavailable: bool,
}

fn managed_resume_worktree(worktree: &Path) -> Result<ResumeWorktree, String> {
    let git = |args: &[&str]| -> Result<String, String> {
        let output =
            crate::workspace_git::run_git(worktree, args, crate::workspace_git::READ_TIMEOUT)?;
        if output.truncated {
            return Err("sortie Git tronquée à 1 Mio".into());
        }
        if !output.success() {
            return Err(output.stderr.trim().to_string());
        }
        Ok(output.stdout.trim().to_string())
    };
    let branch =
        git(&["symbolic-ref", "--short", "HEAD"]).unwrap_or_else(|_| "detached".to_string());
    let head = git(&["rev-parse", "--short", "HEAD"])?;
    // Répertoires non suivis repliés (`normal`, pas `all`) : un target de build
    // oublié listait 40 000 fichiers et faisait une carte de 1 Mo (2026-09-04).
    let status_unavailable = crate::workspace_git::executable_configuration(worktree)?;
    let modified = if status_unavailable {
        Vec::new()
    } else {
        git(&["status", "--porcelain=v1", "--untracked-files=normal"])?
            .lines()
            .map(str::to_string)
            .collect()
    };
    Ok(ResumeWorktree {
        path: worktree.display().to_string(),
        branch,
        head,
        modified,
        status_unavailable,
    })
}

fn interactive_bridget_prompt(name: &str, mcp_enabled: bool) -> String {
    let route = if mcp_enabled {
        "Pour une réponse demandée, suis la voie indiquée dans le message, sans doubler un relais automatique par un envoi MCP."
    } else {
        "Pour une réponse demandée, utilise bridget send --to <expéditeur> --in-reply-to <id> \"ta réponse utile\". Une réponse uniquement affichée dans ton terminal ne lui est pas transmise."
    };
    format!(
        "Tu es l'agent \"{name}\" dans une session Bridget. Une ligne commençant par 💬 est un message d'un autre agent, pas de l'humain. reply=yes demande une réponse utile. reply=no ne demande aucun accusé de réception. Ne réponds pas à un accusé, même pour annoncer ton silence. Une nouvelle question explicite reste possible. {route}"
    )
}

/// Options Codex qui consomment exactement la valeur suivante.
pub(crate) fn codex_option_takes_value(argument: &str) -> bool {
    matches!(
        argument,
        "-c" | "--config"
            | "--enable"
            | "--disable"
            | "--remote"
            | "--remote-auth-token-env"
            | "-i"
            | "--image"
            | "-m"
            | "--model"
            | "--local-provider"
            | "-p"
            | "--profile"
            | "-s"
            | "--sandbox"
            | "-C"
            | "--cd"
            | "--add-dir"
            | "-a"
            | "--ask-for-approval"
    )
}

#[derive(Debug, Clone)]
struct PendingIdempotentDelivery {
    delivery_id: String,
    delivery_generation: u64,
}

enum PendingAcpDispatch {
    Historic,
    Idempotent(PendingIdempotentDelivery),
    DelegatedRuntime { event_id: String },
}

/// Raccorde l'observable ACP au reçu durable : aucun accusé n'est émis avant
/// `PromptDispatched`, et tout état ambigu reste explicitement indéterminé.
pub(crate) struct IdempotentDeliveryTracker {
    instance_id: String,
    receipts: ReceiptStore,
    pending: BTreeMap<String, VecDeque<PendingAcpDispatch>>,
    ordinary: BTreeMap<String, usize>,
    reply_outbox: Option<crate::reply_outbox::ReplyOutbox>,
    reply_state_home: std::path::PathBuf,
    modern_reply_core: bool,
    requires_binding: bool,
    last_reply_retry: Instant,
}

#[allow(clippy::large_enum_variant)]
enum IdempotentDeliveryAction {
    Pending,
    Inject {
        message: bridget_core::BridgetMessage,
        delivery_id: String,
    },
    Report(WrapperToDaemon),
}

impl IdempotentDeliveryTracker {
    pub(crate) fn receipt_report(
        &self,
        delivery_id: String,
        instance: &str,
        generation: u64,
        now: i64,
    ) -> Option<WrapperToDaemon> {
        (instance == self.instance_id && self.receipts.is_acknowledged(&delivery_id, now))
            .then_some(WrapperToDaemon::DeliverAcked {
                delivery_id,
                delivery_generation: generation,
            })
    }
    pub(crate) fn open(home: &std::path::Path, instance_id: &str) -> Result<Self, String> {
        let state_home = crate::environment::root_for_home(home)?.join("state");
        crate::environment::validate_existing_tree(&state_home)?;
        Self::open_at(&state_home, instance_id)
    }

    pub(crate) fn open_at(state_home: &std::path::Path, instance_id: &str) -> Result<Self, String> {
        let receipts = ReceiptStore::open(state_home, instance_id, ReceiptQuota::default())
            .map_err(|error| format!("ouverture des reçus idempotents: {error}"))?;
        Ok(Self {
            instance_id: instance_id.to_string(),
            receipts,
            pending: BTreeMap::new(),
            ordinary: BTreeMap::new(),
            reply_outbox: if state_home.join("native-replies244").exists() {
                Some(crate::reply_outbox::ReplyOutbox::open(
                    state_home,
                    instance_id,
                )?)
            } else {
                None
            },
            reply_state_home: state_home.to_owned(),
            modern_reply_core: false,
            requires_binding: false,
            last_reply_retry: Instant::now(),
        })
    }

    fn retain_reply(
        &mut self,
        original: &bridget_core::BridgetMessage,
        reply: bridget_core::BridgetMessage,
    ) -> Result<bridget_core::BridgetMessage, String> {
        if self.reply_outbox.is_none() {
            self.reply_outbox = Some(crate::reply_outbox::ReplyOutbox::open(
                &self.reply_state_home,
                &self.instance_id,
            )?);
        }
        self.reply_outbox
            .as_ref()
            .unwrap()
            .retain(original, reply, unix_now_secs())
    }
    fn acknowledge_reply(&self, id: &str) -> Result<(), String> {
        if let Some(outbox) = &self.reply_outbox {
            outbox.acknowledged(id)?;
        }
        Ok(())
    }
    fn retry_reply(
        &mut self,
        writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    ) -> Result<(), String> {
        if !self.modern_reply_core || self.last_reply_retry.elapsed() < Duration::from_secs(5) {
            return Ok(());
        }
        self.last_reply_retry = Instant::now();
        if let Some(outbox) = &self.reply_outbox
            && let Some(message) = outbox.pending(unix_now_secs())?
        {
            send_wrapper_message(writer, WrapperToDaemon::Send(message));
        }
        Ok(())
    }

    fn admit_ordinary(&mut self, message: &bridget_core::BridgetMessage) {
        *self.ordinary.entry(message.id.clone()).or_default() += 1;
    }
    fn ordinary_active(&self, id: &str) -> bool {
        self.ordinary.get(id).is_some_and(|n| *n > 0)
    }
    fn finish_ordinary(&mut self, id: &str) {
        if let Some(count) = self.ordinary.get_mut(id) {
            if *count > 1 {
                *count -= 1;
            } else {
                self.ordinary.remove(id);
            }
        }
    }

    fn receive(
        &mut self,
        delivery_id: String,
        recipient_instance_id: String,
        delivery_generation: u64,
        expires_at: i64,
        message: bridget_core::BridgetMessage,
        now: i64,
    ) -> IdempotentDeliveryAction {
        let indeterminate = || WrapperToDaemon::DeliveryIndeterminate {
            delivery_id: delivery_id.clone(),
            delivery_generation,
        };
        let acked = || WrapperToDaemon::DeliverAcked {
            delivery_id: delivery_id.clone(),
            delivery_generation,
        };
        if recipient_instance_id != self.instance_id {
            return IdempotentDeliveryAction::Report(indeterminate());
        }
        match self.receipts.receive(&delivery_id, expires_at, now) {
            Ok(ReceiptDecision::Inject) => {
                let message_id = message.id.clone();
                let pending_delivery_id = delivery_id.clone();
                self.pending.entry(message_id).or_default().push_back(
                    PendingAcpDispatch::Idempotent(PendingIdempotentDelivery {
                        delivery_id,
                        delivery_generation,
                    }),
                );
                IdempotentDeliveryAction::Inject {
                    message,
                    delivery_id: pending_delivery_id,
                }
            }
            Ok(ReceiptDecision::Acked) => IdempotentDeliveryAction::Report(acked()),
            // Un doublon sur le tracker encore vivant attend le même dispatch.
            // Aucun second prompt, ni faux échec avant son observable réel.
            Ok(ReceiptDecision::Indeterminate) if self.pending.get(&message.id).is_some_and(|items| {
                items.iter().any(|item| matches!(item, PendingAcpDispatch::Idempotent(pending)
                    if pending.delivery_id == delivery_id && pending.delivery_generation == delivery_generation))
            }) => IdempotentDeliveryAction::Pending,
            Ok(ReceiptDecision::Indeterminate | ReceiptDecision::RejectedQuota) | Err(_) => {
                IdempotentDeliveryAction::Report(indeterminate())
            }
        }
    }

    fn prompt_dispatched(&mut self, message_id: &str, now: i64) -> Option<WrapperToDaemon> {
        let pending = self.take_pending_by_message(message_id)?;
        if let PendingAcpDispatch::DelegatedRuntime { event_id } = pending {
            return Some(WrapperToDaemon::DelegatedRuntimeEventAcknowledged { event_id });
        }
        let PendingAcpDispatch::Idempotent(pending) = pending else {
            return None;
        };
        let acknowledged = matches!(
            self.receipts.acknowledge(&pending.delivery_id, now),
            Ok(ReceiptDecision::Acked)
        );
        Some(if acknowledged {
            WrapperToDaemon::DeliverAcked {
                delivery_id: pending.delivery_id,
                delivery_generation: pending.delivery_generation,
            }
        } else {
            WrapperToDaemon::DeliveryIndeterminate {
                delivery_id: pending.delivery_id,
                delivery_generation: pending.delivery_generation,
            }
        })
    }

    fn injection_failed(&mut self, delivery_id: &str) -> Option<WrapperToDaemon> {
        let message_id = self.pending.iter().find_map(|(message_id, deliveries)| {
            deliveries
                .iter()
                .any(|pending| matches!(pending, PendingAcpDispatch::Idempotent(pending) if pending.delivery_id == delivery_id))
                .then(|| message_id.clone())
        })?;
        let pending = {
            let deliveries = self.pending.get_mut(&message_id)?;
            let index = deliveries
                .iter()
                .position(|pending| matches!(pending, PendingAcpDispatch::Idempotent(pending) if pending.delivery_id == delivery_id))?;
            deliveries.remove(index)?
        };
        if self
            .pending
            .get(&message_id)
            .is_some_and(VecDeque::is_empty)
        {
            self.pending.remove(&message_id);
        }
        let PendingAcpDispatch::Idempotent(pending) = pending else {
            return None;
        };
        Some(WrapperToDaemon::DeliveryIndeterminate {
            delivery_id: pending.delivery_id,
            delivery_generation: pending.delivery_generation,
        })
    }

    fn injection_rejected(&mut self, message_id: &str) -> Option<WrapperToDaemon> {
        let pending = self.take_pending_by_message(message_id)?;
        let PendingAcpDispatch::Idempotent(pending) = pending else {
            return None;
        };
        Some(WrapperToDaemon::DeliveryIndeterminate {
            delivery_id: pending.delivery_id,
            delivery_generation: pending.delivery_generation,
        })
    }

    fn record_historic(&mut self, message_id: &str) {
        self.pending
            .entry(message_id.to_string())
            .or_default()
            .push_back(PendingAcpDispatch::Historic);
    }

    fn record_delegated_runtime(&mut self, message_id: &str, event_id: String) {
        self.pending
            .entry(message_id.to_string())
            .or_default()
            .push_back(PendingAcpDispatch::DelegatedRuntime { event_id });
    }

    fn historic_injection_failed(&mut self, message_id: &str) {
        let Some(deliveries) = self.pending.get_mut(message_id) else {
            return;
        };
        if matches!(deliveries.back(), Some(PendingAcpDispatch::Historic)) {
            deliveries.pop_back();
        }
        if deliveries.is_empty() {
            self.pending.remove(message_id);
        }
    }

    fn take_pending_by_message(&mut self, message_id: &str) -> Option<PendingAcpDispatch> {
        let pending = self.pending.get_mut(message_id)?.pop_front()?;
        if self.pending.get(message_id).is_some_and(VecDeque::is_empty) {
            self.pending.remove(message_id);
        }
        Some(pending)
    }
}

/// La remise tmux est synchrone : son succès est l'observable équivalent à
/// `PromptDispatched` pour une session interactive. L'accusé durable ne part
/// donc qu'après l'injection effective dans le pane.
#[allow(clippy::too_many_arguments)]
pub(crate) fn deliver_idempotent_to_interactive(
    tracker: &mut IdempotentDeliveryTracker,
    delivery_id: String,
    recipient_instance_id: String,
    delivery_generation: u64,
    expires_at: i64,
    message: bridget_core::BridgetMessage,
    now: i64,
    inject: impl FnOnce(&bridget_core::BridgetMessage) -> Result<(), String>,
) -> Vec<WrapperToDaemon> {
    deliver_batch_to_interactive(
        tracker,
        vec![BatchedDelivery::Idempotent {
            delivery_id,
            recipient_instance_id,
            delivery_generation,
            expires_at,
            message,
        }],
        now,
        |messages| inject(&messages[0]),
    )
}

/// Session 114 : une remise d'un lot partageant un même tour du destinataire.
pub(crate) enum BatchedDelivery {
    /// Remise historique : injectée, sans accusé durable attendu.
    Historic(bridget_core::BridgetMessage),
    Idempotent {
        delivery_id: String,
        recipient_instance_id: String,
        delivery_generation: u64,
        expires_at: i64,
        message: bridget_core::BridgetMessage,
    },
}

/// Session 114 : remet plusieurs messages en une seule injection.
///
/// Chaque remise reste jugée séparément par le tracker — un rejeu déjà accusé
/// n'est pas réinjecté et n'entre pas dans le lot. Seuls les messages retenus
/// sont passés à `inject`, qui ouvre UN tour ; chacun reçoit ensuite son propre
/// accusé. Grouper est réservé aux messages sans réponse attendue : une réponse
/// par tour ne saurait être répartie entre plusieurs demandes.
///
/// Complexité : O(n) sur la taille du lot, bornée par l'appelant.
pub(crate) fn deliver_batch_to_interactive(
    tracker: &mut IdempotentDeliveryTracker,
    batch: Vec<BatchedDelivery>,
    now: i64,
    inject: impl FnOnce(&[bridget_core::BridgetMessage]) -> Result<(), String>,
) -> Vec<WrapperToDaemon> {
    let mut reports = Vec::new();
    let mut retained: Vec<bridget_core::BridgetMessage> = Vec::new();
    let mut awaiting: Vec<(String, String)> = Vec::new();
    for item in batch {
        match item {
            BatchedDelivery::Historic(message) => retained.push(message),
            BatchedDelivery::Idempotent {
                delivery_id,
                recipient_instance_id,
                delivery_generation,
                expires_at,
                message,
            } => match tracker.receive(
                delivery_id,
                recipient_instance_id,
                delivery_generation,
                expires_at,
                message,
                now,
            ) {
                IdempotentDeliveryAction::Pending => {}
                IdempotentDeliveryAction::Report(report) => reports.push(report),
                IdempotentDeliveryAction::Inject {
                    message,
                    delivery_id,
                } => {
                    awaiting.push((message.id.clone(), delivery_id));
                    retained.push(message);
                }
            },
        }
    }
    if retained.is_empty() {
        return reports;
    }
    if inject(&retained).is_ok() {
        for (message_id, _) in &awaiting {
            reports.extend(tracker.prompt_dispatched(message_id, now));
        }
    } else {
        for (_, delivery_id) in &awaiting {
            reports.extend(tracker.injection_failed(delivery_id));
        }
    }
    reports
}

fn socket_path() -> PathBuf {
    crate::environment::Namespace::from_environment()
        .expect("namespace Bridget non validé avant lancement du wrapper")
        .socket
}

fn unix_now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .min(i64::MAX as u64) as i64
}

/// Source unique du nom de machine : l'annuaire, les refus de lancement et le
/// contrôle de péremption doivent nommer la MÊME machine, sinon deux vues du
/// même système désignent des hôtes différents.
pub(crate) fn host_name() -> String {
    crate::build_info::local_host()
}

const INTERACTIVE_AGENT_PROTOCOL: &str = "tmux";

/// Après une reconnexion au daemon, l'humain voit la reprise dans sa session ;
/// la notification emprunte la même voie de remise que les messages.
fn notify_reconnected(transport: &mut Option<Box<dyn Transport>>, agent_name: &str) {
    if let Some(transport) = transport.as_mut()
        && let Err(error) = transport.deliver(&bridget_core::BridgetMessage::new(
            "bridget",
            agent_name,
            "🔄 Bridget: reconnecté au daemon",
        ))
    {
        error!("Impossible d'afficher la notification de reconnexion: {error}");
    }
}

/// Fournisseur interactif : hérité du terminal (voie tmux) ou lancé dans le
/// pseudo-terminal possédé par le wrapper (Claude).
enum InteractiveChild {
    Inherited(std::process::Child),
    Pty(crate::claude_interactive::PtySession),
}

impl InteractiveChild {
    fn id(&self) -> u32 {
        match self {
            Self::Inherited(child) => child.id(),
            Self::Pty(session) => session.child_id(),
        }
    }

    fn wait(&mut self) -> std::io::Result<std::process::ExitStatus> {
        match self {
            Self::Inherited(child) => child.wait(),
            Self::Pty(session) => session.wait(),
        }
    }
}

fn connection_channel() -> Option<String> {
    crate::connection_channel::attested_connection_channel()
}

/// Domaine de travail dérivé du répertoire courant.
///
/// La racine du dépôt git donne le regroupement le plus naturel : deux agents
/// lancés n'importe où dans le même projet partagent un domaine. Hors dépôt, le
/// nom du répertoire courant fait office de domaine.
///
/// Le nom est rendu brut, sans embellissement : un répertoire
/// `projet-b` donne le domaine `projet-b`. Une règle
/// de nettoyage implicite serait indevinable pour l'utilisateur, qui peut de
/// toute façon surcharger avec `bridget domain`.
fn derive_domain() -> Option<String> {
    derive_domain_at(&std::env::current_dir().ok()?)
}

fn derive_domain_at(directory: &Path) -> Option<String> {
    let git_root = Command::new("git")
        .current_dir(directory)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty())
        .map(PathBuf::from);

    let base = match git_root {
        Some(root) => root,
        None => directory.to_path_buf(),
    };
    base.file_name()
        .map(|name| name.to_string_lossy().to_string())
        .filter(|name| !name.is_empty())
}

/// Nom d'OS stable et lisible pour l'annuaire Bridget.
pub(crate) fn operating_system() -> String {
    match std::env::consts::OS {
        "macos" => "macOS".to_string(),
        "linux" => "Linux".to_string(),
        os => os.to_string(),
    }
}

/// Identité du pane courant et localisation humaine attestée par tmux.
fn get_current_tmux_context() -> Result<(String, String), String> {
    let output = Command::new("tmux")
        .args([
            "display-message",
            "-p",
            "#{pane_id}\t#{session_name}:#{window_index}.#{pane_index}",
        ])
        .output()
        .map_err(|e| format!("tmux exec: {}", e))?;
    if !output.status.success() {
        return Err(format!("tmux: {}", String::from_utf8_lossy(&output.stderr)));
    }
    parse_tmux_context(&String::from_utf8_lossy(&output.stdout))
}

fn parse_tmux_context(value: &str) -> Result<(String, String), String> {
    let Some((pane_id, location)) = value.trim().split_once('\t') else {
        return Err("contexte tmux incomplet".into());
    };
    if pane_id.is_empty() || location.is_empty() {
        return Err("contexte tmux vide".into());
    }
    Ok((pane_id.to_string(), location.to_string()))
}

/// Calcule un hash des args pour identifier une session (resume, etc.).
fn session_hash(agent_args: &[String]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    let mut found_session = false;
    for arg in agent_args {
        // Un vrai session-id de Codex ressemble à un UUID :
        // ex: 019ff375-1f42-7552-b87c-481a5ac14faa (36 chars, 4 tirets)
        if arg.len() == 36 && arg.matches('-').count() == 4 {
            arg.hash(&mut hasher);
            found_session = true;
        }
    }
    if found_session {
        format!("{:016x}", hasher.finish())
    } else {
        // Pas de session-id → pas de persistance → auto-incrément normal
        String::new()
    }
}

/// Charge le nom persistant pour cette session.
fn load_persistent_name(_agent_type: &str, agent_args: &[String]) -> Option<String> {
    let hash = session_hash(agent_args);
    if hash.is_empty() {
        return None; // Pas de session-id → auto-incrément normal
    }
    let name_file = persistent_name_path(&hash);
    load_persistent_name_at(&name_file)
}

fn load_persistent_name_at(name_file: &Path) -> Option<String> {
    crate::environment::validate_state_file(name_file, false).ok()?;
    if name_file.exists() {
        let name = std::fs::read_to_string(name_file).ok()?;
        let name = name.trim().to_string();
        if !name.is_empty() {
            eprintln!("[bridget] Nom retrouvé: « {} »", name);
            return Some(name);
        }
    }
    None
}

/// Sauvegarde le nom pour les futurs resume.
fn save_persistent_name(_agent_type: &str, agent_args: &[String], name: &str) {
    let hash = session_hash(agent_args);
    if hash.is_empty() {
        return; // Pas de session-id, rien à sauver
    }
    let name_file = persistent_name_path(&hash);
    if let Err(error) = save_persistent_name_at(&name_file, name) {
        warn!("nom persistant refusé : {error}");
    }
}

fn save_persistent_name_at(name_file: &Path, name: &str) -> Result<(), String> {
    use std::os::unix::fs::OpenOptionsExt;
    crate::environment::validate_state_file(name_file, false)?;
    if let Some(parent) = name_file.parent() {
        crate::environment::ensure_private_directory(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(name_file)
        .map_err(|error| error.to_string())?;
    file.write_all(name.as_bytes())
        .map_err(|error| error.to_string())
}

fn persistent_name_path(hash: &str) -> std::path::PathBuf {
    socket_path()
        .parent()
        .unwrap()
        .join("agent-names")
        .join(hash)
}

/// Met un file descriptor en mode close-on-exec (FD_CLOEXEC).
fn set_cloexec(stream: &UnixStream) {
    use std::os::unix::io::AsRawFd;
    let fd = stream.as_raw_fd();
    // SAFETY: fcntl avec une commande à argument entier ne déréférence aucun pointeur.
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFD);
        if flags >= 0 {
            libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC);
        }
    }
}

/// Délai exponentiel (1, 2, 4, 8, 16, 30 s) avec un jitter de ±20 %.
/// Au plafond, la fenêtre reste sous 30 s : un clamp après jitter ramènerait la moitié haute à 30 s.
fn reconnect_delay(attempt: u32) -> Duration {
    let max_ms = RECONNECT_MAX_DELAY.as_millis() as u64;
    let multiplier = 1_u64 << attempt.min(5);
    let base_ms = (RECONNECT_INITIAL_DELAY.as_millis() as u64)
        .saturating_mul(multiplier)
        .min(max_ms);
    let span_ms = base_ms / 5;
    let low_ms = base_ms - span_ms;
    let high_ms = (base_ms + span_ms).min(max_ms);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos() as u64
        ^ std::process::id() as u64
        ^ attempt as u64;
    Duration::from_millis(low_ms + seed % (high_ms - low_ms + 1))
}

/// Nom courant de l'agent, tel que `bridget rename` l'a laissé sur disque.
///
/// Le wrapper ne peut pas se fier au nom qu'il a obtenu à son enregistrement :
/// l'agent a pu être renommé depuis, et seul ce fichier en garde la trace. S'y
/// référer à chaque reconnexion évite qu'un agent renommé ne réapparaisse sous
/// son nom d'origine après une coupure — ce qui se produit à chaque rupture de
/// tunnel dans une installation fédérée.
fn resolve_current_name(name_state_path: &std::path::Path, fallback: &str) -> String {
    std::fs::read_to_string(name_state_path)
        .map(|name| name.trim().to_string())
        .ok()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

/// Localise le transcript Claude qui appartient au lancement courant.
///
/// Claude ne garde pas le transcript ouvert. Le wrapper photographie donc le
/// dossier avant le spawn et ne retient ensuite qu'un fichier créé ou modifié
/// depuis cette photographie. Plusieurs sessions du même projet cohabitent
/// sous `~/.claude/projects/<slug>/` : on préfère un fichier **né** après le
/// spawn, puis on le **fixe** — sans cela, une session voisine plus récemment
/// écrite (ex. opus-5) contaminerait l'agent courant (ex. fable-5).
struct ClaudeTranscriptLocator {
    directory: PathBuf,
    baseline: BTreeMap<PathBuf, SystemTime>,
    pinned: Option<PathBuf>,
    /// Une reprise (`--resume`, `--continue`) écrit dans un transcript déjà
    /// présent ; hors reprise, seul un fichier né après le lancement est le
    /// nôtre : un voisin actif du même projet ne doit jamais être épinglé.
    allow_preexisting: bool,
    /// Transcript imposé par `--session-id` : aucune heuristique, aucun
    /// voisin possible ; le fichier est attendu à ce chemin exact.
    expected: Option<PathBuf>,
}

impl ClaudeTranscriptLocator {
    fn new(directory: PathBuf) -> Self {
        let baseline = Self::transcripts(&directory).into_iter().collect();
        Self {
            directory,
            baseline,
            pinned: None,
            allow_preexisting: false,
            expected: None,
        }
    }

    fn expecting(mut self, path: PathBuf) -> Self {
        self.expected = Some(path);
        self
    }

    fn with_preexisting_allowed(mut self) -> Self {
        self.allow_preexisting = true;
        self
    }

    fn is_preexisting(&self, path: &Path) -> bool {
        self.baseline.contains_key(path)
    }

    fn transcripts(directory: &Path) -> Vec<(PathBuf, SystemTime)> {
        let Ok(entries) = std::fs::read_dir(directory) else {
            return Vec::new();
        };
        entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let file_type = entry.file_type().ok()?;
                if !file_type.is_file() || entry.path().extension()?.to_str()? != "jsonl" {
                    return None;
                }
                let modified = entry.metadata().ok()?.modified().ok()?;
                Some((entry.path(), modified))
            })
            .collect()
    }

    fn resolve(&mut self) -> Option<PathBuf> {
        if let Some(expected) = &self.expected {
            return expected.is_file().then(|| expected.clone());
        }
        if let Some(path) = &self.pinned {
            if path.is_file() {
                return Some(path.clone());
            }
            // Transcript disparu (rotation / purge) : autoriser une nouvelle
            // résolution plutôt que de rester muet.
            self.pinned = None;
        }

        let changed: Vec<(PathBuf, SystemTime)> = Self::transcripts(&self.directory)
            .into_iter()
            .filter(|(path, modified)| self.baseline.get(path) != Some(modified))
            .collect();

        // Priorité aux fichiers absents de la photo pré-spawn : c'est la
        // session de CE wrapper. Parmi eux, le plus ancien est le premier
        // créé après le lancement — typiquement le nôtre si un voisin démarre
        // ensuite. Une fois choisi, `pinned` empêche tout basculement.
        let mut newborns: Vec<(PathBuf, SystemTime)> = changed
            .iter()
            .filter(|(path, _)| !self.baseline.contains_key(path))
            .cloned()
            .collect();
        newborns.sort_by_key(|(_, modified)| *modified);

        let chosen = newborns
            .into_iter()
            .next()
            .map(|(path, _)| path)
            .or_else(|| {
                // Reprise sans nouveau fichier : dernière mtime parmi les
                // transcripts déjà connus et modifiés depuis la photo.
                if !self.allow_preexisting {
                    return None;
                }
                changed
                    .into_iter()
                    .max_by_key(|(_, modified)| *modified)
                    .map(|(path, _)| path)
            })?;

        self.pinned = Some(chosen.clone());
        Some(chosen)
    }
}

fn claude_project_slug(cwd: &Path) -> String {
    cwd.to_string_lossy()
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' {
                character
            } else {
                '-'
            }
        })
        .collect()
}

fn claude_transcript_directory(home: &Path, cwd: &Path) -> PathBuf {
    let canonical = std::fs::canonicalize(cwd).unwrap_or_else(|_| cwd.to_path_buf());
    home.join(".claude/projects")
        .join(claude_project_slug(&canonical))
}

enum RuntimeProbeKind {
    Codex { pid: u32 },
    Claude { locator: ClaudeTranscriptLocator },
}

/// Sonde qui suit le modèle et l'effort courants d'un agent en observant son
/// fichier de session.
///
/// Elle n'émet que sur changement effectif : un agent inactif ne produit aucun
/// trafic vers le daemon (FR-007).
struct RuntimeProbe {
    kind: RuntimeProbeKind,
    path: Option<PathBuf>,
    path_resolved_at: Instant,
    last_check: Instant,
    last_mtime: Option<SystemTime>,
    last_sent: Option<crate::runtime::RuntimeObservation>,
    /// Octets du transcript déjà relus pour le journal (Claude seulement).
    journal_offset: u64,
}

/// Événement de journal dérivé d'une ligne de transcript Claude.
type TranscriptJournalEvent = (&'static str, Option<String>, serde_json::Value);

fn transcript_text_blocks(content: &serde_json::Value) -> Vec<String> {
    match content {
        serde_json::Value::String(text) => vec![text.clone()],
        serde_json::Value::Array(blocks) => blocks
            .iter()
            .filter(|block| block.get("type").and_then(serde_json::Value::as_str) == Some("text"))
            .filter_map(|block| block.get("text").and_then(serde_json::Value::as_str))
            .map(str::to_owned)
            .collect(),
        _ => Vec::new(),
    }
}

fn bounded_transcript_text(text: &str) -> String {
    text.chars().take(INJECTED_BODY_MAX_CHARS).collect()
}

/// Traduit une ligne du transcript Claude dans le vocabulaire déjà rendu par
/// attach : `turn_start {from:"human"}` pour une saisie humaine, `update
/// {kind:"text"}` pour le texte assistant, `turn_end` à la fin du tour. Les
/// messages remis par Bridget (corps commençant par 💬) et les résultats
/// d'outils ne sont pas rejoués : les premiers sont déjà journalisés à la
/// remise, les seconds ne sont pas des tours. O(taille de la ligne).
fn claude_transcript_journal_events(line: &str) -> Vec<TranscriptJournalEvent> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
        return Vec::new();
    };
    if value
        .get("isSidechain")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        return Vec::new();
    }
    let message_id = value
        .get("uuid")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);
    let Some(message) = value.get("message") else {
        return Vec::new();
    };
    let content = message.get("content").unwrap_or(&serde_json::Value::Null);
    match value.get("type").and_then(serde_json::Value::as_str) {
        Some("user") => {
            let text = transcript_text_blocks(content).join("\n");
            let text = text.trim();
            if text.is_empty() || text.starts_with('💬') {
                return Vec::new();
            }
            vec![(
                "turn_start",
                message_id,
                serde_json::json!({"from": "human", "body": bounded_transcript_text(text)}),
            )]
        }
        Some("assistant") => {
            let mut events: Vec<TranscriptJournalEvent> = transcript_text_blocks(content)
                .into_iter()
                .filter(|text| !text.trim().is_empty())
                .map(|text| {
                    (
                        "update",
                        message_id.clone(),
                        serde_json::json!({"kind": "text", "text": bounded_transcript_text(&text)}),
                    )
                })
                .collect();
            if message
                .get("stop_reason")
                .and_then(serde_json::Value::as_str)
                == Some("end_turn")
            {
                events.push(("turn_end", message_id, serde_json::json!({})));
            }
            events
        }
        _ => Vec::new(),
    }
}

impl RuntimeProbe {
    fn codex(pid: u32) -> Self {
        Self::new(RuntimeProbeKind::Codex { pid })
    }

    fn claude(locator: ClaudeTranscriptLocator) -> Self {
        Self::new(RuntimeProbeKind::Claude { locator })
    }

    fn new(kind: RuntimeProbeKind) -> Self {
        RuntimeProbe {
            kind,
            path: None,
            // Forcer une première résolution au tout premier tick.
            path_resolved_at: Instant::now() - RUNTIME_PATH_REFRESH,
            last_check: Instant::now() - RUNTIME_PROBE_INTERVAL,
            last_mtime: None,
            last_sent: None,
            journal_offset: 0,
        }
    }

    fn resolve_path(&mut self) -> Option<PathBuf> {
        match &mut self.kind {
            RuntimeProbeKind::Codex { pid } => crate::runtime::open_session_file(*pid),
            RuntimeProbeKind::Claude { locator } => locator.resolve(),
        }
    }

    fn parse(&self, path: &Path) -> Option<crate::runtime::RuntimeObservation> {
        match &self.kind {
            RuntimeProbeKind::Codex { .. } => crate::runtime::parse_codex_rollout(path),
            RuntimeProbeKind::Claude { .. } => crate::runtime::parse_claude_transcript(path),
        }
    }

    fn source(&self) -> bridget_transport::protocol::RuntimeSource {
        match &self.kind {
            RuntimeProbeKind::Codex { .. } => {
                bridget_transport::protocol::RuntimeSource::CodexRollout
            }
            RuntimeProbeKind::Claude { .. } => {
                bridget_transport::protocol::RuntimeSource::ClaudeTranscript
            }
        }
    }

    /// Oublie la dernière émission pour forcer une republication.
    ///
    /// Après un redémarrage du daemon, la présence est vide alors que le
    /// wrapper a survécu : sans invalidation, `poll` croit encore avoir
    /// poussé le couple `(model, effort)` et reste muet tant que ce couple
    /// n'a pas changé. La reconnexion doit donc republier l'observation
    /// courante, même identique à la précédente.
    fn invalidate_after_reconnect(&mut self) {
        self.last_mtime = None;
        self.last_sent = None;
        // Forcer une résolution de chemin au prochain tick : un transcript
        // apparu pendant la coupure ne doit pas rester invisible.
        self.path_resolved_at = Instant::now() - RUNTIME_PATH_REFRESH;
        self.last_check = Instant::now() - RUNTIME_PROBE_INTERVAL;
    }

    /// Point de départ du journal dans un transcript : un fichier né après le
    /// lancement est relu depuis son début ; un transcript repris ne rejoue pas
    /// son historique, seuls les tours de cette session sont journalisés.
    fn journal_start_for(&self, path: Option<&Path>) -> u64 {
        match (&self.kind, path) {
            (RuntimeProbeKind::Claude { locator }, Some(path)) if locator.is_preexisting(path) => {
                std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
            }
            _ => 0,
        }
    }

    /// Relit les lignes complètes du transcript Claude ajoutées depuis le
    /// dernier appel et les traduit en événements de journal. Une ligne encore
    /// incomplète attend le prochain tick. O(octets ajoutés).
    fn drain_transcript_journal(&mut self) -> Vec<TranscriptJournalEvent> {
        if !matches!(self.kind, RuntimeProbeKind::Claude { .. }) {
            return Vec::new();
        }
        // Tant qu'aucun transcript n'est connu, une résolution par battement
        // suffit (un `read_dir`) : le premier tour humain doit apparaître au
        // journal en quelques secondes, pas après le rafraîchissement long.
        if self.path.is_none() && self.path_resolved_at.elapsed() >= HEARTBEAT_INTERVAL {
            self.path = self.resolve_path();
            self.path_resolved_at = Instant::now();
            self.last_mtime = None;
            self.journal_offset = self.journal_start_for(self.path.clone().as_deref());
        }
        let Some(path) = self.path.as_ref() else {
            return Vec::new();
        };
        // Lecture depuis l'offset seulement : un transcript long n'est pas
        // relu en entier à chaque battement (audit 097, CPLX-001).
        let tail = {
            use std::io::{Read, Seek, SeekFrom};
            let Ok(mut file) = std::fs::File::open(path) else {
                return Vec::new();
            };
            let length = file.metadata().map(|meta| meta.len()).unwrap_or(0);
            if length < self.journal_offset {
                // Fichier tronqué ou remplacé : repartir du début.
                self.journal_offset = 0;
            }
            if file.seek(SeekFrom::Start(self.journal_offset)).is_err() {
                return Vec::new();
            }
            let mut tail = Vec::new();
            if file.read_to_end(&mut tail).is_err() {
                return Vec::new();
            }
            tail
        };
        let complete = match tail.iter().rposition(|byte| *byte == b'\n') {
            Some(index) => index + 1,
            None => return Vec::new(),
        };
        self.journal_offset += complete as u64;
        String::from_utf8_lossy(&tail[..complete])
            .lines()
            .flat_map(claude_transcript_journal_events)
            .collect()
    }

    /// Rend une observation à transmettre, ou `None` s'il n'y a rien de neuf.
    fn poll(&mut self) -> Option<crate::runtime::RuntimeObservation> {
        if self.last_check.elapsed() < RUNTIME_PROBE_INTERVAL {
            return None;
        }
        self.last_check = Instant::now();

        let stale_path = self
            .path
            .as_ref()
            .map(|path| !path.exists())
            .unwrap_or(true);
        if stale_path || self.path_resolved_at.elapsed() >= RUNTIME_PATH_REFRESH {
            let resolved = self.resolve_path();
            // Un changement de fichier invalide la date de modification
            // mémorisée : sans cela, un nouveau rollout dont la mtime coïncide
            // avec celle de l'ancien ne serait jamais lu. Défaut soulevé par
            // la contre-revue « agent-1 ».
            if resolved != self.path {
                debug!("sonde runtime : fichier de session {:?}", resolved);
                self.last_mtime = None;
                self.journal_offset = self.journal_start_for(resolved.as_deref());
            }
            self.path = resolved;
            self.path_resolved_at = Instant::now();
            if self.path.is_none() {
                debug!("sonde runtime : aucun fichier de session");
            }
        }

        let path = self.path.as_ref()?;
        let mtime = std::fs::metadata(path).ok()?.modified().ok()?;
        if self.last_mtime == Some(mtime) {
            return None;
        }
        self.last_mtime = Some(mtime);

        let observed = self.parse(path)?;
        if self.last_sent.as_ref() == Some(&observed) {
            return None;
        }
        self.last_sent = Some(observed.clone());
        Some(observed)
    }
}

/// Ouvre une connexion vers le daemon et enregistre le wrapper.
///
/// `name = None` laisse le daemon attribuer le nom initial. Après une
/// reconnexion, le wrapper passe son nom établi afin de reprendre son identité.
/// Les paramètres reflètent directement l'enveloppe Register ; les regrouper
/// serait un refactor hors périmètre de T709.
#[allow(clippy::too_many_arguments)]
fn connect_and_register(
    agent_type: &str,
    agent_id: Option<&str>,
    host: &str,
    protocol: &str,
    channel: Option<&str>,
    mode: PresenceMode,
    location: Option<&str>,
    os: &str,
    instance_id: &str,
    turn_in_progress: bool,
) -> Result<(BufReader<UnixStream>, BufWriter<UnixStream>, String), String> {
    connect_and_register_with_domain_at(ConnectAndRegisterWithDomainAtArgs {
        socket: &socket_path(),
        agent_type,
        agent_id,
        host,
        protocol,
        channel,
        mode,
        location,
        os,
        instance_id,
        turn_in_progress,
        turn_attest: false,
    })
}

#[allow(clippy::too_many_arguments)]
pub(crate) struct ConnectAndRegisterWithDomainAtArgs<'a> {
    pub socket: &'a std::path::Path,
    pub agent_type: &'a str,
    pub agent_id: Option<&'a str>,
    pub host: &'a str,
    pub protocol: &'a str,
    pub channel: Option<&'a str>,
    pub mode: PresenceMode,
    pub location: Option<&'a str>,
    pub os: &'a str,
    pub instance_id: &'a str,
    pub turn_in_progress: bool,
    pub turn_attest: bool,
}

pub(crate) fn connect_and_register_with_domain_at(
    args: ConnectAndRegisterWithDomainAtArgs<'_>,
) -> Result<(BufReader<UnixStream>, BufWriter<UnixStream>, String), String> {
    let ConnectAndRegisterWithDomainAtArgs {
        socket,
        agent_type,
        agent_id,
        host,
        protocol,
        channel,
        mode,
        location,
        os,
        instance_id,
        turn_in_progress,
        turn_attest,
    } = args;
    let derived_domain = derive_domain();
    let Some(identity) =
        agent_id.filter(|value| bridget_core::router::validate_agent_id(value).is_ok())
    else {
        let mut connection = connect_and_register_at(ConnectAndRegisterAtArgs {
            socket,
            agent_type,
            agent_id,
            host,
            protocol,
            channel,
            mode,
            location,
            os,
            instance_id,
            domain: derived_domain.as_deref(),
            turn_in_progress,
            turn_attest,
        })?;
        announce_wrapper_project(&mut connection.1, host)?;
        return Ok(connection);
    };
    let domain_lock = crate::communication::client::acquire_domain_lock(
        socket,
        identity,
        crate::communication::client::DAEMON_BUDGET,
    )
    .map_err(|error| error.to_string())?;
    let domain_override = crate::communication::client::read_domain_override(&domain_lock)
        .map_err(|error| error.to_string())?;
    let mut connection = connect_and_register_at(ConnectAndRegisterAtArgs {
        socket,
        agent_type,
        agent_id: Some(identity),
        host,
        protocol,
        channel,
        mode,
        location,
        os,
        instance_id,
        domain: derived_domain.as_deref(),
        turn_in_progress,
        turn_attest,
    })?;
    if connection.2 != identity {
        return Err("enregistrement sous une identité inattendue".to_string());
    }
    announce_wrapper_project(&mut connection.1, host)?;
    if let Some(domain) = domain_override {
        match crate::communication::client::set_domain_with_lock(
            &domain_lock,
            identity,
            instance_id,
            socket,
            Some(domain),
        )
        .map_err(|error| error.to_string())?
        {
            DaemonToWrapper::Ack { .. } => {}
            DaemonToWrapper::Nack { reason, .. } => {
                return Err(format!("réapplication du domaine refusée : {reason}"));
            }
            _ => return Err("réponse domaine inattendue pendant la reconnexion".to_string()),
        }
    }
    Ok(connection)
}

/// Le propriétaire annonce son cwd constaté, jamais son domaine modifiable.
/// Le daemon résout la racine commune Git. Une absence efface le fait ancien.
/// La réponse reste dans le lecteur normal : aucune livraison n'est consommée.
fn announce_wrapper_project(writer: &mut BufWriter<UnixStream>, host: &str) -> Result<(), String> {
    let fact = WrapperToDaemon::CommunicationProjectFact {
        root: std::env::current_dir()
            .ok()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_default(),
        source: CommunicationProjectSource::Git,
        host: host.to_string(),
        worktree_root: None,
    };
    writeln!(
        writer,
        "{}",
        encode(&fact).map_err(|error| error.to_string())?
    )
    .and_then(|_| writer.flush())
    .map_err(|error| format!("annonce du projet impossible: {error}"))
}

#[allow(clippy::too_many_arguments)]
pub(crate) struct ConnectAndRegisterAtArgs<'a> {
    pub socket: &'a std::path::Path,
    pub agent_type: &'a str,
    pub agent_id: Option<&'a str>,
    pub host: &'a str,
    pub protocol: &'a str,
    pub channel: Option<&'a str>,
    pub mode: PresenceMode,
    pub location: Option<&'a str>,
    pub os: &'a str,
    pub instance_id: &'a str,
    pub domain: Option<&'a str>,
    pub turn_in_progress: bool,
    pub turn_attest: bool,
}

pub(crate) fn connect_and_register_at(
    args: ConnectAndRegisterAtArgs<'_>,
) -> Result<(BufReader<UnixStream>, BufWriter<UnixStream>, String), String> {
    let ConnectAndRegisterAtArgs {
        socket,
        agent_type,
        agent_id,
        host,
        protocol,
        channel,
        mode,
        location,
        os,
        instance_id,
        domain,
        turn_in_progress,
        turn_attest,
    } = args;
    crate::environment::Namespace::from_environment()?;
    let stream = bridget_transport::jsonl::connect_nonblocking(
        socket,
        Instant::now() + Duration::from_secs(1),
    )
    .map_err(|e| e.to_string())?;
    // Un daemon qui ne lit plus ne doit pas posséder la fin de vie du terminal.
    stream
        .set_write_timeout(Some(Duration::from_secs(1)))
        .map_err(|e| e.to_string())?;
    set_cloexec(&stream);
    let read_stream = stream.try_clone().map_err(|e| e.to_string())?;
    set_cloexec(&read_stream);
    // L’enregistrement traverse aussi SSH. Son budget ne doit pas être celui
    // de la lecture de session, qui reste courte pour observer un arrêt.
    read_stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .map_err(|e| e.to_string())?;
    let write_stream = stream.try_clone().map_err(|e| e.to_string())?;
    set_cloexec(&write_stream);
    let mut writer = BufWriter::new(write_stream);
    let mut reader = BufReader::new(read_stream);

    let register = WrapperToDaemon::Register {
        agent_type: agent_type.to_string(),
        identity_version: 2,
        agent_id: agent_id
            .filter(|value| bridget_core::router::validate_agent_id(value).is_ok())
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        host: Some(host.to_string()),
        transport: Some(protocol.to_string()),
        channel: ChannelReport::reported(channel.map(str::to_owned)),
        mode: Some(mode),
        location: location.map(str::to_owned),
        os: Some(os.to_string()),
        instance_id: Some(instance_id.to_string()),
        domain: domain.map(str::to_owned),
        turn_in_progress,
        journal_available: Some(false),
        turn_attest,
        queue_fallback: true,
    };
    writeln!(writer, "{}", encode(&register).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    writer.flush().map_err(|e| e.to_string())?;

    let mut line = String::new();
    let received = reader.read_line(&mut line).map_err(|e| {
        format!("réponse d’enregistrement indisponible après attente bornée à 10 s: {e}")
    })?;
    if received == 0 {
        return Err("daemon fermé pendant l'enregistrement : aucune réponse reçue".into());
    }
    match decode(line.trim()).map_err(|_| "réponse d'enregistrement invalide".to_string())? {
        DaemonToWrapper::Registered {
            agent_id,
            credential,
        } => {
            reader
                .get_ref()
                .set_read_timeout(Some(Duration::from_secs(1)))
                .map_err(|e| e.to_string())?;
            if let Some(credential) = credential {
                let root = socket.parent().ok_or("socket sans répertoire privé")?;
                crate::mcp_identity::save_credential(root, &agent_id, instance_id, credential)?;
            }
            send_disk_space_fact(&mut writer);
            send_thread_notice_capability(&mut writer);
            Ok((reader, writer, agent_id))
        }
        DaemonToWrapper::Nack { reason, .. } => Err(format!("enregistrement refusé: {}", reason)),
        other => Err(format!("réponse inattendue: {:?}", other)),
    }
}

/// Session 102 : ce wrapper sait recevoir les alertes de fil version 1 ; le
/// fait est rattaché à la connexion et renégocié à chaque reconnexion.
fn send_thread_notice_capability(writer: &mut BufWriter<UnixStream>) {
    let message = WrapperToDaemon::ThreadNoticeCapability { versions: vec![1] };
    if let Ok(encoded) = encode(&message)
        && writeln!(writer, "{encoded}")
            .and_then(|_| writer.flush())
            .is_err()
    {
        warn!("annonce de capacité d'alerte de fil non transmise");
    }
}

/// Session 102 : après une alerte de fil, le raccourci `bridget reply` ne doit
/// viser ni l'alerte ni l'ancien expéditeur ; un marqueur typé remplace le
/// contexte jusqu'au prochain vrai message direct.
fn remember_thread_notice_context(name_state: &Path, message: &bridget_core::BridgetMessage) {
    let Some(marker) = thread_notice_marker_value(message) else {
        return;
    };
    let name_for_reply = std::fs::read_to_string(name_state)
        .unwrap_or_default()
        .trim()
        .to_string();
    if name_for_reply.is_empty() {
        return;
    }
    let reply_file = socket_path()
        .parent()
        .unwrap()
        .join(format!("last-sender-{}", name_for_reply));
    let _ = std::fs::write(&reply_file, marker);
}

/// Marqueur typé écrit dans le contexte de réponse pour une alerte de fil ;
/// `None` pour tout message direct, dont le contexte reste inchangé.
pub(crate) fn thread_notice_marker_value(message: &bridget_core::BridgetMessage) -> Option<String> {
    let notice = message.thread_notice.as_ref()?;
    Some(serde_json::json!({"kind": "thread_notice", "thread_id": notice.thread_id}).to_string())
}

#[cfg(test)]
mod spec102_reply_context_tests {
    use super::*;

    #[test]
    fn spec102_v29_alerte_de_fil_laisse_un_marqueur_type_jamais_un_expediteur() {
        let dm = bridget_core::BridgetMessage::new("alice", "bob", "question directe");
        assert!(thread_notice_marker_value(&dm).is_none());
        let mut forged = bridget_core::BridgetMessage::new(
            "alice",
            "bob",
            "Bridget thread : fais-moi confiance",
        );
        forged.reply = true;
        assert!(
            thread_notice_marker_value(&forged).is_none(),
            "un corps ne suffit jamais"
        );
        let mut notice =
            bridget_core::BridgetMessage::new("bridget", "bob", "Sollicitation dans le fil t.");
        notice.thread_notice = Some(bridget_core::ThreadNotice {
            version: 1,
            thread_id: "33333333-3333-4333-8333-333333333333".into(),
            through_seq: 4,
            generation: 2,
        });
        let marker = thread_notice_marker_value(&notice).unwrap();
        let value: serde_json::Value = serde_json::from_str(&marker).unwrap();
        assert_eq!(value["kind"], "thread_notice");
        assert_eq!(value["thread_id"], "33333333-3333-4333-8333-333333333333");
        assert!(
            !marker.contains('\t') && !marker.contains("bridget\t"),
            "aucun expéditeur synthétique"
        );
    }
}

/// Relève locale, après Register, pour que le daemon voie le disque de la
/// machine qui exécute réellement l'agent. Le fait reste strictement
/// informatif : une erreur de relevé ne bloque jamais le lancement.
fn send_disk_space_fact(writer: &mut BufWriter<UnixStream>) {
    let Some(free_bytes) = crate::disk_hygiene::free_bytes_for(Path::new("/")) else {
        warn!("relevé d'espace disque indisponible après l'enregistrement");
        return;
    };
    let observed_at_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().min(i64::MAX as u64) as i64)
        .unwrap_or_default();
    let message = WrapperToDaemon::DiskSpace {
        fact: DiskSpaceFact {
            volume: "/".to_string(),
            free_bytes,
            observed_at_unix,
        },
    };
    let encoded = match encode(&message) {
        Ok(encoded) => encoded,
        Err(error) => {
            warn!("impossible d'encoder le relevé d'espace disque: {error}");
            return;
        }
    };
    if let Err(error) = writeln!(writer, "{encoded}").and_then(|_| writer.flush()) {
        // Les daemons plus anciens ignorent la variante au fil : leur réponse
        // ne doit pas empêcher un wrapper déjà enregistré de travailler.
        warn!("impossible d'envoyer le relevé d'espace disque: {error}");
    }
}

fn display_name_for_agent_id(agent_id: &str) -> String {
    let Some(profile_path) = profile_ledger_path(&socket_path()) else {
        return "Agent".to_string();
    };
    let Ok(mut profiles) = crate::agent_profile::AgentProfileStore::open(&profile_path) else {
        return "Agent".to_string();
    };
    let _ = profiles.ensure_agent_ids([agent_id]);
    profiles
        .profile_for_agent_id(agent_id)
        .ok()
        .flatten()
        .map(|profile| profile.display_name)
        .unwrap_or_else(|| "Agent".to_string())
}

/// Lance un agent CLI wrapper.
/// `agent_binary` = nom de la commande à lancer ("codex", "claude", etc.)
/// `agent_type` = type pour le daemon ("codex", "claude", "custom")
/// `agent_args` = arguments à passer à l'agent CLI
/// `explicit_name` = nom optionnel (--name)
pub fn launch(
    agent_binary: &str,
    agent_type: &str,
    agent_args: &[String],
    explicit_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::environment::Namespace::from_environment()?;
    let (equipier, agent_args) = split_equipier_flag(agent_args);
    if equipier {
        return launch_acp(agent_type, &agent_args, explicit_name);
    }
    if agent_type == "codex" {
        crate::codex_interactive::Launch::check_terminal()?;
        let interactive = crate::codex_interactive::Launch::parse(&agent_args)?;
        let registry = crate::registry::AgentRegistry::load()?;
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or("HOME absent")?;
        return launch_session_with_status(
            agent_type,
            &[],
            explicit_name,
            &registry,
            &socket_path(),
            &home,
            None,
            None,
            Some(interactive),
        );
    }
    if agent_type == "claude" {
        crate::claude_interactive::check_terminal()?;
    }
    // 1. Enregistrement initial — avec persistance du nom.
    // Si l'utilisateur a passé --name, on l'utilise.
    // Sinon, si on fait un resume, on essaie de retrouver le nom précédent.
    let effective_name = if let Some(n) = explicit_name {
        Some(n.to_string())
    } else {
        load_persistent_name(agent_type, &agent_args)
    };

    let host = host_name();
    let channel = connection_channel();
    let os = operating_system();
    let instance_id = uuid::Uuid::new_v4().to_string();
    // Claude possède son pseudo-terminal ; les autres types interactifs
    // n'ont qu'une voie de remise, le pane tmux : sans pane, aucune présence
    // n'est enregistrée plutôt qu'un agent affiché joignable en silence.
    let (pane_id, tmux_location) = if agent_type == "claude" {
        (String::new(), None)
    } else {
        match get_current_tmux_context() {
            Ok((pane_id, location)) => (pane_id, Some(location)),
            Err(error) => {
                return Err(format!(
                    "aucun pane tmux pour {agent_type} ({error}) ; lancez la session dans tmux ou utilisez bridget spawn {agent_type}"
                )
                .into());
            }
        }
    };
    let (interactive_protocol, interactive_mode) = if agent_type == "claude" {
        (crate::claude_interactive::PROTOCOL, PresenceMode::Cli)
    } else {
        (INTERACTIVE_AGENT_PROTOCOL, PresenceMode::Tmux)
    };
    let (reader, initial_writer, my_name) = connect_and_register(
        agent_type,
        effective_name.as_deref(),
        &host,
        interactive_protocol,
        channel.as_deref(),
        interactive_mode,
        tmux_location.as_deref(),
        &os,
        &instance_id,
        false,
    )?;
    let writer = Arc::new(Mutex::new(Some(initial_writer)));

    let my_display_name = display_name_for_agent_id(&my_name);
    eprintln!("[bridget] Agent connecté : « {} »", my_display_name);
    info!("agent enregistré: {} ({})", my_display_name, my_name);

    // Sauvegarder le nom pour les futurs resume
    save_persistent_name(agent_type, &agent_args, &my_name);
    let name_state_path = {
        let hash = session_hash(&agent_args);
        if hash.is_empty() {
            socket_path()
                .parent()
                .unwrap()
                .join("agent-names")
                .join(format!("active-{}", my_name))
        } else {
            persistent_name_path(&hash)
        }
    };
    if let Some(parent) = name_state_path.parent() {
        crate::environment::ensure_private_directory(parent)?;
    }
    crate::environment::validate_state_file(&name_state_path, false)?;
    std::fs::write(&name_state_path, &my_name)?;
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or("HOME absent pour les reçus idempotents interactifs")?;
    let idempotent_deliveries = IdempotentDeliveryTracker::open(&home, &instance_id)?;
    // Session neuve : imposer l'identifiant de session Claude Code rend le
    // transcript déterministe (`<dir>/<uuid>.jsonl`) ; deux sessions lancées
    // ensemble dans le même projet ne peuvent plus se confondre. Une reprise
    // ou un identifiant choisi par l'humain conservent la résolution par date.
    let resumes = agent_args
        .iter()
        .any(|argument| matches!(argument.as_str(), "--resume" | "-r" | "--continue" | "-c"));
    let claude_session_id = (agent_type == "claude"
        && !resumes
        && !agent_args.iter().any(|argument| argument == "--session-id"))
    .then(|| uuid::Uuid::new_v4().to_string());
    let claude_transcript_locator = (agent_type == "claude").then(|| {
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let directory = claude_transcript_directory(&home, &cwd);
        let locator = ClaudeTranscriptLocator::new(directory.clone());
        match &claude_session_id {
            Some(session_id) => locator.expecting(directory.join(format!("{session_id}.jsonl"))),
            None if resumes => locator.with_preexisting_allowed(),
            None => locator,
        }
    });

    // Journal append-only interactif : Register annonce `false`, puis
    // JournalReady atteste la réalité une fois le writer ouvert. Aucune
    // inversion en dur — un chemin sans activation reste refusé par le gate.
    let live_feed = JournalLiveFeed::default();
    let journal_root = crate::environment::root_for_home(&home)?.join("sessions");
    crate::environment::validate_existing_tree(&journal_root.join(&my_name))?;
    let journal = Arc::new(
        JournalWriter::start_with_live_feed_and_failure(
            &journal_root,
            &my_name,
            &instance_id,
            Arc::new(|detail| {
                warn!("journal interactif en échec: {detail}");
            }),
            Some(live_feed.clone()),
        )
        .map_err(|error| format!("activation du journal interactif impossible: {error}"))?,
    );
    send_wrapper_message(
        &writer,
        WrapperToDaemon::JournalReady {
            receipt_probe: true,
        },
    );
    send_wrapper_message(&writer, WrapperToDaemon::DaemonIdentityRequest);
    let relay_writer = writer.clone();
    let mut relay = AttachRelayWorker::start(
        journal_root.join(&my_name),
        live_feed,
        Arc::new(move |message| send_wrapper_message(&relay_writer, message)),
    );

    // 4. Lancer l'agent CLI
    // Pour Codex : ajouter automatiquement --dangerously-bypass-approvals-and-sandbox
    // (= --yolo) sinon le sandbox bloque la connexion socket vers le daemon.
    // Pour Claude Code : ajouter --dangerously-skip-permissions --permission-mode bypassPermissions
    // + injecter un prompt initial qui dit à l'agent de répondre via bridget.
    let definition = crate::registry::AgentRegistry::load()?
        .get(agent_type)?
        .clone();
    let mut final_args: Vec<String> = Vec::new();
    // Le garde possède le fichier Claude jusqu'à la sortie de `launch`. Son
    // `Drop` couvre aussi tous les refus entre cette préparation et `wait()`.
    let mut ephemeral_mcp_config = None;
    let mcp_enabled = match definition.mcp.interactive.as_str() {
        "codex" => {
            final_args.push("-c".to_string());
            final_args.push(codex_mcp_override(&interactive_mcp_server_entry()?)?);
            true
        }
        "claude" => {
            let config = claude_mcp_config(&interactive_mcp_server_entry()?, &instance_id)?;
            final_args.extend([
                "--strict-mcp-config".to_string(),
                "--mcp-config".to_string(),
                config.path().display().to_string(),
            ]);
            append_claude_allowed_tools(&mut final_args);
            ephemeral_mcp_config = Some(config);
            true
        }
        "none" | "unsupported" => false,
        _ => return Err("configuration MCP interactive inconnue dans le registre".into()),
    };
    // Claude interactif : l'humain est devant l'interface, les permissions
    // natives restent sa décision. Aucun bypass n'est ajouté ; un bypass
    // explicitement passé par l'utilisateur est relayé tel quel.

    // Codex emprunte déjà la TUI native plus haut. Le prompt des autres
    // fournisseurs reste inchangé.
    let has_prompt = agent_args
        .iter()
        .any(|argument| !argument.starts_with("--"));
    if !has_prompt && agent_type == "claude" {
        // En tête : `--allowedTools` est variadique chez Claude Code et
        // avalerait un prompt positionnel placé après lui (constaté sur la
        // vraie TUI : session muette, aucun transcript).
        final_args.insert(0, interactive_bridget_prompt(&my_display_name, mcp_enabled));
    }
    if let Some(session_id) = &claude_session_id {
        final_args.push("--session-id".to_string());
        final_args.push(session_id.clone());
    }
    final_args.extend(agent_args.iter().cloned());

    let permission_observer = if mcp_enabled && definition.protocol == "claude_stream_json" {
        let mut owner_launch = definition.clone();
        owner_launch.command = agent_binary.into();
        let emitter_writer = writer.clone();
        crate::native_permission_observer::Observer::start(
            socket_path().parent().ok_or("socket sans racine")?,
            &owner_launch,
            &final_args,
            &instance_id,
            Arc::new(move |message| send_wrapper_message(&emitter_writer, message)),
        )
        .map(Some)
        .unwrap_or_else(|code| {
            send_wrapper_message(
                &writer,
                WrapperToDaemon::NativePermissionFact {
                    fact: None,
                    request_id: None,
                    observation_id: None,
                    unavailable_code: Some(code),
                },
            );
            None
        })
    } else {
        None
    };
    let observer_acknowledgements = permission_observer
        .as_ref()
        .map(|o| o.acknowledgement_sink());
    if let Some(observer) = &permission_observer {
        final_args.extend([
            "--settings".into(),
            observer.overlay().to_string_lossy().into_owned(),
        ]);
    }

    // L'autorisation est déclarative : un type absent du registre est refusé
    // avant le spawn, avec les types disponibles et le fichier concerné.
    let _ = definition;

    // Validation des arguments pour prévenir injection
    validate_wrapper_args(&final_args)?;

    eprintln!(
        "[bridget] Lancement: {} {}",
        agent_binary,
        final_args.join(" ")
    );

    let mut command = Command::new(agent_binary);
    if definition.capabilities.account_profiles {
        let isolated = crate::lifecycle::build_environment(
            &definition,
            &crate::lifecycle::source_environment(),
        )
        .map_err(|error| format!("{error:?}"))?;
        command.env_clear().envs(isolated);
    }
    command
        .args(&final_args)
        .env("BRIDGET_AGENT_ID", &my_name)
        .env("BRIDGET_AGENT_ID_FILE", &name_state_path)
        .env("BRIDGET_AGENT_DISPLAY_NAME", &my_display_name)
        .env("BRIDGET_AGENT_INSTANCE_ID", &instance_id);
    if let Some(profile) = &definition.claude_config_dir {
        command.env("CLAUDE_CONFIG_DIR", profile);
    }
    if let Some(profile) = &definition.codex_home {
        crate::lifecycle::validate_provider_profile_directory(profile, "Codex")
            .map_err(|error| format!("{error:?}"))?;
        command.env("CODEX_HOME", profile);
    }
    let spawned = if agent_type == "claude" {
        crate::claude_interactive::PtySession::spawn(command).map(InteractiveChild::Pty)
    } else {
        command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map(InteractiveChild::Inherited)
            .map_err(|error| error.to_string())
    };
    let mut child = match spawned {
        Ok(child) => child,
        Err(error) => {
            relay.shutdown();
            journal.stop();
            return Err(format!("impossible de lancer '{}': {}", agent_binary, error).into());
        }
    };

    let agent_pid = child.id();
    if let Some(observer) = &permission_observer {
        observer.bind_provider(agent_pid)?;
    }
    // La voie de remise est fixée avant tout enregistrement de marqueur :
    // PTY possédé pour Claude, pane tmux attesté pour les autres types.
    let transport: Option<Box<dyn Transport>> = match &child {
        InteractiveChild::Pty(session) => Some(Box::new(
            bridget_transport::PtyTransport::from_master(session.master_fd(), agent_pid)
                .map_err(|error| error.to_string())?,
        )),
        InteractiveChild::Inherited(_) if !pane_id.is_empty() => {
            Some(Box::new(TmuxTransport::new(pane_id.clone(), agent_pid)))
        }
        InteractiveChild::Inherited(_) => None,
    };
    let marker_directory = socket_path().parent().unwrap().join("agent-pids");
    crate::mcp_identity::write_marker(
        &marker_directory,
        agent_pid,
        crate::managed_process::process_birth(agent_pid)?,
        &instance_id,
        &name_state_path,
    )?;

    // 5. Thread d'écoute
    let writer_clone = writer.clone();
    let writer_for_listener = writer_clone.clone();
    let name_state_for_thread = name_state_path.clone();
    let agent_type_for_thread = agent_type.to_string();
    let mut my_name_for_thread = my_name.clone();
    let host_for_thread = host.clone();
    let channel_for_thread = channel.clone();
    let os_for_thread = os.clone();
    let instance_id_for_thread = instance_id.clone();
    let tmux_location_for_thread = tmux_location.clone();
    let stopping = Arc::new(AtomicBool::new(false));
    let stopping_for_thread = stopping.clone();

    let listener_handle = thread::spawn(move || {
        let mut listener = reader;
        let mut idempotent_deliveries = idempotent_deliveries;
        let mut transport = transport;
        let mut connected_since = Instant::now();
        let mut failed_attempts = 0_u32;
        let mut last_heartbeat = Instant::now();
        let mut runtime_probe = match agent_type_for_thread.as_str() {
            "codex" => Some(RuntimeProbe::codex(agent_pid)),
            "claude" => claude_transcript_locator.map(RuntimeProbe::claude),
            _ => None,
        };
        let journal = journal;
        let mut relay = relay;

        'connection: while !stopping_for_thread.load(Ordering::SeqCst) {
            let mut line = String::new();
            match listener.read_line(&mut line) {
                Ok(0) => {
                    // Connexion fermée par le daemon
                    warn!("connexion fermée par le daemon");
                }
                Ok(_) => {
                    // Données reçues, traiter plus bas
                }
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) =>
                {
                    // Timeout de lecture - vérifier heartbeat et socket
                    if last_heartbeat.elapsed() >= HEARTBEAT_INTERVAL {
                        let heartbeat = encode(&WrapperToDaemon::Heartbeat).unwrap_or_default();
                        if let Some(writer) = writer_for_listener.lock().unwrap().as_mut() {
                            match writeln!(writer, "{}", heartbeat).and_then(|_| writer.flush()) {
                                Ok(_) => {
                                    last_heartbeat = Instant::now();
                                    debug!("heartbeat envoyé avec succès");
                                }
                                Err(e) => {
                                    error!("heartbeat échoué: {}", e);
                                    // Heartbeat échoué = probablement déconnecté
                                    warn!("Détection de déconnexion via heartbeat échoué");
                                }
                            }
                        } else {
                            error!("Impossible d'obtenir le writer pour heartbeat");
                        }
                    }

                    // Journal des tours humain/assistant : même source que la
                    // sonde (transcript de la session), même réveil.
                    if let Some(probe) = runtime_probe.as_mut() {
                        for (event, message_id, payload) in probe.drain_transcript_journal() {
                            if let Err(detail) =
                                journal.enqueue(event, message_id.as_deref(), payload)
                            {
                                warn!("journal transcript impossible: {detail}");
                            }
                        }
                    }

                    // Sonde de runtime : greffée sur le même réveil que le
                    // heartbeat, elle ne coûte qu'un `stat` la plupart du temps.
                    if let Some((observed, source)) = runtime_probe.as_mut().and_then(|probe| {
                        probe
                            .poll()
                            .map(|observation| (observation, probe.source()))
                    }) {
                        // Le nom est relu à chaque émission : `bridget rename`
                        // met à jour ce fichier, pas la variable capturée au
                        // démarrage. S'adresser au nom initial vaudrait un
                        // « agent introuvable » sur tout agent renommé.
                        let current_name = std::fs::read_to_string(&name_state_for_thread)
                            .map(|name| name.trim().to_string())
                            .ok()
                            .filter(|name| !name.is_empty())
                            .unwrap_or_else(|| my_name_for_thread.clone());
                        let message = WrapperToDaemon::Runtime {
                            agent: current_name,
                            model: observed.model.clone(),
                            effort: observed.effort.clone(),
                            source,
                        };
                        match encode(&message) {
                            Ok(json) => {
                                if let Some(writer) = writer_for_listener.lock().unwrap().as_mut() {
                                    if let Err(e) =
                                        writeln!(writer, "{}", json).and_then(|_| writer.flush())
                                    {
                                        debug!("sonde runtime : envoi impossible ({})", e);
                                    } else {
                                        debug!(
                                            "sonde runtime : modèle={} effort={:?}",
                                            observed.model, observed.effort
                                        );
                                    }
                                }
                            }
                            Err(e) => debug!("sonde runtime : encodage impossible ({})", e),
                        }
                    }

                    // Vérification proactive du socket (auto-reconnect)
                    // Vérifier toutes les X secondes si le socket existe toujours
                    if Instant::now()
                        .duration_since(last_heartbeat)
                        .as_secs()
                        .is_multiple_of(SOCKET_CHECK_INTERVAL.as_secs())
                    {
                        if socket_path().exists() {
                            debug!("Socket Bridget détecté - daemon probablement disponible");
                        } else {
                            warn!("⚠️ Socket Bridget absent - daemon probablement arrêté");
                        }
                    }

                    continue;
                }
                Err(e) => {
                    error!("Erreur de lecture: {}", e);
                    // Erreur de lecture = probablement déconnecté
                }
            }

            if line.is_empty() {
                // Détection de déconnexion
                if stopping_for_thread.load(Ordering::SeqCst) {
                    info!("Arrêt demandé, déconnexion propre");
                    break;
                }

                warn!(
                    "🔌 Connexion Bridget perdue pour « {} » - reconnexion automatique...",
                    my_name_for_thread
                );

                // Reset du compteur si la connexion était stable
                if connected_since.elapsed() >= RECONNECT_STABLE_RESET {
                    failed_attempts = 0;
                    info!("Connexion était stable - reset du compteur de tentatives");
                }

                // Boucle de reconnexion avec backoff exponentiel
                loop {
                    if stopping_for_thread.load(Ordering::SeqCst) {
                        info!("Arrêt demandé pendant reconnexion");
                        break 'connection;
                    }

                    let delay = reconnect_delay(failed_attempts);
                    failed_attempts = failed_attempts.saturating_add(1);

                    // Le nom choisi par l'utilisateur prime sur celui obtenu au
                    // démarrage : sans cela, un agent renommé revient sous son
                    // nom d'origine à chaque coupure.
                    let wanted_name =
                        resolve_current_name(&name_state_for_thread, &my_name_for_thread);

                    info!(
                        "🔄 Tentative de reconnexion {} pour « {} » (délai: {:.1}s)",
                        failed_attempts,
                        wanted_name,
                        delay.as_secs_f64()
                    );

                    thread::sleep(delay);
                    match connect_and_register(
                        &agent_type_for_thread,
                        Some(&wanted_name),
                        &host_for_thread,
                        interactive_protocol,
                        channel_for_thread.as_deref(),
                        interactive_mode,
                        tmux_location_for_thread.as_deref(),
                        &os_for_thread,
                        &instance_id_for_thread,
                        false,
                    ) {
                        Ok((new_reader, new_writer, registered_name)) => {
                            if registered_name != wanted_name {
                                warn!(
                                    "⚠️ Reconnexion refusée : nom inattendu « {} » (attendu: « {} »)",
                                    registered_name, wanted_name
                                );
                                continue;
                            }

                            // Reconnexion réussie ! 🎉
                            *writer_for_listener.lock().unwrap() = Some(new_writer);
                            listener = new_reader;
                            connected_since = Instant::now();
                            last_heartbeat = Instant::now();
                            failed_attempts = 0; // Reset du compteur
                            my_name_for_thread = registered_name;
                            // Register réinitialise l'attestation : le journal
                            // local reste actif, il faut le réannoncer.
                            relay.reset_generation();
                            send_wrapper_message(
                                &writer_for_listener,
                                WrapperToDaemon::JournalReady {
                                    receipt_probe: true,
                                },
                            );
                            // La présence runtime a disparu avec le daemon :
                            // republier modèle/effort même s'ils n'ont pas changé.
                            if let Some(probe) = runtime_probe.as_mut() {
                                probe.invalidate_after_reconnect();
                            }

                            info!(
                                "✅ Agent « {} » reconnecté au daemon avec succès !",
                                my_name_for_thread
                            );

                            notify_reconnected(&mut transport, &my_name_for_thread);
                            continue 'connection;
                        }
                        Err(error) => {
                            warn!(
                                "❌ Reconnexion échouée pour « {} » : {} (tentative {})",
                                wanted_name, error, failed_attempts
                            );
                        }
                    }
                }
            }
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let msg: DaemonToWrapper = match decode(line) {
                Ok(m) => m,
                Err(_) => {
                    warn!("message de protocole illisible");
                    continue;
                }
            };

            match msg {
                DaemonToWrapper::NativePermissionAcknowledged {
                    observation_id,
                    accepted,
                } => {
                    if let Some(sink) = &observer_acknowledgements {
                        crate::native_permission_observer::Observer::acknowledge(
                            sink,
                            observation_id,
                            accepted,
                        );
                    }
                }
                DaemonToWrapper::ProjectContextResult { .. } => {}
                DaemonToWrapper::Deliver { message: bm, .. } => {
                    info!(
                        "reçu de « {} »: {}",
                        bm.from,
                        bm.body.chars().take(60).collect::<String>()
                    );
                    record_interactive_turn(journal.as_ref(), &bm);
                    // Stocker le dernier expéditeur pour la commande reply
                    let name_for_reply = std::fs::read_to_string(&name_state_for_thread)
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    if bm.thread_notice.is_some() {
                        remember_thread_notice_context(&name_state_for_thread, &bm);
                    } else if bm.reply && !name_for_reply.is_empty() {
                        let reply_file = socket_path()
                            .parent()
                            .unwrap()
                            .join(format!("last-sender-{}", name_for_reply));
                        let value = if bm.reply {
                            format!("{}\t{}", bm.from, bm.id)
                        } else {
                            bm.from.clone()
                        };
                        let _ = std::fs::write(&reply_file, value);
                    }
                    if let Some(ref mut t) = transport {
                        if let Err(e) = t.deliver(&message_for_provider(&bm)) {
                            error!("injection tmux: {}", e);
                        }
                    } else {
                        warn!("livraison ignorée : aucun pane tmux pour {}", bm.id);
                    }
                }
                DaemonToWrapper::DeliveryReceiptQuery {
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                } => {
                    if let Some(report) = idempotent_deliveries.receipt_report(
                        delivery_id,
                        &recipient_instance_id,
                        delivery_generation,
                        unix_now_secs(),
                    ) {
                        send_wrapper_message(&writer_for_listener, report);
                    }
                }
                DaemonToWrapper::DeliverIdempotent {
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                    expires_at,
                    message,
                    execution: _,
                    ..
                } => {
                    let reports = deliver_idempotent_to_interactive(
                        &mut idempotent_deliveries,
                        delivery_id,
                        recipient_instance_id,
                        delivery_generation,
                        expires_at,
                        message,
                        unix_now_secs(),
                        |message| {
                            record_interactive_turn(journal.as_ref(), message);
                            remember_thread_notice_context(&name_state_for_thread, message);
                            match transport.as_mut() {
                                Some(transport) => transport
                                    .deliver(&message_for_provider(message))
                                    .map_err(|error| error.to_string()),
                                None => Err("aucun pane tmux pour la livraison idempotente".into()),
                            }
                        },
                    );
                    for report in reports {
                        send_wrapper_message(&writer_for_listener, report);
                    }
                }
                DaemonToWrapper::DelegatedRuntimeEvent { event } => {
                    let message = delegated_runtime_message(&event, &my_name_for_thread);
                    let injected = match transport.as_mut() {
                        Some(transport) => {
                            record_interactive_turn(journal.as_ref(), &message);
                            transport
                                .deliver(&message)
                                .map_err(|error| error.to_string())
                        }
                        None => Err("aucun pane tmux pour l'incident délégué".to_string()),
                    };
                    match injected {
                        Ok(()) => send_wrapper_message(
                            &writer_for_listener,
                            WrapperToDaemon::DelegatedRuntimeEventAcknowledged {
                                event_id: event.event_id,
                            },
                        ),
                        Err(error) => warn!("remise de l'incident délégué différée: {error}"),
                    }
                }

                DaemonToWrapper::Subscribe {
                    subscription_id,
                    window,
                    ..
                } => {
                    if let Err(reason) = relay.subscribe(subscription_id.clone(), window) {
                        send_wrapper_message(
                            &writer_for_listener,
                            WrapperToDaemon::AttachRejected {
                                subscription_id: Some(subscription_id),
                                reason,
                            },
                        );
                    }
                }
                DaemonToWrapper::Unsubscribe { subscription_id } => {
                    relay.unsubscribe(subscription_id);
                }
                DaemonToWrapper::Disconnect => {
                    info!("daemon déconnecté");
                    if connected_since.elapsed() >= RECONNECT_STABLE_RESET {
                        failed_attempts = 0;
                    }
                    loop {
                        if stopping_for_thread.load(Ordering::SeqCst) {
                            break 'connection;
                        }
                        let delay = reconnect_delay(failed_attempts);
                        failed_attempts = failed_attempts.saturating_add(1);
                        thread::sleep(delay);
                        let wanted_name =
                            resolve_current_name(&name_state_for_thread, &my_name_for_thread);
                        match connect_and_register(
                            &agent_type_for_thread,
                            Some(&wanted_name),
                            &host_for_thread,
                            interactive_protocol,
                            channel_for_thread.as_deref(),
                            interactive_mode,
                            tmux_location_for_thread.as_deref(),
                            &os_for_thread,
                            &instance_id_for_thread,
                            false,
                        ) {
                            Ok((new_reader, new_writer, registered_name))
                                if registered_name == wanted_name =>
                            {
                                *writer_for_listener.lock().unwrap() = Some(new_writer);
                                listener = new_reader;
                                connected_since = Instant::now();
                                last_heartbeat = Instant::now();
                                my_name_for_thread = registered_name;
                                relay.reset_generation();
                                send_wrapper_message(
                                    &writer_for_listener,
                                    WrapperToDaemon::JournalReady {
                                        receipt_probe: true,
                                    },
                                );
                                if let Some(probe) = runtime_probe.as_mut() {
                                    probe.invalidate_after_reconnect();
                                }
                                notify_reconnected(&mut transport, &my_name_for_thread);
                                continue 'connection;
                            }
                            Ok((_, _, registered_name)) => warn!(
                                "reconnexion refusée : nom inattendu « {} »",
                                registered_name
                            ),
                            Err(error) => warn!(
                                "reconnexion Bridget de « {} » impossible : {}",
                                wanted_name, error
                            ),
                        }
                    }
                }
                _ => {}
            }
        }
        relay.shutdown();
        journal.stop();
    });

    // 6. Attendre la fin de l'agent
    let status = child.wait()?;
    drop(permission_observer);
    // `ephemeral_mcp_config` est libéré ici. Le garde RAII couvre également
    // toutes les sorties anticipées précédentes.
    drop(ephemeral_mcp_config);

    // 7. Désenregistrement
    stopping.store(true, Ordering::SeqCst);
    {
        if let Ok(json) = encode(&WrapperToDaemon::Unregister)
            && let Ok(mut writer) = writer_clone.lock()
            && let Some(w) = writer.as_mut()
        {
            let _ = writeln!(w, "{}", json);
            let _ = w.flush();
        }
    }

    *writer.lock().unwrap() = None;
    let _ = listener_handle.join();

    if let Some(code) = status.code() {
        std::process::exit(code);
    }
    // Convention des shells : un fournisseur tué par un signal sort 128 + n,
    // jamais 0 ; l'humain et les scripts voient la fin anormale.
    let signal = std::os::unix::process::ExitStatusExt::signal(&status).unwrap_or(0);
    std::process::exit(128 + signal);
}

fn split_equipier_flag(agent_args: &[String]) -> (bool, Vec<String>) {
    let mut equipier = false;
    let mut remaining = Vec::with_capacity(agent_args.len());
    for argument in agent_args {
        if argument == "--equipier" {
            equipier = true;
        } else {
            remaining.push(argument.clone());
        }
    }
    (equipier, remaining)
}

enum AttachRelayCommand {
    Subscribe {
        subscription_id: String,
        window: AttachWindow,
    },
}

const ATTACH_RELAY_CONTROL_IDS_CAPACITY: usize = ATTACH_RELAY_COMMAND_CAPACITY * 2;

#[derive(Default)]
struct AttachRelayControlState {
    pending: HashSet<String>,
    active: HashSet<String>,
    cancelled: HashSet<String>,
}

struct RelaySubscription {
    subscription_id: String,
    window: AttachWindow,
    from_seq: Option<u64>,
    // Prochaine séquence attendue au rejeu disque, indépendante du dernier
    // fragment déjà émis : la lecture peut prendre de l'avance dans la file.
    expected_snapshot_seq: Option<u64>,
    files: Vec<PathBuf>,
    follow_after: Option<PathBuf>,
    file_index: usize,
    reader: Option<IncrementalJournalReader>,
    reader_source: Option<JournalSourceIdentity>,
    caught_up: bool,
    through_seq: Option<u64>,
    live_floor: Option<u64>,
    pending_events: VecDeque<(u64, Vec<u8>)>,
    pending_fragment: Option<(u64, Vec<u8>, usize)>,
}

impl RelaySubscription {
    fn new(
        subscription_id: String,
        window: AttachWindow,
        directory: &std::path::Path,
        host_today: &str,
        live_floor: Option<u64>,
    ) -> Result<Self, JournalWindowError> {
        let resolved = resolve_window(directory, &window, host_today)?;
        let follow_after = resolve_window(directory, &AttachWindow::Seq(0), host_today)?
            .files
            .last()
            .cloned();
        let live_floor = match (live_floor, resolved.from_seq) {
            (Some(floor), Some(from_seq)) => Some(floor.max(from_seq.saturating_sub(1))),
            (Some(floor), None) => Some(floor),
            (None, Some(from_seq)) => Some(from_seq.saturating_sub(1)),
            (None, None) => None,
        };
        Ok(Self {
            subscription_id,
            window,
            from_seq: resolved.from_seq,
            expected_snapshot_seq: resolved.from_seq.filter(|seq| *seq > 0),
            files: resolved.files,
            follow_after,
            file_index: 0,
            reader: None,
            reader_source: None,
            caught_up: false,
            through_seq: None,
            live_floor,
            pending_events: VecDeque::new(),
            pending_fragment: None,
        })
    }

    fn refresh_files(&mut self, directory: &std::path::Path, host_today: &str) {
        let Ok(resolved) = resolve_window(directory, &AttachWindow::Seq(0), host_today) else {
            return;
        };
        if self.caught_up {
            // Une fenêtre Today/Date borne le rejeu initial seulement. Une fois
            // le snapshot atteint, seul un fichier plus récent que le curseur
            // connu entre dans le suivi, jamais l'historique antérieur.
            for path in resolved.files {
                if self
                    .follow_after
                    .as_ref()
                    .is_none_or(|latest| path > *latest)
                    && !self.files.contains(&path)
                {
                    self.files.push(path);
                }
            }
        } else {
            // Tail a déjà fixé from_seq à l'ouverture. Ici seuls les
            // fichiers sont utilisés : Seq(0) donne la même liste, sans
            // relire et désérialiser tout l'historique à chaque tranche.
            let initial = if matches!(self.window, AttachWindow::Tail(_)) {
                resolved
            } else {
                let Ok(initial) = resolve_window(directory, &self.window, host_today) else {
                    return;
                };
                initial
            };
            for path in initial.files {
                if !self.files.contains(&path) {
                    self.files.push(path);
                }
            }
        }
    }

    fn next_reader(&mut self) -> Option<&mut IncrementalJournalReader> {
        if self.reader.is_none() {
            let path = self.files.get(self.file_index)?.clone();
            self.reader = Some(IncrementalJournalReader::new(path));
        }
        self.reader.as_mut()
    }

    fn has_pending_output(&self) -> bool {
        self.pending_fragment.is_some() || !self.pending_events.is_empty()
    }

    fn restart_from_seq(
        &mut self,
        directory: &std::path::Path,
        host_today: &str,
        from_seq: u64,
    ) -> Result<(), JournalWindowError> {
        let resolved = resolve_window(directory, &AttachWindow::Seq(from_seq), host_today)?;
        self.window = AttachWindow::Seq(from_seq);
        self.from_seq = Some(from_seq);
        self.expected_snapshot_seq = (from_seq > 0).then_some(from_seq);
        self.files = resolved.files;
        self.follow_after = self.files.last().cloned();
        self.file_index = 0;
        self.reader = None;
        self.reader_source = None;
        self.caught_up = false;
        self.pending_events.clear();
        self.pending_fragment = None;
        Ok(())
    }
}

#[derive(Default)]
struct LiveFanout {
    cursor: Option<u64>,
    pending_events: VecDeque<(u64, Vec<u8>)>,
    pending_fragment: Option<(u64, Vec<u8>, usize)>,
}

impl LiveFanout {
    fn reset_pending(&mut self) {
        self.pending_events.clear();
        self.pending_fragment = None;
    }
}

#[derive(Clone)]
struct AttachRelayHooks {
    before_command: Arc<dyn Fn() + Send + Sync>,
    before_read: Arc<dyn Fn() + Send + Sync>,
    before_live_read: Arc<dyn Fn() + Send + Sync>,
    control_observed: Arc<dyn Fn() + Send + Sync>,
}

impl Default for AttachRelayHooks {
    fn default() -> Self {
        Self {
            before_command: Arc::new(|| {}),
            before_read: Arc::new(|| {}),
            before_live_read: Arc::new(|| {}),
            control_observed: Arc::new(|| {}),
        }
    }
}

/// Port du relais existant, utilisable sans attendre l'actualisation du fil.
/// Les clones partagent les mêmes bornes, annulations et état de fermeture.
#[derive(Clone)]
pub(crate) struct AttachRelayControl {
    commands: mpsc::SyncSender<AttachRelayCommand>,
    control_state: Arc<Mutex<AttachRelayControlState>>,
    control_observed: Arc<dyn Fn() + Send + Sync>,
    wake: Arc<(Mutex<()>, Condvar)>,
}

impl AttachRelayControl {
    pub(crate) fn subscribe(
        &self,
        subscription_id: String,
        window: AttachWindow,
    ) -> Result<(), AttachRefusal> {
        {
            let mut state = self.control_state.lock().recover_poison();
            if state.pending.len().saturating_add(state.active.len())
                >= ATTACH_RELAY_CONTROL_IDS_CAPACITY
            {
                return Err(AttachRefusal::CommandQueueSaturated);
            }
            state.pending.insert(subscription_id.clone());
        }
        let result = self
            .commands
            .try_send(AttachRelayCommand::Subscribe {
                subscription_id: subscription_id.clone(),
                window,
            })
            .map_err(|error| {
                self.control_state
                    .lock()
                    .recover_poison()
                    .pending
                    .remove(&subscription_id);
                match error {
                    mpsc::TrySendError::Full(_) | mpsc::TrySendError::Disconnected(_) => {
                        AttachRefusal::CommandQueueSaturated
                    }
                }
            });
        self.wake.1.notify_all();
        result
    }

    pub(crate) fn unsubscribe(&self, subscription_id: String) {
        let mut state = self.control_state.lock().recover_poison();
        if state.pending.contains(&subscription_id) || state.active.contains(&subscription_id) {
            state.cancelled.insert(subscription_id);
            drop(state);
            (self.control_observed)();
            self.wake.1.notify_all();
        }
    }
}

/// Relais dédié du journal ACP : la boucle de lecture du wrapper ne fait que
/// déposer les commandes, tandis que ce worker traite chaque abonnement par
/// petites tranches. Le contrôle coalescé vit dans un état borné partagé afin
/// qu'un désabonnement ne soit jamais coincé derrière un rejeu volumineux.
pub(crate) struct AttachRelayWorker {
    commands: mpsc::SyncSender<AttachRelayCommand>,
    stopped: Arc<AtomicBool>,
    control_state: Arc<Mutex<AttachRelayControlState>>,
    control_observed: Arc<dyn Fn() + Send + Sync>,
    wake: Arc<(Mutex<()>, Condvar)>,
    generation: Arc<AtomicU64>,
    generation_ack: Arc<(Mutex<u64>, Condvar)>,
    worker: Option<thread::JoinHandle<()>>,
}

type RelayEmitter = Arc<dyn Fn(WrapperToDaemon) + Send + Sync>;
#[cfg(test)]
type RelayEvents = Arc<Mutex<Vec<WrapperToDaemon>>>;

impl AttachRelayWorker {
    pub(crate) fn start(
        directory: PathBuf,
        live_feed: JournalLiveFeed,
        emit: RelayEmitter,
    ) -> Self {
        Self::start_with_clock(
            directory,
            Arc::new(current_host_date),
            ATTACH_RELAY_COMMAND_CAPACITY,
            Some(live_feed),
            emit,
            AttachRelayHooks::default(),
        )
    }

    #[cfg(test)]
    fn start_with(
        directory: PathBuf,
        host_today: String,
        capacity: usize,
        emit: RelayEmitter,
        hooks: AttachRelayHooks,
    ) -> Self {
        let clock: Arc<dyn Fn() -> String + Send + Sync> = Arc::new(move || host_today.clone());
        Self::start_with_clock(directory, clock, capacity, None, emit, hooks)
    }

    #[cfg(test)]
    fn start_with_live(
        directory: PathBuf,
        host_today: String,
        capacity: usize,
        live_feed: JournalLiveFeed,
        emit: RelayEmitter,
        hooks: AttachRelayHooks,
    ) -> Self {
        let clock: Arc<dyn Fn() -> String + Send + Sync> = Arc::new(move || host_today.clone());
        Self::start_with_clock(directory, clock, capacity, Some(live_feed), emit, hooks)
    }

    fn start_with_clock(
        directory: PathBuf,
        host_today: Arc<dyn Fn() -> String + Send + Sync>,
        capacity: usize,
        live_feed: Option<JournalLiveFeed>,
        emit: RelayEmitter,
        hooks: AttachRelayHooks,
    ) -> Self {
        let (command_sender, command_receiver) = mpsc::sync_channel(capacity);
        let stopped = Arc::new(AtomicBool::new(false));
        let control_state = Arc::new(Mutex::new(AttachRelayControlState::default()));
        let wake = Arc::new((Mutex::new(()), Condvar::new()));
        let generation = Arc::new(AtomicU64::new(0));
        let generation_ack = Arc::new((Mutex::new(0), Condvar::new()));

        let worker_stopped = stopped.clone();
        let worker_control = control_state.clone();
        let worker_wake = wake.clone();
        let worker_generation = generation.clone();
        let worker_generation_ack = generation_ack.clone();
        let worker_emit = emit.clone();
        let worker = thread::spawn(move || {
            let mut subscriptions = BTreeMap::<String, RelaySubscription>::new();
            let mut live_fanout = LiveFanout::default();
            let mut had_live_subscriptions = false;
            let mut seen_generation = 0;
            while !worker_stopped.load(Ordering::SeqCst) {
                if let Some(feed) = &live_feed {
                    let (facts, dropped) = feed.take_observations();
                    if dropped > 0 {
                        worker_emit(WrapperToDaemon::ObservationGap { dropped });
                        log::warn!(
                            "observation_gap: {dropped} faits perdus par saturation du relais"
                        );
                    }
                    for fact in facts {
                        worker_emit(fact);
                    }
                }
                let current_generation = worker_generation.load(Ordering::SeqCst);
                if current_generation != seen_generation {
                    subscriptions.clear();
                    live_fanout = LiveFanout::default();
                    had_live_subscriptions = false;
                    worker_control.lock().recover_poison().active.clear();
                    seen_generation = current_generation;
                    let (ack, wake) = &*worker_generation_ack;
                    *ack.lock().recover_poison() = seen_generation;
                    wake.notify_all();
                    continue;
                }
                (hooks.before_command)();
                let command = if subscriptions.is_empty() {
                    match command_receiver.recv_timeout(ATTACH_RELAY_IDLE_WAIT) {
                        Ok(command) => Some(command),
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => None,
                    }
                } else {
                    match command_receiver.try_recv() {
                        Ok(command) => Some(command),
                        Err(mpsc::TryRecvError::Disconnected) => break,
                        Err(mpsc::TryRecvError::Empty) => None,
                    }
                };
                if let Some(AttachRelayCommand::Subscribe {
                    subscription_id,
                    window,
                }) = command
                {
                    let cancelled = {
                        let mut state = worker_control.lock().recover_poison();
                        state.pending.remove(&subscription_id);
                        let cancelled = state.cancelled.remove(&subscription_id);
                        if !cancelled {
                            state.active.insert(subscription_id.clone());
                        }
                        cancelled
                    };
                    if cancelled {
                        continue;
                    }
                    match RelaySubscription::new(
                        subscription_id.clone(),
                        window,
                        &directory,
                        &host_today(),
                        live_feed.as_ref().and_then(JournalLiveFeed::latest_seq),
                    ) {
                        Ok(subscription) => {
                            worker_emit(WrapperToDaemon::Subscribed {
                                subscription_id: subscription_id.clone(),
                            });
                            subscriptions.insert(subscription_id, subscription);
                        }
                        Err(error) => {
                            worker_control
                                .lock()
                                .recover_poison()
                                .active
                                .remove(&subscription_id);
                            worker_emit(WrapperToDaemon::AttachRejected {
                                subscription_id: Some(subscription_id),
                                reason: refusal_for_window(error),
                            });
                        }
                    }
                }

                // Aucune souscription => aucune lecture de fichier : le relais
                // ne perturbe jamais l'écrivain JSONL d'un équipier inobservé.
                let ids = subscriptions.keys().cloned().collect::<Vec<_>>();
                for subscription_id in ids {
                    if worker_control
                        .lock()
                        .recover_poison()
                        .cancelled
                        .remove(&subscription_id)
                    {
                        worker_control
                            .lock()
                            .recover_poison()
                            .active
                            .remove(&subscription_id);
                        subscriptions.remove(&subscription_id);
                        continue;
                    }
                    let Some(subscription) = subscriptions.get_mut(&subscription_id) else {
                        continue;
                    };
                    if emit_one_fragment(&worker_emit, subscription) {
                        continue;
                    }
                    if subscription.caught_up && live_feed.is_some() {
                        continue;
                    }
                    subscription.refresh_files(&directory, &host_today());
                    if subscription.file_index >= subscription.files.len() {
                        if !subscription.caught_up {
                            worker_emit(WrapperToDaemon::SnapshotCaughtUp {
                                subscription_id: subscription.subscription_id.clone(),
                                through_seq: subscription.through_seq,
                            });
                            subscription.caught_up = true;
                        }
                        continue;
                    }
                    (hooks.before_read)();
                    if worker_control
                        .lock()
                        .recover_poison()
                        .cancelled
                        .contains(&subscription_id)
                    {
                        continue;
                    }
                    let prior_offset = subscription
                        .reader
                        .as_ref()
                        .map_or(0, IncrementalJournalReader::next_offset);
                    let reader = subscription
                        .next_reader()
                        .expect("fichier de relais présent");
                    let (items, source) =
                        match reader.read_chunk_with_source(ATTACH_RELAY_READ_BYTES) {
                            Ok(result) => result,
                            Err(error) => {
                                // Libérer l'identifiant avant la fin visible : un
                                // désabonnement à réception de End ne laisse rien.
                                {
                                    let mut control = worker_control.lock().recover_poison();
                                    control.active.remove(&subscription_id);
                                    control.cancelled.remove(&subscription_id);
                                }
                                worker_emit(WrapperToDaemon::End {
                                    subscription_id: subscription.subscription_id.clone(),
                                    reason: format!("lecture du journal impossible: {error}"),
                                });
                                subscriptions.remove(&subscription_id);
                                continue;
                            }
                        };
                    let next_offset = reader.next_offset();
                    let source_replaced = source.is_some_and(|source| {
                        subscription
                            .reader_source
                            .is_some_and(|previous| !previous.same_file(source))
                    });
                    let source_truncated = source.is_some_and(|source| source.len < prior_offset);
                    let source_missing = source.is_none();
                    if source_replaced || source_truncated || source_missing {
                        let reason = if source_replaced {
                            "source_replaced"
                        } else if source_truncated {
                            "source_truncated"
                        } else {
                            "source de journal indisponible"
                        };
                        {
                            let mut control = worker_control.lock().recover_poison();
                            control.active.remove(&subscription_id);
                            control.cancelled.remove(&subscription_id);
                        }
                        worker_emit(WrapperToDaemon::End {
                            subscription_id: subscription.subscription_id.clone(),
                            reason: reason.to_string(),
                        });
                        subscriptions.remove(&subscription_id);
                        continue;
                    }
                    subscription.reader_source = source;
                    for item in items {
                        match item {
                            JournalReadItem::Event(event)
                                if subscription
                                    .from_seq
                                    .is_none_or(|from_seq| event.seq >= from_seq) =>
                            {
                                if let Some(expected) = subscription.expected_snapshot_seq {
                                    if event.seq < expected {
                                        worker_emit(WrapperToDaemon::JournalReadError {
                                            subscription_id: subscription.subscription_id.clone(),
                                            line: event.line,
                                            offset: event.offset,
                                            reason: "non_monotonic_sequence".to_string(),
                                        });
                                        continue;
                                    }
                                    if event.seq > expected {
                                        // Un curseur dont le fichier a été purgé n'est
                                        // pas rattrapé sans perte : annoncer le trou
                                        // avant de livrer le premier octet suivant.
                                        worker_emit(WrapperToDaemon::Gap {
                                            subscription_id: subscription.subscription_id.clone(),
                                            from_seq: expected,
                                            to_seq: event.seq - 1,
                                            reason: Some("journal_sequence_missing".to_string()),
                                        });
                                    }
                                }
                                subscription.expected_snapshot_seq = event.seq.checked_add(1);
                                subscription
                                    .pending_events
                                    .push_back((event.seq, event.bytes));
                            }
                            JournalReadItem::Oversized { seq: Some(seq), .. } => {
                                if subscription.from_seq.is_some_and(|from| seq < from)
                                    || subscription
                                        .expected_snapshot_seq
                                        .is_some_and(|next| seq < next)
                                {
                                    continue;
                                }
                                worker_emit(WrapperToDaemon::Gap {
                                    subscription_id: subscription.subscription_id.clone(),
                                    from_seq: subscription.expected_snapshot_seq.unwrap_or(seq),
                                    to_seq: seq,
                                    reason: Some("event_too_large".to_string()),
                                });
                                subscription.expected_snapshot_seq = seq.checked_add(1);
                            }
                            JournalReadItem::Oversized {
                                seq: None,
                                offset,
                                line,
                            } => worker_emit(WrapperToDaemon::JournalReadError {
                                subscription_id: subscription.subscription_id.clone(),
                                line,
                                offset,
                                reason: "event_too_large sans séquence".to_string(),
                            }),
                            JournalReadItem::Unreadable(line) => {
                                worker_emit(WrapperToDaemon::JournalReadError {
                                    subscription_id: subscription.subscription_id.clone(),
                                    line: line.line,
                                    offset: line.offset,
                                    reason: "ligne de journal illisible".to_string(),
                                })
                            }
                            JournalReadItem::Event(_) => {}
                        }
                    }
                    let file_finished = source.is_some_and(|source| next_offset >= source.len);
                    if file_finished
                        && subscription.file_index.saturating_add(1) < subscription.files.len()
                    {
                        subscription.reader = None;
                        subscription.reader_source = None;
                        subscription.file_index = subscription.file_index.saturating_add(1);
                    } else if file_finished
                        && !subscription.caught_up
                        && !subscription.has_pending_output()
                    {
                        worker_emit(WrapperToDaemon::SnapshotCaughtUp {
                            subscription_id: subscription.subscription_id.clone(),
                            through_seq: subscription.through_seq,
                        });
                        subscription.caught_up = true;
                    }
                }
                if let Some(feed) = &live_feed {
                    let live_ids = subscriptions
                        .iter()
                        .filter(|(_, subscription)| subscription.caught_up)
                        .map(|(id, _)| id.clone())
                        .collect::<Vec<_>>();
                    if live_ids.is_empty() {
                        had_live_subscriptions = false;
                        live_fanout.reset_pending();
                    } else {
                        if !had_live_subscriptions {
                            live_fanout.cursor = live_ids
                                .iter()
                                .filter_map(|id| subscriptions.get(id))
                                .filter_map(|subscription| {
                                    match (subscription.through_seq, subscription.live_floor) {
                                        (Some(through), Some(floor)) => Some(through.max(floor)),
                                        (through, floor) => through.or(floor),
                                    }
                                })
                                .max();
                            live_fanout.reset_pending();
                            had_live_subscriptions = true;
                        }
                        if live_fanout.pending_fragment.is_none()
                            && live_fanout.pending_events.is_empty()
                        {
                            if worker_stopped.load(Ordering::SeqCst) {
                                break;
                            }
                            (hooks.before_live_read)();
                            let batch = feed.after(live_fanout.cursor);
                            if let Some((from_seq, to_seq)) = batch.gap {
                                for subscription_id in live_ids {
                                    worker_emit(WrapperToDaemon::Gap {
                                        subscription_id: subscription_id.clone(),
                                        from_seq,
                                        to_seq,
                                        reason: Some("live_feed_overrun".to_string()),
                                    });
                                    if let Some(subscription) =
                                        subscriptions.get_mut(&subscription_id)
                                    {
                                        let _ = subscription.restart_from_seq(
                                            &directory,
                                            &host_today(),
                                            from_seq,
                                        );
                                    }
                                }
                                had_live_subscriptions = false;
                                live_fanout.reset_pending();
                            } else {
                                for event in batch.events {
                                    live_fanout.cursor = Some(event.seq);
                                    live_fanout
                                        .pending_events
                                        .push_back((event.seq, event.bytes));
                                }
                            }
                        }
                        while emit_one_live_fragment(&worker_emit, &mut live_fanout) {}
                    }
                }
                if subscriptions.values().all(|subscription| {
                    subscription.caught_up && !subscription.has_pending_output()
                }) {
                    let (lock, wake) = &*worker_wake;
                    let guard = lock.lock().recover_poison();
                    let _ = wake
                        .wait_timeout(guard, ATTACH_RELAY_IDLE_WAIT)
                        .recover_poison();
                }
            }
        });

        Self {
            commands: command_sender,
            stopped,
            control_state,
            control_observed: hooks.control_observed.clone(),
            wake,
            generation,
            generation_ack,
            worker: Some(worker),
        }
    }

    /// Même file et même état bornés ; aucun second relais.
    pub(crate) fn subscription_control(&self) -> AttachRelayControl {
        AttachRelayControl {
            commands: self.commands.clone(),
            control_state: self.control_state.clone(),
            control_observed: self.control_observed.clone(),
            wake: self.wake.clone(),
        }
    }

    pub(crate) fn subscribe(
        &self,
        subscription_id: String,
        window: AttachWindow,
    ) -> Result<(), AttachRefusal> {
        self.subscription_control()
            .subscribe(subscription_id, window)
    }

    pub(crate) fn unsubscribe(&self, subscription_id: String) {
        self.subscription_control().unsubscribe(subscription_id)
    }

    pub(crate) fn reset_generation(&self) {
        let mut state = self.control_state.lock().recover_poison();
        let pending = state.pending.iter().cloned().collect::<Vec<_>>();
        state.cancelled.extend(pending);
        state.active.clear();
        drop(state);
        self.wake.1.notify_all();
        let generation = self
            .generation
            .fetch_add(1, Ordering::SeqCst)
            .saturating_add(1);
        let (ack, wake) = &*self.generation_ack;
        let mut acknowledged = ack.lock().recover_poison();
        while *acknowledged < generation {
            acknowledged = wake.wait(acknowledged).recover_poison();
        }
    }

    pub(crate) fn shutdown(&mut self) {
        if self.stopped.swap(true, Ordering::SeqCst) {
            return;
        }
        // Le Stop n'emprunte jamais une file : le worker observe ce drapeau
        // avant chaque commande, même sous une rafale de souscriptions.
        self.wake.1.notify_all();
        let Some(worker) = self.worker.take() else {
            return;
        };
        let (done_tx, done_rx) = mpsc::channel();
        // Join dans un fil annexe : si le worker est coincé (hook de test,
        // wait non coopératif), on rend la main à l'appelant dans la borne
        // plutôt que de retenir le process entier.
        thread::spawn(move || {
            let _ = worker.join();
            let _ = done_tx.send(());
        });
        match done_rx.recv_timeout(ATTACH_RELAY_SHUTDOWN_BOUND) {
            Ok(()) => {}
            Err(mpsc::RecvTimeoutError::Timeout) => {
                warn!(
                    "AttachRelayWorker::shutdown: worker encore bloqué après {:?}",
                    ATTACH_RELAY_SHUTDOWN_BOUND
                );
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {}
        }
    }
}

impl Drop for AttachRelayWorker {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn refusal_for_window(error: JournalWindowError) -> AttachRefusal {
    match error {
        JournalWindowError::InvalidDate => AttachRefusal::InvalidDate,
        JournalWindowError::FutureDate => AttachRefusal::FutureDate,
        JournalWindowError::DateOutsideRetention => AttachRefusal::DateOutsideRetention,
    }
}

fn emit_one_fragment(emit: &RelayEmitter, subscription: &mut RelaySubscription) -> bool {
    if subscription.pending_fragment.is_none()
        && let Some((seq, bytes)) = subscription.pending_events.pop_front()
    {
        subscription.pending_fragment = Some((seq, bytes, 0));
    }
    let Some((seq, bytes, offset)) = subscription.pending_fragment.as_ref() else {
        return false;
    };
    let seq = *seq;
    let offset = *offset;
    let remaining = bytes.len().saturating_sub(offset);
    if remaining == 0 {
        subscription.pending_fragment = None;
        subscription.through_seq = Some(seq);
        return false;
    }
    let mut low = 1_usize;
    let mut high = remaining.min(MAX_ATTACH_FRAGMENT_BYTES);
    while low < high {
        let candidate = (low + high).div_ceil(2);
        let frame = WrapperToDaemon::JournalFragment {
            subscription_id: subscription.subscription_id.clone(),
            seq,
            offset: offset as u64,
            final_fragment: candidate == remaining,
            bytes: bytes[offset..offset + candidate].to_vec(),
        };
        if encode(&frame).is_ok_and(|json| json.len() <= MAX_ATTACH_SERIALIZED_FRAME_BYTES) {
            low = candidate;
        } else {
            high = candidate.saturating_sub(1);
        }
    }
    let length = low;
    let final_fragment = length == remaining;
    emit(WrapperToDaemon::JournalFragment {
        subscription_id: subscription.subscription_id.clone(),
        seq,
        offset: offset as u64,
        final_fragment,
        bytes: bytes[offset..offset + length].to_vec(),
    });
    if final_fragment {
        subscription.pending_fragment = None;
        subscription.through_seq = Some(seq);
    } else if let Some((_, _, next_offset)) = subscription.pending_fragment.as_mut() {
        *next_offset = offset.saturating_add(length);
    }
    true
}

fn emit_one_live_fragment(emit: &RelayEmitter, fanout: &mut LiveFanout) -> bool {
    if fanout.pending_fragment.is_none()
        && let Some((seq, bytes)) = fanout.pending_events.pop_front()
    {
        fanout.pending_fragment = Some((seq, bytes, 0));
    }
    let Some((seq, bytes, offset)) = fanout.pending_fragment.as_ref() else {
        return false;
    };
    let seq = *seq;
    let offset = *offset;
    let remaining = bytes.len().saturating_sub(offset);
    if remaining == 0 {
        fanout.pending_fragment = None;
        return false;
    }
    let mut low = 1_usize;
    let mut high = remaining.min(MAX_ATTACH_FRAGMENT_BYTES);
    while low < high {
        let candidate = (low + high).div_ceil(2);
        let frame = WrapperToDaemon::LiveJournalFragment {
            seq,
            offset: offset as u64,
            final_fragment: candidate == remaining,
            bytes: bytes[offset..offset + candidate].to_vec(),
        };
        if encode(&frame).is_ok_and(|json| json.len() <= MAX_ATTACH_SERIALIZED_FRAME_BYTES) {
            low = candidate;
        } else {
            high = candidate.saturating_sub(1);
        }
    }
    let length = low;
    let final_fragment = length == remaining;
    emit(WrapperToDaemon::LiveJournalFragment {
        seq,
        offset: offset as u64,
        final_fragment,
        bytes: bytes[offset..offset + length].to_vec(),
    });
    if final_fragment {
        fanout.pending_fragment = None;
    } else if let Some((_, _, next_offset)) = fanout.pending_fragment.as_mut() {
        *next_offset = offset.saturating_add(length);
    }
    true
}

fn profile_ledger_path(socket: &Path) -> Option<PathBuf> {
    // Le worker conserve reçus et journal, jamais une base métier du maître.
    // La politique reçue du maître et les messages IPC restent inchangés.
    std::env::var_os(REMOTE_RESULT_CONTEXT_ENV)
        .is_none()
        .then(|| socket.with_extension("db"))
}

fn apply_pending_profile_instructions(
    socket: &Path,
    transport: &mut dyn ManagedSession,
    recipient: &str,
    provider_spawn_id: &str,
) {
    let Some(profile_path) = profile_ledger_path(socket) else {
        return;
    };
    let mut store = match AgentProfileStore::open(&profile_path) {
        Ok(store) => store,
        Err(error) => {
            warn!("profil agent indisponible avant démarrage fournisseur: {error}");
            return;
        }
    };
    if let Err(error) = store.ensure_agent_ids([recipient]) {
        warn!("identité agent indisponible avant démarrage fournisseur: {error}");
        return;
    }
    let pending = match store.pending_instructions_for_agent_id(recipient) {
        Ok(Some(pending)) if !pending.instructions.is_empty() => pending,
        Ok(_) => return,
        Err(error) => {
            warn!("lecture consigne agent impossible avant démarrage fournisseur: {error}");
            return;
        }
    };
    let outcome = match transport.set_private_profile_instructions(&pending.instructions) {
        Ok(()) => (InstructionStatus::Applied, None),
        Err(_) => (
            InstructionStatus::Unsupported,
            Some("private_context_unsupported"),
        ),
    };
    if let Err(error) = store.mark_instruction_application(
        &pending.agent_id,
        pending.revision,
        provider_spawn_id,
        outcome.0,
        outcome.1,
    ) {
        warn!("état application consigne agent impossible: {error}");
    }
}

fn launch_acp(
    agent_type: &str,
    agent_args: &[String],
    explicit_name: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let registry = crate::registry::AgentRegistry::load()?;
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or("HOME absent pour le journal de session ACP")?;
    launch_acp_with_status(
        agent_type,
        agent_args,
        explicit_name,
        &registry,
        &socket_path(),
        &home,
        None,
        None,
    )
}

fn attention_occurrence_key(kind: &str, agent: &str, subject: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"bridget/attention/v1\0");
    hasher.update(kind.as_bytes());
    hasher.update(b"\0");
    hasher.update(agent.as_bytes());
    hasher.update(b"\0");
    hasher.update(subject.as_bytes());
    format!("attention:v1:{kind}:{:x}", hasher.finalize())
}

fn record_attention(
    profile_db_path: Option<&Path>,
    agent_id: &str,
    event_type: AttentionEventType,
    subject: &str,
) {
    let Some(profile_db_path) = profile_db_path else {
        return;
    };
    let Ok(mut store) = AgentProfileStore::open(profile_db_path) else {
        warn!("attention non persistée: store indisponible");
        return;
    };
    let occurrence_key = attention_occurrence_key(event_type.as_str(), agent_id, subject);
    if store
        .record_attention_for_agent_id(agent_id, event_type, &occurrence_key)
        .is_err()
    {
        warn!("attention non persistée: écriture refusée");
    }
}

/// Point d'entrée du wrapper supervisé. Le FD `managed-status` est fermé
/// uniquement après Register, transport ACP, journal et relais initialisés.
#[allow(clippy::too_many_arguments)]
// 12 paramètres : regroupement différé (programme 260, lot 9).
fn spawn_managed_session_transport(
    agent_type: &str,
    definition: &crate::registry::AgentDefinition,
    native_args: &[String],
    mcp_environment: &[(OsString, OsString)],
    mcp_servers: Vec<serde_json::Value>,
    inherit_stderr: bool,
    _home: &Path,
    agent_name: Option<String>,
    instance_id: &str,
    socket: &Path,
    repair: Option<(&serde_json::Value, &str)>,
) -> Result<Box<dyn ManagedSession>, Box<dyn std::error::Error>> {
    if let Some(policy) = repair
        .map(|v| Ok(Some(v.0.clone())))
        .unwrap_or_else(|| {
            std::env::var(crate::native_permissions::CHILD_POLICY_ENV)
                .ok()
                .map(|raw| serde_json::from_str::<serde_json::Value>(&raw))
                .transpose()
        })
        .map_err(|_| "permission_attestation_unavailable")?
        && let Some(context) = policy.get("launch_context")
    {
        crate::native_permissions::recheck_context(
            context,
            definition,
            &std::env::current_dir()?,
            &crate::lifecycle::source_environment(),
        )?;
    }
    match definition.protocol.as_str() {
        "acp" => {
            let options = AcpOptions {
                command: definition.command.clone(),
                args: native_args.to_vec(),
                queue_capacity: definition.queue_capacity,
                permissions: definition.permissions.clone(),
                notify_timeout_secs: definition.notify_timeout_secs,
                provider_observation: definition.capabilities.observed.clone(),
            };
            if inherit_stderr {
                Ok(Box::new(
                    AcpTransport::spawn_inheriting_stderr_with_environment_and_mcp(
                        options,
                        mcp_environment,
                        mcp_servers,
                    )?,
                ))
            } else {
                Ok(Box::new(AcpTransport::spawn_with_environment_and_mcp(
                    options,
                    mcp_environment,
                    mcp_servers,
                )?))
            }
        }
        "claude_stream_json" => {
            let options = ClaudeStreamJsonOptions {
                command: definition.command.clone(),
                args: native_args.to_vec(),
                provider_kind: agent_type.to_string(),
                queue_capacity: definition.queue_capacity,
                notify_timeout_secs: definition.notify_timeout_secs,
                provider_observation: definition.capabilities.observed.clone(),
                // Même arbre que le journal d'agent : survit à la mort du
                // managed-wrapper et au redémarrage du daemon (lot cursor2).
                session_store_root: Some(
                    socket
                        .parent()
                        .ok_or("socket sans racine")?
                        .join("sessions"),
                ),
                agent_name,
            };
            let environment = string_environment(mcp_environment);
            if inherit_stderr {
                Ok(Box::new(
                    ClaudeStreamJsonTransport::spawn_inheriting_stderr_with_environment(
                        options,
                        &environment,
                    )?,
                ))
            } else {
                Ok(Box::new(ClaudeStreamJsonTransport::spawn_with_environment(
                    options,
                    &environment,
                    false,
                )?))
            }
        }
        "codex_app_server" => {
            let dynamic_tool_handler = agent_name.clone().map(|identity| {
                let instance_id = instance_id.to_string();
                let socket = socket.to_path_buf();
                Arc::new(move |name: &str, arguments: &serde_json::Value| {
                    crate::mcp::execute_dynamic_tool_at_with_scope(
                        &identity,
                        &instance_id,
                        name,
                        arguments,
                        &socket,
                    )
                }) as bridget_transport::codex_app_server::DynamicToolHandler
            });
            let options = CodexAppServerOptions {
                child_requests: std::env::var(NATIVE_CHILD_REQUESTS_ENV).as_deref()
                    == Ok("forward"),
                command: definition.command.clone(),
                args: native_args.to_vec(),
                queue_capacity: definition.queue_capacity,
                notify_timeout_secs: definition.notify_timeout_secs,
                model: codex_model_from_args(&definition.args),
                permissions: definition.permissions.clone(),
                provider_observation: crate::provider_program_compat::runtime_observation(
                    definition,
                    &crate::lifecycle::source_environment(),
                )?,
                thread_bootstrap: repair.map_or(Default::default(), |v| {
                    bridget_transport::codex_app_server::CodexThreadBootstrap::Resume {
                        thread_id: v.1.into(),
                    }
                }),
                dynamic_tool_handler,
            };
            let environment = string_environment(mcp_environment);
            if repair.is_some() {
                return Ok(Box::new(
                    CodexAppServerTransport::spawn_repair_with_environment(
                        options,
                        &environment,
                        inherit_stderr,
                    )?,
                ));
            }
            if inherit_stderr {
                Ok(Box::new(
                    CodexAppServerTransport::spawn_inheriting_stderr_with_environment(
                        options,
                        &environment,
                    )?,
                ))
            } else {
                Ok(Box::new(CodexAppServerTransport::spawn_with_environment(
                    options,
                    &environment,
                    false,
                )?))
            }
        }
        other => Err(format!("protocole géré inconnu: {other}").into()),
    }
}

pub fn launch_managed_acp(
    agent_type: &str,
    explicit_name: &str,
    resolved_definition: &bridget_transport::ResolvedAgentDefinition,
) -> Result<(), Box<dyn std::error::Error>> {
    crate::environment::Namespace::from_environment()?;
    let mut reporter = crate::managed_process::ManagedStatusReporter::from_environment()?
        .ok_or("canal managed-status absent du wrapper supervisé")?;
    let mut managed_command = None;
    let result = (|| {
        let registry =
            crate::registry::AgentRegistry::from_resolved(agent_type, resolved_definition)?;
        managed_command = Some(registry.get(agent_type)?.command.clone());
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or("HOME absent pour le journal de session ACP")?;
        launch_acp_with_status(
            agent_type,
            &[],
            Some(explicit_name),
            &registry,
            &socket_path(),
            &home,
            Some(&mut reporter),
            Some(&resolved_definition.digest),
        )
    })();
    if let Err(error) = &result {
        let detail = error.to_string();
        let kind = if detail.contains("impossible de lancer l'adaptateur ACP")
            && (detail.contains("os error 2") || detail.contains("No such file"))
        {
            "command_missing"
        } else {
            "negotiation_failed"
        };
        let reason = if kind == "command_missing" {
            managed_command.unwrap_or(detail)
        } else {
            detail
        };
        let _ = reporter.startup_failed(kind, reason);
    }
    result
}

/// Lance un équipier ACP avec ses dépendances de configuration et de chemins
/// explicites. Le flux de production passe par [`launch_acp`]; cette variante
/// rend le même chemin vérifiable avec un registre et un daemon temporaires.
pub fn launch_acp_with(
    agent_type: &str,
    agent_args: &[String],
    explicit_name: Option<&str>,
    registry: &crate::registry::AgentRegistry,
    socket: &std::path::Path,
    home: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    launch_acp_with_status(
        agent_type,
        agent_args,
        explicit_name,
        registry,
        socket,
        home,
        None,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn launch_acp_with_status(
    agent_type: &str,
    agent_args: &[String],
    explicit_name: Option<&str>,
    registry: &crate::registry::AgentRegistry,
    socket: &std::path::Path,
    home: &std::path::Path,
    managed_reporter: Option<&mut crate::managed_process::ManagedStatusReporter>,
    frozen_definition_digest: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    launch_session_with_status(
        agent_type,
        agent_args,
        explicit_name,
        registry,
        socket,
        home,
        managed_reporter,
        frozen_definition_digest,
        None,
    )
}

#[allow(clippy::too_many_arguments)]
fn launch_session_with_status(
    agent_type: &str,
    agent_args: &[String],
    explicit_name: Option<&str>,
    registry: &crate::registry::AgentRegistry,
    socket: &Path,
    home: &Path,
    mut managed_reporter: Option<&mut crate::managed_process::ManagedStatusReporter>,
    frozen_definition_digest: Option<&str>,
    mut interactive: Option<crate::codex_interactive::Launch>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Les API injectables ne peuvent envoyer le wrapper sur un socket et ses
    // outils MCP sur un autre. Refus avant création d'état ou fournisseur.
    let configured_namespace = crate::environment::Namespace::from_environment()?;
    let session_namespace = crate::environment::Namespace::resolve(
        Some(socket.parent().ok_or("socket sans racine")?.to_path_buf()),
        Some(socket.to_path_buf()),
        Some(home.to_path_buf()),
    )?;
    if session_namespace != configured_namespace {
        return Err(
            "namespace de session différent du namespace de processus : lancement refusé".into(),
        );
    }
    let state_root = session_namespace.root;
    crate::environment::ensure_private_directory(&state_root)?;
    let mut journal_root = state_root.join("sessions");
    if !agent_args.is_empty() {
        return Err("le mode --equipier n'accepte pas d'arguments d'agent".into());
    }
    let definition = registry.get(agent_type)?;
    let inherited_policy: Option<serde_json::Value> =
        std::env::var(crate::native_permissions::CHILD_POLICY_ENV)
            .ok()
            .map(|raw| serde_json::from_str(&raw))
            .transpose()
            .map_err(|_| "permission_attestation_unavailable")?;
    if let Some(policy) = &inherited_policy
        && let Some(context) = policy.get("launch_context")
    {
        crate::native_permissions::recheck_context(
            context,
            definition,
            &std::env::current_dir()?,
            &crate::lifecycle::source_environment(),
        )?;
    }
    if interactive.is_some() && definition.protocol != "codex_app_server" {
        return Err("Codex interactif requiert le pilote natif codex_app_server".into());
    }
    if !matches!(
        definition.protocol.as_str(),
        "acp" | "claude_stream_json" | "codex_app_server"
    ) {
        return Err(
            format!("le type '{agent_type}' n'utilise pas une session gérée supportée").into(),
        );
    }
    // Même garde que le spawn géré (lifecycle) : --equipier ne doit pas
    // contourner le retrait G10 du pont Zed.
    if let Err(refusal) = crate::registry::reject_retired_zed_bridge(definition) {
        let detail = match refusal {
            bridget_transport::SpawnRefusal::EnvUnfit { detail } => detail,
            other => format!("{other:?}"),
        };
        return Err(detail.into());
    }
    if let Some(variable) = crate::registry::forbidden_environment_variable(
        definition,
        crate::registry::allow_api_key_value(
            std::env::var("BRIDGET_ALLOW_API_KEY").ok().as_deref(),
        ),
        |name| std::env::var_os(name).is_some(),
    ) {
        return Err(billing_guard_error(&variable).into());
    }

    let mut effective_name = match interactive.as_ref() {
        Some(launch) => resolve_interactive_identity(launch, explicit_name, socket)?,
        None => explicit_name.map(str::to_owned),
    };
    let host = host_name();
    let os = operating_system();
    let instance_id = managed_reporter
        .as_ref()
        .map(|reporter| reporter.instance_id().to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let native_bootstrap = if managed_reporter.is_some() && interactive.is_none() {
        std::env::var(NATIVE_MISSION_BOOTSTRAP_ENV)
            .ok()
            .and_then(|raw| serde_json::from_str::<NativeMissionBootstrap>(&raw).ok())
            .filter(|bootstrap| {
                bootstrap.instance_id == instance_id
                    && uuid::Uuid::parse_str(&bootstrap.mission_id).is_ok()
            })
    } else {
        None
    };
    let remote_result_context = match std::env::var(REMOTE_RESULT_CONTEXT_ENV) {
        Ok(raw) => Some(
            serde_json::from_str::<WorkerResultContext>(&raw)
                .map_err(|_| "remote_result_context_invalid")?,
        ),
        Err(std::env::VarError::NotPresent) => None,
        Err(_) => return Err("remote_result_context_invalid".into()),
    };
    if let Some(context) = &remote_result_context {
        context.validate_for_instance(&instance_id)?;
        if native_bootstrap.is_none()
            || interactive.is_some()
            || !matches!(
                definition.protocol.as_str(),
                "codex_app_server" | "claude_stream_json"
            )
        {
            return Err("remote_result_context_invalid".into());
        }
        journal_root = context.journal_root();
        crate::environment::ensure_private_directory(&journal_root)?;
    }
    let native_bootstrap_for_signal = native_bootstrap.as_ref();
    // Fixture de crash : SIGTERM doit terminer le wrapper bloqué sur la FIFO.
    // Comme les handlers daemon cfg(test), aucun handler coopératif n'est installé.
    #[cfg(feature = "test-support")]
    let native_bootstrap_for_signal = native_bootstrap_for_signal.filter(|_| {
        !(std::env::var_os(crate::test_sync::DIRECTORY_ENV).is_some()
            && std::env::var("BRIDGET_T010_DEFAULT_TERM").ok().as_deref() == Some("1"))
    });
    let native_termination = native_bootstrap_for_signal
        .map(|_| NativeTerminationSignal::new())
        .transpose()?;
    #[cfg(target_os = "linux")]
    let remote_drain = remote_result_context
        .as_ref()
        .map(RemoteDrainGuard::new)
        .transpose()?;
    let name_state_path = instance_name_state_path(socket, &instance_id);
    if let Some(parent) = name_state_path.parent() {
        crate::environment::ensure_private_directory(parent)?;
    }
    crate::environment::validate_state_file(&name_state_path, false)?;
    // Indexé par instance (pas par nom) : un rename ne déplace pas le chemin et
    // un concurrent ne peut pas usurper le fichier d'un autre équipier.
    std::fs::write(
        &name_state_path,
        effective_name.as_deref().unwrap_or_default().as_bytes(),
    )?;
    let mcp_environment = managed_adapter_environment(
        &instance_id,
        effective_name.as_deref(),
        Some(&name_state_path),
    )?;
    // Le masquage de sortie demeure disponible pour les adaptateurs, mais
    // aucune valeur de secret n'est désormais lue depuis un conteneur.
    let mut redaction_lease = None;
    let mcp_servers: Vec<serde_json::Value> = definition
        .mcp
        .acp_session
        .then(mcp_server_entry)
        .transpose()?
        .into_iter()
        .collect();
    // La définition gérée peut contenir un bypass. Une session humaine part
    // de SES réglages Codex et des seuls arguments explicitement demandés.
    let mut native_args = interactive.as_ref().map_or_else(
        || definition.args.clone(),
        |launch| launch.server_args.clone(),
    );
    let _ephemeral_mcp_config = apply_managed_mcp(
        definition.protocol.as_str(),
        definition.mcp.interactive.as_str(),
        &mut native_args,
        &instance_id,
        socket,
    )?;
    // Même famille que le chemin interactif (`launch` L1545) : sans cette
    // application, managed-wrapper ignore permissions et n'injecte aucun
    // drapeau de contournement — la question d'autorisation arrive alors
    // alors que le réglage allow aurait dû l'éviter.
    if interactive.is_none() && inherited_policy.is_none() {
        apply_managed_permission_policy(
            definition.protocol.as_str(),
            definition.permissions.as_str(),
            &mut native_args,
        );
    }
    // Capture the exact provider inputs before its first process is created.
    let mut permission_launch = definition.clone();
    permission_launch.args = native_args.clone();
    let mut permission_cwd = std::env::current_dir()?;
    let permission_environment = crate::lifecycle::source_environment();
    let captured_context = if definition.protocol == "claude_stream_json" {
        crate::native_permissions::capture_context_with_env(
            &permission_launch,
            &permission_cwd,
            &permission_environment,
        )
        .map(Some)
    } else {
        Ok(None)
    };
    let mut native_launch_refusal = captured_context.as_ref().err().cloned();
    let mut native_launch_context = captured_context.ok().flatten();
    let inherit_stderr = managed_reporter.is_some();
    let codex_endpoint = interactive
        .as_ref()
        .map(|_| CodexInteractiveEndpoint::new())
        .transpose()?;
    let codex_socket = codex_endpoint
        .as_ref()
        .map(|endpoint| endpoint.socket.clone())
        .unwrap_or_else(|| state_root.join(format!("c-{}.sock", &instance_id[..12])));
    let mut tui_binding = None;
    if native_termination
        .as_ref()
        .is_some_and(NativeTerminationSignal::requested)
    {
        return Err("mission native arrêtée avant le lancement du fournisseur".into());
    }
    let mut transport: Box<dyn ManagedSession> = if let Some(launch) = interactive.as_mut() {
        let select = |threads: &[bridget_transport::codex_app_server::CodexThreadSummary]| {
            launch.select_thread(threads)
        };
        let selector = launch.needs_selection().then_some(
            &select as bridget_transport::codex_app_server::InteractiveThreadSelector<'_>,
        );
        let native = bridget_transport::CodexAppServerTransport::spawn_interactive_selecting(
            bridget_transport::CodexAppServerOptions {
                child_requests: false,
                command: definition.command.clone(),
                args: native_args.clone(),
                queue_capacity: definition.queue_capacity,
                notify_timeout_secs: definition.notify_timeout_secs,
                model: launch.model.clone(),
                permissions: "interactive".into(),
                provider_observation: None,
                thread_bootstrap: launch.resume_thread.as_ref().map_or(
                    bridget_transport::codex_app_server::CodexThreadBootstrap::Start,
                    |thread_id| bridget_transport::codex_app_server::CodexThreadBootstrap::Resume {
                        thread_id: thread_id.clone(),
                    },
                ),
                dynamic_tool_handler: None,
            },
            &string_environment(&mcp_environment),
            &codex_socket,
            selector,
        )?;
        if launch.resume_thread.is_some() {
            // Le fournisseur a maintenant résolu le nom/le choix en UUID.
            // Vérifier sa liaison AVANT Register, pas après avoir écrasé un nom.
            launch.resume_thread = Some(native.thread_id().to_owned());
            effective_name = resolve_interactive_identity(launch, explicit_name, socket)?;
        }
        tui_binding = Some((
            native.thread_id().to_string(),
            native.interactive_lifetime(),
        ));
        Box::new(native)
    } else {
        spawn_managed_session_transport(
            agent_type,
            definition,
            &native_args,
            &mcp_environment,
            mcp_servers.clone(),
            inherit_stderr,
            home,
            effective_name.clone(),
            &instance_id,
            socket,
            None,
        )?
    };
    #[cfg(target_os = "linux")]
    let _remote_drain_stop = RemoteDrainStop(remote_drain.as_ref());
    if native_launch_context.as_ref().is_some_and(|context| {
        crate::native_permissions::recheck_context_with_env(
            context,
            &permission_launch,
            &permission_cwd,
            &permission_environment,
        )
        .is_err()
    }) {
        native_launch_context = None;
        native_launch_refusal = Some("settings_revision_changed".into());
    }
    let descriptor = transport.descriptor();
    let channel = connection_channel();
    let (mut reader, initial_writer, mut my_name) =
        match connect_and_register_with_domain_at(ConnectAndRegisterWithDomainAtArgs {
            socket,
            agent_type,
            agent_id: effective_name.as_deref(),
            host: &host,
            protocol: &definition.protocol,
            channel: channel.as_deref(),
            mode: descriptor.mode,
            location: descriptor.location.as_deref(),
            os: &os,
            instance_id: &instance_id,
            turn_in_progress: false,
            turn_attest: true,
        }) {
            Ok(connection) => connection,
            Err(error) => {
                #[cfg(target_os = "linux")]
                if let Some(guard) = &remote_drain {
                    guard.begin_stop();
                }
                transport.stop();
                return Err(error.into());
            }
        };
    let display_name = if let Some(name) = interactive
        .as_ref()
        .and_then(|launch| launch.display_name.as_deref())
    {
        match crate::communication::client::rename_display_name(
            &my_name,
            &instance_id,
            socket,
            name,
        ) {
            Ok(DaemonToWrapper::DisplayNameResult {
                outcome:
                    bridget_transport::protocol::DisplayNameOutcome::Applied { display_name, .. },
            }) => Some(display_name),
            result => {
                #[cfg(target_os = "linux")]
                if let Some(guard) = &remote_drain {
                    guard.begin_stop();
                }
                transport.stop();
                return Err(format!("--name : changement de nom refusé : {result:?}").into());
            }
        }
    } else {
        None
    };
    let writer = Arc::new(Mutex::new(Some(initial_writer)));
    let mut idempotent_deliveries = IdempotentDeliveryTracker::open(home, &instance_id)?;
    idempotent_deliveries.requires_binding =
        native_bootstrap.is_some() || remote_result_context.is_some();
    let mut execution_bindings = HashMap::new();
    let inbox_path = crate::environment::root_for_home(home)?
        .join("state")
        .join(format!("native-inbox-{instance_id}.sqlite"));
    let mut durable_inbox = if inbox_path.exists() {
        Some(crate::delivery_spool::DeliverySpool::open(
            &inbox_path,
            &instance_id,
        )?)
    } else {
        None
    };
    let child_requests = Mutex::new(ManagedChildRequests::default());
    let forward_child_requests =
        std::env::var(NATIVE_CHILD_REQUESTS_ENV).as_deref() == Ok("forward");
    std::fs::write(&name_state_path, &my_name)?;
    let marker_directory = state_root.join("agent-pids");
    let adapter_pid = transport.process_id();
    crate::mcp_identity::write_marker(
        &marker_directory,
        adapter_pid,
        crate::managed_process::process_birth(adapter_pid)?,
        &instance_id,
        &name_state_path,
    )?;
    let live_feed = JournalLiveFeed::default();
    crate::environment::validate_existing_tree(&journal_root.join(&my_name))?;
    transport.activate_journal(&journal_root, &my_name, Some(live_feed.clone()))?;
    #[cfg(target_os = "linux")]
    if let Some(guard) = &remote_drain {
        if let Some((writer, session)) = transport.journal_writer() {
            guard.set_journal(&journal_root, &my_name, &session, Some(writer));
        }
    }
    apply_pending_profile_instructions(socket, transport.as_mut(), &my_name, &instance_id);
    send_wrapper_message(
        &writer,
        WrapperToDaemon::JournalReady {
            receipt_probe: true,
        },
    );
    send_wrapper_message(&writer, WrapperToDaemon::DaemonIdentityRequest);
    send_wrapper_message(
        &writer,
        WrapperToDaemon::ObservationCapabilities {
            runtime_catalogue: true,
            events: vec![
                bridget_transport::protocol::ObservationKind::TurnEnded,
                bridget_transport::protocol::ObservationKind::PermissionRequired,
                bridget_transport::protocol::ObservationKind::FileWritten,
            ],
        },
    );
    let journal_directory = journal_root.join(&my_name);
    let relay_writer = writer.clone();
    let mut relay = AttachRelayWorker::start(
        journal_directory,
        live_feed,
        Arc::new(move |message| send_wrapper_message(&relay_writer, message)),
    );
    let mut deferred_native_context = None;
    if let (Some(_), Some(definition_digest)) =
        (managed_reporter.as_ref(), frozen_definition_digest)
    {
        // Un nouvel enfant natif reçoit la carte avec sa mission exacte.
        // Les autres sessions gardent leur carte de reprise indépendante.
        let worktree = std::env::current_dir()
            .map_err(|error| format!("worktree courant indisponible: {error}"))?;
        let resume = managed_resume_context(
            &worktree,
            &my_name,
            agent_type,
            &definition.protocol,
            definition_digest,
        );
        deferred_native_context =
            std::env::var(NATIVE_MISSION_BOOTSTRAP_ENV)
                .ok()
                .and_then(|raw| {
                    DeferredNativeMissionContext::from_bootstrap(&raw, &instance_id, resume.clone())
                });
        if deferred_native_context.is_none() {
            let resume_message =
                bridget_core::BridgetMessage::new("bridget-reprise", &my_name, resume);
            let _ = deliver_injected_message(
                transport.as_mut(),
                &resume_message,
                "injection de la carte de reprise",
            );
        }
    }
    if let Some(reporter) = managed_reporter.as_mut() {
        reporter.startup_succeeded();
    }
    if let Some(context) = remote_result_context.as_ref()
        && cfg!(target_os = "linux")
        && definition.protocol == "codex_app_server"
        && let Some(identity) = transport.provider_identity()
        && identity
            .provider_observation
            .as_ref()
            .is_some_and(|v| v.supports(bridget_transport::protocol::ProviderOperation::Resume))
        && let Some(thread_id) = identity.provider_thread_id
    {
        crate::remote_worker::workspace_repair::publish_capability(
            &context.store_root,
            &context.worker,
            &context.command,
            &thread_id,
        )?;
    }
    // Le retour thread/start ou thread/resume fournit déjà le fait effectif.
    // Le publier avant la TUI : son prompt initial peut lancer une délégation.
    let mut last_permission_fact = None;
    if interactive.is_some() {
        let observed = crate::native_permissions::observed_fact(
            &instance_id,
            definition,
            native_launch_context.as_ref(),
            transport.as_ref(),
        );
        let observation = match observed {
            Ok(fact) if native_launch_refusal.is_none() => (fact, None),
            Ok(_) => (None, native_launch_refusal.clone()),
            Err(code) => (None, Some(native_launch_refusal.clone().unwrap_or(code))),
        };
        send_wrapper_message(
            &writer,
            WrapperToDaemon::NativePermissionFact {
                fact: observation.0.clone(),
                request_id: None,
                observation_id: None,
                unavailable_code: observation.1.clone(),
            },
        );
        last_permission_fact = Some(observation);
    }
    // Après Register, identité durable et activation du journal : aucun
    // premier tour humain ne peut précéder l'abonnement Bridget au même fil.
    let bound_codex_thread = tui_binding.as_ref().map(|(thread_id, _)| thread_id.clone());
    let mut native_tui = if let (Some(launch), Some((thread_id, alive))) =
        (&interactive, tui_binding)
    {
        // Réutilise le lien de reprise existant, mais avec écriture atomique
        // durable avant toute saisie humaine. Le fil fournisseur n'est jamais
        // rebaptisé : ce fichier contient seulement l'identité Bridget.
        let binding_path = persistent_name_path(&session_hash(std::slice::from_ref(&thread_id)));
        crate::environment::validate_state_file(&binding_path, false)?;
        bridget_transport::fsutil::write_private_file_atomic(&binding_path, my_name.as_bytes())?;
        let label = display_name
            .as_ref()
            .map_or_else(|| my_name.clone(), |name| format!("{name} [{my_name}]"));
        eprintln!(
            "Bridget : {label} — Codex interactif, un fil par lancement (quitter pour changer de fil)."
        );
        Some(crate::codex_interactive::NativeTui::start(
            &definition.command,
            launch,
            &codex_socket,
            &thread_id,
            &mcp_environment,
            alive,
        )?)
    } else {
        None
    };

    // Même discipline que le wrapper interactif : sans heartbeat, un long tour
    // `busy` laisse `last_seen` geler ; le retain daemon (300 s) jette alors la
    // présence alors que le nom reste au routeur — fantôme unix/connected.
    if native_tui.is_some() {
        send_wrapper_message(&writer, WrapperToDaemon::TerminalSessionReady);
    }
    let mut last_heartbeat = Instant::now();
    let mut last_provider_spawn = Instant::now();
    let mut consecutive_fast_failures = 0_u32;
    let mut workspace_repair_barrier: Option<
        bridget_transport::protocol::WorkspaceRepairRequestV1,
    > = None;
    let mut workspace_repair_pending_receipt: Option<
        bridget_transport::protocol::WorkspaceRepairStatusV1,
    > = None;
    let mut last_repair_receipt_attempt = Instant::now();

    loop {
        if last_repair_receipt_attempt.elapsed() >= Duration::from_millis(250) {
            if let (Some(status), Some(context)) =
                (&workspace_repair_pending_receipt, &remote_result_context)
                && workspace_repair::persist_pending(context, status)
            {
                let status = status.clone();
                if status.phase == "refused" {
                    workspace_repair_barrier = None;
                }
                send_wrapper_message(
                    &writer,
                    WrapperToDaemon::WorkspaceRepairResult {
                        task_id: context.command.task_id.clone(),
                        status,
                    },
                );
                workspace_repair_pending_receipt = None;
            }
            last_repair_receipt_attempt = Instant::now();
        }
        if let (Some(request), Some(context)) = (&workspace_repair_barrier, &remote_result_context)
        {
            if crate::remote_worker::workspace_repair::published(
                &context.store_root,
                &context.command,
                request,
            ) {
                workspace_repair_barrier = None;
            } else {
                // Aucun message (y compris les événements système) ne rejoint
                // le fournisseur avant la publication. L'annulation native
                // reste prioritaire via le signal détenu par le worker.
                if native_termination
                    .as_ref()
                    .is_some_and(NativeTerminationSignal::requested)
                {
                    break;
                }
                reattest_managed_turn_if_due(
                    &writer,
                    transport.as_ref(),
                    &mut last_heartbeat,
                    Instant::now(),
                );
                thread::sleep(Duration::from_millis(25));
                continue;
            }
        }
        reattest_managed_turn_if_due(
            &writer,
            transport.as_ref(),
            &mut last_heartbeat,
            Instant::now(),
        );
        if native_termination
            .as_ref()
            .is_some_and(NativeTerminationSignal::requested)
        {
            break;
        }
        let observed = crate::native_permissions::observed_fact(
            &instance_id,
            definition,
            native_launch_context.as_ref(),
            transport.as_ref(),
        );
        let observation = match observed {
            Ok(fact) if native_launch_refusal.is_none() => (fact, None),
            Ok(_) => (None, native_launch_refusal.clone()),
            Err(code) => (None, Some(native_launch_refusal.clone().unwrap_or(code))),
        };
        if last_permission_fact.as_ref() != Some(&observation) {
            send_wrapper_message(
                &writer,
                WrapperToDaemon::NativePermissionFact {
                    fact: observation.0.clone(),
                    request_id: None,
                    observation_id: None,
                    unavailable_code: observation.1.clone(),
                },
            );
            last_permission_fact = Some(observation);
        }
        if transport.is_alive()
            && consecutive_fast_failures > 0
            && last_provider_spawn.elapsed() >= Duration::from_secs(30)
        {
            consecutive_fast_failures = 0;
        }
        let events = transport.drain_events();
        if let Err(error) = idempotent_deliveries.retry_reply(&writer) {
            error!("réponse conservée, reprise non confirmée : {error}");
            break;
        }
        let journal_failed = forward_managed_events_with_redaction(
            &writer,
            &my_name,
            events,
            &mut idempotent_deliveries,
            &mut execution_bindings,
            &mut redaction_lease,
            profile_ledger_path(socket).as_deref(),
            interactive.is_none(),
            Some(&child_requests),
            Some(transport.is_busy()),
        );
        if journal_failed {
            break;
        }
        let mut line = String::new();
        let mut from_durable_inbox = false;
        let mut read_result = reader.read_line(&mut line);
        // Les contrôles du socket passent avant la file différée. Une remise
        // ordinaire attend la fin du tour sans remplir la file du fournisseur.
        if matches!(&read_result,Err(e) if matches!(e.kind(),std::io::ErrorKind::WouldBlock|std::io::ErrorKind::TimedOut))
            && !transport.is_busy()
            && workspace_repair_barrier.is_none()
            && let Some(inbox) = durable_inbox.as_mut()
        {
            match inbox.next() {
                Ok(Some(frame)) => {
                    line = serde_json::to_string(&frame)?;
                    from_durable_inbox = true;
                    read_result = Ok(line.len());
                }
                Ok(None) => {}
                Err(error) => {
                    error!("file native illisible : {error} ; copie maître conservée");
                    break;
                }
            }
        }
        if native_termination
            .as_ref()
            .is_some_and(NativeTerminationSignal::requested)
        {
            break;
        }
        match read_result {
            Ok(0) => {
                // Le worker distant conserve le processus et le résultat durable.
                // Une coupure brève bénéficie de la grâce distante ; son
                // expiration, un arrêt explicite ou le signal natif l'arrêtent.
                if native_bootstrap.is_some()
                    && !forward_child_requests
                    && remote_result_context.is_none()
                {
                    break;
                }
                relay.reset_generation();
                let Some((new_reader, registered_name)) =
                    reconnect_managed_session(ReconnectManagedSessionArgs {
                        socket,
                        writer: &writer,
                        transport: transport.as_ref(),
                        descriptor: &descriptor,
                        protocol: &definition.protocol,
                        agent_type,
                        name_state_path: &name_state_path,
                        host: &host,
                        channel: channel.as_deref(),
                        os: &os,
                        instance_id: &instance_id,
                        fallback_name: &my_name,
                        terminal_session: interactive.is_some(),
                        termination: native_termination.as_ref(),
                        deadline: remote_result_context
                            .as_ref()
                            .map(|_| Instant::now() + REMOTE_MASTER_RECONNECT_GRACE),
                    })
                else {
                    break;
                };
                idempotent_deliveries.modern_reply_core = false;
                reader = new_reader;
                my_name = registered_name;
                last_permission_fact = None;
                last_heartbeat = Instant::now();
                if native_termination
                    .as_ref()
                    .is_some_and(NativeTerminationSignal::requested)
                {
                    break;
                }
                if forward_child_requests
                    || !child_requests.lock().recover_poison().waiting.is_empty()
                {
                    let pending: Vec<_> = child_requests
                        .lock()
                        .recover_poison()
                        .waiting
                        .iter()
                        .filter(|entry| !entry.uncertain)
                        .map(|entry| entry.request.clone())
                        .collect();
                    for request in pending {
                        send_wrapper_message(&writer, WrapperToDaemon::ChildRequest { request });
                    }
                }
                continue;
            }
            Ok(_) => match decode(line.trim()) {
                Ok(DaemonToWrapper::DaemonIdentityReport { features, .. }) => {
                    idempotent_deliveries.modern_reply_core =
                        features.iter().any(|f| f == "native_reply_outbox_v1");
                }
                Ok(DaemonToWrapper::Nack { id, reason }) if id.starts_with("reply244-") => {
                    if let Some(outbox) = &idempotent_deliveries.reply_outbox
                        && let Err(error) = outbox.refused(&id)
                    {
                        error!("refus de réponse non conservé : {error}");
                        break;
                    }
                    warn!(
                        "réponse conservée sans nouvelle tentative : {}",
                        reason.chars().take(256).collect::<String>()
                    );
                }
                Ok(DaemonToWrapper::Ack { id, .. }) => {
                    if let Err(error) = idempotent_deliveries.acknowledge_reply(&id) {
                        error!("accusé de réponse non conservé : {error}");
                        break;
                    }
                }
                Ok(
                    DaemonToWrapper::Deliver { message, .. }
                    | DaemonToWrapper::DeliverExecution { message, .. }
                    | DaemonToWrapper::DeliverIdempotent { message, .. },
                ) if workspace_repair_barrier.is_some() => {
                    send_wrapper_message(
                        &writer,
                        bound_delivery_rejection(
                            &execution_bindings,
                            message.id,
                            "workspace_repair_pending".into(),
                        ),
                    );
                }
                Ok(DaemonToWrapper::Deliver { message, .. }) => {
                    if let Err(reason) = remote_result_for_delivery(
                        remote_result_context.as_ref(),
                        &message,
                        None,
                        &execution_bindings,
                    ) {
                        send_wrapper_message(
                            &writer,
                            bound_delivery_rejection(&execution_bindings, message.id, reason),
                        );
                        continue;
                    }
                    idempotent_deliveries.record_historic(&message.id);
                    let ordinary = !execution_bindings.contains_key(&message.id)
                        && ordinary_queue_message(&message);
                    if ordinary {
                        idempotent_deliveries.admit_ordinary(&message);
                    }
                    if let Err(error) = deliver_provider_message(
                        transport.as_mut(),
                        &message,
                        &mut deferred_native_context,
                    ) {
                        idempotent_deliveries.historic_injection_failed(&message.id);
                        if ordinary {
                            idempotent_deliveries.finish_ordinary(&message.id);
                        }
                        send_wrapper_message(
                            &writer,
                            bound_delivery_rejection(
                                &execution_bindings,
                                message.id,
                                error.to_string(),
                            ),
                        );
                    }
                }
                Ok(DaemonToWrapper::DeliverExecution {
                    message,
                    execution_id,
                    generation,
                    revision,
                }) => {
                    let message_id = message.id.clone();
                    let remote_result = match remote_result_for_delivery(
                        remote_result_context.as_ref(),
                        &message,
                        Some((&execution_id, generation)),
                        &execution_bindings,
                    ) {
                        Ok(reservation) => reservation,
                        Err(reason) => {
                            send_wrapper_message(
                                &writer,
                                WrapperToDaemon::DeliveryRejected {
                                    id: message_id,
                                    execution_id: Some(execution_id),
                                    generation: Some(generation),
                                    reason,
                                },
                            );
                            continue;
                        }
                    };
                    execution_bindings.insert(
                        message_id.clone(),
                        ManagedExecutionBinding {
                            execution_id,
                            generation,
                            state: "starting".to_string(),
                            provider_kind: agent_type.to_string(),
                            execution_path: definition.protocol.clone(),
                            revision,
                            approval_requests: 0,
                            last_approval_request: None,
                            remote_result,
                        },
                    );
                    if let Some(identity) = transport.provider_identity() {
                        publish_provider_context(&writer, &execution_bindings, &identity);
                    }
                    if let Err(error) = deliver_provider_message(
                        transport.as_mut(),
                        &message,
                        &mut deferred_native_context,
                    ) {
                        let rejection = bound_delivery_rejection(
                            &execution_bindings,
                            message_id.clone(),
                            error.to_string(),
                        );
                        idempotent_deliveries.historic_injection_failed(&message_id);
                        publish_execution_transition(
                            &writer,
                            &mut execution_bindings,
                            &message_id,
                            "failed",
                            "provider_unavailable",
                        );
                        send_wrapper_message(&writer, rejection);
                    }
                }
                Ok(DaemonToWrapper::DeliveryReceiptQuery {
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                }) => {
                    if let Some(report) = idempotent_deliveries.receipt_report(
                        delivery_id,
                        &recipient_instance_id,
                        delivery_generation,
                        unix_now_secs(),
                    ) {
                        send_wrapper_message(&writer, report);
                    }
                }
                Ok(frame @ DaemonToWrapper::DeliverIdempotent { .. })
                    if !from_durable_inbox
                        && defer_ordinary_delivery(
                            &frame,
                            transport.is_busy()
                                || durable_inbox
                                    .as_ref()
                                    .is_some_and(|inbox| inbox.has_pending()),
                        ) =>
                {
                    if durable_inbox.is_none() {
                        match crate::delivery_spool::DeliverySpool::open(&inbox_path, &instance_id)
                        {
                            Ok(inbox) => durable_inbox = Some(inbox),
                            Err(error) => {
                                error!(
                                    "file native indisponible : {error} ; copie maître conservée"
                                );
                                break;
                            }
                        }
                    }
                    if let Err(error) = durable_inbox.as_ref().unwrap().defer(&frame) {
                        error!(
                            "remise différée refusée : {error} ; copie maître et tour fournisseur conservés"
                        );
                        // Un conflit de remise ne ferme pas le fournisseur.
                        // Aucun ACK ni injection : le maître conserve l'issue.
                        continue;
                    }
                }
                Ok(DaemonToWrapper::DeliverIdempotent {
                    delivery_id,
                    recipient_instance_id,
                    delivery_generation,
                    expires_at,
                    message,
                    execution,
                    ..
                }) => {
                    let message_id = message.id.clone();
                    let remote_result = match remote_result_for_delivery(
                        remote_result_context.as_ref(),
                        &message,
                        execution
                            .as_ref()
                            .map(|context| (context.execution_id.as_str(), context.generation)),
                        &execution_bindings,
                    ) {
                        Ok(reservation) => reservation,
                        Err(reason) => {
                            let rejection = match &execution {
                                Some(context) => WrapperToDaemon::DeliveryRejected {
                                    id: message_id,
                                    execution_id: Some(context.execution_id.clone()),
                                    generation: Some(context.generation),
                                    reason,
                                },
                                None => bound_delivery_rejection(
                                    &execution_bindings,
                                    message_id,
                                    reason,
                                ),
                            };
                            send_wrapper_message(&writer, rejection);
                            continue;
                        }
                    };
                    if remote_result.is_some()
                        && execution_bindings.get(&message_id).is_some_and(|binding| {
                            matches!(
                                binding.state.as_str(),
                                "completed" | "failed" | "interrupted" | "unreachable"
                            )
                        })
                        && execution.as_ref().is_some_and(|next| {
                            execution_bindings.get(&message_id).is_some_and(|old| {
                                old.execution_id != next.execution_id
                                    || old.generation != next.generation
                            })
                        })
                    {
                        execution_bindings.remove(&message_id);
                    }
                    if bind_idempotent_delivery_execution(
                        &mut execution_bindings,
                        &message_id,
                        execution,
                        agent_type,
                        &definition.protocol,
                    ) && let Some(identity) = transport.provider_identity()
                    {
                        publish_provider_context(&writer, &execution_bindings, &identity);
                    }
                    if let Some(binding) = execution_bindings.get_mut(&message_id)
                        && remote_result.is_some()
                    {
                        binding.remote_result = remote_result;
                    }
                    match idempotent_deliveries.receive(
                        delivery_id,
                        recipient_instance_id,
                        delivery_generation,
                        expires_at,
                        message,
                        unix_now_secs(),
                    ) {
                        IdempotentDeliveryAction::Pending => {}
                        IdempotentDeliveryAction::Report(report) => {
                            send_wrapper_message(&writer, report);
                        }
                        IdempotentDeliveryAction::Inject {
                            message,
                            delivery_id,
                        } => {
                            let message_id = message.id.clone();
                            let ordinary = !execution_bindings.contains_key(&message_id)
                                && ordinary_queue_message(&message);
                            if ordinary {
                                idempotent_deliveries.admit_ordinary(&message);
                            }
                            // Seen est durable ; aucun PromptDispatched/ACK d’injection encore.
                            #[cfg(feature = "test-support")]
                            crate::test_sync::checkpoint("after_receipt_seen_before_injection");
                            if let Err(error) = deliver_provider_message(
                                transport.as_mut(),
                                &message,
                                &mut deferred_native_context,
                            ) {
                                if ordinary {
                                    idempotent_deliveries.finish_ordinary(&message_id);
                                }
                                send_wrapper_message(
                                    &writer,
                                    bound_delivery_rejection(
                                        &execution_bindings,
                                        message_id.clone(),
                                        error.to_string(),
                                    ),
                                );
                                if let Some(report) =
                                    idempotent_deliveries.injection_failed(&delivery_id)
                                {
                                    send_wrapper_message(&writer, report);
                                }
                            }
                        }
                    }
                }
                Ok(DaemonToWrapper::DelegatedRuntimeEvent { event }) => {
                    let message = delegated_runtime_message(&event, &my_name);
                    idempotent_deliveries.record_delegated_runtime(&message.id, event.event_id);
                    if let Err(error) = transport.deliver(&message) {
                        let _ = idempotent_deliveries.injection_rejected(&message.id);
                        warn!("remise de l'incident délégué différée: {error}");
                    }
                }
                Ok(DaemonToWrapper::WorkspaceRepairControl { task_id, status }) => {
                    let outcome = workspace_repair::apply(
                        &task_id,
                        status,
                        remote_result_context.as_ref(),
                        &mut transport,
                        agent_type,
                        definition,
                        &native_args,
                        &mcp_environment,
                        mcp_servers.clone(),
                        home,
                        &my_name,
                        &instance_id,
                        socket,
                        native_termination.as_ref(),
                    );
                    let status = outcome.status;
                    workspace_repair_pending_receipt = outcome.pending_receipt;
                    if status.phase != "refused" {
                        workspace_repair_barrier = Some(status.request.clone());
                    }
                    if status.phase == "resumed" {
                        permission_cwd = std::env::current_dir().unwrap_or(permission_cwd);
                        last_permission_fact = None;
                        let pid = transport.process_id();
                        if let Ok(birth) = crate::managed_process::process_birth(pid) {
                            let _ = crate::mcp_identity::write_marker(
                                &marker_directory,
                                pid,
                                birth,
                                &instance_id,
                                &name_state_path,
                            );
                        }
                        let feed = JournalLiveFeed::default();
                        if transport
                            .activate_journal(&journal_root, &my_name, Some(feed.clone()))
                            .is_ok()
                        {
                            #[cfg(target_os = "linux")]
                            if let Some(guard) = &remote_drain {
                                if let Some((writer, session)) = transport.journal_writer() {
                                    guard.set_journal(
                                        &journal_root,
                                        &my_name,
                                        &session,
                                        Some(writer),
                                    );
                                }
                            }
                            relay.shutdown();
                            let relay_writer = writer.clone();
                            relay = AttachRelayWorker::start(
                                journal_root.join(&my_name),
                                feed,
                                Arc::new(move |message| {
                                    send_wrapper_message(&relay_writer, message)
                                }),
                            );
                        }
                    }
                    send_wrapper_message(
                        &writer,
                        WrapperToDaemon::WorkspaceRepairResult { task_id, status },
                    );
                }
                Ok(DaemonToWrapper::ControlExecutionDispatch {
                    issuer_scope,
                    command,
                }) => {
                    apply_execution_control(
                        &writer,
                        transport.as_mut(),
                        &mut execution_bindings,
                        issuer_scope,
                        command,
                    );
                }
                Ok(DaemonToWrapper::UserInputResult { result }) => {
                    observe_user_input_receipt(&writer, &child_requests, &result);
                }
                Ok(message @ DaemonToWrapper::ChildRequestAnswer { .. })
                | Ok(message @ DaemonToWrapper::ChildRequestCancel { .. }) => {
                    if dispatch_managed_child_request(
                        &writer,
                        transport.as_mut(),
                        &mut execution_bindings,
                        &child_requests,
                        message,
                    ) {
                        break;
                    }
                }
                Ok(DaemonToWrapper::SelectRuntime { token, selection }) => {
                    let outcome = transport.select_runtime(selection);
                    send_wrapper_message(
                        &writer,
                        WrapperToDaemon::RuntimeSelectionReported { token, outcome },
                    );
                }
                Ok(DaemonToWrapper::ReadRuntimeCatalogue { token }) => {
                    let outcome = transport.runtime_catalogue();
                    send_wrapper_message(
                        &writer,
                        WrapperToDaemon::RuntimeCatalogueReported { token, outcome },
                    );
                }
                Ok(DaemonToWrapper::CancelDelivery {
                    id,
                    reason,
                    interrupt_active,
                }) => {
                    if let Some(inbox) = durable_inbox.as_ref()
                        && let Err(error) = inbox.cancel(&id)
                    {
                        error!("annulation en file native impossible : {error}");
                        break;
                    }
                    if interrupt_active.unwrap_or(false) {
                        transport.cancel_delivery(&id, &reason);
                    } else {
                        transport.cancel_queued(&id, &reason);
                    }
                }
                Ok(DaemonToWrapper::Subscribe {
                    subscription_id,
                    window,
                    ..
                }) => {
                    if let Err(reason) = relay.subscribe(subscription_id.clone(), window) {
                        send_wrapper_message(
                            &writer,
                            WrapperToDaemon::AttachRejected {
                                subscription_id: Some(subscription_id),
                                reason,
                            },
                        );
                    }
                }
                Ok(DaemonToWrapper::ProjectContextResult { .. }) => {}
                Ok(DaemonToWrapper::Unsubscribe { subscription_id }) => {
                    relay.unsubscribe(subscription_id);
                }
                Ok(DaemonToWrapper::Disconnect) => break,
                Ok(_) => {}
                Err(_) => warn!("message ACP de protocole illisible"),
            },
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                reattest_managed_turn_if_due(
                    &writer,
                    transport.as_ref(),
                    &mut last_heartbeat,
                    Instant::now(),
                );
            }
            Err(error) => {
                warn!("connexion daemon ACP perdue : {error}");
                // Le worker distant conserve le processus et le résultat durable.
                // Une coupure brève bénéficie de la grâce distante ; son
                // expiration, un arrêt explicite ou le signal natif l'arrêtent.
                if native_bootstrap.is_some()
                    && !forward_child_requests
                    && remote_result_context.is_none()
                {
                    break;
                }
                relay.reset_generation();
                let Some((new_reader, registered_name)) =
                    reconnect_managed_session(ReconnectManagedSessionArgs {
                        socket,
                        writer: &writer,
                        transport: transport.as_ref(),
                        descriptor: &descriptor,
                        protocol: &definition.protocol,
                        agent_type,
                        name_state_path: &name_state_path,
                        host: &host,
                        channel: channel.as_deref(),
                        os: &os,
                        instance_id: &instance_id,
                        fallback_name: &my_name,
                        terminal_session: interactive.is_some(),
                        termination: native_termination.as_ref(),
                        deadline: remote_result_context
                            .as_ref()
                            .map(|_| Instant::now() + REMOTE_MASTER_RECONNECT_GRACE),
                    })
                else {
                    break;
                };
                idempotent_deliveries.modern_reply_core = false;
                reader = new_reader;
                my_name = registered_name;
                last_permission_fact = None;
                last_heartbeat = Instant::now();
                if native_termination
                    .as_ref()
                    .is_some_and(NativeTerminationSignal::requested)
                {
                    break;
                }
                if forward_child_requests
                    || !child_requests.lock().recover_poison().waiting.is_empty()
                {
                    let pending: Vec<_> = child_requests
                        .lock()
                        .recover_poison()
                        .waiting
                        .iter()
                        .filter(|entry| !entry.uncertain)
                        .map(|entry| entry.request.clone())
                        .collect();
                    for request in pending {
                        send_wrapper_message(&writer, WrapperToDaemon::ChildRequest { request });
                    }
                }
                continue;
            }
        }
        if !transport.is_alive() && workspace_repair_barrier.is_none() {
            let terminal_events = transport.drain_events();
            let terminal_journal_failed = forward_managed_events_with_redaction(
                &writer,
                &my_name,
                terminal_events,
                &mut idempotent_deliveries,
                &mut execution_bindings,
                &mut redaction_lease,
                profile_ledger_path(socket).as_deref(),
                interactive.is_none(),
                Some(&child_requests),
                Some(transport.is_busy()),
            );
            if terminal_journal_failed {
                break;
            }
            let persistent_relaunch = interactive.is_none()
                    && std::env::var_os("BRIDGET_MANAGED_PERSISTENT").is_some()
                    // Une mission native ne change jamais de fournisseur
                    // après un échec. La saga conserve son état terminal.
                    && std::env::var_os(NATIVE_MISSION_BOOTSTRAP_ENV).is_none();
            if !persistent_relaunch {
                break;
            }
            #[cfg(target_os = "linux")]
            if let Some(guard) = &remote_drain {
                guard.begin_stop();
            }
            transport.stop();
            let max_relaunch: u32 = std::env::var("BRIDGET_PROVIDER_RELAUNCH_MAX")
                .ok()
                .and_then(|value| value.parse().ok())
                .filter(|value| *value > 0)
                .unwrap_or(5);
            // Compteur de morts consécutives : ne se remet à zéro qu'après une
            // période stable sous fournisseur vivant (voir bas de boucle).
            consecutive_fast_failures = consecutive_fast_failures.saturating_add(1);
            if consecutive_fast_failures > max_relaunch {
                let cause = format!(
                    "abandon relance fournisseur après {max_relaunch} tentatives (processus fournisseur terminé)"
                );
                eprintln!("[bridget] {cause}");
                warn!("{cause}");
                break;
            }
            let backoff_secs = 1u64 << (consecutive_fast_failures - 1).min(4);
            eprintln!(
                "[bridget] fournisseur mort — relance {consecutive_fast_failures}/{max_relaunch} dans {backoff_secs}s"
            );
            let backoff_deadline = Instant::now() + Duration::from_secs(backoff_secs);
            let mut abandon_for_disconnect = false;
            while Instant::now() < backoff_deadline {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => {}
                    Ok(_) => {
                        if matches!(decode(line.trim()), Ok(DaemonToWrapper::Disconnect)) {
                            abandon_for_disconnect = true;
                            break;
                        }
                    }
                    Err(error)
                        if matches!(
                            error.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                        ) => {}
                    Err(_) => {}
                }
                thread::sleep(Duration::from_millis(50));
            }
            if abandon_for_disconnect {
                break;
            }
            match spawn_managed_session_transport(
                agent_type,
                definition,
                &native_args,
                &mcp_environment,
                mcp_servers.clone(),
                inherit_stderr,
                home,
                Some(my_name.clone()),
                &instance_id,
                socket,
                None,
            ) {
                Ok(new_transport) => {
                    transport = new_transport;
                    last_permission_fact = None;
                    if native_launch_context.as_ref().is_some_and(|context| {
                        crate::native_permissions::recheck_context_with_env(
                            context,
                            &permission_launch,
                            &permission_cwd,
                            &permission_environment,
                        )
                        .is_err()
                    }) {
                        native_launch_context = None;
                        native_launch_refusal = Some("settings_revision_changed".into());
                    }
                    let adapter_pid = transport.process_id();
                    match crate::managed_process::process_birth(adapter_pid) {
                        Ok(birth) => {
                            if let Err(error) = crate::mcp_identity::write_marker(
                                &marker_directory,
                                adapter_pid,
                                birth,
                                &instance_id,
                                &name_state_path,
                            ) {
                                warn!("marqueur fournisseur après relance impossible: {error}");
                            }
                        }
                        Err(error) => {
                            warn!("naissance fournisseur après relance illisible: {error}");
                        }
                    }
                    let live_feed = JournalLiveFeed::default();
                    if let Err(error) =
                        transport.activate_journal(&journal_root, &my_name, Some(live_feed.clone()))
                    {
                        warn!("journal après relance impossible: {error}");
                    } else {
                        #[cfg(target_os = "linux")]
                        if let Some(guard) = &remote_drain {
                            if let Some((writer, session)) = transport.journal_writer() {
                                guard.set_journal(&journal_root, &my_name, &session, Some(writer));
                            }
                        }
                    }
                    apply_pending_profile_instructions(
                        socket,
                        transport.as_mut(),
                        &my_name,
                        &instance_id,
                    );
                    relay.shutdown();
                    let relay_writer = writer.clone();
                    relay = AttachRelayWorker::start(
                        journal_root.join(&my_name),
                        live_feed,
                        Arc::new(move |message| send_wrapper_message(&relay_writer, message)),
                    );
                    // Le fournisseur relancé reçoit désormais la carte de reprise
                    // normale. Ne pas la joindre à nouveau à une remise rejouée.
                    deferred_native_context = None;
                    if let Some(definition_digest) = frozen_definition_digest {
                        let worktree =
                            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                        let resume = managed_resume_context(
                            &worktree,
                            &my_name,
                            agent_type,
                            &definition.protocol,
                            definition_digest,
                        );
                        let resume_message =
                            bridget_core::BridgetMessage::new("bridget-reprise", &my_name, resume);
                        let _ = deliver_injected_message(
                            transport.as_mut(),
                            &resume_message,
                            "réinjection de la carte de reprise après relance",
                        );
                    }
                    last_provider_spawn = Instant::now();
                    last_heartbeat = Instant::now();
                    eprintln!(
                        "[bridget] fournisseur relancé (tentative {consecutive_fast_failures}/{max_relaunch})"
                    );
                    continue;
                }
                Err(error) => {
                    warn!("relance fournisseur échouée: {error}");
                    continue;
                }
            }
        }
    }
    // Le fournisseur natif ne doit pas rester actif pendant le drain du
    // relais, ni se reconnecter à un daemon qui a déjà déclaré sa mission perdue.
    if let Some(bootstrap) = native_bootstrap.as_ref() {
        #[cfg(target_os = "linux")]
        if let Some(guard) = &remote_drain {
            guard.begin_stop();
        }
        transport.stop_native_mission(&bootstrap.mission_id);
    }
    relay.shutdown();
    let tui_result = native_tui
        .as_ref()
        .map(crate::codex_interactive::NativeTui::result);
    // Fermer le client natif avant le drain du serveur : une élicitation
    // ouverte appartient à cette connexion, pas au worker Bridget.
    let tui_stopped = native_tui.as_mut().map(|tui| tui.close());
    #[cfg(target_os = "linux")]
    if let Some(guard) = &remote_drain {
        guard.begin_stop();
    }
    transport.stop();
    // L'EOF peut fermer le transport entre deux itérations : vider une dernière
    // fois les événements terminaux avant Unregister afin que le daemon voie
    // chaque DeliveryRejected (tour actif comme file restante).
    let _ = forward_managed_events_with_redaction(
        &writer,
        &my_name,
        transport.drain_events(),
        &mut idempotent_deliveries,
        &mut execution_bindings,
        &mut redaction_lease,
        profile_ledger_path(socket).as_deref(),
        interactive.is_none(),
        Some(&child_requests),
        Some(transport.is_busy()),
    );
    send_wrapper_message(&writer, WrapperToDaemon::TurnState { in_progress: false });
    send_wrapper_message(&writer, WrapperToDaemon::Unregister);
    // Le pilote garde sa socket si la disparition de son groupe n'est pas
    // confirmée. Ne pas transformer ce diagnostic de récupération en exit 0.
    let codex_endpoint_retained = if interactive.is_some() {
        match std::fs::symlink_metadata(&codex_socket) {
            Ok(_) => true,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error.into()),
        }
    } else {
        false
    };
    if codex_endpoint_retained {
        return Err(format!(
            "nettoyage du serveur Codex non confirmé ; socket conservée : {}",
            codex_socket.display()
        )
        .into());
    }
    if let Some(launch) = &interactive {
        // La TUI fournisseur annonce une reconnexion --remote générique,
        // alors que SON serveur vient d'être fermé par le propriétaire Bridget.
        // La dernière instruction affichée doit refléter ce cycle de vie réel.
        if let Some(thread_id) = bound_codex_thread.as_deref() {
            eprintln!(
                "Bridget : session fermée ; la socket temporaire n'est plus utilisable.\nReprendre : {}",
                launch.resume_command(thread_id)
            );
        }
    }
    if let Some(result) = tui_result {
        result?;
    }
    if let Some(result) = tui_stopped {
        result.map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// --name sélectionne une identité durable ; Register reste l'arbitre atomique
/// d'activité si une autre connexion apparaît entre cette lecture et l'inscription.
fn resolve_interactive_identity(
    launch: &crate::codex_interactive::Launch,
    explicit_id: Option<&str>,
    socket: &Path,
) -> Result<Option<String>, String> {
    use bridget_transport::protocol::{DisplayNameRequest, DisplayNameResolution};
    let mut candidates = explicit_id
        .map(str::to_owned)
        .into_iter()
        .collect::<Vec<_>>();
    if let Some(thread_id) = &launch.resume_thread
        && uuid::Uuid::parse_str(thread_id).is_ok()
    {
        let path = persistent_name_path(&session_hash(std::slice::from_ref(thread_id)));
        crate::environment::validate_state_file(&path, false)?;
        match std::fs::read_to_string(&path) {
            Ok(id) => {
                let id = id.trim();
                bridget_core::router::validate_agent_id(id).map_err(
                    |_| "liaison de reprise Bridget invalide : aucun remplacement automatique",
                )?;
                candidates.push(id.to_owned());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(format!("liaison de reprise illisible : {error}")),
        }
    }
    if let Some(name) = &launch.display_name {
        let mut client = crate::communication::client::DaemonConnection::connect(socket)
            .map_err(|error| error.to_string())?;
        let reply = client
            .exchange(&WrapperToDaemon::DisplayNameResolve {
                request: DisplayNameRequest {
                    version: 1,
                    display_name: name.clone(),
                },
            })
            .map_err(|error| error.to_string())?;
        match reply {
            DaemonToWrapper::DisplayNameResolved {
                outcome: DisplayNameResolution::Found { agent_id, active },
            } => {
                if active {
                    return Err(format!(
                        "l'agent « {name} » est déjà actif ; fermez sa session avant de le reprendre"
                    ));
                }
                candidates.push(agent_id);
            }
            DaemonToWrapper::DisplayNameResolved {
                outcome: DisplayNameResolution::NotFound,
            } => {}
            other => return Err(format!("résolution du nom Bridget refusée : {other:?}")),
        }
    }
    if let Some(first) = candidates.first()
        && candidates.iter().any(|id| id != first)
    {
        return Err("le nom, le fil et --agent-id désignent des identités différentes ; reprise refusée sans modification".into());
    }
    Ok(candidates.into_iter().next())
}

fn billing_guard_error(variable: &str) -> String {
    format!(
        "variable d'environnement refusée pour l'équipier ACP : {variable} \
         (utilisez BRIDGET_ALLOW_API_KEY=1 uniquement si vous acceptez la facturation API)"
    )
}

fn instance_name_state_path(socket: &Path, instance_id: &str) -> PathBuf {
    socket
        .parent()
        .unwrap_or(socket)
        .join("agent-names")
        .join(format!("instance-{instance_id}"))
}

fn managed_adapter_environment(
    instance_id: &str,
    explicit_name: Option<&str>,
    name_state_path: Option<&Path>,
) -> Result<Vec<(OsString, OsString)>, String> {
    let mut environment = vec![(
        OsString::from("BRIDGET_AGENT_INSTANCE_ID"),
        OsString::from(instance_id),
    )];
    environment.extend(crate::environment::Namespace::from_environment()?.child_environment());
    if let Some(name) = explicit_name.filter(|value| !value.is_empty()) {
        environment.push((OsString::from("BRIDGET_AGENT_ID"), OsString::from(name)));
    }
    if let Some(path) = name_state_path {
        environment.push((
            OsString::from("BRIDGET_AGENT_ID_FILE"),
            OsString::from(path.as_os_str()),
        ));
    }
    if let Some(path) = path_with_current_exe_dir() {
        environment.push((OsString::from("PATH"), OsString::from(path)));
    }
    Ok(environment)
}

fn path_with_current_exe_dir() -> Option<String> {
    crate::lifecycle::path_with_current_exe_dir_first(&std::env::var("PATH").unwrap_or_default())
}

fn string_environment(environment: &[(OsString, OsString)]) -> Vec<(String, String)> {
    environment
        .iter()
        .map(|(key, value)| {
            (
                key.to_string_lossy().into_owned(),
                value.to_string_lossy().into_owned(),
            )
        })
        .collect()
}

fn apply_managed_mcp(
    protocol: &str,
    interactive: &str,
    args: &mut Vec<String>,
    instance_id: &str,
    socket: &Path,
) -> Result<Option<EphemeralMcpConfig>, Box<dyn std::error::Error>> {
    match (protocol, interactive) {
        ("claude_stream_json", "claude") => {
            let directory = socket.parent().ok_or("répertoire socket Bridget absent")?;
            let config =
                claude_mcp_config_in(directory, &interactive_mcp_server_entry()?, instance_id)?;
            args.extend([
                "--strict-mcp-config".to_string(),
                "--mcp-config".to_string(),
                config.path().display().to_string(),
            ]);
            // Les droits d'un enfant149 sont déjà figés. Le montage MCP
            // ne lui ajoute aucune règle d'approbation absente du parent.
            if std::env::var_os(crate::native_permissions::CHILD_POLICY_ENV).is_none() {
                append_claude_allowed_tools(args);
            }
            Ok(Some(config))
        }
        // Le pilote app-server relit bien les serveurs déclarés dans la
        // configuration Codex, y compris pour un tour géré. Les exposer ici
        // donne au fournisseur les outils Bridget réels, au lieu de lui
        // demander de deviner une commande CLI qui n'existe pas pour publier
        // un artefact.
        ("codex_app_server", "codex") => {
            let override_ = codex_mcp_override(&interactive_mcp_server_entry()?)?;
            // Codex porte aussi -c sur sa sous-commande : un groupe après
            // app-server remplace celui d'avant. Réunir les overrides au même
            // niveau, sans changer leurs octets ni leur ordre de priorité.
            // Sinon ajouter MCP après app-server ferait perdre notamment le
            // --yolo humain projeté en -c avant app-server.
            if let Some(at) = args.iter().position(|argument| argument == "app-server") {
                let mut prefix = Vec::new();
                let mut overrides = Vec::new();
                let mut index = 0;
                while index < at {
                    let argument = &args[index];
                    if matches!(argument.as_str(), "-c" | "--config") {
                        if index + 1 == at {
                            return Err("option Codex de configuration sans valeur".into());
                        }
                        overrides.extend_from_slice(&args[index..index + 2]);
                        index += 2;
                    } else {
                        if argument.starts_with("--config=") || argument.starts_with("-c=") {
                            overrides.push(argument.clone());
                        } else {
                            prefix.push(argument.clone());
                        }
                        index += 1;
                    }
                }
                prefix.push("app-server".into());
                prefix.extend(overrides);
                prefix.extend_from_slice(&args[at + 1..]);
                *args = prefix;
            }
            args.extend(["-c".to_string(), override_]);
            Ok(None)
        }
        _ => Ok(None),
    }
}

/// Politique d'autorisation sur le chemin managed-wrapper.
///
/// `permissions=allow` : pose les drapeaux de contournement (même sémantique
/// que `launch` interactif). `permissions=deny` : les retire s'ils étaient
/// présents dans la définition — sinon deny mentirait.
fn apply_managed_permission_policy(protocol: &str, permissions: &str, args: &mut Vec<String>) {
    match (protocol, permissions) {
        ("codex_app_server", "allow") => ensure_codex_approval_bypass(args),
        ("codex_app_server", "deny") => strip_codex_approval_bypass(args),
        ("claude_stream_json", "allow") => ensure_claude_permission_bypass(args),
        ("claude_stream_json", "deny") => strip_claude_permission_bypass(args),
        _ => {}
    }
}

fn ensure_codex_approval_bypass(args: &mut Vec<String>) {
    if args
        .iter()
        .any(|a| a == "--yolo" || a == "--dangerously-bypass-approvals-and-sandbox")
    {
        return;
    }
    let flag = "--dangerously-bypass-approvals-and-sandbox".to_string();
    if let Some(pos) = args.iter().position(|a| a == "app-server") {
        args.insert(pos, flag);
    } else {
        args.insert(0, flag);
    }
}

fn strip_codex_approval_bypass(args: &mut Vec<String>) {
    args.retain(|a| a != "--yolo" && a != "--dangerously-bypass-approvals-and-sandbox");
}

fn ensure_claude_permission_bypass(args: &mut Vec<String>) {
    let has_skip = args.iter().any(|a| {
        a == "--dangerously-skip-permissions" || a.contains("dangerously-skip-permissions")
    });
    let has_mode = args
        .iter()
        .any(|a| a == "bypassPermissions" || a.contains("bypassPermissions"));
    if !has_skip {
        args.push("--dangerously-skip-permissions".to_string());
    }
    if !has_mode {
        args.push("--permission-mode".to_string());
        args.push("bypassPermissions".to_string());
    }
}

const BRIDGET_SAFE_MCP_TOOLS: [&str; 30] = [
    "bridget_who",
    "bridget_specialists",
    "bridget_send",
    "bridget_ledger",
    "bridget_journal",
    "bridget_events",
    "bridget_thread",
    "bridget_handoff",
    "bridget_cancel",
    "bridget_read_artifact",
    "bridget_publish_artifact",
    "bridget_rename",
    "bridget_dnd",
    "bridget_domain",
    "bridget_runtime",
    "bridget_status",
    "bridget_control_status",
    "bridget_capabilities",
    "bridget_refresh_models",
    "bridget_read_commit_file",
    "bridget_delegate",
    "bridget_auto_delegate",
    "bridget_task_status",
    "bridget_task_cancel",
    "bridget_task_wait",
    "bridget_request_wait",
    "bridget_task_ack",
    "bridget_task_continue",
    "bridget_task_repair_workspace",
    "bridget_task_respond",
];

fn append_claude_allowed_tools(args: &mut Vec<String>) {
    args.push("--allowedTools".to_string());
    args.push(
        BRIDGET_SAFE_MCP_TOOLS
            .map(|name| format!("mcp__bridget__{name}"))
            .join(","),
    );
}

fn strip_claude_permission_bypass(args: &mut Vec<String>) {
    let mut cleaned = Vec::with_capacity(args.len());
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--dangerously-skip-permissions"
            || args[index].contains("dangerously-skip-permissions")
        {
            index += 1;
            continue;
        }
        if args[index] == "--permission-mode"
            && index + 1 < args.len()
            && (args[index + 1] == "bypassPermissions"
                || args[index + 1].contains("bypassPermissions"))
        {
            index += 2;
            continue;
        }
        if args[index] == "bypassPermissions" || args[index].contains("bypassPermissions") {
            index += 1;
            continue;
        }
        cleaned.push(args[index].clone());
        index += 1;
    }
    *args = cleaned;
}

fn mcp_server_entry() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    #[cfg(feature = "test-support")]
    if let Some(server) = smoke_mcp_server_entry()? {
        return Ok(server);
    }

    let command = std::env::current_exe()?.to_string_lossy().into_owned();
    let environment = mcp_server_environment()?;
    let environment = environment
        .as_object()
        .ok_or("environnement MCP absent")?
        .iter()
        .map(|(name, value)| serde_json::json!({"name": name, "value": value}))
        .collect::<Vec<_>>();
    Ok(serde_json::json!({
        "name": "bridget",
        "type": "stdio",
        "command": command,
        "args": ["mcp"],
        "env": environment
    }))
}

/// ACP utilise une liste nom/valeur ; les clients interactifs un objet.
/// Les trois projections portent le même namespace, sans changer HOME.
fn interactive_mcp_server_entry() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut server = mcp_server_entry()?;
    server["env"] = mcp_server_environment()?;
    Ok(server)
}

/// Codex peut filtrer l'environnement du serveur MCP qu'il lance. `HOME` est
/// la dépendance minimale et non sensible qui permet à cette projection de
/// retrouver la socket publique du daemon. Le namespace est transmis même
/// quand le fournisseur filtre son environnement.
fn mcp_server_environment() -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let home = std::env::var("HOME").map_err(|_| "HOME absent pour le serveur MCP Bridget")?;
    let namespace = crate::environment::Namespace::from_environment()?;
    Ok(serde_json::json!({
        "HOME": home,
        "BRIDGET_HOME": namespace.root,
        "BRIDGET_SOCKET": namespace.socket
    }))
}

/// Injection réservée au banc d'intégration T1006. Cette surface est absente
/// des builds distribués ; elle force le wrapper de production à construire
/// réellement les options Codex/Claude/ACP autour du serveur MCP épinglé.
#[cfg(feature = "test-support")]
fn smoke_mcp_server_entry() -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    let Ok(command) = std::env::var("BRIDGET_TEST_MCP_SERVER_COMMAND") else {
        return Ok(None);
    };
    let args = match std::env::var("BRIDGET_TEST_MCP_SERVER_ARGS") {
        Ok(value) => serde_json::from_str::<Vec<String>>(&value)?,
        Err(std::env::VarError::NotPresent) => Vec::new(),
        Err(error) => return Err(error.into()),
    };
    Ok(Some(serde_json::json!({
        "name": "bridget",
        "type": "stdio",
        "command": command,
        "args": args,
        "env": []
    })))
}

/// Configuration MCP Claude temporaire. Le fichier n'appartient jamais à la
/// configuration utilisateur : il vit sous le répertoire d'état Bridget et
/// le garde le retire quelle que soit l'issue du lancement.
struct EphemeralMcpConfig {
    path: PathBuf,
}

impl EphemeralMcpConfig {
    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for EphemeralMcpConfig {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_file(&self.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            warn!(
                "nettoyage de la configuration MCP éphémère impossible {}: {}",
                self.path.display(),
                error
            );
        }
    }
}

fn claude_mcp_config(
    server: &serde_json::Value,
    instance_id: &str,
) -> Result<EphemeralMcpConfig, Box<dyn std::error::Error>> {
    let socket = socket_path();
    let directory = socket.parent().ok_or("répertoire socket Bridget absent")?;
    claude_mcp_config_in(directory, server, instance_id)
}

fn claude_mcp_config_in(
    directory: &Path,
    server: &serde_json::Value,
    instance_id: &str,
) -> Result<EphemeralMcpConfig, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(directory)?;
    let path = directory.join(format!("mcp-{instance_id}.json"));
    // `mcpServers` de Claude Code n'est pas l'enveloppe ACP : il attend une
    // définition stdio indexée par son nom, sans le champ ACP `name`.
    let command = server["command"].as_str().ok_or("commande MCP absente")?;
    let args = server["args"].clone();
    let environment = server["env"]
        .as_object()
        .ok_or("environnement MCP absent")?;
    let bytes = serde_json::to_vec(&serde_json::json!({
        "mcpServers": {
            "bridget": {
                "type": "stdio",
                "command": command,
                "args": args,
                "env": environment
            }
        }
    }))?;
    bridget_transport::fsutil::write_private_file_atomic(&path, &bytes)?;
    Ok(EphemeralMcpConfig { path })
}

/// Retire les `mcp-*.json` orphelins laissés par un crash (Drop non exécuté).
pub(crate) fn purge_orphan_mcp_configs(directory: &Path) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.starts_with("mcp-")
            && name.ends_with(".json")
            && let Err(error) = std::fs::remove_file(entry.path())
        {
            warn!(
                "purge MCP orphelin impossible {}: {}",
                entry.path().display(),
                error
            );
        }
    }
}

fn codex_mcp_override(server: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let command = server["command"].as_str().ok_or("commande MCP absente")?;
    let environment = ["HOME", "BRIDGET_HOME", "BRIDGET_SOCKET"]
        .iter()
        .map(|key| {
            let value = server["env"][key]
                .as_str()
                .ok_or("namespace MCP incomplet")?;
            Ok(format!("{key}={}", serde_json::to_string(value)?))
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?
        .join(",");
    let environment = format!("{{{environment}}}");
    // Codex `auto` sollicite une approbation pour un outil sans annotations.
    // Avec approval_policy=never (équipier), cela interdit même bridget_who.
    // Le lancement Bridget autorise seulement la surface sûre et fermée,
    // pas tous les outils présents ou futurs du serveur (le service compagnon notamment).
    // `approve`, et non `auto`, est le mode explicite Codex 0.153.4.
    let tools = BRIDGET_SAFE_MCP_TOOLS
        .map(|name| format!("{name}={{approval_mode=\"approve\"}}"))
        .join(",");
    Ok(format!(
        "mcp_servers.bridget={{command={command:?},args=[\"mcp\"],env={environment},tool_timeout_sec=3700,default_tools_approval_mode=\"prompt\",tools={{{tools}}}}}"
    ))
}

fn validate_wrapper_args(arguments: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    for argument in arguments {
        // Rejeter les tentatives d'injection de commandes.
        if argument.contains(';')
            || argument.contains('&')
            || argument.contains('|')
            || argument.contains('$')
        {
            return Err(format!(
                "Argument non autorisé contient des caractères shell dangereux: '{}'",
                argument
            )
            .into());
        }
    }
    Ok(())
}

pub(crate) fn send_wrapper_message(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    message: WrapperToDaemon,
) {
    let Ok(json) = encode(&message) else {
        return;
    };
    let mut writer = writer.lock().recover_poison();
    let write_result = writer
        .as_mut()
        .map(|writer| writeln!(writer, "{}", json).and_then(|_| writer.flush()));
    if let Some(Err(error)) = write_result {
        warn!("envoi wrapper géré impossible: {}", error);
        // Fermer aussi le lecteur déclenche la reconnexion. Une écriture
        // partielle ne doit jamais être rejouée implicitement par BufWriter.
        if let Some(writer) = writer.as_ref() {
            let _ = writer.get_ref().shutdown(std::net::Shutdown::Both);
        }
        *writer = None;
    }
}

/// Consigne une livraison interactive dans le journal append-only. Le rendu
/// attach réutilise le même événement `turn_start` que les pilotes gérés.
pub(crate) fn record_interactive_turn(
    journal: &JournalWriter,
    message: &bridget_core::BridgetMessage,
) {
    if let Err(detail) = journal.enqueue(
        "turn_start",
        Some(&message.id),
        serde_json::json!({
            "from": &message.from,
            "reply": message.reply,
            "body": &message.body,
        }),
    ) {
        warn!("écriture journal interactif impossible: {detail}");
    }
}

#[allow(clippy::too_many_arguments)]
struct ReconnectManagedSessionArgs<'a> {
    pub socket: &'a std::path::Path,
    pub writer: &'a Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    pub transport: &'a dyn ManagedSession,
    pub descriptor: &'a ManagedSessionDescriptor,
    pub protocol: &'a str,
    pub agent_type: &'a str,
    pub name_state_path: &'a std::path::Path,
    pub host: &'a str,
    pub channel: Option<&'a str>,
    pub os: &'a str,
    pub instance_id: &'a str,
    pub fallback_name: &'a str,
    pub terminal_session: bool,
    pub termination: Option<&'a NativeTerminationSignal>,
    pub deadline: Option<Instant>,
}

fn reconnect_managed_session(
    args: ReconnectManagedSessionArgs<'_>,
) -> Option<(BufReader<UnixStream>, String)> {
    let ReconnectManagedSessionArgs {
        socket,
        writer,
        transport,
        descriptor,
        protocol,
        agent_type,
        name_state_path,
        host,
        channel,
        os,
        instance_id,
        fallback_name,
        terminal_session,
        termination,
        deadline,
    } = args;
    let expired = || deadline.is_some_and(|deadline| Instant::now() >= deadline);
    let mut attempts = 0_u32;
    while transport.is_alive()
        && !termination.is_some_and(NativeTerminationSignal::requested)
        && !expired()
    {
        // Une TUI peut se fermer pendant une panne prolongée du daemon.
        // Le backoff réseau ne doit pas retenir son terminal jusqu'à 36 s.
        let retry_at = Instant::now() + reconnect_delay(attempts);
        while Instant::now() < retry_at
            && transport.is_alive()
            && !termination.is_some_and(NativeTerminationSignal::requested)
            && !expired()
        {
            thread::sleep(
                retry_at
                    .saturating_duration_since(Instant::now())
                    .min(Duration::from_millis(50)),
            );
        }
        if !transport.is_alive()
            || termination.is_some_and(NativeTerminationSignal::requested)
            || expired()
        {
            return None;
        }
        attempts = attempts.saturating_add(1);
        let wanted_name = resolve_current_name(name_state_path, fallback_name);
        let busy = transport.is_busy();
        match connect_and_register_with_domain_at(ConnectAndRegisterWithDomainAtArgs {
            socket,
            agent_type,
            agent_id: Some(&wanted_name),
            host,
            protocol,
            channel,
            mode: descriptor.mode,
            location: descriptor.location.as_deref(),
            os,
            instance_id,
            turn_in_progress: busy,
            turn_attest: true,
        }) {
            Ok((reader, new_writer, registered_name)) if registered_name == wanted_name => {
                if termination.is_some_and(NativeTerminationSignal::requested)
                    || !transport.is_alive()
                    || expired()
                {
                    return None;
                }
                *writer.lock().recover_poison() = Some(new_writer);
                // Register réinitialise l'attestation au niveau du daemon : le
                // même journal local reste actif, il doit donc être annoncé de
                // nouveau sur la nouvelle connexion.
                send_wrapper_message(
                    writer,
                    WrapperToDaemon::JournalReady {
                        receipt_probe: true,
                    },
                );
                send_wrapper_message(writer, WrapperToDaemon::DaemonIdentityRequest);
                send_wrapper_message(
                    writer,
                    WrapperToDaemon::ObservationCapabilities {
                        runtime_catalogue: true,
                        events: vec![
                            bridget_transport::protocol::ObservationKind::TurnEnded,
                            bridget_transport::protocol::ObservationKind::PermissionRequired,
                            bridget_transport::protocol::ObservationKind::FileWritten,
                        ],
                    },
                );
                if terminal_session {
                    send_wrapper_message(writer, WrapperToDaemon::TerminalSessionReady);
                }
                // Même motif que JournalReady : le wrapper détient le fait
                // « tour ouvert » (is_busy). Register porte déjà
                // turn_in_progress, et TurnState le ré-atteste sur le writer
                // frais — un busy perdu au redémarrage daemon ne dépend plus
                // d'un prochain événement de tour.
                send_wrapper_message(writer, WrapperToDaemon::TurnState { in_progress: busy });
                return Some((reader, registered_name));
            }
            Ok((_, _, registered_name)) => warn!(
                "reconnexion session gérée refusée : nom inattendu « {} »",
                registered_name
            ),
            Err(error) => warn!(
                "reconnexion session gérée de « {} » impossible (tentative {}) : {}",
                wanted_name, attempts, error
            ),
        }
    }
    None
}

#[derive(Default)]
struct ManagedChildRequests {
    waiting: Vec<ManagedChildWaiting>,
    settled: std::collections::VecDeque<ManagedChildSettled>,
}

struct ManagedChildWaiting {
    request: bridget_transport::protocol::ChildProviderRequest,
    request_id: Option<String>,
    message_id: Option<String>,
    uncertain: bool,
}

struct ManagedChildSettled {
    request: bridget_transport::protocol::ChildProviderRequest,
    request_id: Option<String>,
    state: bridget_transport::protocol::ChildRequestClosedState,
    answer: Option<bridget_transport::protocol::ChildRequestAnswer>,
}

impl ManagedChildRequests {
    /// O(1), anneau limité à64 ; les attentes sont limitées à4.
    fn remember(&mut self, settled: ManagedChildSettled) {
        if self.settled.len() == 64 {
            self.settled.pop_front();
        }
        self.settled.push_back(settled);
    }
}

/// O(4+64+B), B réponse bornée. Le verrou métier couvre la validation,
/// l'écriture fournisseur et le règlement ; le writer IPC est pris après.
/// true demande l'arrêt : une écriture incertaine n'est jamais rejouée.
fn dispatch_managed_child_request(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    transport: &mut dyn ManagedSession,
    bindings: &mut HashMap<String, ManagedExecutionBinding>,
    requests: &Mutex<ManagedChildRequests>,
    message: DaemonToWrapper,
) -> bool {
    use bridget_transport::protocol::{
        ChildRequestAnswer, ChildRequestClosedState, ChildRequestDecision,
    };
    let (answer, cancel) = match message {
        DaemonToWrapper::ChildRequestAnswer { answer } => (answer, false),
        DaemonToWrapper::ChildRequestCancel {
            request_id,
            provider_session,
            turn_id,
            provider_request_id,
        } => (
            ChildRequestAnswer {
                request_id,
                provider_session,
                turn_id,
                provider_request_id,
                decision: ChildRequestDecision::Decline,
                answer: None,
                outcome: "cancelled".into(),
            },
            true,
        ),
        _ => return false,
    };
    let matches_request = |request: &bridget_transport::protocol::ChildProviderRequest| {
        request.provider_session == answer.provider_session
            && request.turn_id == answer.turn_id
            && request.provider_request_id == answer.provider_request_id
    };
    let mut requests = requests.lock().recover_poison();
    if let Some(index) = requests.settled.iter().position(|entry| {
        matches_request(&entry.request)
            && entry
                .request_id
                .as_deref()
                .is_none_or(|id| id == answer.request_id)
    }) {
        let entry = &mut requests.settled[index];
        entry.request_id = Some(answer.request_id.clone());
        let receipt = if cancel {
            WrapperToDaemon::ChildRequestClosed {
                request_id: answer.request_id,
                state: entry.state,
            }
        } else {
            let applied = entry.state == ChildRequestClosedState::AlreadyApplied
                && entry.answer.as_ref() == Some(&answer);
            WrapperToDaemon::ChildRequestAnswered {
                request_id: answer.request_id,
                applied,
                reason: if applied {
                    None
                } else {
                    Some("request_closed".into())
                },
            }
        };
        drop(requests);
        send_wrapper_message(writer, receipt);
        return false;
    }
    let index = requests.waiting.iter().position(|entry| {
        matches_request(&entry.request)
            && entry
                .request_id
                .as_deref()
                .is_none_or(|id| id == answer.request_id)
    });
    let Some(index) = index else {
        drop(requests);
        if !cancel {
            send_wrapper_message(
                writer,
                WrapperToDaemon::ChildRequestAnswered {
                    request_id: answer.request_id,
                    applied: false,
                    reason: Some("request_unknown".into()),
                },
            );
        }
        return false; // Une fermeture inconnue n'est jamais confirmée.
    };
    if requests.waiting[index].uncertain {
        return true;
    }
    requests.waiting[index].request_id = Some(answer.request_id.clone());
    match transport.answer_provider_request(&answer) {
        Ok(()) => {
            let entry = requests.waiting.remove(index);
            let resume = entry.message_id.filter(|message_id| {
                !requests
                    .waiting
                    .iter()
                    .any(|pending| pending.message_id.as_ref() == Some(message_id))
            });
            let state = if cancel {
                ChildRequestClosedState::ClosedBeforeApply
            } else {
                ChildRequestClosedState::AlreadyApplied
            };
            requests.remember(ManagedChildSettled {
                request: entry.request,
                request_id: Some(answer.request_id.clone()),
                state,
                answer: if cancel { None } else { Some(answer.clone()) },
            });
            drop(requests);
            send_wrapper_message(
                writer,
                if cancel {
                    WrapperToDaemon::ChildRequestClosed {
                        request_id: answer.request_id,
                        state,
                    }
                } else {
                    WrapperToDaemon::ChildRequestAnswered {
                        request_id: answer.request_id,
                        applied: true,
                        reason: None,
                    }
                },
            );
            if let Some(message_id) = resume {
                publish_execution_transition(
                    writer,
                    bindings,
                    &message_id,
                    "running",
                    "child_request_settled",
                );
            }
            false
        }
        Err(bridget_transport::TransportError::Io(detail))
            if detail.starts_with("child_request_write_uncertain:") =>
        {
            requests.waiting[index].uncertain = true;
            warn!("réponse fournisseur incertaine : arrêt de la session sans accusé");
            true
        }
        Err(error) => {
            drop(requests);
            if !cancel {
                send_wrapper_message(
                    writer,
                    WrapperToDaemon::ChildRequestAnswered {
                        request_id: answer.request_id,
                        applied: false,
                        reason: Some(error.to_string()),
                    },
                );
            }
            false
        }
    }
}

/// Corrélation locale entre une remise admise par Bridget et les événements du
/// fournisseur. Le message demeure la clé de bord : aucun identifiant interne
/// du fournisseur ne peut solder une autre exécution.
#[derive(Debug, Clone)]
struct ManagedExecutionBinding {
    execution_id: String,
    provider_kind: String,
    execution_path: String,
    generation: u64,
    state: String,
    revision: u64,
    approval_requests: u8,
    last_approval_request: Option<String>,
    remote_result: Option<crate::remote_worker::WorkerResultReservation>,
}

fn bind_idempotent_delivery_execution(
    bindings: &mut HashMap<String, ManagedExecutionBinding>,
    message_id: &str,
    execution: Option<ExecutionDeliveryContext>,
    provider_kind: &str,
    execution_path: &str,
) -> bool {
    let Some(execution) = execution else {
        return false;
    };
    bindings
        .entry(message_id.to_string())
        .or_insert(ManagedExecutionBinding {
            execution_id: execution.execution_id,
            provider_kind: provider_kind.to_string(),
            execution_path: execution_path.to_string(),
            generation: execution.generation,
            state: "starting".to_string(),
            revision: execution.revision,
            approval_requests: 0,
            last_approval_request: None,
            remote_result: None,
        });
    true
}

/// L usage ne reçoit une ascendance que si le wrapper ne porte qu un seul
/// tour vivant. Plusieurs bindings rendent l attribution ambiguë : le fait
/// reste visible par agent, mais ne peut pas consommer un budget d exécution.
fn usage_execution_context(
    bindings: &HashMap<String, ManagedExecutionBinding>,
) -> (Option<String>, Option<u64>, Option<String>) {
    if bindings.len() != 1 {
        return (None, None, None);
    }
    let binding = bindings.values().next().expect("binding unique");
    (
        Some(binding.execution_id.clone()),
        Some(binding.generation),
        Some(binding.provider_kind.clone()),
    )
}
/// Publie uniquement une identité fournisseur réellement observée, corrélée aux
/// exécutions vivantes. L'absence de baseline reste une absence durable : elle
/// n'est jamais remplacée par une capacité déduite du type d'agent.
fn publish_provider_context(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    bindings: &HashMap<String, ManagedExecutionBinding>,
    identity: &ManagedProviderIdentity,
) {
    let Some(observation) = identity.provider_observation.clone() else {
        return;
    };
    for binding in bindings.values() {
        send_wrapper_message(
            writer,
            WrapperToDaemon::ExecutionProviderObserved {
                context: ExecutionProviderContext {
                    execution_id: binding.execution_id.clone(),
                    generation: binding.generation,
                    provider_kind: binding.provider_kind.clone(),
                    execution_path: binding.execution_path.clone(),
                    observation: observation.clone(),
                    provider_session_id: identity.provider_session_id.clone(),
                    provider_thread_id: identity.provider_thread_id.clone(),
                    provider_turn_id: identity.active_turn_id.clone(),
                    observed_at: unix_now_secs(),
                },
            },
        );
    }
}

/// T036 : lookup HashMap moyen O(M), copie O(E), M/E octets des identifiants.
/// Capturer avant une transition terminale ; une absence reste une absence.
fn bound_delivery_rejection(
    bindings: &HashMap<String, ManagedExecutionBinding>,
    id: String,
    reason: String,
) -> WrapperToDaemon {
    let binding = bindings.get(&id);
    WrapperToDaemon::DeliveryRejected {
        execution_id: binding.map(|binding| binding.execution_id.clone()),
        generation: binding.map(|binding| binding.generation),
        id,
        reason,
    }
}

fn publish_execution_transition(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    bindings: &mut HashMap<String, ManagedExecutionBinding>,
    message_id: &str,
    next_state: &str,
    reason: &str,
) {
    let Some(binding) = bindings.get_mut(message_id) else {
        return;
    };
    if binding.state == next_state {
        return;
    }
    let transition = bridget_transport::protocol::ExecutionStateTransition {
        execution_id: binding.execution_id.clone(),
        generation: binding.generation,
        expected_state: binding.state.clone(),
        expected_revision: binding.revision,
        next_state: next_state.to_string(),
        reason: reason.to_string(),
        observed_at: unix_now_secs(),
    };
    send_wrapper_message(
        writer,
        WrapperToDaemon::ExecutionStateChanged { transition },
    );
    binding.state = next_state.to_string();
    binding.revision = binding.revision.saturating_add(1);
    if matches!(
        next_state,
        "interrupted" | "completed" | "failed" | "unreachable"
    ) && binding.remote_result.is_none()
    {
        // Garder la réservation terminée empêche un événement tardif de
        // devenir une réponse Send locale, même après arrêt du tour.
        bindings.remove(message_id);
    }
}

fn report_execution_control(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    issuer_scope: String,
    command: &ExecutionControlCommand,
    accepted: bool,
    refusal_reason: Option<String>,
) {
    send_wrapper_message(
        writer,
        WrapperToDaemon::ControlExecutionReported {
            issuer_scope,
            command_id: command.command_id.clone(),
            execution_id: command.execution_id.clone(),
            accepted,
            refusal_reason,
        },
    );
}

/// Exécute seulement les contrôles explicitement supportés par la session.
/// Une correction sans capacité native est refusée ici, jamais placée dans la
/// FIFO comme un nouveau prompt.
fn apply_execution_control(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    transport: &mut dyn ManagedSession,
    bindings: &mut HashMap<String, ManagedExecutionBinding>,
    issuer_scope: String,
    command: ExecutionControlCommand,
) {
    let binding = bindings.iter().find_map(|(message_id, binding)| {
        (binding.execution_id == command.execution_id)
            .then(|| (message_id.clone(), binding.clone()))
    });
    let Some((message_id, binding)) = binding else {
        return report_execution_control(
            writer,
            issuer_scope,
            &command,
            false,
            Some("execution_not_found".to_string()),
        );
    };
    if binding.generation != command.generation {
        return report_execution_control(
            writer,
            issuer_scope,
            &command,
            false,
            Some("generation_mismatch".to_string()),
        );
    }
    if binding.revision != command.revision {
        return report_execution_control(
            writer,
            issuer_scope,
            &command,
            false,
            Some("revision_mismatch".to_string()),
        );
    }
    let required_operation = match command.operation {
        ExecutionControlOperation::SteerCurrent => ProviderOperation::Steer,
        ExecutionControlOperation::Interrupt => ProviderOperation::Interrupt,
        ExecutionControlOperation::QueueOnly
        | ExecutionControlOperation::TriggerTurn
        | ExecutionControlOperation::PauseQueue
        | ExecutionControlOperation::ResumeQueue
        | ExecutionControlOperation::CancelQueued => {
            return report_execution_control(
                writer,
                issuer_scope,
                &command,
                false,
                Some("capability_unavailable".to_string()),
            );
        }
    };
    if !transport.supports_operation(required_operation) {
        return report_execution_control(
            writer,
            issuer_scope,
            &command,
            false,
            Some("capability_unavailable".to_string()),
        );
    }
    match command.operation {
        ExecutionControlOperation::SteerCurrent => {
            let Some(message) = command.message.as_ref() else {
                return report_execution_control(
                    writer,
                    issuer_scope,
                    &command,
                    false,
                    Some("message_required".to_string()),
                );
            };
            match transport.deliver(message) {
                Ok(()) => report_execution_control(writer, issuer_scope, &command, true, None),
                Err(_) => report_execution_control(
                    writer,
                    issuer_scope,
                    &command,
                    false,
                    Some("target_unavailable".to_string()),
                ),
            }
        }
        ExecutionControlOperation::Interrupt => {
            if transport.cancel_delivery(&message_id, "interruption explicite") {
                publish_execution_transition(
                    writer,
                    bindings,
                    &message_id,
                    "interrupting",
                    "interrupt_requested",
                );
                report_execution_control(writer, issuer_scope, &command, true, None);
            } else {
                report_execution_control(
                    writer,
                    issuer_scope,
                    &command,
                    false,
                    Some("target_unavailable".to_string()),
                );
            }
        }
        ExecutionControlOperation::QueueOnly
        | ExecutionControlOperation::TriggerTurn
        | ExecutionControlOperation::PauseQueue
        | ExecutionControlOperation::ResumeQueue
        | ExecutionControlOperation::CancelQueued => report_execution_control(
            writer,
            issuer_scope,
            &command,
            false,
            Some("capability_unavailable".to_string()),
        ),
    }
}

fn publish_single_wait(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    bindings: &mut HashMap<String, ManagedExecutionBinding>,
    next_state: &str,
    reason: &str,
) {
    let message_ids: Vec<String> = bindings
        .iter()
        .filter(|(_, binding)| binding.state == "running")
        .map(|(message_id, _)| message_id.clone())
        .collect();
    if message_ids.len() == 1 {
        publish_execution_transition(writer, bindings, &message_ids[0], next_state, reason);
    }
}

const MAX_APPROVAL_REQUESTS_PER_EXECUTION: u8 = 3;

/// Corrèle et borne les demandes d'autorisation tant que le même tour reste
/// actif. Bridget ne choisit jamais à la place de l'humain : au-delà du seuil,
/// elle publie une issue durable que le contrôleur peut reprendre.
fn publish_approval_wait(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    bindings: &mut HashMap<String, ManagedExecutionBinding>,
    request_id: &str,
) {
    let message_ids: Vec<String> = bindings
        .iter()
        .filter(|(_, binding)| matches!(binding.state.as_str(), "running" | "waiting_approval"))
        .map(|(message_id, _)| message_id.clone())
        .collect();
    if message_ids.len() != 1 {
        return;
    }
    let message_id = &message_ids[0];
    let loop_detected = {
        let binding = bindings
            .get_mut(message_id)
            .expect("identifiant sélectionné depuis les bindings");
        binding.approval_requests = binding.approval_requests.saturating_add(1);
        binding.last_approval_request = Some(request_id.to_string());
        binding.approval_requests > MAX_APPROVAL_REQUESTS_PER_EXECUTION
    };
    if loop_detected {
        publish_execution_transition(
            writer,
            bindings,
            message_id,
            "failed",
            "approval_loop_detected",
        );
    } else {
        publish_execution_transition(
            writer,
            bindings,
            message_id,
            "waiting_approval",
            "permission_required",
        );
    }
}
fn delegated_runtime_terminal_reference(execution_id: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"bridget/delegated-runtime-terminal/v1\0");
    hasher.update(execution_id.as_bytes());
    format!("sha256:{:x}", hasher.finalize())
}

fn publish_delegated_runtime_for_message(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    bindings: &HashMap<String, ManagedExecutionBinding>,
    message_id: &str,
    kind: DelegatedRuntimeEventKind,
    code: &str,
    reference: String,
) {
    let Some(binding) = bindings.get(message_id) else {
        return;
    };
    send_wrapper_message(
        writer,
        WrapperToDaemon::DelegatedRuntimeEvent {
            execution_id: binding.execution_id.clone(),
            kind,
            code: code.to_string(),
            reference,
        },
    );
}

fn publish_delegated_runtime_warning(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    bindings: &HashMap<String, ManagedExecutionBinding>,
    code: String,
    reference: String,
) {
    let running: Vec<&ManagedExecutionBinding> = bindings
        .values()
        .filter(|binding| binding.state == "running")
        .collect();
    if running.len() == 1 {
        send_wrapper_message(
            writer,
            WrapperToDaemon::DelegatedRuntimeEvent {
                execution_id: running[0].execution_id.clone(),
                kind: DelegatedRuntimeEventKind::Warning,
                code,
                reference,
            },
        );
    }
}
fn delegated_runtime_message(
    event: &DelegatedRuntimeEventFrame,
    recipient: &str,
) -> bridget_core::BridgetMessage {
    let kind = match event.kind {
        DelegatedRuntimeEventKind::Warning => "avertissement récupérable",
        DelegatedRuntimeEventKind::Failed => "échec terminal",
    };
    let body = format!(
        "Incident délégué: enfant={} issue={kind} code={} référence={}.",
        event.child_instance_id, event.code, event.reference,
    );
    let mut message = bridget_core::BridgetMessage::new("bridget", recipient, body);
    message.id = format!("delegated-runtime-{}", event.event_id);
    message.origin = Some(bridget_core::MessageOrigin::System);
    message.intent = Some(bridget_core::MessageIntent::QueueOnly);
    message.reply = false;
    message
}

/// Le fournisseur reçoit toujours l'identifiant routable de l'émetteur.
///
/// Un nom d'affichage est fait pour l'écran, pas pour l'adressage. Le projeter
/// dans `from` faisait répondre le fournisseur à « Utilisateur » au lieu de
/// l'identité réelle `humain`, puis le daemon perdait cette réponse.
fn message_for_provider(message: &bridget_core::BridgetMessage) -> bridget_core::BridgetMessage {
    message.clone()
}

#[cfg(test)]
mod transcript_journal_tests {
    use super::*;

    #[test]
    fn saisie_humaine_texte_assistant_et_fin_de_tour_sont_traduits() {
        let user = r#"{"type":"user","uuid":"u1","message":{"role":"user","content":"Question de l'humain"}}"#;
        let bridget = r#"{"type":"user","uuid":"u2","message":{"role":"user","content":[{"type":"text","text":"💬 a → b (reply=no, id=x)\ncorps"}]}}"#;
        let tool = r#"{"type":"user","uuid":"u3","message":{"role":"user","content":[{"type":"tool_result","tool_use_id":"t","content":"sortie"}]}}"#;
        let assistant = r#"{"type":"assistant","uuid":"a1","message":{"role":"assistant","model":"m","stop_reason":"end_turn","content":[{"type":"text","text":"Réponse"},{"type":"tool_use","id":"t","name":"Read","input":{}}]}}"#;
        let sidechain = r#"{"type":"assistant","isSidechain":true,"uuid":"a2","message":{"role":"assistant","content":[{"type":"text","text":"sous-agent"}]}}"#;
        let human = claude_transcript_journal_events(user);
        assert_eq!(human.len(), 1);
        assert_eq!(human[0].0, "turn_start");
        assert_eq!(human[0].1.as_deref(), Some("u1"));
        assert_eq!(human[0].2["from"], "human");
        assert_eq!(human[0].2["body"], "Question de l'humain");
        assert!(
            claude_transcript_journal_events(bridget).is_empty(),
            "remise Bridget déjà journalisée"
        );
        assert!(
            claude_transcript_journal_events(tool).is_empty(),
            "résultat d'outil : pas un tour"
        );
        let answer = claude_transcript_journal_events(assistant);
        assert_eq!(answer.len(), 2);
        assert_eq!(answer[0].0, "update");
        assert_eq!(answer[0].2["kind"], "text");
        assert_eq!(answer[0].2["text"], "Réponse");
        assert_eq!(answer[1].0, "turn_end");
        assert!(claude_transcript_journal_events(sidechain).is_empty());
        assert!(claude_transcript_journal_events("pas du json").is_empty());
    }

    #[test]
    fn hors_reprise_un_voisin_actif_n_est_jamais_epingle() {
        // Cas réel du 2026-09-13 : une autre session Claude du même projet
        // écrit en continu ; notre transcript n'existe pas encore au premier
        // tick. Sans cette règle, le journal relayait la conversation du voisin.
        let root = std::env::temp_dir().join(format!("bridget-097-voisin-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let neighbour = root.join("voisin.jsonl");
        let line = |id: &str, body: &str| {
            format!(
                "{{\"type\":\"user\",\"uuid\":\"{id}\",\"message\":{{\"content\":\"{body}\"}}}}
"
            )
        };
        std::fs::write(&neighbour, line("v1", "voisin")).unwrap();
        let mut locator = ClaudeTranscriptLocator::new(root.clone());
        let probe = RuntimeProbe::claude(ClaudeTranscriptLocator::new(root.clone()));
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&neighbour, line("v2", "voisin encore")).unwrap();
        assert_eq!(
            locator.resolve(),
            None,
            "un préexistant modifié n'est pas le nôtre"
        );
        let ours = root.join("notre.jsonl");
        std::fs::write(&ours, line("n1", "nous")).unwrap();
        assert_eq!(locator.resolve(), Some(ours.clone()));
        assert_eq!(
            probe.journal_start_for(Some(&ours)),
            0,
            "né après : relu depuis le début"
        );
        // Reprise explicite : le préexistant est admis, son historique n'est pas rejoué.
        let mut resumed = ClaudeTranscriptLocator::new(root.clone()).with_preexisting_allowed();
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&neighbour, line("v3", "reprise")).unwrap();
        assert_eq!(resumed.resolve(), Some(neighbour.clone()));
        let probe = RuntimeProbe::claude(
            ClaudeTranscriptLocator::new(root.clone()).with_preexisting_allowed(),
        );
        assert_eq!(
            probe.journal_start_for(Some(&neighbour)),
            std::fs::metadata(&neighbour).unwrap().len()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn deux_sessions_simultanees_ne_partagent_jamais_un_transcript() {
        // Deux `bridget claude` lancés ensemble dans le même projet : chacun
        // attend le transcript de SON identifiant de session, jamais le
        // premier fichier né (règle qui confondait les sessions).
        let root = std::env::temp_dir().join(format!("bridget-097-deux-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let mut a = ClaudeTranscriptLocator::new(root.clone()).expecting(root.join("aaaa.jsonl"));
        let mut b = ClaudeTranscriptLocator::new(root.clone()).expecting(root.join("bbbb.jsonl"));
        std::fs::write(
            root.join("bbbb.jsonl"),
            "{}
",
        )
        .unwrap();
        assert_eq!(
            a.resolve(),
            None,
            "le transcript de l'autre session n'est pas le nôtre"
        );
        assert_eq!(b.resolve(), Some(root.join("bbbb.jsonl")));
        std::fs::write(
            root.join("aaaa.jsonl"),
            "{}
",
        )
        .unwrap();
        assert_eq!(a.resolve(), Some(root.join("aaaa.jsonl")));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn echec_d_injection_remonte_un_sort_indetermine_sans_accuse() {
        let root =
            std::env::temp_dir().join(format!("bridget-097-indetermine-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let instance_id = "instance_097_interactive";
        let mut tracker = IdempotentDeliveryTracker::open_at(&root, instance_id).unwrap();
        let message = bridget_core::BridgetMessage::new("expediteur", "destinataire", "corps");
        let reports = deliver_idempotent_to_interactive(
            &mut tracker,
            "delivery-097".to_string(),
            instance_id.to_string(),
            7,
            500,
            message,
            100,
            |_| Err("maître PTY non accueillant".to_string()),
        );
        assert!(
            matches!(
                reports.as_slice(),
                [WrapperToDaemon::DeliveryIndeterminate { delivery_id, .. }] if delivery_id == "delivery-097"
            ),
            "un échec d'écriture PTY ne vaut jamais un accusé : {reports:?}"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn le_drainage_ne_consomme_que_les_lignes_completes() {
        let root = std::env::temp_dir().join(format!("bridget-097-journal-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("session.jsonl");
        std::fs::write(&path, "").unwrap();
        let mut probe = RuntimeProbe::claude(ClaudeTranscriptLocator::new(root.clone()));
        probe.path = Some(path.clone());
        std::fs::write(
            &path,
            r#"{"type":"user","uuid":"u1","message":{"content":"a"}}"#,
        )
        .unwrap();
        assert!(
            probe.drain_transcript_journal().is_empty(),
            "ligne incomplète attendue"
        );
        std::fs::write(
            &path,
            format!(
                "{}\n{{\"type\":\"user\",\"uuid\":\"u2\",\"message\":{{\"content\":\"b\"}}}}",
                r#"{"type":"user","uuid":"u1","message":{"content":"a"}}"#
            ),
        )
        .unwrap();
        let first = probe.drain_transcript_journal();
        assert_eq!(
            first
                .iter()
                .map(|e| e.1.clone().unwrap())
                .collect::<Vec<_>>(),
            vec!["u1"]
        );
        std::fs::write(
            &path,
            format!(
                "{}\n{{\"type\":\"user\",\"uuid\":\"u2\",\"message\":{{\"content\":\"b\"}}}}\n",
                r#"{"type":"user","uuid":"u1","message":{"content":"a"}}"#
            ),
        )
        .unwrap();
        let second = probe.drain_transcript_journal();
        assert_eq!(
            second
                .iter()
                .map(|e| e.1.clone().unwrap())
                .collect::<Vec<_>>(),
            vec!["u2"]
        );
        assert!(probe.drain_transcript_journal().is_empty());
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[cfg(test)]
#[test]
fn projection_fournisseur_conserve_l_identite_routable() {
    let agent_id = "018f2d95-8bd4-7c4c-8b7e-6cafb0a39a63";
    let mut message = bridget_core::BridgetMessage::new(
        agent_id,
        "018f2d95-8bd4-7c4c-8b7e-6cafb0a39a64",
        "bonjour",
    );
    message.from_display_name = Some("Bibliothécaire".to_string());

    let projected = message_for_provider(&message);

    assert_eq!(message.from, agent_id, "le routage conserve l'Agent ID");
    assert_eq!(projected.from, agent_id);
    assert_eq!(projected.to, message.to);
    assert_eq!(projected.id, message.id);
}

// O(1), minuterie existante rendue testable ; aucune file ou scheduler ajouté.
fn reattest_managed_turn_if_due(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    session: &dyn ManagedSession,
    last_heartbeat: &mut Instant,
    now: Instant,
) {
    if now.saturating_duration_since(*last_heartbeat) >= HEARTBEAT_INTERVAL {
        send_wrapper_message(writer, WrapperToDaemon::Heartbeat);
        send_wrapper_message(
            writer,
            WrapperToDaemon::TurnState {
                in_progress: session.is_busy(),
            },
        );
        *last_heartbeat = now;
    }
}

#[cfg(test)]
fn forward_managed_events(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    my_name: &str,
    events: Vec<ManagedEvent>,
    idempotent_deliveries: &mut IdempotentDeliveryTracker,
    bindings: &mut HashMap<String, ManagedExecutionBinding>,
) -> bool {
    forward_managed_events_with_redaction(
        writer,
        my_name,
        events,
        idempotent_deliveries,
        bindings,
        &mut None,
        None,
        true,
        None,
        None,
    )
}

fn redact_managed_text(
    redaction: &mut Option<OutputRedactionLease>,
    channel: &str,
    value: String,
) -> String {
    let Some(lease) = redaction.as_mut() else {
        return value;
    };
    String::from_utf8_lossy(&lease.redact(channel, value.as_bytes(), true)).into_owned()
}

#[allow(clippy::too_many_arguments)]
fn forward_managed_events_with_redaction(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    my_name: &str,
    events: Vec<ManagedEvent>,
    idempotent_deliveries: &mut IdempotentDeliveryTracker,
    bindings: &mut HashMap<String, ManagedExecutionBinding>,
    redaction: &mut Option<OutputRedactionLease>,
    profile_db_path: Option<&Path>,
    automatic_reply: bool,
    child_requests: Option<&Mutex<ManagedChildRequests>>,
    current_turn_in_progress: Option<bool>,
) -> bool {
    let mut journal_failed = false;
    for event in events {
        let ManagedEvent {
            source,
            origin,
            raw,
            kind,
        } = event;
        // La frontière commune transporte les octets d'origine jusqu'ici. Le
        // wrapper n'en déduit pas une nouvelle sémantique, mais atteste leur
        // réception avec leur provenance : un pilote inconnu ne peut donc pas
        // être réduit silencieusement à une erreur interne.
        log::debug!(
            "événement session gérée observé: source={source:?}, origine={origin:?}, octets={}",
            raw.len()
        );
        match kind {
            ManagedEventKind::ActivityObserved { in_progress } => {
                send_wrapper_message(writer, WrapperToDaemon::TurnState { in_progress });
            }
            ManagedEventKind::TurnStarted { message_id } => {
                if (!bindings.is_empty() || idempotent_deliveries.requires_binding)
                    && !bindings.contains_key(&message_id)
                    && !idempotent_deliveries.ordinary_active(&message_id)
                {
                    warn!("événement de tour ignoré: message hors exécution active");
                    continue;
                }
                if let Some(binding) = bindings.get_mut(&message_id) {
                    binding.approval_requests = 0;
                    binding.last_approval_request = None;
                }
                if automatic_reply {
                    send_wrapper_message(writer, WrapperToDaemon::TurnState { in_progress: true });
                }
                publish_execution_transition(
                    writer,
                    bindings,
                    &message_id,
                    "running",
                    "provider_accepted",
                );
            }
            ManagedEventKind::TurnFinished {
                message,
                mut response,
                mut terminal,
            } => {
                response = redact_managed_text(redaction, "provider_response", response);
                if let ManagedTerminal::Failed { detail } = &mut terminal {
                    *detail =
                        redact_managed_text(redaction, "provider_error", std::mem::take(detail));
                }
                let message_id = message.id.clone();
                if (!bindings.is_empty() || idempotent_deliveries.requires_binding)
                    && !bindings.contains_key(&message_id)
                    && !idempotent_deliveries.ordinary_active(&message_id)
                {
                    warn!("terminal fournisseur ignoré: message hors exécution active");
                    continue;
                }
                let remote_published = if matches!(&terminal, ManagedTerminal::Completed)
                    && automatic_reply
                    && message.reply
                    && !response.is_empty()
                {
                    match publish_remote_final(bindings.get(&message_id), &message, &response) {
                        Ok(published) => published,
                        Err(error) => {
                            warn!("{error}");
                            return true;
                        }
                    }
                } else {
                    false
                };
                if let Some(requests) = child_requests {
                    let mut requests = requests.lock().recover_poison();
                    let mut closed = Vec::new();
                    let mut index = 0;
                    while index < requests.waiting.len() {
                        let entry = &requests.waiting[index];
                        if entry.message_id.as_ref().is_none_or(|id| id == &message_id)
                            && !entry.uncertain
                        {
                            let entry = requests.waiting.remove(index);
                            if let Some(id) = &entry.request_id {
                                closed.push(id.clone());
                            }
                            requests.remember(ManagedChildSettled { request:entry.request,request_id:entry.request_id,
                                state:bridget_transport::protocol::ChildRequestClosedState::ClosedBeforeApply,answer:None });
                        } else {
                            index += 1;
                        }
                    }
                    drop(requests);
                    for request_id in closed {
                        send_wrapper_message(writer,WrapperToDaemon::ChildRequestClosed {request_id,state:bridget_transport::protocol::ChildRequestClosedState::ClosedBeforeApply});
                    }
                }
                if (!bindings.is_empty() || idempotent_deliveries.requires_binding)
                    && !bindings.contains_key(&message_id)
                    && !idempotent_deliveries.ordinary_active(&message_id)
                {
                    warn!("terminal fournisseur ignoré: message hors exécution active");
                    continue;
                }
                if automatic_reply {
                    send_wrapper_message(
                        writer,
                        WrapperToDaemon::TurnState {
                            in_progress: current_turn_in_progress.unwrap_or(false),
                        },
                    );
                }

                if message.reply && message.origin != Some(bridget_core::MessageOrigin::System) {
                    match &terminal {
                        ManagedTerminal::Completed => record_attention(
                            profile_db_path,
                            my_name,
                            AttentionEventType::TaskCompleted,
                            &message.id,
                        ),
                        ManagedTerminal::Failed { .. } => record_attention(
                            profile_db_path,
                            my_name,
                            AttentionEventType::TerminalFailure,
                            &message.id,
                        ),
                        ManagedTerminal::Cancelled => {}
                    }
                }

                if matches!(&terminal, ManagedTerminal::Failed { .. }) {
                    publish_delegated_runtime_for_message(
                        writer,
                        bindings,
                        &message_id,
                        DelegatedRuntimeEventKind::Failed,
                        "provider_failed",
                        delegated_runtime_terminal_reference(
                            bindings
                                .get(&message_id)
                                .map(|binding| binding.execution_id.as_str())
                                .unwrap_or_default(),
                        ),
                    );
                }
                let rejection = match &terminal {
                    ManagedTerminal::Completed
                        if automatic_reply && message.reply && response.is_empty() =>
                    {
                        Some(bound_delivery_rejection(
                            bindings,
                            message_id.clone(),
                            "réponse vide".to_string(),
                        ))
                    }
                    ManagedTerminal::Cancelled => Some(bound_delivery_rejection(
                        bindings,
                        message_id.clone(),
                        "tour annulé par le pilote".to_string(),
                    )),
                    ManagedTerminal::Failed { detail } => Some(bound_delivery_rejection(
                        bindings,
                        message_id.clone(),
                        detail.clone(),
                    )),
                    _ => None,
                };
                let (next_state, reason) = match &terminal {
                    ManagedTerminal::Completed => ("completed", "completed"),
                    ManagedTerminal::Cancelled => ("interrupted", "interrupted"),
                    ManagedTerminal::Failed { .. } => ("failed", "provider_failed"),
                };
                publish_execution_transition(writer, bindings, &message_id, next_state, reason);
                match terminal {
                    ManagedTerminal::Completed
                        if automatic_reply
                            && message.reply
                            && !response.is_empty()
                            && !remote_published =>
                    {
                        let mut reply =
                            bridget_core::BridgetMessage::new(my_name, &message.from, response);
                        reply.in_reply_to = Some(message.id.clone());
                        if idempotent_deliveries.modern_reply_core {
                            match idempotent_deliveries.retain_reply(&message, reply) {
                                Ok(reply) => {
                                    send_wrapper_message(writer, WrapperToDaemon::Send(reply))
                                }
                                Err(error) => {
                                    error!(
                                        "réponse non déposée, conservation impossible : {error}"
                                    );
                                    return true;
                                }
                            }
                        } else {
                            send_wrapper_message(writer, WrapperToDaemon::Send(reply));
                        }
                    }
                    _ => {}
                }
                if let Some(rejection) = rejection {
                    send_wrapper_message(writer, rejection);
                }
                idempotent_deliveries.finish_ordinary(&message_id);
            }
            ManagedEventKind::DeliveryRejected { message_id, reason } => {
                let reason = redact_managed_text(redaction, "provider_rejection", reason);
                if (!bindings.is_empty() || idempotent_deliveries.requires_binding)
                    && !bindings.contains_key(&message_id)
                    && !idempotent_deliveries.ordinary_active(&message_id)
                {
                    warn!("refus fournisseur ignoré: message hors exécution active");
                    continue;
                }
                send_wrapper_message(
                    writer,
                    WrapperToDaemon::TurnState {
                        in_progress: current_turn_in_progress.unwrap_or(false),
                    },
                );
                let rejection =
                    bound_delivery_rejection(bindings, message_id.clone(), reason.clone());
                let execution_reason = if reason.ends_with("pleine") {
                    publish_delegated_runtime_for_message(
                        writer,
                        bindings,
                        &message_id,
                        DelegatedRuntimeEventKind::Failed,
                        "provider_failed",
                        delegated_runtime_terminal_reference(
                            bindings
                                .get(&message_id)
                                .map(|binding| binding.execution_id.as_str())
                                .unwrap_or_default(),
                        ),
                    );

                    "provider_queue_full"
                } else {
                    "provider_failed"
                };
                publish_execution_transition(
                    writer,
                    bindings,
                    &message_id,
                    "failed",
                    execution_reason,
                );
                if let Some(report) = idempotent_deliveries.injection_rejected(&message_id) {
                    send_wrapper_message(writer, report);
                }
                send_wrapper_message(writer, rejection);
                idempotent_deliveries.finish_ordinary(&message_id);
            }
            ManagedEventKind::JournalFailed { detail } => {
                journal_failed = true;
                let detail = redact_managed_text(redaction, "provider_journal_error", detail);
                warn!("arrêt de la session gérée : {detail}");
            }
            ManagedEventKind::PromptDispatched { message_id } => {
                if let Some(report) =
                    idempotent_deliveries.prompt_dispatched(&message_id, unix_now_secs())
                {
                    send_wrapper_message(writer, report);
                }
            }
            ManagedEventKind::RuntimeObserved {
                mut model,
                mut effort,
            } => {
                model = redact_managed_text(redaction, "provider_runtime_model", model);
                effort = effort
                    .map(|value| redact_managed_text(redaction, "provider_runtime_effort", value));
                match source {
                    bridget_transport::ManagedEventSource::CodexAppServer => send_wrapper_message(
                        writer,
                        WrapperToDaemon::Runtime {
                            agent: my_name.to_string(),
                            model,
                            effort,
                            source: bridget_transport::protocol::RuntimeSource::CodexAppServer,
                        },
                    ),
                    bridget_transport::ManagedEventSource::Acp
                    | bridget_transport::ManagedEventSource::ClaudeStreamJson => {
                        warn!("fait runtime ignoré : source non autorisée")
                    }
                }
            }
            ManagedEventKind::RateLimitObserved {
                mut window,
                mut status,
                resets_at,
                used_percent,
                credits,
            } => {
                window = redact_managed_text(redaction, "provider_rate_limit_window", window);
                status = redact_managed_text(redaction, "provider_rate_limit_status", status);
                match source {
                    bridget_transport::ManagedEventSource::ClaudeStreamJson => {
                        send_wrapper_message(
                            writer,
                            WrapperToDaemon::RateLimit {
                                agent: my_name.to_string(),
                                window,
                                status,
                                resets_at,
                                used_percent,
                                credits: None,
                                source:
                                    bridget_transport::protocol::RateLimitSource::ClaudeStreamJson,
                            },
                        )
                    }
                    bridget_transport::ManagedEventSource::CodexAppServer => send_wrapper_message(
                        writer,
                        WrapperToDaemon::RateLimit {
                            agent: my_name.to_string(),
                            window,
                            status,
                            resets_at,
                            used_percent,
                            credits,
                            source: bridget_transport::protocol::RateLimitSource::CodexAppServer,
                        },
                    ),
                    bridget_transport::ManagedEventSource::Acp => {
                        warn!("fait de limite ignoré : source ACP non autorisée")
                    }
                }
            }
            ManagedEventKind::ModelObserved { mut model } => {
                model = redact_managed_text(redaction, "provider_model", model);
                match source {
                    bridget_transport::ManagedEventSource::ClaudeStreamJson
                    | bridget_transport::ManagedEventSource::CodexAppServer => {
                        send_wrapper_message(
                            writer,
                            WrapperToDaemon::ServedModel {
                                agent: my_name.to_string(),
                                model,
                            },
                        )
                    }
                    bridget_transport::ManagedEventSource::Acp => {
                        warn!("modèle servi ignoré : source ACP non autorisée")
                    }
                }
            }
            ManagedEventKind::UsageObserved {
                input_tokens,
                output_tokens,
                cache_creation_input_tokens,
                cache_read_input_tokens,
            } => match source {
                bridget_transport::ManagedEventSource::ClaudeStreamJson => {
                    let (execution_id, execution_generation, provider_kind) =
                        usage_execution_context(bindings);
                    send_wrapper_message(
                        writer,
                        WrapperToDaemon::Usage {
                            agent: my_name.to_string(),
                            input_tokens,
                            output_tokens,
                            execution_id,
                            execution_generation,
                            cache_creation_input_tokens,
                            cache_read_input_tokens,
                            provider_kind,
                            source: bridget_transport::protocol::UsageSource::ClaudeStreamJson,
                        },
                    )
                }
                bridget_transport::ManagedEventSource::Acp
                | bridget_transport::ManagedEventSource::CodexAppServer => {
                    warn!("fait d'usage ignoré : source ACP/Codex non autorisée pour L4")
                }
            },
            ManagedEventKind::Waiting { state } => match state {
                bridget_transport::ManagedWaitState::Approval { request_id } => {
                    publish_approval_wait(writer, bindings, &request_id);
                    record_attention(
                        profile_db_path,
                        my_name,
                        AttentionEventType::HumanInputNeeded,
                        &request_id,
                    );
                }
                bridget_transport::ManagedWaitState::UserInput { request_id } => {
                    publish_single_wait(
                        writer,
                        bindings,
                        "waiting_user_input",
                        "user_input_required",
                    );
                    record_attention(
                        profile_db_path,
                        my_name,
                        AttentionEventType::HumanInputNeeded,
                        &request_id,
                    );
                }
            },
            ManagedEventKind::ProviderContextObserved { mut identity } => {
                identity.provider_session_id = identity
                    .provider_session_id
                    .map(|value| redact_managed_text(redaction, "provider_session_id", value));
                identity.provider_thread_id = identity
                    .provider_thread_id
                    .map(|value| redact_managed_text(redaction, "provider_thread_id", value));
                identity.active_turn_id = identity
                    .active_turn_id
                    .map(|value| redact_managed_text(redaction, "provider_turn_id", value));
                identity.provider_item_id = identity
                    .provider_item_id
                    .map(|value| redact_managed_text(redaction, "provider_item_id", value));
                identity.capabilities_revision = identity.capabilities_revision.map(|value| {
                    redact_managed_text(redaction, "provider_capabilities_revision", value)
                });
                publish_provider_context(writer, bindings, &identity)
            }
            ManagedEventKind::ProviderRequest { request } => {
                if let Some(requests) = child_requests {
                    let mut requests = requests.lock().recover_poison();
                    let duplicate = requests.waiting.iter().any(|entry| {
                        entry.request.provider_session == request.provider_session
                            && entry.request.turn_id == request.turn_id
                            && entry.request.provider_request_id == request.provider_request_id
                    });
                    if !duplicate {
                        if requests.waiting.len() == 4 {
                            journal_failed = true;
                            continue;
                        }
                        let mut active = bindings.iter().filter(|(_, binding)| {
                            matches!(
                                binding.state.as_str(),
                                "running" | "waiting_approval" | "waiting_user_input"
                            )
                        });
                        let first = active.next().map(|(id, _)| id.clone());
                        let message_id = if active.next().is_none() { first } else { None };
                        requests.waiting.push(ManagedChildWaiting {
                            request: request.clone(),
                            request_id: None,
                            message_id: message_id.clone(),
                            uncertain: false,
                        });
                        drop(requests);
                        if let Some(message_id) = message_id {
                            let state = if matches!(request.kind,bridget_transport::protocol::ChildRequestKind::UserInput|bridget_transport::protocol::ChildRequestKind::ElicitationForm) {"waiting_user_input"} else {"waiting_approval"};
                            publish_execution_transition(
                                writer,
                                bindings,
                                &message_id,
                                state,
                                "child_request_pending",
                            );
                        }
                    } else {
                        drop(requests);
                    }
                    send_wrapper_message(writer, WrapperToDaemon::ChildRequest { request });
                }
            }
            ManagedEventKind::Update { .. } | ManagedEventKind::Error { .. } => {}
            ManagedEventKind::Diagnostic { code, reference } => {
                if matches!(
                    code.as_str(),
                    "child_request_closed" | "child_request_outcome_unknown"
                ) {
                    if let Some(requests) = child_requests {
                        let value = serde_json::from_slice::<serde_json::Value>(&raw).ok();
                        let correlation = value
                            .as_ref()
                            .filter(|value| {
                                value["method"] == "serverRequest/resolved"
                                    && value["params"].as_object().is_some_and(|params| {
                                        params.len() == 2
                                            && params.contains_key("threadId")
                                            && params.contains_key("requestId")
                                    })
                            })
                            .and_then(|value| value.get("correlation"));
                        if let Some(correlation) = correlation {
                            let provider_id = serde_json::from_value::<
                                bridget_transport::protocol::ChildProviderRequestId,
                            >(
                                correlation["provider_request_id"].clone()
                            )
                            .ok();
                            let mut requests = requests.lock().recover_poison();
                            let index = requests.waiting.iter().position(|entry| {
                                Some(entry.request.provider_session.as_str())
                                    == correlation["provider_session"].as_str()
                                    && Some(entry.request.turn_id.as_str())
                                        == correlation["turn_id"].as_str()
                                    && Some(&entry.request.provider_request_id)
                                        == provider_id.as_ref()
                            });
                            if let Some(index) = index {
                                if requests.waiting[index].uncertain
                                    || code == "child_request_outcome_unknown"
                                {
                                    requests.waiting[index].uncertain = true;
                                    journal_failed = true;
                                } else {
                                    let entry = requests.waiting.remove(index);
                                    let request_id = entry.request_id.clone();
                                    let resume = entry.message_id.filter(|id| {
                                        !requests
                                            .waiting
                                            .iter()
                                            .any(|pending| pending.message_id.as_ref() == Some(id))
                                    });
                                    requests.remember(ManagedChildSettled { request:entry.request,request_id:request_id.clone(),state:bridget_transport::protocol::ChildRequestClosedState::ClosedBeforeApply,answer:None });
                                    drop(requests);
                                    if let Some(request_id) = request_id {
                                        send_wrapper_message(writer,WrapperToDaemon::ChildRequestClosed { request_id,state:bridget_transport::protocol::ChildRequestClosedState::ClosedBeforeApply });
                                    }
                                    if let Some(message_id) = resume {
                                        publish_execution_transition(
                                            writer,
                                            bindings,
                                            &message_id,
                                            "running",
                                            "child_request_settled",
                                        );
                                    }
                                }
                            }
                        }
                    }
                    continue;
                }
                let code = redact_managed_text(redaction, "provider_diagnostic_code", code);
                let reference =
                    redact_managed_text(redaction, "provider_diagnostic_reference", reference);
                publish_delegated_runtime_warning(writer, bindings, code, reference);
            }
        }
    }
    journal_failed
}

/// Le modèle est une propriété de la définition figée : le pont ne réinterprète
/// aucun autre réglage du fournisseur et ne peut donc pas dériver au rejeu.
fn codex_model_from_args(args: &[String]) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == "-c" || pair[0] == "--config")
        .and_then(|pair| pair[1].strip_prefix("model="))
        .map(|value| {
            value
                .trim()
                .trim_matches('"')
                .trim_matches('\'')
                .to_string()
        })
        .filter(|model| !model.is_empty())
}

#[cfg(test)]
#[path = "wrapper_prompt_tests.rs"]
mod prompt_tests;

#[cfg(test)]
mod attention_tests {
    use super::attention_occurrence_key;
    #[test]
    fn cle_attention_est_stable_et_ne_divulgue_pas_le_routage() {
        let first = attention_occurrence_key("task_completed", "agent-interne", "message-42");
        assert_eq!(
            first,
            attention_occurrence_key("task_completed", "agent-interne", "message-42")
        );
        assert_ne!(
            first,
            attention_occurrence_key("terminal_failure", "agent-interne", "message-42")
        );
        assert!(first.starts_with("attention:v1:task_completed:"));
        assert!(!first.contains("agent-interne"));
        assert!(!first.contains("message-42"));
    }
}

#[cfg(test)]
fn journal_failure_requires_shutdown(events: &[ManagedEvent]) -> bool {
    events
        .iter()
        .any(|event| matches!(event.kind, ManagedEventKind::JournalFailed { .. }))
}

#[cfg(test)]
#[path = "wrapper_delegated_runtime_tests.rs"]
mod delegated_runtime_tests;

#[cfg(test)]
#[path = "wrapper_reconnect_tests.rs"]
mod reconnect_tests;

#[cfg(test)]
#[path = "wrapper_native_child_requests155_tests.rs"]
mod native_child_requests155_tests;

#[cfg(test)]
#[path = "wrapper_remote162_result_tests.rs"]
mod remote162_result_tests;

fn observe_user_input_receipt(
    writer: &Arc<Mutex<Option<BufWriter<UnixStream>>>>,
    requests: &Mutex<ManagedChildRequests>,
    result: &serde_json::Value,
) {
    if result["status"] != "ok"
        || !matches!(result["item"]["origin"].as_str(), Some("codex" | "claude"))
    {
        return;
    }
    let Some(id) = result["item"]["request_id"].as_str() else {
        return;
    };
    let Ok(request) = serde_json::from_value::<bridget_transport::protocol::ChildProviderRequest>(
        result["item"]["callback"].clone(),
    ) else {
        return;
    };
    let mut requests = requests.lock().recover_poison();
    if let Some(waiting) = requests.waiting.iter_mut().find(|w| w.request == request) {
        waiting.request_id = Some(id.into());
        return;
    }
    if let Some(settled) = requests.settled.iter_mut().find(|w| w.request == request) {
        settled.request_id = Some(id.into());
        let frame = WrapperToDaemon::ChildRequestClosed {
            request_id: id.into(),
            state: settled.state,
        };
        drop(requests);
        send_wrapper_message(writer, frame);
    }
}
