use bridget_daemon::project_runtime::validate_project_profile_backend;
use bridget_transport::protocol::{ProjectBackend, ProjectResourceKind, ProjectResourceRef};

fn resource() -> ProjectResourceRef {
    ProjectResourceRef {
        resource_id: "fixture".to_string(),
        kind: ProjectResourceKind::SecretFile,
        source_ref: "catalog:fixture".to_string(),
        destination: "/run/bridget/secrets/fixture".to_string(),
        version: Some("v1".to_string()),
        content_digest: None,
        generation: 1,
    }
}

#[test]
fn spec_067_backend_host_refuse_un_profil_a_ressource_et_garde_le_profil_vide() {
    assert!(validate_project_profile_backend(ProjectBackend::Host, &[resource()]).is_err());
    assert!(validate_project_profile_backend(ProjectBackend::Host, &[]).is_ok());
    assert!(validate_project_profile_backend(ProjectBackend::Docker, &[resource()]).is_ok());
}
