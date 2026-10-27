//! Dernier refus observé par le maître. Cache volatil, jamais autorité de remise.
use super::DaemonState;
use crate::delegation::Task;
use bridget_transport::protocol::{WorkerCommandPhase, WorkerRefusal};
use serde::Serialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Write,
    sync::{Arc, Mutex},
};
const MAX_RECORDS: usize = 128;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Stage {
    Preparation,
    ParentProof,
    CommandIdentity,
    WorkerObservation,
    WorkerPermissions,
    FinalPermissions,
    Control,
    ResultReservation,
}
#[derive(Clone, Debug, PartialEq, Serialize)]
pub(super) struct WorkerFact {
    pub phase: WorkerCommandPhase,
    pub native_ready: bool,
    pub permission_digest_present: bool,
    pub permission_digest_matches: Option<bool>,
    pub workspace_report_present: bool,
}
#[derive(Clone, PartialEq, Eq)]
struct Identity {
    owner: String,
    owner_instance: String,
    child_instance: String,
    mission: String,
    round: u32,
    placement_digest: String,
}
impl Identity {
    fn of(task: &Task) -> Option<Self> {
        let placement = task.remote_placement.as_ref()?;
        let repair = task
            .workspace_repairs
            .last()
            .map(|r| (&r.request.request_id, &r.phase));
        let bytes = serde_json::to_vec(&(placement, repair)).ok()?;
        Some(Self {
            owner: task.owner.clone(),
            owner_instance: task.owner_instance.clone(),
            child_instance: task.child_instance.clone()?,
            mission: task.mission.clone(),
            round: u32::from(task.round),
            placement_digest: format!("{:x}", Sha256::digest(bytes)),
        })
    }
}
#[derive(Clone)]
struct Record {
    identity: Identity,
    stage: Stage,
    code: String,
    worker: Option<WorkerFact>,
    observed_at: i64,
}
#[derive(Default)]
pub(super) struct Cache {
    entries: BTreeMap<String, Record>,
}
fn pending(task: &Task) -> bool {
    task.state == "mission_pending"
        && task.mission_issued_at.is_none()
        && task.remote_placement.is_some()
}
fn closed_code(code: &str) -> String {
    if code.len() > 128 {
        return "unclassified".into();
    }
    if matches!(
        code,
        "workspace_repair_pending"
            | "permission_attestation_unavailable"
            | "remote_placement_missing"
            | "origin_binding_unavailable"
            | "origin_worker_unavailable"
            | "origin_request_invalid"
            | "origin_proof_unavailable"
            | "origin_proof_busy"
            | "remote_parent_proof_invalid"
            | "remote_parent_proof_changed"
            | "permission_revision_changed"
            | "settings_revision_changed"
            | "permission_source_unavailable"
            | "permission_not_inherited"
            | "remote_execution_unconfirmed"
            | "remote_execution_ended"
            | "remote_workspace_unconfirmed"
            | "remote_permissions_unconfirmed"
            | "provider_quota_exhausted"
            | "invalid_record"
            | "referent_paused"
            | "delegations_halted"
            | "remote_result_slot_limit"
            | "remote_execution_missing"
            | "remote_execution_payload_missing"
            | "remote_result_slot_mismatch"
            | "remote_result_slot_unconfirmed"
    ) {
        return code.into();
    }
    let suffix = code
        .strip_prefix("remote_permissions:")
        .or_else(|| code.strip_prefix("remote_parent_proof:"))
        .or_else(|| code.strip_prefix("remote_result_slot:"))
        .unwrap_or(code);
    let mut snake = String::new();
    for (i, c) in suffix.chars().enumerate() {
        if !c.is_ascii_alphabetic() && c != '_' {
            return "unclassified".into();
        }
        if c.is_ascii_uppercase() && i > 0 {
            snake.push('_');
        }
        snake.push(c.to_ascii_lowercase());
    }
    if serde_json::from_value::<WorkerRefusal>(json!(snake)).is_ok() {
        code.into()
    } else {
        "unclassified".into()
    }
}
impl Record {
    fn projection(&self) -> Value {
        let (message, recovery) = match self.stage {
            Stage::ParentProof => (
                "La preuve du parent n’a pas été confirmée pour cette remise.",
                "Vérifiez le worker de l’hôte d’origine et les sources de droits du parent admis. Une attestation générique antérieure ne remplace pas la réponse de ce contrôle. Ne répétez pas les attestations sans fait nouveau.",
            ),
            Stage::CommandIdentity => (
                "Le maître n’a pas relié la commande persistée à cette exécution.",
                "Vérifiez commande, instance, génération et lien parent dans les reçus natifs. Ne recréez ni identité ni commande pour remplacer une preuve manquante.",
            ),
            Stage::WorkerObservation => (
                "Le maître n’a pas obtenu l’observation distante attendue pour cette remise.",
                "Vérifiez le code de refus, l’identité et l’autorisation de la machine dans le registre, puis le contact du worker et la durée du contrôle corrélé. Un délai ne prouve ni panne réseau ni arrêt de l’exécution. Conservez la même tâche et ses fichiers.",
            ),
            Stage::WorkerPermissions => (
                "L’observation reçue du worker ne confirme pas les conditions de remise.",
                "Consultez les faits joints : phase, préparation native, présence de preuve de droits et de rapport Git. Faites vérifier les sources ou le dossier par leur responsable ; ne forcez pas la garde.",
            ),
            Stage::FinalPermissions => (
                "La dernière garde du maître a refusé la remise.",
                "Vérifiez le code constaté : autorité admise, révocations ou quota de cette exécution. Conservez la tâche ; ne changez pas ses droits ou son moteur pour contourner ce refus.",
            ),
            Stage::Control => (
                "Le contrôle natif du maître retient la remise.",
                "Consultez l’état natif de contrôle et sa décision humaine. Ne le modifiez pas pour contourner ce refus.",
            ),
            Stage::ResultReservation => (
                "Le maître n’a pas confirmé la réservation du résultat.",
                "Vérifiez le reçu et la capacité de retour de cette même exécution. Aucun travail de remplacement n’est autorisé pendant une issue inconnue.",
            ),
            Stage::Preparation => (
                "Une condition du maître empêche la vérification de remise.",
                "Vérifiez le code fermé et la préparation native de cette tâche. La cause précise reste inconnue si le code est unclassified ; faites inspecter ce point sans reconstruire le dossier ni la mission.",
            ),
        };
        json!({"version":1,"scope":"last_observed_master_refusal","authorizes_dispatch":false,
            "observed_at":self.observed_at,"stage":self.stage,"code":self.code,"worker":self.worker,
            "message":message,"recovery":recovery,
            "human_action":"Si la preuve ne peut pas être rétablie dans votre périmètre, demandez au responsable ou à l’utilisateur la décision précise sur la source, la capacité ou la maintenance concernée. Gardez la tâche, sa clé et ses fichiers ; aucune annulation d’autrui ni remplacement automatique."})
    }
}
impl Cache {
    fn record(
        &mut self,
        task: &Task,
        stage: Stage,
        code: &str,
        worker: Option<WorkerFact>,
        now: i64,
    ) -> Option<Value> {
        if !pending(task) {
            return None;
        }
        let identity = Identity::of(task)?;
        let code = closed_code(code);
        if let Some(old) = self.entries.get_mut(&task.task_id)
            && old.identity == identity
            && old.stage == stage
            && old.code == code
            && old.worker == worker
        {
            old.observed_at = now;
            return None;
        }
        if !self.entries.contains_key(&task.task_id)
            && self.entries.len() >= MAX_RECORDS
            && let Some(key) = self
                .entries
                .iter()
                .min_by_key(|(_, r)| r.observed_at)
                .map(|(key, _)| key.clone())
        {
            self.entries.remove(&key);
        }
        let record = Record {
            identity,
            stage,
            code,
            worker,
            observed_at: now,
        };
        let mut event = record.projection();
        event["task_id"] = json!(task.task_id);
        event["round"] = json!(task.round);
        self.entries.insert(task.task_id.clone(), record);
        Some(event)
    }
    pub(super) fn attach(&self, task: &Task, view: &mut Value, full: bool) {
        if !full || !pending(task) {
            return;
        }
        let value=self.entries.get(&task.task_id).filter(|r|Identity::of(task).as_ref()==Some(&r.identity))
            .map(Record::projection).unwrap_or_else(||json!({"version":1,"scope":"last_observed_master_refusal",
                "authorizes_dispatch":false,"state":"unobserved_since_start",
                "message":"Aucun refus maître corrélé à cette version de la tâche n’est disponible depuis ce démarrage.",
                "recovery":"L’absence de diagnostic ne prouve ni remise ni échec. Conservez la tâche ; le prochain contrôle natif pourra fournir un fait nouveau sans nouvelle mission."}));
        view["detail"]["dispatch_refusal"] = value;
    }
}
pub(super) fn record_locked(
    st: &mut DaemonState,
    task: &Task,
    stage: Stage,
    code: &str,
    worker: Option<WorkerFact>,
) -> Option<Value> {
    st.dispatch_diagnostics
        .record(task, stage, code, worker, super::unix_timestamp())
}
pub(super) fn emit(event: Value) {
    // Hors verrou maître. Une trace fermée par changement contourne la perte
    // des WARN190 sans recopier de politique ni de message métier.
    let _ = writeln!(
        std::io::stderr().lock(),
        "bridget_dispatch_refusal190 {event}"
    );
}
pub(super) fn record(
    state: &Arc<Mutex<DaemonState>>,
    expected: &Task,
    stage: Stage,
    code: &str,
    worker: Option<WorkerFact>,
) {
    let event = {
        let mut st = state.lock().unwrap_or_else(|e| e.into_inner());
        if let Ok(Some(current)) = st.delegation_store.get(&expected.task_id)
            && Identity::of(&current) == Identity::of(expected)
        {
            record_locked(&mut st, &current, stage, code, worker)
        } else {
            None
        }
    };
    if let Some(event) = event {
        emit(event);
    }
}
pub(super) fn observe<T>(
    state: &Arc<Mutex<DaemonState>>,
    task: &Task,
    stage: Stage,
    outcome: Result<T, String>,
) -> Result<T, String> {
    if let Err(code) = &outcome {
        record(state, task, stage, code, None);
    }
    outcome
}
