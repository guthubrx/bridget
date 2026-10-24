//! Adaptation humaine des artefacts natifs : aucun chemin reçu ni identité d'agent empruntée.
use super::*;
use crate::{
    artifact_store::{ArtifactStore, ArtifactStoreError},
    artifact_types::{
        ArtifactKind, ArtifactPublicationV1, ArtifactSourceAccess, ArtifactSourceKind,
        ArtifactSourceV1, PublicationReason, sha256_hex,
    },
    mobile::store::{Device, Operation, Store},
};
use bridget_transport::protocol::{MobileFileUpload, MobileOperation, MobileOperationRequest};
use serde_json::{Value, json};
use std::path::Path;

const MAX_FILE_BYTES: usize = 8 * 1024 * 1024;

fn fail(code: &str) -> DaemonToWrapper {
    DaemonToWrapper::MobileOperationResult {
        result: json!({"status":"refused","code":code}),
    }
}
fn success(value: Value) -> DaemonToWrapper {
    DaemonToWrapper::MobileOperationResult { result: value }
}
fn uuid(id: &str) -> bool {
    Uuid::parse_str(id).is_ok_and(|v| v.get_version_num() == 4 && v.to_string() == id)
}
fn upload_key(device: &str, id: &str) -> String {
    format!("mobile177:file:{device}:{id}")
}

/// Le contexte est dérivé de la route attestée, jamais d'un project_id fourni par HTTP.
#[derive(Clone, PartialEq)]
struct Binding {
    agent: String,
    instance: String,
    project: String,
    conversation: String,
}
fn binding(
    st: &DaemonState,
    device: &Device,
    agent: &str,
    op: Operation,
) -> Result<Binding, &'static str> {
    if !uuid(agent) {
        return Err("invalid_request");
    }
    let conn = st
        .router
        .get_agent(agent)
        .ok_or("not_found")?
        .connection_id
        .clone();
    let (actual, instance, project) = artifact_access_scope(st, &conn).map_err(|_| "not_found")?;
    if actual != agent {
        return Err("not_found");
    }
    let host = &st.presences.get(&instance).ok_or("not_found")?.host;
    if !device
        .scope
        .permits(op, host, agent_communication_project(st, agent).as_ref())
    {
        return Err("not_found");
    }
    Ok(Binding {
        agent: actual,
        instance,
        conversation: format!("conversation:{project}:{agent}"),
        project,
    })
}
fn valid_file_name(name: &str) -> bool {
    !name.trim().is_empty()
        && name.len() <= 240
        && name != "."
        && name != ".."
        && !name
            .chars()
            .any(|c| c.is_control() || c == '/' || c == '\\')
}
fn allowed_media(media: &str) -> bool {
    matches!(
        media,
        "application/pdf" | "text/plain" | "image/png" | "image/jpeg"
    )
}
fn valid_bytes(media: &str, bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes.len() <= MAX_FILE_BYTES
        && match media {
            "application/pdf" => bytes.starts_with(b"%PDF-"),
            "image/png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
            "image/jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
            "text/plain" => std::str::from_utf8(bytes).is_ok_and(|s| {
                !s.chars()
                    .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
            }),
            _ => false,
        }
}
fn current_device(root: &Path, request: &MobileOperationRequest) -> Result<Device, &'static str> {
    Store::open(&root.join("mobile-devices.db"))
        .and_then(|s| s.authenticate(request.credential.expose_for_attestation(), unix_now_secs()))
        .map_err(|e| e.code())
}
fn metadata(
    store: &ArtifactStore,
    binding: &Binding,
    artifact: &str,
    version: &str,
) -> Result<Value, &'static str> {
    if artifact.len() > 256 || version.len() > 256 {
        return Err("invalid_request");
    }
    let detail = store
        .version_detail(&binding.project, version)
        .map_err(|_| "storage_unavailable")?
        .filter(|d| {
            d.item.artifact_ref == artifact && d.item.conversation_reference == binding.conversation
        })
        .ok_or("not_found")?;
    if !matches!(detail.item.kind, ArtifactKind::Image | ArtifactKind::File) {
        return Err("file_type_unavailable");
    }
    let payload = &detail.publication.payload;
    let digest = payload
        .get("blob_digest")
        .and_then(Value::as_str)
        .ok_or("file_type_unavailable")?;
    let blob = store
        .version_blob(&binding.project, version, digest)
        .map_err(|_| "storage_unavailable")?
        .ok_or("not_found")?;
    if !allowed_media(&blob.media_type) {
        return Err("file_type_unavailable");
    }
    if blob.byte_length > MAX_FILE_BYTES as u64 {
        return Err("payload_too_large");
    }
    Ok(
        json!({"artifact_ref":artifact,"version_ref":version,"title":detail.item.title,
        "state":detail.item.state,"media_type":blob.media_type,"byte_length":blob.byte_length,"digest":digest}),
    )
}

pub(super) fn handle(
    request: &MobileOperationRequest,
    device: &Device,
    root: &Path,
    state: &Arc<Mutex<DaemonState>>,
) -> DaemonToWrapper {
    let agent = match &request.operation {
        MobileOperation::ArtifactUpload { upload } => &upload.agent_id,
        MobileOperation::ArtifactList { agent_id, .. }
        | MobileOperation::ArtifactRead { agent_id, .. } => agent_id,
        MobileOperation::ArtifactLookup { command_id } => {
            if !uuid(command_id) {
                return fail("invalid_request");
            }
            if !device.scope.operations.contains(&Operation::Message) {
                return fail("forbidden_scope");
            }
            let db = state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .db_path
                .clone();
            let value = ArtifactStore::open(&db)
                .and_then(|s| s.publication_by_key(&upload_key(&device.device_id, command_id)));
            if let Err(code) = current_device(root, request) {
                return fail(code);
            }
            return match value {
                Ok(Some((_, _, receipt))) => success(
                    json!({"status":"ok","command_id":command_id,"receipt":receipt,"replayed":true}),
                ),
                Ok(None) => fail("not_found"),
                Err(_) => fail("storage_unavailable"),
            };
        }
        _ => return fail("invalid_request"),
    };
    let upload = matches!(request.operation, MobileOperation::ArtifactUpload { .. });
    let op = if upload {
        Operation::Message
    } else {
        Operation::Read
    };
    if !upload && !device.scope.permits_federation(Operation::Read) {
        return fail("history_scope_unavailable");
    }
    let (snapshot, db, blobs) = {
        let st = state.lock().unwrap_or_else(|e| e.into_inner());
        let binding = if upload {
            match binding(&st, device, agent, op) {
                Ok(b) => b,
                Err(c) => return fail(c),
            }
        } else {
            if !uuid(agent) {
                return fail("invalid_request");
            }
            Binding {
                agent: agent.clone(),
                instance: String::new(),
                project: String::new(),
                conversation: String::new(),
            }
        };
        (binding, st.db_path.clone(), st.artifact_root.clone())
    };
    let store = match ArtifactStore::open(&db) {
        Ok(s) => s,
        Err(_) => return fail("storage_unavailable"),
    };
    let outcome = match &request.operation {
        MobileOperation::ArtifactList { after, limit, .. } => {
            if !(1..=50).contains(limit) || after.as_ref().is_some_and(|s| s.len() > 2048) {
                return fail("invalid_request");
            }
            let cursor = match after
                .as_ref()
                .map(|s| serde_json::from_str::<(i64, String)>(s))
                .transpose()
            {
                Ok(cursor) => cursor,
                Err(_) => return fail("invalid_request"),
            };
            let refs = match store.list_agent_references(
                &snapshot.agent,
                usize::from(*limit) + 1,
                cursor.as_ref().map(|(t, id)| (*t, id.as_str())),
            ) {
                Ok(r) => r,
                Err(_) => return fail("storage_unavailable"),
            };
            let more = refs.len() > usize::from(*limit);
            let next = if more {
                let last = &refs[usize::from(*limit) - 1];
                serde_json::to_string(&(last.version_created_at, &last.reference_id)).ok()
            } else {
                None
            };
            let items=refs.into_iter().take(usize::from(*limit)).map(|r| {
                let binding=Binding {project:r.project_id.clone(),conversation:r.conversation_reference.clone(),..snapshot.clone()};
                metadata(&store,&binding,&r.artifact_ref,&r.version_ref).unwrap_or_else(|code|
                    json!({"artifact_ref":r.artifact_ref,"version_ref":r.version_ref,"title":r.title,"code":code}))
            }).collect::<Vec<_>>();
            json!({"status":"ok","items":items,"next_cursor":next,"maximum_file_bytes":MAX_FILE_BYTES})
        }
        MobileOperation::ArtifactRead {
            artifact_ref,
            version_ref,
            offset,
            limit,
            ..
        } => {
            let project = match store.project_for_agent_version(agent, version_ref) {
                Ok(Some(p)) => p,
                Ok(None) => return fail("not_found"),
                Err(_) => return fail("storage_unavailable"),
            };
            let read_binding = Binding {
                conversation: format!("conversation:{project}:{agent}"),
                project,
                ..snapshot.clone()
            };
            let info = match metadata(&store, &read_binding, artifact_ref, version_ref) {
                Ok(m) => m,
                Err(c) => return fail(c),
            };
            let request = ArtifactReadRequest {
                version: 1,
                artifact_ref: artifact_ref.clone(),
                version_ref: version_ref.clone(),
                kind: bridget_transport::protocol::ArtifactReadKind::Blob {
                    digest: info["digest"].as_str().unwrap().into(),
                },
                offset: *offset,
                limit: *limit,
            };
            match ArtifactService::open(&db, &blobs, ArtifactPolicy::default())
                .map_err(|_| ())
                .and_then(|s| s.read(&read_binding.project, &request).map_err(|_| ()))
            {
                Ok(chunk) => json!({"status":"ok","metadata":info,"chunk":chunk}),
                Err(_) => return fail("artifact_unavailable"),
            }
        }
        MobileOperation::ArtifactUpload { upload } => {
            if upload.device_id != device.device_id {
                return fail("command_origin_changed");
            }
            if !uuid(&upload.command_id)
                || !valid_file_name(&upload.file_name)
                || !valid_bytes(&upload.media_type, &upload.bytes)
                || sha256_hex(&upload.bytes) != upload.expected_digest
            {
                return fail("invalid_file");
            }
            let key = upload_key(&device.device_id, &upload.command_id);
            let publication = publication(upload, &key);
            let prior = match store.publication_by_key(&key) {
                Ok(v) => v,
                Err(_) => return fail("storage_unavailable"),
            };
            if let Some((project, digest, receipt)) = prior {
                if project != snapshot.project || digest != publication.content_digest() {
                    return fail("envelope_mismatch");
                }
                json!({"status":"ok","command_id":upload.command_id,"receipt":receipt,"replayed":true})
            } else {
                let current = match store.total_published_blob_bytes() {
                    Ok(v) => v,
                    Err(_) => return fail("storage_unavailable"),
                };
                let additional = if store
                    .blob_reference_count(&upload.expected_digest)
                    .ok()
                    .flatten()
                    .unwrap_or(0)
                    > 0
                {
                    0
                } else {
                    upload.bytes.len() as u64
                };
                if ArtifactPolicy::default().publication_capacity(current, additional)
                    == crate::artifact_policy::PublicationCapacity::Blocked
                {
                    return fail("publication_capacity_reached");
                }
                let mut service =
                    match ArtifactService::open(&db, &blobs, ArtifactPolicy::default()) {
                        Ok(s) => s,
                        Err(_) => return fail("storage_unavailable"),
                    };
                if service.ingest_blob(&upload.bytes).is_err() {
                    return fail("storage_unavailable");
                }
                // Les gros octets sont déjà privés ; dernière garde avant l'unique publication native.
                let st = state.lock().unwrap_or_else(|e| e.into_inner());
                let current = match current_device(root, request) {
                    Ok(d) => d,
                    Err(c) => return fail(c),
                };
                if binding(&st, &current, agent, op).ok().as_ref() != Some(&snapshot) {
                    return fail("not_found");
                }
                // La clé est globale à cet appareil, même si deux projets arrivent en parallèle.
                match store.publication_by_key(&key) {
                    Ok(Some((project, digest, receipt))) => {
                        return if project == snapshot.project
                            && digest == publication.content_digest()
                        {
                            success(
                                json!({"status":"ok","command_id":upload.command_id,"receipt":receipt,"replayed":true}),
                            )
                        } else {
                            fail("envelope_mismatch")
                        };
                    }
                    Ok(None) => {}
                    Err(_) => return fail("storage_unavailable"),
                }
                let context = ArtifactPublicationContext {
                    project_id: snapshot.project.clone(),
                    conversation_reference: snapshot.conversation.clone(),
                    turn_reference: format!("turn:{key}"),
                    created_by: format!("mobile:{}", device.device_id),
                    origin_instance: format!("mobile:{}", device.device_id),
                    observed_at: unix_timestamp(),
                };
                match service.publish(context, publication) {
                    Ok(ArtifactPersistResult::Created(receipt)) => {
                        json!({"status":"ok","command_id":upload.command_id,"receipt":receipt,"replayed":false})
                    }
                    Ok(ArtifactPersistResult::Replayed(receipt)) => {
                        json!({"status":"ok","command_id":upload.command_id,"receipt":receipt,"replayed":true})
                    }
                    Err(ArtifactServiceError::Store(ArtifactStoreError::IdempotencyConflict)) => {
                        return fail("envelope_mismatch");
                    }
                    Err(ArtifactServiceError::PolicyBlocked) => {
                        return fail("publication_capacity_reached");
                    }
                    Err(_) => return fail("storage_unavailable"),
                }
            }
        }
        _ => return fail("invalid_request"),
    };
    let current = match current_device(root, request) {
        Ok(d) => d,
        Err(c) => return fail(c),
    };
    let st = state.lock().unwrap_or_else(|e| e.into_inner());
    if upload {
        if binding(&st, &current, agent, op).ok().as_ref() != Some(&snapshot) {
            return fail("not_found");
        }
    } else if !current.scope.permits_federation(Operation::Read) {
        return fail("forbidden_scope");
    }
    success(outcome)
}
fn publication(upload: &MobileFileUpload, key: &str) -> ArtifactPublicationV1 {
    ArtifactPublicationV1 {
        idempotency_key: key.into(),
        kind: if upload.media_type.starts_with("image/") {
            ArtifactKind::Image
        } else {
            ArtifactKind::File
        },
        title: upload.file_name.clone(),
        payload: json!({"blob_digest":upload.expected_digest,"media_type":upload.media_type,"agent_id":upload.agent_id,
            "file_name":upload.file_name,"byte_length":upload.bytes.len()}),
        sources: vec![ArtifactSourceV1 {
            source_kind: ArtifactSourceKind::UserSupplied,
            locator: format!("mobile:{}", upload.device_id),
            fetched_at: None,
            content_digest: Some(upload.expected_digest.clone()),
            citation: "Fichier fourni par l’utilisateur".into(),
            units: None,
            transformations: vec![],
            access_status: ArtifactSourceAccess::Available,
        }],
        quality_notices: vec![],
        parent_artifact_ref: None,
        publication_reason: PublicationReason::Initial,
    }
}
