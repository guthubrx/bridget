use bridget_desktop::artifact_sandbox::{
    MAX_INLINE_HEIGHT, SandboxArtifactVersionV1, SandboxRuntimePolicyV1, SandboxRuntimeStateV1,
    SandboxRuntimeStatusV1, validate_runtime_state,
};
use uuid::Uuid;

fn runtime_state(state: serde_json::Value, height: u16) -> SandboxRuntimeStateV1 {
    SandboxRuntimeStateV1 {
        frame_instance_id: Uuid::new_v4(),
        artifact_id: Uuid::new_v4(),
        version_id: Uuid::new_v4(),
        ui_state: state,
        requested_height: height,
        status: SandboxRuntimeStatusV1::Ready,
    }
}

#[test]
fn le_contrat_refuse_les_etats_qui_portent_une_navigation_ou_depassement() {
    assert!(validate_runtime_state(&runtime_state(serde_json::json!({"zoom": 2}), 480)).is_ok());
    assert!(validate_runtime_state(&runtime_state(serde_json::json!({"url": "file:///tmp/no"}), 480)).is_err());
    assert!(validate_runtime_state(&runtime_state(serde_json::json!({"zoom": 2}), MAX_INLINE_HEIGHT + 1)).is_err());
}

#[test]
fn la_version_sandbox_ne_porte_ni_cookie_ni_capability() {
    let version = SandboxArtifactVersionV1 {
        artifact_id: Uuid::new_v4(),
        version_id: Uuid::new_v4(),
        html_blob_sha256: "a".repeat(64),
        data_blob_sha256: None,
        runtime_policy: SandboxRuntimePolicyV1::SandboxV1,
        inline_height_hint: 420,
        source_refs: vec!["source-1".into()],
        parent_version_id: None,
    };
    let serialized = serde_json::to_string(&version).unwrap();
    assert!(!serialized.contains("cookie"));
    assert!(!serialized.contains("tauri"));
}
