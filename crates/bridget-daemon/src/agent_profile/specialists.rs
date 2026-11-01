//! Métadonnées222 dans AgentProfileStore ; les exécutions restent dans ExecutionStore.
use super::*;
use bridget_transport::specialists::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[derive(Clone)]
pub struct SpecialistPrincipal {
    pub agent_id: Option<String>,
    pub human_approval_ref: Option<String>,
    pub allowed_projects: Vec<String>,
}
impl SpecialistPrincipal {
    fn human(&self) -> bool {
        self.human_approval_ref.is_some()
    }
    fn owner(&self) -> &str {
        self.agent_id.as_deref().unwrap_or("human")
    }
    fn permits(&self, project: &str) -> bool {
        self.allowed_projects
            .iter()
            .any(|p| p == "*" || p == project)
    }
    fn owns(&self, agent: &str) -> bool {
        self.human() || self.agent_id.as_deref() == Some(agent)
    }
    // Une autorité humaine reste limitée par la portée de son appareil.
    pub(crate) fn permits_record(&self, record: &SpecialistRecord) -> bool {
        !self.human()
            || self.allowed_projects.iter().any(|p| p == "*")
            || (!record.scope.transversal && record.scope.projects.iter().all(|p| self.permits(p)))
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecialistError(pub &'static str);
impl From<rusqlite::Error> for SpecialistError {
    fn from(_: rusqlite::Error) -> Self {
        Self("storage_unavailable")
    }
}
impl From<serde_json::Error> for SpecialistError {
    fn from(_: serde_json::Error) -> Self {
        Self("invalid_request")
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefinitionRevision {
    pub definition: SpecialistDefinition,
    pub revision: u64,
    pub digest: String,
    pub approved_ref: Option<String>,
    pub created_by: String,
    pub created_at: i64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MandateGrant {
    #[serde(default)]
    pub profile_id: Option<String>,
    #[serde(default)]
    pub profile_revision: u64,
    pub grant_id: String,
    pub revision: u64,
    pub approved_by: String,
    pub approval_ref: String,
    pub allowed_projects: SpecialistScope,
    pub launch_policy: Option<SpecialistLaunchPolicy>,
    pub issued_at: i64,
    pub revoked_at: Option<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecialistRecord {
    #[serde(default)]
    pub capture_history: bool,
    pub agent_id: String,
    pub owner: String,
    pub profile_id: Option<String>,
    pub profile_revision: u64,
    pub revision: u64,
    pub scope: SpecialistScope,
    pub durable: bool,
    pub enabled: bool,
    pub auto_start: bool,
    pub mandate: String,
    pub budget: SpecialistBudget,
    pub grant: Option<MandateGrant>,
    pub updated_at: i64,
}
fn digest(value: &impl Serialize) -> Result<String, SpecialistError> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(value)?)))
}
fn slug(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_' || b == b'.')
}
fn specialist_projection(record: &SpecialistRecord, principal: &SpecialistPrincipal) -> Value {
    let mut view = json!(record);
    if !principal.owns(&record.agent_id) {
        if let Some(object) = view.as_object_mut() {
            object.remove("grant");
            object.remove("owner");
        }
    }
    view
}
fn valid_uuid(s: &str) -> bool {
    Uuid::parse_str(s).is_ok_and(|id| id.to_string() == s)
}
fn validate_scope(s: &SpecialistScope) -> Result<(), SpecialistError> {
    if s.projects.len() > 64
        || s.projects
            .iter()
            .any(|p| p.is_empty() || p.len() > 256 || p.chars().any(char::is_control))
    {
        return Err(SpecialistError("invalid_request"));
    }
    Ok(())
}
fn validate_definition(d: &SpecialistDefinition) -> Result<(), SpecialistError> {
    if !valid_uuid(&d.profile_id)
        || d.name.is_empty()
        || d.name.chars().count() > 80
        || d.instructions.chars().count() > 8_000
        || d.roles.len() > 32
        || d.roles.iter().any(|r| !slug(r))
        || d.procedures.len() > 32
        || d.requirements.len() > 32
        || d.members.len() > 32
    {
        return Err(SpecialistError("invalid_request"));
    }
    if d.procedures
        .iter()
        .any(|p| p.name.is_empty() || p.name.len() > 128 || p.instructions.chars().count() > 8_000)
    {
        return Err(SpecialistError("invalid_request"));
    }
    if d.kind == SpecialistDefinitionKind::Profile && !d.members.is_empty() {
        return Err(SpecialistError("invalid_request"));
    }
    let keys: HashSet<_> = d.members.iter().map(|m| m.key.as_str()).collect();
    if keys.len() != d.members.len()
        || d.members.iter().any(|m| {
            !slug(&m.key)
                || !slug(&m.role_id)
                || m.deliverable.is_empty()
                || m.deliverable.len() > 4000
                || m.depends_on
                    .iter()
                    .any(|k| !keys.contains(k.as_str()) || k == &m.key)
        })
    {
        return Err(SpecialistError("invalid_request"));
    }
    // Kahn borné à32 membres ; un cycle ne devient jamais une équipe partiellement active.
    let mut done = HashSet::new();
    for _ in 0..d.members.len() {
        let before = done.len();
        for m in &d.members {
            if m.depends_on.iter().all(|k| done.contains(k)) {
                done.insert(m.key.clone());
            }
        }
        if done.len() == before {
            break;
        }
    }
    if done.len() != d.members.len() {
        return Err(SpecialistError("team_dependency_cycle"));
    }
    Ok(())
}
pub fn default_mandate(roles: &[String]) -> String {
    if roles.iter().any(|r| r == "mainteneur-bridget") {
        "Diagnostiquer les incidents Bridget, corriger les défauts confirmés et livrer dans le périmètre autorisé. Préserver les missions et résultats. Demander une décision ciblée pour tout accès, dépense ou privilège nouveau.".into()
    } else if roles.iter().any(|r| r.contains("recherche")) {
        "Rechercher, comparer les sources et remettre une synthèse sourcée avec ses limites. Ne pas acheter, publier ni modifier des services sans autorisation.".into()
    } else {
        "Mener la demande autorisée jusqu'au résultat utilisable. Préserver les travaux existants et demander une décision ciblée pour toute action hors du mandat.".into()
    }
}
impl AgentProfileStore {
    pub(crate) fn init_specialists(&self) -> Result<(), AgentProfileError> {
        self.conn.execute_batch("CREATE TABLE IF NOT EXISTS specialist_definitions (
            profile_id TEXT NOT NULL,revision INTEGER NOT NULL,definition_json TEXT NOT NULL,digest TEXT NOT NULL,
            approved_ref TEXT,created_by TEXT NOT NULL,created_at INTEGER NOT NULL,PRIMARY KEY(profile_id,revision));
        CREATE TABLE IF NOT EXISTS specialist_temporary_requests (
            owner TEXT NOT NULL,request_id TEXT NOT NULL,profile_id TEXT NOT NULL,profile_revision INTEGER NOT NULL,
            canonical BLOB NOT NULL,delegation TEXT NOT NULL,source_project TEXT NOT NULL,PRIMARY KEY(owner,request_id),UNIQUE(profile_id,profile_revision));
        CREATE TABLE IF NOT EXISTS specialist_initial_activations (
            profile_id TEXT NOT NULL,profile_revision INTEGER NOT NULL,task_id TEXT NOT NULL,child_agent_id TEXT NOT NULL,
            PRIMARY KEY(profile_id,profile_revision));
        CREATE TABLE IF NOT EXISTS specialists (
            agent_id TEXT PRIMARY KEY REFERENCES agent_profiles(agent_id),record_json TEXT NOT NULL,revision INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS specialist_grant_revisions (
            grant_id TEXT NOT NULL,revision INTEGER NOT NULL,agent_id TEXT NOT NULL,grant_json TEXT NOT NULL,
            PRIMARY KEY(grant_id,revision));
        CREATE INDEX IF NOT EXISTS specialist_profile_lookup ON specialists(json_extract(record_json,'$.profile_id'),json_extract(record_json,'$.profile_revision'));
        CREATE TABLE IF NOT EXISTS specialist_roles (
            agent_id TEXT NOT NULL REFERENCES agent_profiles(agent_id),role_id TEXT NOT NULL,label TEXT NOT NULL,scope_json TEXT NOT NULL,
            official INTEGER NOT NULL DEFAULT 0,designation_ref TEXT,PRIMARY KEY(agent_id,role_id));
        CREATE INDEX IF NOT EXISTS specialist_roles_lookup ON specialist_roles(role_id,agent_id);
        CREATE TABLE IF NOT EXISTS specialist_memory (
            memory_id TEXT PRIMARY KEY,agent_id TEXT NOT NULL,project_id TEXT NOT NULL,shared INTEGER NOT NULL,
            kind TEXT NOT NULL,text TEXT NOT NULL,normalized TEXT NOT NULL,source_json TEXT NOT NULL,source_key TEXT NOT NULL,
            revision INTEGER NOT NULL,forgotten INTEGER NOT NULL DEFAULT 0,updated_at INTEGER NOT NULL,
            UNIQUE(agent_id,source_key));
        CREATE INDEX IF NOT EXISTS specialist_memory_agent ON specialist_memory(agent_id,forgotten,project_id,updated_at);
        CREATE TABLE IF NOT EXISTS specialist_memory_revisions (
            memory_id TEXT NOT NULL,revision INTEGER NOT NULL,text TEXT NOT NULL,kind TEXT NOT NULL,updated_at INTEGER NOT NULL,
            PRIMARY KEY(memory_id,revision));
        CREATE TABLE IF NOT EXISTS specialist_memory_sources (
            agent_id TEXT PRIMARY KEY,observation TEXT NOT NULL,observed_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS specialist_mutations (
            issuer TEXT NOT NULL,request_id TEXT NOT NULL,canonical BLOB NOT NULL,response TEXT NOT NULL,PRIMARY KEY(issuer,request_id));")
            .map_err(AgentProfileError::Sqlite)
    }
    pub fn get_specialist(&self, id: &str) -> Result<Option<SpecialistRecord>, SpecialistError> {
        let row: Option<String> = self
            .conn
            .query_row(
                "SELECT record_json FROM specialists WHERE agent_id=?1",
                [id],
                |r| r.get(0),
            )
            .optional()?;
        row.map(|r| serde_json::from_str(&r).map_err(Into::into))
            .transpose()
    }
    pub fn get_definition(
        &self,
        id: &str,
        revision: u64,
    ) -> Result<Option<DefinitionRevision>, SpecialistError> {
        self.conn.query_row("SELECT definition_json,digest,approved_ref,created_by,created_at FROM specialist_definitions WHERE profile_id=?1 AND revision=?2",
            params![id,revision],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,String>(3)?,r.get::<_,i64>(4)?))).optional()?.map(|r|Ok(DefinitionRevision{definition:serde_json::from_str(&r.0)?,revision,digest:r.1,approved_ref:r.2,created_by:r.3,created_at:r.4})).transpose()
    }
    fn save_specialist(&self, r: &SpecialistRecord) -> Result<(), SpecialistError> {
        self.conn.execute("INSERT INTO specialists(agent_id,record_json,revision) VALUES(?1,?2,?3) ON CONFLICT(agent_id) DO UPDATE SET record_json=excluded.record_json,revision=excluded.revision",
            params![r.agent_id,serde_json::to_string(r)?,r.revision])?;
        if let Some(grant) = &r.grant {
            self.conn.execute("INSERT OR IGNORE INTO specialist_grant_revisions(grant_id,revision,agent_id,grant_json) VALUES(?1,?2,?3,?4)",
                params![grant.grant_id,grant.revision,r.agent_id,serde_json::to_string(grant)?])?;
        }
        Ok(())
    }
    fn ensure_specialist(
        &mut self,
        id: &str,
        now: i64,
    ) -> Result<SpecialistRecord, SpecialistError> {
        if !valid_uuid(id) {
            return Err(SpecialistError("invalid_request"));
        }
        if let Some(r) = self.get_specialist(id)? {
            return Ok(r);
        }
        self.ensure_agent_ids([id])
            .map_err(|_| SpecialistError("storage_unavailable"))?;
        let r = SpecialistRecord {
            capture_history: false,
            agent_id: id.into(),
            owner: id.into(),
            profile_id: None,
            profile_revision: 0,
            revision: 0,
            scope: SpecialistScope::default(),
            durable: false,
            enabled: true,
            auto_start: false,
            mandate: String::new(),
            budget: SpecialistBudget::default(),
            grant: None,
            updated_at: now,
        };
        self.save_specialist(&r)?;
        Ok(r)
    }
    /// O(n log n), n = titulaires du rôle ; un JOIN charge aussi la désignation.
    pub fn specialist_candidates(
        &self,
        role: &str,
        project: &str,
    ) -> Result<Vec<(SpecialistRecord, bool)>, SpecialistError> {
        let mut q=self.conn.prepare("SELECT s.record_json,r.scope_json,r.official FROM specialist_roles r JOIN specialists s ON s.agent_id=r.agent_id WHERE r.role_id=?1 ORDER BY s.agent_id")?;
        let rows = q.query_map([role], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, bool>(2)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (a, b, official) = row?;
            let record: SpecialistRecord = serde_json::from_str(&a)?;
            let scope: SpecialistScope = serde_json::from_str(&b)?;
            if scope.permits(project) && record.scope.permits(project) {
                out.push((record, official));
            }
        }
        out.sort_by_key(|(s, official)| (!*official, s.scope.transversal, s.agent_id.clone()));
        Ok(out)
    }
    pub fn specialist_apply(
        &mut self,
        q: &SpecialistRequest,
        p: &SpecialistPrincipal,
        now: i64,
    ) -> Result<Value, SpecialistError> {
        q.validate().map_err(SpecialistError)?;
        if q.requires_human() && !p.human() {
            return Err(SpecialistError("human_principal_required"));
        }
        let raw = serde_json::to_value(q)?;
        if p.human() && q.requires_human() {
            let permits_scope = |scope: &SpecialistScope| {
                !scope.transversal && scope.projects.iter().all(|v| p.permits(v))
            };
            if !p.allowed_projects.iter().any(|v| v == "*") {
                if let Some(scope) = raw.get("scope") {
                    let scope: SpecialistScope = serde_json::from_value(scope.clone())?;
                    if !permits_scope(&scope) {
                        return Err(SpecialistError("role_scope_denied"));
                    }
                }
                if let Some(id) = raw["specialist_agent_id"].as_str() {
                    if let Some(record) = self.get_specialist(id)? {
                        if !permits_scope(&record.scope) {
                            return Err(SpecialistError("role_scope_denied"));
                        }
                    }
                }
            }
        }
        let canonical = Sha256::digest(serde_json::to_vec(q)?).to_vec();
        let request_id = raw.get("request_id").and_then(Value::as_str);
        if !q.is_read() {
            let id = request_id.ok_or(SpecialistError("invalid_request"))?;
            if let Some((stored,result))=self.conn.query_row("SELECT canonical,response FROM specialist_mutations WHERE issuer=?1 AND request_id=?2",params![p.owner(),id],|r|Ok((r.get::<_,Vec<u8>>(0)?,r.get::<_,String>(1)?))).optional()? {
                if stored!=canonical{return Err(SpecialistError("envelope_mismatch"));}
                return Ok(serde_json::from_str(&result)?);
            }
        }
        // Un seul écrivain sous la garde DaemonState ; savepoint annule aussi les sous-opérations.
        self.conn.execute_batch("SAVEPOINT specialist_command")?;
        let result = self.apply_specialist_inner(q, p, now);
        match result {
            Ok(value) => {
                let saved = (|| -> Result<(), SpecialistError> {
                    if !q.is_read() {
                        self.conn.execute("INSERT INTO specialist_mutations(issuer,request_id,canonical,response) VALUES(?1,?2,?3,?4)",params![p.owner(),request_id,canonical,serde_json::to_string(&value)?])?;
                    }
                    self.conn.execute_batch("RELEASE specialist_command")?;
                    Ok(())
                })();
                if let Err(error) = saved {
                    let _ = self.conn.execute_batch(
                        "ROLLBACK TO specialist_command; RELEASE specialist_command",
                    );
                    return Err(error);
                }
                Ok(value)
            }
            Err(error) => {
                self.conn
                    .execute_batch("ROLLBACK TO specialist_command; RELEASE specialist_command")?;
                Err(error)
            }
        }
    }
    fn apply_specialist_inner(
        &mut self,
        q: &SpecialistRequest,
        p: &SpecialistPrincipal,
        now: i64,
    ) -> Result<Value, SpecialistError> {
        use SpecialistRequest::*;
        let ok = |value: Value| Ok(json!({"version":1,"status":"ok","data":value}));
        match q {
            Profiles { cursor, limit, .. } => {
                let limit = limit.unwrap_or(20).clamp(1, 50);
                let mut q=self.conn.prepare("SELECT d.profile_id,d.revision,d.definition_json,d.approved_ref,d.created_by,d.created_at FROM specialist_definitions d WHERE d.profile_id>?1 AND d.revision=(SELECT MAX(v.revision) FROM specialist_definitions v WHERE v.profile_id=d.profile_id) ORDER BY d.profile_id LIMIT ?2")?;
                let rows =
                    q.query_map(params![cursor.as_deref().unwrap_or(""), limit + 1], |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, u64>(1)?,
                            r.get::<_, String>(2)?,
                            r.get::<_, Option<String>>(3)?,
                            r.get::<_, String>(4)?,
                            r.get::<_, i64>(5)?,
                        ))
                    })?;
                let mut items = Vec::new();
                for row in rows {
                    let r = row?;
                    let d: SpecialistDefinition = serde_json::from_str(&r.2)?;
                    items.push(json!({"profile_id":r.0,"revision":r.1,"name":d.name,"kind":d.kind,"roles":d.roles,"approved":r.3.is_some(),"created_by":r.4,"created_at":r.5,"activation":"separate_human_decision"}));
                }
                let more = items.len() > limit as usize;
                items.truncate(limit as usize);
                let next = more.then(|| {
                    items.last().unwrap()["profile_id"]
                        .as_str()
                        .unwrap()
                        .to_owned()
                });
                ok(json!({"items":items,"next_cursor":next}))
            }
            ProfileVersions {
                profile_id,
                cursor,
                limit,
                ..
            } => {
                let before = cursor
                    .as_ref()
                    .map(|s| s.parse::<u64>())
                    .transpose()
                    .map_err(|_| SpecialistError("invalid_request"))?
                    .unwrap_or(i64::MAX as u64);
                let limit = limit.unwrap_or(5).clamp(1, 10);
                let mut q=self.conn.prepare("SELECT revision FROM specialist_definitions WHERE profile_id=?1 AND revision<?2 ORDER BY revision DESC LIMIT ?3")?;
                let versions = q
                    .query_map(params![profile_id, before, limit + 1], |r| {
                        r.get::<_, u64>(0)
                    })?
                    .collect::<Result<Vec<_>, _>>()?;
                let mut items = Vec::new();
                for revision in versions {
                    let d = self
                        .get_definition(profile_id, revision)?
                        .ok_or(SpecialistError("profile_not_found"))?;
                    if p.human() || d.created_by == p.owner() || d.approved_ref.is_some() {
                        items.push(json!(d));
                    }
                }
                let more = items.len() > limit as usize;
                items.truncate(limit as usize);
                let next = more.then(|| {
                    items.last().unwrap()["revision"]
                        .as_u64()
                        .unwrap()
                        .to_string()
                });
                ok(json!({"items":items,"next_cursor":next}))
            }
            AttachProfile {
                profile_id,
                profile_revision,
                expected_revision,
                ..
            } => {
                let agent = p
                    .agent_id
                    .as_deref()
                    .ok_or(SpecialistError("identity_unavailable"))?;
                let d = self
                    .get_definition(profile_id, *profile_revision)?
                    .ok_or(SpecialistError("profile_not_found"))?;
                let initial=self.conn.query_row("SELECT EXISTS(SELECT 1 FROM specialist_initial_activations WHERE profile_id=?1 AND profile_revision=?2 AND child_agent_id=?3)",params![profile_id,profile_revision,agent],|r|r.get::<_,bool>(0))?;
                let reusable = d.approved_ref.is_some()
                    && self.reusable_definition(profile_id, *profile_revision, p)?;
                if d.created_by != agent && !reusable && !initial {
                    return Err(SpecialistError("human_reuse_approval_required"));
                }
                let mut r = self.ensure_specialist(agent, now)?;
                if r.revision != *expected_revision || r.durable {
                    return Err(SpecialistError("profile_version_conflict"));
                }
                r.grant = None;
                r.auto_start = false;
                r.capture_history = true;
                r.profile_id = Some(profile_id.clone());
                r.profile_revision = *profile_revision;
                r.scope = SpecialistScope {
                    projects: p
                        .allowed_projects
                        .iter()
                        .filter(|s| s.as_str() != "*")
                        .cloned()
                        .collect(),
                    transversal: false,
                };
                r.revision += 1;
                r.updated_at = now;
                self.save_specialist(&r)?;
                for role in &d.definition.roles {
                    self.conn.execute("INSERT INTO specialist_roles(agent_id,role_id,label,scope_json) VALUES(?1,?2,?2,?3) ON CONFLICT(agent_id,role_id) DO UPDATE SET scope_json=excluded.scope_json",params![agent,role,serde_json::to_string(&r.scope)?])?;
                }
                ok(
                    json!({"specialist":r,"activation":"temporary_current_identity","future_reactivation":"human_approval_required"}),
                )
            }
            List { cursor, limit, .. } | Search { cursor, limit, .. } => {
                let role = match q {
                    Search { role_id, .. } => Some(role_id.as_str()),
                    _ => None,
                };
                let project = match q {
                    Search { project_id, .. } => project_id.as_deref(),
                    _ => None,
                };
                if project.is_some_and(|v| !p.permits(v)) {
                    return Err(SpecialistError("role_scope_denied"));
                }
                let limit = limit.unwrap_or(30).clamp(1, 100) as usize;
                // Page bornée à101 identités, chacune à32 rôles : accès locaux bornés.
                let mut stmt=self.conn.prepare("SELECT record_json FROM specialists WHERE agent_id>?1 ORDER BY agent_id LIMIT 101")?;
                let rows =
                    stmt.query_map([cursor.as_deref().unwrap_or("")], |r| r.get::<_, String>(0))?;
                let mut items = Vec::new();
                let mut scanned = None;
                let mut exhausted = true;
                let mut scanned_count = 0;
                for row in rows {
                    scanned_count += 1;
                    let r: SpecialistRecord = serde_json::from_str(&row?)?;
                    scanned = Some(r.agent_id.clone());
                    if scanned_count == 101 {
                        exhausted = false;
                    }
                    if !p.permits_record(&r) {
                        continue;
                    }
                    if !p.owns(&r.agent_id)
                        && !r.scope.transversal
                        && !r.scope.projects.iter().any(|v| p.permits(v))
                    {
                        continue;
                    }
                    if project.is_some_and(|v| !r.scope.permits(v)) {
                        continue;
                    }
                    let mut roles = self.roles_for(&r.agent_id)?;
                    if !p.owns(&r.agent_id) {
                        for role in &mut roles {
                            if let Some(role) = role.as_object_mut() {
                                role.remove("designation_ref");
                            }
                        }
                    }
                    if role.is_some_and(|role| !roles.iter().any(|v| v["role_id"] == role)) {
                        continue;
                    }
                    let display_name = self
                        .profile_for_agent_id(&r.agent_id)
                        .map_err(|_| SpecialistError("storage_unavailable"))?
                        .map(|profile| profile.display_name);
                    items.push(json!({"specialist":specialist_projection(&r,p),"display_name":display_name,"roles":roles}));
                    if items.len() == limit {
                        exhausted = false;
                        break;
                    }
                }
                ok(
                    json!({"items":items,"next_cursor":if exhausted {None}else{scanned},"presence_scope":"runtime_observation_required"}),
                )
            }
            Get {
                specialist_agent_id,
                ..
            } => {
                let r = self
                    .get_specialist(specialist_agent_id)?
                    .ok_or(SpecialistError("specialist_not_found"))?;
                if !p.permits_record(&r) {
                    return Err(SpecialistError("role_scope_denied"));
                }
                if !p.owns(&r.agent_id)
                    && !r.scope.transversal
                    && !r.scope.projects.iter().any(|v| p.permits(v))
                {
                    return Err(SpecialistError("role_scope_denied"));
                }
                let definition = r
                    .profile_id
                    .as_ref()
                    .map(|id| self.get_definition(id, r.profile_revision))
                    .transpose()?
                    .flatten();
                let mut roles = self.roles_for(specialist_agent_id)?;
                let mut definition = serde_json::to_value(definition)?;
                if !p.owns(&r.agent_id) {
                    for role in &mut roles {
                        if let Some(role) = role.as_object_mut() {
                            role.remove("designation_ref");
                        }
                    }
                    if let Some(definition) = definition.as_object_mut() {
                        definition.remove("approved_ref");
                    }
                }
                let display_name = self
                    .profile_for_agent_id(&r.agent_id)
                    .map_err(|_| SpecialistError("storage_unavailable"))?
                    .map(|profile| profile.display_name);
                ok(
                    json!({"specialist":specialist_projection(&r,p),"display_name":display_name,"roles":roles,"definition":definition}),
                )
            }
            DeclareRoles {
                roles,
                expected_revision,
                ..
            } => {
                let agent = p
                    .agent_id
                    .as_deref()
                    .ok_or(SpecialistError("identity_unavailable"))?;
                let mut r = self.ensure_specialist(agent, now)?;
                if r.revision != *expected_revision {
                    return Err(SpecialistError("profile_version_conflict"));
                }
                if roles.len() > 32 {
                    return Err(SpecialistError("invalid_request"));
                }
                for role in roles {
                    validate_scope(&role.scope)?;
                    let approved_scope = r
                        .grant
                        .as_ref()
                        .filter(|g| g.revoked_at.is_none())
                        .map(|g| &g.allowed_projects);
                    if !p.human()
                        && ((role.scope.transversal
                            && !approved_scope.is_some_and(|s| s.transversal))
                            || role.scope.projects.iter().any(|project| {
                                !p.permits(project)
                                    && !approved_scope.is_some_and(|s| s.permits(project))
                            }))
                    {
                        return Err(SpecialistError("role_scope_denied"));
                    }
                    if !slug(&role.role_id) || role.label.is_empty() || role.label.len() > 160 {
                        return Err(SpecialistError("invalid_request"));
                    }
                }
                // Les désignations officielles sont retirées avec leur claim ; jamais héritées d'un remplacement.
                self.conn
                    .execute("DELETE FROM specialist_roles WHERE agent_id=?1", [agent])?;
                for role in roles {
                    self.conn.execute("INSERT INTO specialist_roles(agent_id,role_id,label,scope_json) VALUES(?1,?2,?3,?4)",params![agent,role.role_id,role.label,serde_json::to_string(&role.scope)?])?;
                }
                if r.profile_id.is_none() {
                    r.scope = SpecialistScope {
                        projects: roles
                            .iter()
                            .flat_map(|v| v.scope.projects.clone())
                            .collect(),
                        transversal: roles.iter().any(|v| v.scope.transversal),
                    };
                }
                r.revision += 1;
                r.updated_at = now;
                self.save_specialist(&r)?;
                ok(json!(r))
            }
            ProposeProfile { definition, .. }
            | Import { definition, .. }
            | ProposeRevision { definition, .. } => {
                validate_definition(definition)?;
                let latest:Option<(u64,String)>=self.conn.query_row("SELECT revision,created_by FROM specialist_definitions WHERE profile_id=?1 ORDER BY revision DESC LIMIT 1",[&definition.profile_id],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
                let revision = match q {
                    ProposeRevision {
                        expected_revision, ..
                    } => {
                        let (rev, owner) = latest.ok_or(SpecialistError("profile_not_found"))?;
                        if rev != *expected_revision {
                            return Err(SpecialistError("profile_version_conflict"));
                        }
                        if !p.human() && owner != p.owner() {
                            return Err(SpecialistError("role_scope_denied"));
                        }
                        rev + 1
                    }
                    _ => {
                        if latest.is_some() {
                            return Err(SpecialistError("profile_version_conflict"));
                        }
                        1
                    }
                };
                self.conn.execute("INSERT INTO specialist_definitions(profile_id,revision,definition_json,digest,created_by,created_at) VALUES(?1,?2,?3,?4,?5,?6)",
                    params![definition.profile_id,revision,serde_json::to_string(definition)?,digest(definition)?,p.owner(),now])?;
                ok(
                    json!({"profile_id":definition.profile_id,"revision":revision,"activation":"inactive","approved":false,"digest":digest(definition)?}),
                )
            }
            ApproveRevision {
                profile_id,
                revision,
                ..
            } => {
                if self.get_definition(profile_id, *revision)?.is_none() {
                    return Err(SpecialistError("profile_not_found"));
                }
                self.conn.execute("UPDATE specialist_definitions SET approved_ref=?3 WHERE profile_id=?1 AND revision=?2",params![profile_id,revision,p.human_approval_ref])?;
                ok(json!({"profile_id":profile_id,"revision":revision,"approved":true}))
            }
            Activate {
                specialist_agent_id,
                profile_id,
                profile_revision,
                expected_revision,
                scope,
                durable,
                auto_start,
                mandate,
                budget,
                launch_policy,
                ..
            } => {
                validate_scope(scope)?;
                let d = self
                    .get_definition(profile_id, *profile_revision)?
                    .ok_or(SpecialistError("profile_not_found"))?;
                if d.approved_ref.is_none() {
                    return Err(SpecialistError("profile_approval_required"));
                }
                let mut r = self.ensure_specialist(specialist_agent_id, now)?;
                if r.revision != *expected_revision {
                    return Err(SpecialistError("profile_version_conflict"));
                }
                validate_launch(launch_policy.as_ref())?;
                r.capture_history = true;
                r.profile_id = Some(profile_id.clone());
                r.profile_revision = *profile_revision;
                r.scope = scope.clone();
                r.durable = *durable;
                r.enabled = true;
                r.auto_start = *auto_start;
                r.mandate = mandate
                    .clone()
                    .unwrap_or_else(|| default_mandate(&d.definition.roles));
                r.budget = budget.clone().unwrap_or_default();
                r.revision += 1;
                r.updated_at = now;
                r.grant = Some(new_grant(&r, p, launch_policy.clone(), now)?);
                self.save_specialist(&r)?;
                for role in &d.definition.roles {
                    self.conn.execute("INSERT INTO specialist_roles(agent_id,role_id,label,scope_json) VALUES(?1,?2,?2,?3) ON CONFLICT(agent_id,role_id) DO UPDATE SET scope_json=excluded.scope_json",params![specialist_agent_id,role,serde_json::to_string(scope)?])?;
                }
                ok(json!(r))
            }
            ConfigureMandate {
                specialist_agent_id,
                expected_revision,
                mandate,
                budget,
                scope,
                auto_start,
                launch_policy,
                ..
            } => {
                validate_scope(scope)?;
                validate_launch(launch_policy.as_ref())?;
                if mandate.len() > 16_000 {
                    return Err(SpecialistError("invalid_request"));
                }
                let mut r = self
                    .get_specialist(specialist_agent_id)?
                    .ok_or(SpecialistError("specialist_not_found"))?;
                if r.revision != *expected_revision {
                    return Err(SpecialistError("profile_version_conflict"));
                }
                r.scope = scope.clone();
                r.mandate = mandate.clone();
                r.budget = budget.clone().unwrap_or_default();
                r.auto_start = *auto_start;
                r.revision += 1;
                r.updated_at = now;
                r.grant = Some(new_grant(&r, p, launch_policy.clone(), now)?);
                self.save_specialist(&r)?;
                ok(json!(r))
            }
            Disable {
                specialist_agent_id,
                expected_revision,
                ..
            }
            | Revoke {
                specialist_agent_id,
                expected_revision,
                ..
            } => {
                let mut r = self
                    .get_specialist(specialist_agent_id)?
                    .ok_or(SpecialistError("specialist_not_found"))?;
                if r.revision != *expected_revision {
                    return Err(SpecialistError("profile_version_conflict"));
                }
                r.enabled = false;
                r.auto_start = false;
                r.revision += 1;
                r.updated_at = now;
                if matches!(q, Revoke { .. })
                    && let Some(g) = &mut r.grant
                {
                    g.revoked_at = Some(now);
                    g.revision = r.revision;
                }
                self.save_specialist(&r)?;
                ok(json!({"specialist":r,"running_tasks":"unchanged"}))
            }
            DesignateRole {
                specialist_agent_id,
                role_id,
                official,
                ..
            } => {
                let count=self.conn.execute("UPDATE specialist_roles SET official=?3,designation_ref=?4 WHERE agent_id=?1 AND role_id=?2",params![specialist_agent_id,role_id,official,p.human_approval_ref])?;
                if count == 0 {
                    return Err(SpecialistError("role_not_found"));
                }
                ok(json!({"agent_id":specialist_agent_id,"role_id":role_id,"official":official}))
            }
            MemoryWrite {
                specialist_agent_id,
                project_id,
                shared,
                kind,
                text,
                source,
                ..
            } => {
                if !p.owns(specialist_agent_id) || !p.permits(project_id) {
                    return Err(SpecialistError("role_scope_denied"));
                }
                if text.is_empty() || text.chars().count() > 16_000 {
                    return Err(SpecialistError("invalid_request"));
                }
                let source_key = digest(source)?;
                if let Some((id,forgotten))=self.conn.query_row("SELECT memory_id,forgotten FROM specialist_memory WHERE agent_id=?1 AND source_key=?2",params![specialist_agent_id,source_key],|r|Ok((r.get::<_,String>(0)?,r.get::<_,bool>(1)?))).optional()? {
                    return ok(json!({"memory_id":id,"status":if forgotten{"forgotten"}else{"already_captured"}}));
                }
                let id = Uuid::new_v4().to_string();
                let kind = serde_json::to_string(kind)?;
                self.conn.execute("INSERT INTO specialist_memory(memory_id,agent_id,project_id,shared,kind,text,normalized,source_json,source_key,revision,forgotten,updated_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,1,0,?10)",
                    params![id,specialist_agent_id,project_id,shared,kind,text,fold_for_search(text),serde_json::to_string(source)?,source_key,now])?;
                self.conn.execute("INSERT INTO specialist_memory_revisions(memory_id,revision,text,kind,updated_at) VALUES(?1,1,?2,?3,?4)",params![id,text,kind,now])?;
                ok(
                    json!({"memory_id":id,"revision":1,"kind":serde_json::from_str::<Value>(&kind)?}),
                )
            }
            MemoryCorrect {
                memory_id,
                expected_revision,
                text,
                kind,
                ..
            } => {
                let (agent,revision,forgotten,project):(String,u64,bool,String)=self.conn.query_row("SELECT agent_id,revision,forgotten,project_id FROM specialist_memory WHERE memory_id=?1",[memory_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?.ok_or(SpecialistError("memory_not_found"))?;
                if !p.owns(&agent) || (p.human() && !p.permits(&project)) {
                    return Err(SpecialistError("role_scope_denied"));
                }
                if revision != *expected_revision || forgotten {
                    return Err(SpecialistError("profile_version_conflict"));
                }
                if text.is_empty() || text.chars().count() > 16_000 {
                    return Err(SpecialistError("invalid_request"));
                }
                let kind = serde_json::to_string(kind)?;
                self.conn.execute("UPDATE specialist_memory SET text=?2,normalized=?3,kind=?4,revision=revision+1,updated_at=?5 WHERE memory_id=?1",params![memory_id,text,fold_for_search(text),kind,now])?;
                self.conn.execute("INSERT INTO specialist_memory_revisions(memory_id,revision,text,kind,updated_at) VALUES(?1,?2,?3,?4,?5)",params![memory_id,revision+1,text,kind,now])?;
                ok(json!({"memory_id":memory_id,"revision":revision+1}))
            }
            MemoryForget {
                memory_id,
                expected_revision,
                ..
            } => {
                let (agent, revision, project): (String, u64, String) = self
                    .conn
                    .query_row(
                        "SELECT agent_id,revision,project_id FROM specialist_memory WHERE memory_id=?1",
                        [memory_id],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                    )
                    .optional()?
                    .ok_or(SpecialistError("memory_not_found"))?;
                if !p.owns(&agent) || (p.human() && !p.permits(&project)) {
                    return Err(SpecialistError("role_scope_denied"));
                }
                if revision != *expected_revision {
                    return Err(SpecialistError("profile_version_conflict"));
                }
                self.conn.execute("UPDATE specialist_memory SET text='',normalized='',forgotten=1,revision=revision+1,updated_at=?2 WHERE memory_id=?1",params![memory_id,now])?;
                self.conn.execute(
                    "DELETE FROM specialist_memory_revisions WHERE memory_id=?1",
                    [memory_id],
                )?;
                ok(
                    json!({"memory_id":memory_id,"revision":revision+1,"status":"forgotten","source_products":"unchanged"}),
                )
            }

            MemoryVersions {
                memory_id,
                cursor,
                limit,
                ..
            } => {
                let (agent,project,forgotten):(String,String,bool)=self.conn.query_row(
                    "SELECT agent_id,project_id,forgotten FROM specialist_memory WHERE memory_id=?1",[memory_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))
                    .optional()?.ok_or(SpecialistError("memory_not_found"))?;
                if !p.owns(&agent) || (p.human() && !p.permits(&project)) {
                    return Err(SpecialistError("role_scope_denied"));
                }
                if forgotten {
                    return ok(
                        json!({"memory_id":memory_id,"items":[],"forgotten":true,"next_cursor":Value::Null}),
                    );
                }
                let limit = limit.unwrap_or(10).clamp(1, 50);
                let before = cursor.unwrap_or(i64::MAX as u64).min(i64::MAX as u64);
                let mut q=self.conn.prepare("SELECT revision,text,kind,updated_at FROM specialist_memory_revisions WHERE memory_id=?1 AND revision<?2 ORDER BY revision DESC LIMIT ?3")?;
                let rows = q.query_map(params![memory_id, before, limit + 1], |r| {
                    Ok((
                        r.get::<_, u64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, i64>(3)?,
                    ))
                })?;
                let mut items = Vec::new();
                for row in rows {
                    let row = row?;
                    items.push(json!({"revision":row.0,"text":row.1,"kind":serde_json::from_str::<Value>(&row.2)?,"updated_at":row.3}));
                }
                let more = items.len() > limit as usize;
                items.truncate(limit as usize);
                let next = if more {
                    items.last().and_then(|v| v["revision"].as_u64())
                } else {
                    None
                };
                ok(
                    json!({"memory_id":memory_id,"items":items,"forgotten":false,"next_cursor":next}),
                )
            }
            MemoryRecall {
                specialist_agent_id,
                project_id,
                query,
                cursor,
                limit,
                ..
            } => {
                if !p.permits(project_id) {
                    return Err(SpecialistError("role_scope_denied"));
                }
                let record = self
                    .get_specialist(specialist_agent_id)?
                    .ok_or(SpecialistError("specialist_not_found"))?;
                if !p.owns(specialist_agent_id) && !record.scope.permits(project_id) {
                    return Err(SpecialistError("role_scope_denied"));
                }
                let terms = fold_for_search(query)
                    .split_whitespace()
                    .take(12)
                    .map(str::to_owned)
                    .collect::<Vec<_>>();
                let limit = limit.unwrap_or(12).clamp(1, 50);
                let offset = cursor
                    .as_ref()
                    .map(|s| s.parse::<u32>())
                    .transpose()
                    .map_err(|_| SpecialistError("invalid_request"))?
                    .unwrap_or(0)
                    .min(1_000_000);
                let owner_recall = p.owns(specialist_agent_id);
                let all_sources = owner_recall && (record.scope.transversal || p.human());
                let source_projects = if owner_recall {
                    record.scope.projects.clone()
                } else {
                    Vec::new()
                };
                let score = if terms.is_empty() {
                    "0".into()
                } else {
                    terms
                        .iter()
                        .enumerate()
                        .map(|(i, _)| format!("(instr(normalized,?{})>0)", i + 5))
                        .collect::<Vec<_>>()
                        .join("+")
                };
                let source_filter = if all_sources {
                    "1".to_owned()
                } else if source_projects.is_empty() {
                    "0".to_owned()
                } else {
                    format!(
                        "project_id IN ({})",
                        (0..source_projects.len())
                            .map(|i| format!("?{}", 5 + terms.len() + i))
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                };
                let relevant = if terms.is_empty() {
                    "1".to_owned()
                } else {
                    format!("({score})>0")
                };
                let sql = format!(
                    "SELECT memory_id,project_id,kind,text,source_json,revision,updated_at,({score}) AS relevance FROM specialist_memory WHERE agent_id=?1 AND forgotten=0 AND (project_id=?2 OR shared=1 OR {source_filter}) AND {relevant} ORDER BY relevance DESC,(project_id=?2) DESC,updated_at DESC,memory_id LIMIT ?3 OFFSET ?4"
                );
                let mut values = vec![
                    rusqlite::types::Value::Text(specialist_agent_id.clone()),
                    rusqlite::types::Value::Text(project_id.clone()),
                    rusqlite::types::Value::Integer((limit + 1) as i64),
                    rusqlite::types::Value::Integer(offset as i64),
                ];
                values.extend(terms.into_iter().map(rusqlite::types::Value::Text));
                if !all_sources {
                    values.extend(
                        source_projects
                            .into_iter()
                            .map(rusqlite::types::Value::Text),
                    );
                }
                let mut stmt = self.conn.prepare(&sql)?;
                let rows = stmt.query_map(rusqlite::params_from_iter(values), |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, String>(4)?,
                        r.get::<_, u64>(5)?,
                        r.get::<_, i64>(6)?,
                        r.get::<_, i64>(7)?,
                    ))
                })?;
                let mut items = Vec::new();
                for row in rows {
                    let r = row?;
                    items.push(json!({"memory_id":r.0,"project_id":r.1,"kind":serde_json::from_str::<Value>(&r.2)?,"text":r.3,"source":serde_json::from_str::<Value>(&r.4)?,"revision":r.5,"updated_at":r.6,"relevance":r.7,"ranking":"lexical_then_current_project_then_date"}));
                }
                let more = items.len() > limit as usize;
                items.truncate(limit as usize);
                ok(
                    json!({"items":items,"next_cursor":more.then(||(offset+limit).to_string()),"window_limit":limit,"retention_limit":Value::Null}),
                )
            }
            Export {
                profile_id,
                revision,
                ..
            } => {
                let r = self
                    .get_definition(profile_id, *revision)?
                    .ok_or(SpecialistError("profile_not_found"))?;
                if !p.human() && r.created_by != p.owner() {
                    return Err(SpecialistError("role_scope_denied"));
                }
                ok(
                    json!({"definition":r.definition,"revision":r.revision,"digest":r.digest,"activation":"not_exported","memory":"not_exported","grants":"not_exported"}),
                )
            }
            Convert {
                specialist_agent_id,
                profile_id,
                profile_revision,
                expected_revision,
                ..
            } => {
                let d = self
                    .get_definition(profile_id, *profile_revision)?
                    .ok_or(SpecialistError("profile_not_found"))?;
                let mut r = self.ensure_specialist(specialist_agent_id, now)?;
                if r.revision != *expected_revision {
                    return Err(SpecialistError("profile_version_conflict"));
                }
                r.grant = None;
                r.durable = false;
                r.auto_start = false;
                r.capture_history = true;
                r.profile_id = Some(profile_id.clone());
                r.profile_revision = d.revision;
                r.revision += 1;
                r.updated_at = now;
                self.save_specialist(&r)?;
                ok(
                    json!({"specialist":r,"history":"preserved","memory_capture":"source_observation_required","activation":"approval_required"}),
                )
            }
            RequestTemporary { .. }
            | RequestApproval { .. }
            | CaptureHistory { .. }
            | RequestRole { .. }
            | RequestStatus { .. }
            | RequestTeam { .. } => Err(SpecialistError("native_admission_required")),
        }
    }
    pub fn roles_for(&self, id: &str) -> Result<Vec<Value>, SpecialistError> {
        let mut s=self.conn.prepare("SELECT role_id,label,scope_json,official,designation_ref FROM specialist_roles WHERE agent_id=?1 ORDER BY role_id")?;
        let rows = s.query_map([id], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, bool>(3)?,
                r.get::<_, Option<String>>(4)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let r = row?;
            out.push(json!({"role_id":r.0,"label":r.1,"scope":serde_json::from_str::<Value>(&r.2)?,"official":r.3,"designation_ref":r.4}));
        }
        Ok(out)
    }
}
fn validate_launch(policy: Option<&SpecialistLaunchPolicy>) -> Result<(), SpecialistError> {
    if policy.is_some_and(|p| {
        p.agent_type.is_empty()
            || p.agent_type.len() > 256
            || !std::path::Path::new(&p.cwd).is_absolute()
    }) {
        return Err(SpecialistError("invalid_request"));
    }
    Ok(())
}
fn new_grant(
    r: &SpecialistRecord,
    p: &SpecialistPrincipal,
    launch_policy: Option<SpecialistLaunchPolicy>,
    now: i64,
) -> Result<MandateGrant, SpecialistError> {
    let approval_ref = p
        .human_approval_ref
        .clone()
        .ok_or(SpecialistError("human_principal_required"))?;
    Ok(MandateGrant {
        profile_id: r.profile_id.clone(),
        profile_revision: r.profile_revision,
        grant_id: r
            .grant
            .as_ref()
            .map(|g| g.grant_id.clone())
            .unwrap_or_else(|| Uuid::new_v4().to_string()),
        revision: r.revision,
        approved_by: approval_ref
            .rsplit_once(':')
            .map(|v| v.0)
            .unwrap_or("human-control")
            .into(),
        approval_ref,
        allowed_projects: r.scope.clone(),
        launch_policy,
        issued_at: now,
        revoked_at: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn definition(id: &str) -> SpecialistDefinition {
        SpecialistDefinition {
            profile_id: id.into(),
            name: "Mainteneur".into(),
            kind: SpecialistDefinitionKind::Profile,
            instructions: "Conserver les travaux et corriger dans le mandat.".into(),
            roles: vec!["mainteneur-bridget".into()],
            procedures: vec![],
            requirements: vec![],
            preferences: SpecialistPreferences::default(),
            members: vec![],
            provenance: None,
        }
    }
    #[test]
    fn spec222_specialist_human_name_tracks_profile_in_list_search_and_detail() {
        let root = std::env::temp_dir().join(format!("bridget222-names-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut store = AgentProfileStore::open(&root.join("s.db")).unwrap();
        let agent = Uuid::new_v4().to_string();
        let p = SpecialistPrincipal {
            agent_id: Some(agent.clone()), human_approval_ref: None,
            allowed_projects: vec!["bridget".into()],
        };
        let declare = serde_json::from_value(json!({"version":1,"action":"declare_roles",
            "request_id":"names-declare","expected_revision":0,
            "roles":[{"role_id":"patate","label":"Patate",
                "scope":{"projects":["bridget"],"transversal":false}}]})).unwrap();
        store.specialist_apply(&declare, &p, 10).unwrap();
        store.rename_display_name(&agent, "bridge1016", |_| false).unwrap();
        let before = store.get_specialist(&agent).unwrap().unwrap();
        let queries = [
            json!({"version":1,"action":"list"}),
            json!({"version":1,"action":"search","role_id":"patate","project_id":"bridget"}),
            json!({"version":1,"action":"get","specialist_agent_id":agent}),
        ];
        // Le nom vient du profil persistant, même sans présence en ligne.
        for name in ["bridge1016", "Gentil"] {
            store.rename_display_name(&agent, name, |_| false).unwrap();
            for value in &queries {
                let q = serde_json::from_value(value.clone()).unwrap();
                let v = store.specialist_apply(&q, &p, 11).unwrap();
                let item = if value["action"] == "get" { &v["data"] }
                    else { &v["data"]["items"][0] };
                assert_eq!(item["display_name"], name);
                assert_eq!(item["specialist"]["agent_id"], agent);
                assert_eq!(item["roles"][0]["role_id"], "patate");
                assert_eq!(serde_json::to_value(store.get_specialist(&agent).unwrap().unwrap()).unwrap(),
                    serde_json::to_value(&before).unwrap());
            }
        }
        // L'enrichissement intervient après le filtre d'autorisation, sans fuite du nom.
        let other = SpecialistPrincipal {
            agent_id: Some(Uuid::new_v4().to_string()), human_approval_ref: None,
            allowed_projects: vec!["autre-projet".into()],
        };
        let q = serde_json::from_value(queries[0].clone()).unwrap();
        let hidden = store.specialist_apply(&q, &other, 12).unwrap();
        assert!(hidden["data"]["items"].as_array().unwrap().is_empty());
        assert!(!hidden.to_string().contains("Gentil"));
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn spec222_profiles_grants_and_versions_require_human() {
        let root = std::env::temp_dir().join(format!("bridget222-profile-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut store = AgentProfileStore::open(&root.join("s.db")).unwrap();
        let agent = Uuid::new_v4().to_string();
        let id = Uuid::new_v4().to_string();
        let p = SpecialistPrincipal {
            agent_id: Some(agent.clone()),
            human_approval_ref: None,
            allowed_projects: vec!["p".into()],
        };
        let h = SpecialistPrincipal {
            agent_id: None,
            human_approval_ref: Some("human:approval".into()),
            allowed_projects: vec!["*".into()],
        };
        let propose = SpecialistRequest::ProposeProfile {
            version: 1,
            request_id: "proposal".into(),
            definition: definition(&id),
        };
        assert_eq!(
            store.specialist_apply(&propose, &p, 1).unwrap()["data"]["revision"],
            1
        );
        assert_eq!(
            store.specialist_apply(&propose, &p, 2).unwrap()["data"]["revision"],
            1
        );
        let activation = SpecialistRequest::Activate {
            version: 1,
            request_id: "activate".into(),
            specialist_agent_id: agent.clone(),
            profile_id: id.clone(),
            profile_revision: 1,
            expected_revision: 0,
            scope: SpecialistScope {
                projects: vec!["p".into()],
                transversal: false,
            },
            durable: true,
            auto_start: true,
            mandate: None,
            budget: None,
            launch_policy: None,
        };
        assert_eq!(
            store.specialist_apply(&activation, &p, 3).unwrap_err().0,
            "human_principal_required"
        );
        assert_eq!(
            store.specialist_apply(&activation, &h, 3).unwrap_err().0,
            "profile_approval_required"
        );
        store
            .specialist_apply(
                &SpecialistRequest::ApproveRevision {
                    version: 1,
                    request_id: "approve".into(),
                    profile_id: id.clone(),
                    revision: 1,
                },
                &h,
                4,
            )
            .unwrap();
        store.specialist_apply(&activation, &h, 5).unwrap();
        let active = store.get_specialist(&agent).unwrap().unwrap();
        assert!(active.durable);
        assert_eq!(active.budget, SpecialistBudget::default());
        assert!(
            active
                .grant
                .as_ref()
                .unwrap()
                .approval_ref
                .contains("approval")
        );
        let mut changed = definition(&id);
        changed.instructions = "Autre version".into();
        store
            .specialist_apply(
                &SpecialistRequest::ProposeRevision {
                    version: 1,
                    request_id: "rev".into(),
                    expected_revision: 1,
                    definition: changed,
                },
                &p,
                6,
            )
            .unwrap();
        assert_eq!(
            store
                .get_specialist(&agent)
                .unwrap()
                .unwrap()
                .profile_revision,
            1
        );
        assert_eq!(
            store
                .get_definition(&id, 1)
                .unwrap()
                .unwrap()
                .definition
                .instructions,
            definition(&id).instructions
        );
        store
            .specialist_apply(
                &SpecialistRequest::Disable {
                    version: 1,
                    request_id: "disable".into(),
                    specialist_agent_id: agent.clone(),
                    expected_revision: 1,
                },
                &h,
                7,
            )
            .unwrap();
        assert!(!store.get_specialist(&agent).unwrap().unwrap().enabled);
        assert_eq!(
            store
                .get_specialist(&agent)
                .unwrap()
                .unwrap()
                .grant
                .unwrap()
                .revoked_at,
            None
        );
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn spec222_memory_prefers_project_and_forgetting_survives_capture() {
        let root = std::env::temp_dir().join(format!("bridget222-memory-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        let mut store = AgentProfileStore::open(&root.join("s.db")).unwrap();
        let agent = Uuid::new_v4().to_string();
        store.ensure_specialist(&agent, 0).unwrap();
        let p = SpecialistPrincipal {
            agent_id: Some(agent.clone()),
            human_approval_ref: None,
            allowed_projects: vec!["p".into(), "other".into()],
        };
        let write = |key: &str, project: &str| SpecialistRequest::MemoryWrite {
            version: 1,
            request_id: key.into(),
            specialist_agent_id: agent.clone(),
            project_id: project.into(),
            shared: true,
            kind: SpecialistMemoryKind::ReportedResult,
            text: "SQLite verrou de reprise".into(),
            source: SpecialistSource::Statement {
                request_id: key.into(),
            },
        };
        let a = store.specialist_apply(&write("a", "p"), &p, 1).unwrap();
        store
            .specialist_apply(&write("b", "other"), &p, 10)
            .unwrap();
        let recall = SpecialistRequest::MemoryRecall {
            version: 1,
            specialist_agent_id: agent.clone(),
            project_id: "p".into(),
            query: "sqlite reprise".into(),
            cursor: None,
            limit: Some(10),
        };
        let memories = store.specialist_apply(&recall, &p, 11).unwrap();
        assert_eq!(memories["data"]["items"][0]["project_id"], "p");
        assert_eq!(memories["data"]["items"].as_array().unwrap().len(), 2);
        let id = a["data"]["memory_id"].as_str().unwrap();
        let versions = SpecialistRequest::MemoryVersions {
            version: 1,
            memory_id: id.into(),
            cursor: None,
            limit: Some(1),
        };
        assert_eq!(
            store.specialist_apply(&versions, &p, 11).unwrap()["data"]["items"][0]["revision"],
            1
        );
        store
            .specialist_apply(
                &SpecialistRequest::MemoryCorrect {
                    version: 1,
                    request_id: "fix".into(),
                    memory_id: id.into(),
                    expected_revision: 1,
                    text: "SQLite verrou de reprise corrigé".into(),
                    kind: SpecialistMemoryKind::Decision,
                },
                &p,
                11,
            )
            .unwrap();
        let history = store.specialist_apply(&versions, &p, 11).unwrap();
        assert_eq!(history["data"]["items"][0]["revision"], 2);
        assert_eq!(history["data"]["next_cursor"], 2);
        let mut irrelevant = recall.clone();
        if let SpecialistRequest::MemoryRecall { query, .. } = &mut irrelevant {
            *query = "astronomie introuvable".into();
        }
        assert!(
            store.specialist_apply(&irrelevant, &p, 11).unwrap()["data"]["items"]
                .as_array()
                .unwrap()
                .is_empty()
        );

        store
            .specialist_apply(
                &SpecialistRequest::MemoryForget {
                    version: 1,
                    request_id: "forget".into(),
                    memory_id: id.into(),
                    expected_revision: 2,
                },
                &p,
                12,
            )
            .unwrap();
        let mut recapture = write("a", "p");
        if let SpecialistRequest::MemoryWrite { request_id, .. } = &mut recapture {
            *request_id = "recapture".into();
        }
        assert_eq!(
            store.specialist_apply(&recapture, &p, 13).unwrap()["data"]["status"],
            "forgotten"
        );
        assert_eq!(
            store.specialist_apply(&recall, &p, 14).unwrap()["data"]["items"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(
            store.specialist_apply(&versions, &p, 15).unwrap()["data"]["items"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        drop(store);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn spec222_claim_scope_is_bounded_and_authority_is_private() {
        let path = std::env::temp_dir().join(format!("bridget222-scope-{}.db", Uuid::new_v4()));
        let mut store = AgentProfileStore::open(&path).unwrap();
        let agent = Uuid::new_v4().to_string();
        let p = SpecialistPrincipal {
            agent_id: Some(agent.clone()),
            human_approval_ref: None,
            allowed_projects: vec!["p".into()],
        };
        for (n, scope) in [json!({"transversal":true}), json!({"projects":["q"]})]
            .into_iter()
            .enumerate()
        {
            let q=serde_json::from_value(json!({"version":1,"action":"declare_roles","request_id":format!("bad{n}"),"expected_revision":0,
                "roles":[{"role_id":"recherche","label":"Recherche","scope":scope}]})).unwrap();
            assert_eq!(
                store.specialist_apply(&q, &p, 10).unwrap_err().0,
                "role_scope_denied"
            );
        }
        assert!(
            store
                .specialist_candidates("recherche", "q")
                .unwrap()
                .is_empty()
        );
        let q = serde_json::from_value(
            json!({"version":1,"action":"declare_roles","request_id":"own","expected_revision":0,
            "roles":[{"role_id":"recherche","label":"Recherche","scope":{"projects":["p"]}}]}),
        )
        .unwrap();
        store.specialist_apply(&q, &p, 11).unwrap();
        let mut record = store.get_specialist(&agent).unwrap().unwrap();
        let h = SpecialistPrincipal {
            agent_id: None,
            human_approval_ref: Some("native-control".into()),
            allowed_projects: vec!["*".into()],
        };
        record.grant = Some(new_grant(&record, &h, None, 12).unwrap());
        store.save_specialist(&record).unwrap();
        let other = SpecialistPrincipal {
            agent_id: Some(Uuid::new_v4().to_string()),
            human_approval_ref: None,
            allowed_projects: vec!["p".into()],
        };
        let get = SpecialistRequest::Get {
            version: 1,
            specialist_agent_id: agent,
        };
        let limited_human = SpecialistPrincipal {
            agent_id: None,
            human_approval_ref: Some("mobile-scoped".into()),
            allowed_projects: vec!["q".into()],
        };
        assert_eq!(
            store
                .specialist_apply(&get, &limited_human, 13)
                .unwrap_err()
                .0,
            "role_scope_denied"
        );
        let list = SpecialistRequest::List {
            version: 1,
            cursor: None,
            limit: Some(10),
        };
        assert!(
            store.specialist_apply(&list, &limited_human, 13).unwrap()["data"]["items"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let designation = SpecialistRequest::DesignateRole {
            version: 1,
            request_id: "official".into(),
            specialist_agent_id: record.agent_id.clone(),
            role_id: "recherche".into(),
            official: true,
        };
        store.specialist_apply(&designation, &h, 13).unwrap();
        let joined = store.specialist_candidates("recherche", "p").unwrap();
        assert_eq!(joined.len(), 1);
        assert!(
            joined[0].1,
            "le JOIN fournit la désignation sans requête par candidat"
        );
        let view = store.specialist_apply(&get, &other, 13).unwrap();
        assert!(view["data"]["specialist"].get("grant").is_none());
        assert!(view["data"]["specialist"].get("owner").is_none());
        assert_eq!(
            new_grant(&record, &p, None, 14).unwrap_err().0,
            "human_principal_required"
        );
    }
}

impl AgentProfileStore {
    pub(crate) fn source_command_begin(
        &self,
        q: &SpecialistRequest,
        p: &SpecialistPrincipal,
    ) -> Result<Option<Value>, SpecialistError> {
        let raw = serde_json::to_value(q)?;
        let id = raw["request_id"]
            .as_str()
            .ok_or(SpecialistError("invalid_request"))?;
        let canon = Sha256::digest(serde_json::to_vec(q)?).to_vec();
        if let Some((stored,response))=self.conn.query_row("SELECT canonical,response FROM specialist_mutations WHERE issuer=?1 AND request_id=?2",params![p.owner(),id],|r|Ok((r.get::<_,Vec<u8>>(0)?,r.get::<_,String>(1)?))).optional()? {
            if stored!=canon{return Err(SpecialistError("envelope_mismatch"));}
            let response:Value=serde_json::from_str(&response)?;
            return Ok((response["status"]!="source_reading").then_some(response));
        }
        self.conn.execute("INSERT INTO specialist_mutations(issuer,request_id,canonical,response) VALUES(?1,?2,?3,?4)",params![p.owner(),id,canon,json!({"version":1,"status":"source_reading"}).to_string()])?;
        Ok(None)
    }
    pub(crate) fn source_command_finish(
        &self,
        q: &SpecialistRequest,
        p: &SpecialistPrincipal,
        response: &Value,
    ) -> Result<(), SpecialistError> {
        let raw = serde_json::to_value(q)?;
        let id = raw["request_id"]
            .as_str()
            .ok_or(SpecialistError("invalid_request"))?;
        let canon = Sha256::digest(serde_json::to_vec(q)?).to_vec();
        let changed=self.conn.execute("UPDATE specialist_mutations SET response=?4 WHERE issuer=?1 AND request_id=?2 AND canonical=?3",params![p.owner(),id,canon,serde_json::to_string(response)?])?;
        if changed != 1 {
            return Err(SpecialistError("envelope_mismatch"));
        }
        Ok(())
    }
    pub(crate) fn mutation_receipt(
        &self,
        p: &SpecialistPrincipal,
        id: &str,
    ) -> Result<Option<Value>, SpecialistError> {
        let result: Option<String> = self
            .conn
            .query_row(
                "SELECT response FROM specialist_mutations WHERE issuer=?1 AND request_id=?2",
                params![p.owner(), id],
                |r| r.get(0),
            )
            .optional()?;
        result
            .map(|v| serde_json::from_str(&v).map_err(Into::into))
            .transpose()
    }
}

impl AgentProfileStore {
    pub(crate) fn record_capture_observation(
        &self,
        agent: &str,
        result: &Value,
        now: i64,
    ) -> Result<(), SpecialistError> {
        let code = result
            .get("code")
            .and_then(Value::as_str)
            .filter(|s| s.len() <= 80 && s.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'));
        let data = &result["data"];
        let previous = self.capture_observation(agent)?.unwrap_or(Value::Null);
        let cursor = data["next_seq"]
            .as_u64()
            .or_else(|| previous["next_seq"].as_u64());
        let last_success = if result["status"] == "ok" {
            Some(now)
        } else {
            previous["last_success_at"].as_i64()
        };
        let observation = json!({"status":result["status"],"code":code,"observed_at":now,
            "next_seq":cursor,"last_success_at":last_success,"caught_up":data["caught_up"].as_bool(),
            "gap":data["gap"].as_object().map(|gap|json!({"from_seq":gap.get("from_seq").and_then(Value::as_u64),"to_seq":gap.get("to_seq").and_then(Value::as_u64),
                "reason":gap.get("reason").and_then(Value::as_str).filter(|r|matches!(*r,"journal_gap"|"entry_too_large"))}))});
        self.conn.execute("INSERT INTO specialist_memory_sources(agent_id,observation,observed_at) VALUES(?1,?2,?3) ON CONFLICT(agent_id) DO UPDATE SET observation=excluded.observation,observed_at=excluded.observed_at",
            params![agent,observation.to_string(),now])?;
        Ok(())
    }
    pub(crate) fn capture_observation(
        &self,
        agent: &str,
    ) -> Result<Option<Value>, SpecialistError> {
        let row: Option<String> = self
            .conn
            .query_row(
                "SELECT observation FROM specialist_memory_sources WHERE agent_id=?1",
                [agent],
                |r| r.get(0),
            )
            .optional()?;
        row.map(|v| serde_json::from_str(&v).map_err(Into::into))
            .transpose()
    }
}

impl AgentProfileStore {
    /// Le caller a prouvé le tuple enfant/instance/Task en cours ; aucune entrée JSON ne donne cette preuve.
    pub(crate) fn authorize_initial_profile_for_task(
        &self,
        profile: &str,
        revision: u64,
        parent: &str,
        task: &str,
        child: &str,
    ) -> Result<(), SpecialistError> {
        let d = self
            .get_definition(profile, revision)?
            .ok_or(SpecialistError("profile_not_found"))?;
        if d.created_by != parent {
            return Err(SpecialistError("human_reuse_approval_required"));
        }
        let existing:Option<(String,String)>=self.conn.query_row("SELECT task_id,child_agent_id FROM specialist_initial_activations WHERE profile_id=?1 AND profile_revision=?2",params![profile,revision],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        if let Some((recorded_task, recorded_child)) = existing {
            if recorded_task != task || recorded_child != child {
                return Err(SpecialistError("human_reuse_approval_required"));
            }
            return Ok(());
        }
        self.conn.execute("INSERT INTO specialist_initial_activations(profile_id,profile_revision,task_id,child_agent_id) VALUES(?1,?2,?3,?4)",params![profile,revision,task,child])?;
        Ok(())
    }
}

impl AgentProfileStore {
    /// Les trois collecteurs (Task, réponse et journal) gardent toute leur page ou aucun fragment.
    pub(crate) fn write_capture_batch(
        &mut self,
        memories: &[SpecialistRequest],
        actor: &SpecialistPrincipal,
        receipt: Option<(&SpecialistRequest, &SpecialistPrincipal, &Value)>,
        observation: Option<(&str, &Value)>,
        now: i64,
    ) -> Result<(), SpecialistError> {
        if memories.len() > 256
            || memories
                .iter()
                .any(|q| !matches!(q, SpecialistRequest::MemoryWrite { .. }))
        {
            return Err(SpecialistError("invalid_request"));
        }
        self.conn.execute_batch("SAVEPOINT specialist_capture")?;
        let result = (|| -> Result<(), SpecialistError> {
            for memory in memories {
                self.specialist_apply(memory, actor, now)?;
            }
            if let Some((request, principal, value)) = receipt {
                self.source_command_finish(request, principal, value)?;
            }
            if let Some((agent, value)) = observation {
                self.record_capture_observation(agent, value, now)?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.conn.execute_batch("RELEASE specialist_capture")?;
                Ok(())
            }
            Err(error) => {
                self.conn
                    .execute_batch("ROLLBACK TO specialist_capture; RELEASE specialist_capture")?;
                Err(error)
            }
        }
    }
}

impl AgentProfileStore {
    pub(crate) fn reusable_definition(
        &self,
        id: &str,
        revision: u64,
        p: &SpecialistPrincipal,
    ) -> Result<bool, SpecialistError> {
        let mut q=self.conn.prepare("SELECT record_json FROM specialists WHERE json_extract(record_json,'$.profile_id')=?1 AND json_extract(record_json,'$.profile_revision')=?2")?;
        let rows = q.query_map(params![id, revision], |r| r.get::<_, String>(0))?;
        for row in rows {
            let record: SpecialistRecord = serde_json::from_str(&row?)?;
            if !record.enabled || !record.durable {
                continue;
            }
            let Some(grant) = record.grant.as_ref() else {
                continue;
            };
            if grant.revoked_at.is_some()
                || grant.profile_id.as_deref() != Some(id)
                || grant.profile_revision != revision
            {
                continue;
            }
            if record.scope.transversal
                || record
                    .scope
                    .projects
                    .iter()
                    .any(|project| p.permits(project))
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

impl AgentProfileStore {
    pub(crate) fn reserve_temporary(
        &self,
        owner: &str,
        key: &str,
        profile: &str,
        revision: u64,
        canonical: &[u8],
        delegation: &bridget_transport::protocol::AutoDelegateV2,
        source_project: &str,
    ) -> Result<(), SpecialistError> {
        let prior:Option<(Vec<u8>,String)>=self.conn.query_row("SELECT canonical,delegation FROM specialist_temporary_requests WHERE owner=?1 AND request_id=?2",
            params![owner,key],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        if let Some((prior, body)) = prior {
            if prior != canonical || body != serde_json::to_string(delegation)? {
                return Err(SpecialistError("envelope_mismatch"));
            }
            return Ok(());
        }
        let used:bool=self.conn.query_row("SELECT EXISTS(SELECT 1 FROM specialist_temporary_requests WHERE profile_id=?1 AND profile_revision=?2 UNION SELECT 1 FROM specialist_initial_activations WHERE profile_id=?1 AND profile_revision=?2)",params![profile,revision],|r|r.get(0))?;
        if used {
            return Err(SpecialistError("human_reuse_approval_required"));
        }
        self.conn.execute("INSERT INTO specialist_temporary_requests(owner,request_id,profile_id,profile_revision,canonical,delegation,source_project)VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![owner,key,profile,revision,canonical,serde_json::to_string(delegation)?,source_project])?;
        Ok(())
    }
    pub(crate) fn temporary_request(
        &self,
        owner: &str,
        key: &str,
    ) -> Result<Option<Value>, SpecialistError> {
        let body:Option<(String,String)>=self.conn.query_row("SELECT delegation,source_project FROM specialist_temporary_requests WHERE owner=?1 AND request_id=?2",
            params![owner,key],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        body.map(|(v, project)| {
            let mut value: Value = serde_json::from_str(&v)?;
            value["specialist_source_project"] = json!(project);
            Ok(value)
        })
        .transpose()
    }
}
