//! Construction et possession stricte des processus SSH Bridget Desktop.

use crate::profile::{ConnectionProfile, ProfileValidationError, SshIdentityRef};
use std::ffi::OsString;
use std::io;
use std::net::TcpListener;
use std::path::Path;
use std::process::{Child, Command, Stdio};

pub const REMOTE_ENDPOINT_COMMAND: &str = "bridget ui endpoint --json";

#[derive(Debug)]
pub struct SshInvocation {
    args: Vec<OsString>,
}

impl SshInvocation {
    pub fn args(&self) -> &[OsString] {
        &self.args
    }

    pub fn spawn(&self) -> Result<Child, io::Error> {
        Command::new("ssh")
            .args(&self.args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
    }

    /// Les diagnostics exposent une intention, jamais une ligne de commande.
    pub fn diagnostic(&self) -> &'static str {
        "SSH Bridget Desktop en cours"
    }
}

#[derive(Debug)]
pub struct OwnedTunnel {
    child: Child,
    pub local_port: u16,
}

impl OwnedTunnel {
    pub fn open(
        profile: &ConnectionProfile,
        known_hosts: &Path,
        remote_port: u16,
    ) -> Result<Self, SshError> {
        let local_port = reserve_loopback_port()?;
        let invocation = forward_invocation(profile, known_hosts, local_port, remote_port)?;
        let child = invocation.spawn()?;
        Ok(Self { child, local_port })
    }

    pub fn is_running(&mut self) -> Result<bool, SshError> {
        Ok(self.child.try_wait()?.is_none())
    }

    /// Ne touche qu'au processus enfant créé par cette session.
    pub fn close(mut self) -> Result<(), SshError> {
        if self.child.try_wait()?.is_none() {
            self.child.kill()?;
            let _ = self.child.wait()?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum SshError {
    InvalidProfile(ProfileValidationError),
    Io(io::Error),
    NotSshProfile,
}

impl std::fmt::Display for SshError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProfile(error) => error.fmt(formatter),
            Self::Io(error) => write!(formatter, "SSH indisponible : {error}"),
            Self::NotSshProfile => formatter.write_str("Ce profil n'utilise pas SSH."),
        }
    }
}

impl std::error::Error for SshError {}

impl From<ProfileValidationError> for SshError {
    fn from(error: ProfileValidationError) -> Self {
        Self::InvalidProfile(error)
    }
}

impl From<io::Error> for SshError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn discovery_invocation(
    profile: &ConnectionProfile,
    known_hosts: &Path,
) -> Result<SshInvocation, SshError> {
    let mut args = base_arguments(profile, known_hosts)?;
    args.push(OsString::from(ssh_target(profile)?));
    // Chaîne constante du protocole, sans aucune interpolation utilisateur.
    args.push(OsString::from(REMOTE_ENDPOINT_COMMAND));
    Ok(SshInvocation { args })
}

pub fn forward_invocation(
    profile: &ConnectionProfile,
    known_hosts: &Path,
    local_port: u16,
    remote_port: u16,
) -> Result<SshInvocation, SshError> {
    profile.validate()?;
    validate_port(local_port)?;
    validate_port(remote_port)?;
    let mut args = base_arguments(profile, known_hosts)?;
    args.extend([
        OsString::from("-N"),
        OsString::from("-L"),
        OsString::from(format!("127.0.0.1:{local_port}:127.0.0.1:{remote_port}")),
        OsString::from(ssh_target(profile)?),
    ]);
    Ok(SshInvocation { args })
}

fn base_arguments(
    profile: &ConnectionProfile,
    known_hosts: &Path,
) -> Result<Vec<OsString>, SshError> {
    profile.validate()?;
    let (port, identity) = match profile {
        ConnectionProfile::Ssh { port, identity, .. } => (*port, identity),
        ConnectionProfile::Local { .. } => return Err(SshError::NotSshProfile),
    };
    let known_hosts = known_hosts.to_str().ok_or_else(|| {
        SshError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Le chemin known_hosts doit être UTF-8.",
        ))
    })?;
    let mut args = vec![
        OsString::from("-p"),
        OsString::from(port.to_string()),
        OsString::from("-o"),
        OsString::from("BatchMode=yes"),
        OsString::from("-o"),
        OsString::from("ControlMaster=no"),
        OsString::from("-o"),
        OsString::from("ControlPath=none"),
        OsString::from("-o"),
        OsString::from("ExitOnForwardFailure=yes"),
        OsString::from("-o"),
        OsString::from("ServerAliveInterval=20"),
        OsString::from("-o"),
        OsString::from("ServerAliveCountMax=3"),
        OsString::from("-o"),
        OsString::from("StrictHostKeyChecking=yes"),
        OsString::from("-o"),
        OsString::from(format!("UserKnownHostsFile={known_hosts}")),
        OsString::from("-o"),
        OsString::from("LogLevel=ERROR"),
    ];
    if let SshIdentityRef::File { path } = identity {
        args.extend([OsString::from("-i"), path.as_os_str().to_os_string()]);
    }
    Ok(args)
}

fn ssh_target(profile: &ConnectionProfile) -> Result<String, SshError> {
    match profile {
        ConnectionProfile::Ssh { user, host, .. } => Ok(format!("{user}@{host}")),
        ConnectionProfile::Local { .. } => Err(SshError::NotSshProfile),
    }
}

fn validate_port(port: u16) -> Result<(), SshError> {
    if port == 0 {
        return Err(SshError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Le port de tunnel doit être non nul.",
        )));
    }
    Ok(())
}

/// Réserve un port dans l'espace loopback, puis le libère juste avant le spawn
/// SSH. `ExitOnForwardFailure` rend toute course locale observable plutôt que
/// de déclarer une session connectée à tort.
fn reserve_loopback_port() -> Result<u16, SshError> {
    let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))?;
    Ok(listener.local_addr()?.port())
}

#[cfg(test)]
mod tests {
    use super::{OwnedTunnel, REMOTE_ENDPOINT_COMMAND, discovery_invocation, forward_invocation};
    use crate::profile::{ConnectionProfile, ProfileCapability, SshIdentityRef};
    use std::ffi::OsString;
    use std::path::Path;
    use std::process::Command;

    fn profile() -> ConnectionProfile {
        ConnectionProfile::Ssh {
            id: "production".into(),
            label: "Cartae.app".into(),
            host: "cartae.app".into(),
            port: 2222,
            user: "moi".into(),
            identity: SshIdentityRef::Agent,
            host_fingerprint: Some("SHA256:abcdefghijklmnopqrstuvwxyz123456".into()),
            capabilities: vec![ProfileCapability::Ui],
        }
    }

    fn as_strings(args: &[OsString]) -> Vec<String> {
        args.iter()
            .map(|value| value.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn ssh_est_construit_sans_shell_avec_les_gardes_requises() {
        let invocation =
            forward_invocation(&profile(), Path::new("/tmp/known_hosts"), 39001, 17888)
                .expect("invocation");
        let args = as_strings(invocation.args());
        for required in [
            "BatchMode=yes",
            "ControlMaster=no",
            "ControlPath=none",
            "ExitOnForwardFailure=yes",
            "ServerAliveInterval=20",
            "ServerAliveCountMax=3",
            "StrictHostKeyChecking=yes",
            "-N",
            "127.0.0.1:39001:127.0.0.1:17888",
            "moi@cartae.app",
        ] {
            assert!(
                args.iter().any(|argument| argument == required),
                "{required}"
            );
        }
        assert!(
            !args
                .iter()
                .any(|argument| argument.contains("ProxyCommand"))
        );
        assert_eq!(invocation.diagnostic(), "SSH Bridget Desktop en cours");
    }

    #[test]
    fn decouverte_utilise_une_commande_distante_constante() {
        let invocation =
            discovery_invocation(&profile(), Path::new("/tmp/known_hosts")).expect("invocation");
        let args = as_strings(invocation.args());
        assert_eq!(args.last(), Some(&REMOTE_ENDPOINT_COMMAND.to_owned()));
        assert!(!args.iter().any(|argument| argument == "/bin/sh"));
    }

    #[test]
    fn un_hote_qui_pourrait_devenir_une_option_est_refuse() {
        let mut invalid = profile();
        if let ConnectionProfile::Ssh { host, .. } = &mut invalid {
            *host = "-oProxyCommand=evil".into();
        }
        assert!(forward_invocation(&invalid, Path::new("/tmp/known_hosts"), 39001, 17888).is_err());
    }

    #[test]
    fn fermer_un_tunnel_possede_recolte_son_enfant_sans_affecter_un_autre() {
        let first = Command::new("sh")
            .args(["-c", "sleep 2"])
            .spawn()
            .expect("enfant possédé");
        let tunnel = OwnedTunnel {
            child: first,
            local_port: 39174,
        };
        let mut unrelated = Command::new("sh")
            .args(["-c", "sleep 1"])
            .spawn()
            .expect("enfant non possédé");
        tunnel.close().expect("fermeture du seul enfant possédé");
        assert!(
            unrelated
                .try_wait()
                .expect("état enfant non possédé")
                .is_none(),
            "la fermeture d'un tunnel ne doit pas toucher un autre enfant"
        );
        unrelated.wait().expect("fin naturelle enfant non possédé");
    }
}
