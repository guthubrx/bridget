//! Client local du mandat Loop264. Le secret ne passe ni dans argv ni dans les logs.
use super::*;
use crate::cli_table::table_println as println;
use bridget_transport::protocol::{LoopControlOutcome, LoopControlRequest};
use std::io::Read;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthorizationFile {
    version: u16,
    token: String,
    issuer_scope: String,
    project_root: String,
    targets: Vec<String>,
    intent: String,
}

fn read_authorization(path: &std::path::Path) -> Result<AuthorizationFile, String> {
    let file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
        .map_err(|_| "loop_authorization_file_unavailable")?;
    let metadata = file
        .metadata()
        .map_err(|_| "loop_authorization_file_unavailable")?;
    // SAFETY: geteuid ne reçoit aucun pointeur et ne modifie aucune mémoire.
    let uid = unsafe { libc::geteuid() };
    if !metadata.is_file()
        || metadata.uid() != uid
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.len() > 16384
    {
        return Err(
            "loop_authorization_file_not_private : fichier ordinaire privé requis (0600)".into(),
        );
    }
    let mut bytes = Vec::new();
    file.take(16385)
        .read_to_end(&mut bytes)
        .map_err(|_| "loop_authorization_file_unavailable")?;
    let data: AuthorizationFile =
        serde_json::from_slice(&bytes).map_err(|_| "invalid_loop_authorization_file")?;
    if data.version != 1
        || data.token.len() != 64
        || !data.token.bytes().all(|b| b.is_ascii_hexdigit())
        || !matches!(data.intent.as_str(), "steer" | "interrupt")
    {
        return Err("invalid_loop_authorization_file".into());
    }
    Ok(data)
}

pub(super) fn bind(
    connection: &mut DaemonConnection,
    path: &std::path::Path,
) -> Result<(), String> {
    let data = read_authorization(path)?;
    match connection
        .exchange(&WrapperToDaemon::LoopControl {
            request: LoopControlRequest::Use { token: data.token },
        })
        .map_err(|_| "loop_control_bind_unconfirmed")?
    {
        DaemonToWrapper::LoopControlResult {
            result: LoopControlOutcome::Bound { .. },
        } => Ok(()),
        DaemonToWrapper::LoopControlResult {
            result: LoopControlOutcome::Refused { code, recovery },
        } => Err(format!("{code}: {recovery}")),
        DaemonToWrapper::ClientRejected { reason } => Err(format!("loop_control_client_rejected: {reason:?}. Aucun envoi ; vérifier la négociation de cette connexion.")),
        _ => Err("loop_control_response_unexpected : aucun envoi confirmé ; conserver le mandat et vérifier la réponse native".into()),
    }
}

pub(super) fn command(args: &[String]) {
    let result = (|| -> Result<LoopControlOutcome, String> {
        let action = args.first().map(String::as_str).unwrap_or("");
        let valid = matches!(action, "authorize" | "revoke")
            && args.get(1).map(String::as_str) == Some("--file")
            && (action == "revoke" && args.len() == 3
                || action == "authorize" && args.len() == 4 && args[3] == "--approve");
        if !valid {
            return Err("usage: bridget loop-control authorize --file FICHIER_PRIVE --approve | revoke --file FICHIER_PRIVE ; authorize exige le choix explicite de l'utilisateur pour cette boucle".into());
        }
        let data = read_authorization(std::path::Path::new(&args[2]))?;
        let request = if action == "revoke" {
            LoopControlRequest::Revoke { token: data.token }
        } else {
            let project = crate::communication::resolve_communication_project(
                &data.project_root,
                &crate::build_info::local_host(),
                bridget_transport::protocol::CommunicationProjectSource::Git,
                None,
            )
            .ok_or("loop_project_unavailable")?;
            LoopControlRequest::Authorize {
                token: data.token,
                issuer_scope: data.issuer_scope,
                project,
                targets: data.targets,
                intent: if data.intent == "steer" {
                    bridget_core::MessageIntent::SteerCurrent
                } else {
                    bridget_core::MessageIntent::InterruptAndStart
                },
            }
        };
        let mut connection = DaemonConnection::connect(&socket_path())
            .map_err(|_| "loop_control_daemon_unavailable")?;
        let role = if action == "authorize" {
            ConnectionRole::Attach
        } else {
            ConnectionRole::Client
        };
        match connection
            .exchange(&WrapperToDaemon::RoleHandshake { role })
            .map_err(|_| "loop_control_handshake_unconfirmed")?
        {
            DaemonToWrapper::RoleAccepted { .. } => {}
            _ => return Err("loop_control_handshake_refused".into()),
        }
        match connection.exchange(&WrapperToDaemon::LoopControl { request }).map_err(|_| "loop_control_result_unconfirmed : conserver le même fichier et rejouer la même opération")? {
            DaemonToWrapper::LoopControlResult { result } => Ok(result),
            DaemonToWrapper::AttachRejected { reason, .. } => Err(format!("loop_authorization_refused: {reason:?}. Aucun mandat confirmé ; vérifier le droit d'autoriser cette boucle sur la connexion humaine native.")),
            DaemonToWrapper::ClientRejected { reason } => Err(format!("loop_control_client_rejected: {reason:?}. Opération non confirmée ; vérifier cette connexion.")),
            _ => Err("loop_control_response_unexpected : opération non confirmée ; conserver le même fichier et vérifier la réponse native".into()),
        }
    })();
    match result {
        Ok(outcome) => {
            println!(
                "{}",
                serde_json::to_string(&outcome).expect("résultat sérialisable")
            );
            if matches!(outcome, LoopControlOutcome::Refused { .. }) {
                crate::cli_table::exit(1);
            }
        }
        Err(error) => {
            eprintln!("bridget loop-control: {error}");
            crate::cli_table::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec264_loop_authorization_file_must_be_private_and_not_a_symlink() {
        let root = std::env::temp_dir().join(format!("loop264-file-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&root).unwrap();
        let file = root.join("authorization.json");
        let payload = serde_json::json!({"version":1,"token":"a".repeat(64),
            "issuer_scope":"authorization264-scope","project_root":"/fixture",
            "targets":[],"intent":"steer"});
        std::fs::write(&file, payload.to_string()).unwrap();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(read_authorization(&file).is_ok());
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read_authorization(&file).is_err());
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
        let link = root.join("alias.json");
        std::os::unix::fs::symlink(&file, &link).unwrap();
        assert!(read_authorization(&link).is_err());
        std::fs::remove_file(link).unwrap();
        std::fs::remove_file(file).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}
