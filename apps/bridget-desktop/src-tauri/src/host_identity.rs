//! Vérification explicite de la clé hôte SSH, isolée des clés utilisateur.

use crate::profile::{ConnectionProfile, ProfileValidationError};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostIdentityStatus {
    Approved,
    AwaitingApproval(HostIdentityTicket),
    Changed { expected: String, observed: String },
}

/// Ticket uniquement mémoire. Une webview ne peut ni le fabriquer ni faire
/// approuver une autre clé que celle contrôlée juste avant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HostIdentityTicket {
    pub profile_id: String,
    pub fingerprint: String,
    public_key_line: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommandResult {
    pub success: bool,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

pub trait HostKeyCommandRunner {
    fn run(
        &mut self,
        program: &str,
        args: &[OsString],
        stdin: &[u8],
    ) -> Result<CommandResult, HostIdentityError>;
}

pub struct SystemHostKeyCommandRunner;

impl HostKeyCommandRunner for SystemHostKeyCommandRunner {
    fn run(
        &mut self,
        program: &str,
        args: &[OsString],
        stdin: &[u8],
    ) -> Result<CommandResult, HostIdentityError> {
        let mut child = std::process::Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(HostIdentityError::Io)?;
        if !stdin.is_empty() {
            child
                .stdin
                .as_mut()
                .expect("stdin capturé")
                .write_all(stdin)
                .map_err(HostIdentityError::Io)?;
        }
        let output = child.wait_with_output().map_err(HostIdentityError::Io)?;
        Ok(CommandResult {
            success: output.status.success(),
            stdout: output.stdout,
            stderr: output.stderr,
        })
    }
}

#[derive(Debug)]
pub enum HostIdentityError {
    InvalidProfile(ProfileValidationError),
    Io(std::io::Error),
    CommandFailed(&'static str),
    InvalidScan,
    InvalidFingerprint,
    TicketMismatch,
    NotSshProfile,
}

impl std::fmt::Display for HostIdentityError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidProfile(error) => error.fmt(formatter),
            Self::Io(error) => write!(formatter, "Vérification SSH indisponible : {error}"),
            Self::CommandFailed(command) => {
                write!(formatter, "La vérification SSH {command} a échoué.")
            }
            Self::InvalidScan => formatter.write_str("La clé hôte retournée est invalide."),
            Self::InvalidFingerprint => {
                formatter.write_str("L'empreinte SSH retournée est invalide.")
            }
            Self::TicketMismatch => {
                formatter.write_str("Le ticket d'approbation ne correspond plus au profil.")
            }
            Self::NotSshProfile => formatter.write_str("Un relais local n'a pas de clé hôte SSH."),
        }
    }
}

impl std::error::Error for HostIdentityError {}

impl From<ProfileValidationError> for HostIdentityError {
    fn from(error: ProfileValidationError) -> Self {
        Self::InvalidProfile(error)
    }
}

pub fn check_host_identity(
    profile: &ConnectionProfile,
    runner: &mut impl HostKeyCommandRunner,
) -> Result<HostIdentityStatus, HostIdentityError> {
    profile.validate()?;
    let (id, host, port, expected) = match profile {
        ConnectionProfile::Ssh {
            id,
            host,
            port,
            host_fingerprint,
            ..
        } => (id, host, *port, host_fingerprint.as_deref()),
        ConnectionProfile::Local { .. } => return Err(HostIdentityError::NotSshProfile),
    };

    let scan = runner.run(
        "ssh-keyscan",
        &[
            OsString::from("-p"),
            OsString::from(port.to_string()),
            OsString::from("-t"),
            OsString::from("ed25519"),
            OsString::from(host),
        ],
        &[],
    )?;
    if !scan.success {
        return Err(HostIdentityError::CommandFailed("ssh-keyscan"));
    }
    let public_key_line = normalize_public_key_line(&scan.stdout)?;
    let fingerprint_result = runner.run(
        "ssh-keygen",
        &[
            OsString::from("-lf"),
            OsString::from("-"),
            OsString::from("-E"),
            OsString::from("sha256"),
        ],
        public_key_line.as_bytes(),
    )?;
    if !fingerprint_result.success {
        return Err(HostIdentityError::CommandFailed("ssh-keygen"));
    }
    let fingerprint = parse_fingerprint(&fingerprint_result.stdout)?;

    match expected {
        Some(expected) if expected == fingerprint => Ok(HostIdentityStatus::Approved),
        Some(expected) => Ok(HostIdentityStatus::Changed {
            expected: expected.to_owned(),
            observed: fingerprint,
        }),
        None => Ok(HostIdentityStatus::AwaitingApproval(HostIdentityTicket {
            profile_id: id.to_owned(),
            fingerprint,
            public_key_line,
        })),
    }
}

pub fn approve_host_identity(
    profile: &mut ConnectionProfile,
    ticket: HostIdentityTicket,
    application_known_hosts: &Path,
) -> Result<(), HostIdentityError> {
    if profile.id() != ticket.profile_id || profile.is_local() {
        return Err(HostIdentityError::TicketMismatch);
    }
    profile.set_host_fingerprint(ticket.fingerprint.clone())?;
    let parent = application_known_hosts
        .parent()
        .ok_or(HostIdentityError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "known_hosts sans dossier parent",
        )))?;
    fs::create_dir_all(parent).map_err(HostIdentityError::Io)?;
    set_private_directory_permissions(parent).map_err(HostIdentityError::Io)?;
    let existing = fs::read_to_string(application_known_hosts).unwrap_or_default();
    if !existing.lines().any(|line| line == ticket.public_key_line) {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(application_known_hosts)
            .map_err(HostIdentityError::Io)?;
        set_private_file_permissions(&file).map_err(HostIdentityError::Io)?;
        writeln!(file, "{}", ticket.public_key_line).map_err(HostIdentityError::Io)?;
        file.sync_all().map_err(HostIdentityError::Io)?;
    }
    Ok(())
}

fn normalize_public_key_line(bytes: &[u8]) -> Result<String, HostIdentityError> {
    let text = std::str::from_utf8(bytes).map_err(|_| HostIdentityError::InvalidScan)?;
    let line = text
        .lines()
        .find(|line| !line.trim_start().starts_with('#') && !line.trim().is_empty())
        .ok_or(HostIdentityError::InvalidScan)?
        .trim();
    let fields: Vec<_> = line.split_whitespace().collect();
    if fields.len() != 3 || fields[1] != "ssh-ed25519" || !fields[2].starts_with("AAAA") {
        return Err(HostIdentityError::InvalidScan);
    }
    Ok(line.to_owned())
}

fn parse_fingerprint(bytes: &[u8]) -> Result<String, HostIdentityError> {
    let text = std::str::from_utf8(bytes).map_err(|_| HostIdentityError::InvalidFingerprint)?;
    let fingerprint = text
        .split_whitespace()
        .find(|field| field.starts_with("SHA256:"))
        .ok_or(HostIdentityError::InvalidFingerprint)?;
    if fingerprint.len() < 16 || fingerprint.contains(char::is_whitespace) {
        return Err(HostIdentityError::InvalidFingerprint);
    }
    Ok(fingerprint.to_owned())
}

fn set_private_file_permissions(file: &File) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.set_permissions(fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

fn set_private_directory_permissions(path: &Path) -> Result<(), std::io::Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        CommandResult, HostIdentityStatus, HostKeyCommandRunner, approve_host_identity,
        check_host_identity,
    };
    use crate::profile::{ConnectionProfile, ProfileCapability, SshIdentityRef};
    use std::collections::VecDeque;
    use std::ffi::OsString;
    use std::fs;
    use uuid::Uuid;

    const KEY_LINE: &str =
        "[cartae.app]:2222 ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFixturePublicKeyOnly";
    const FINGERPRINT: &str = "SHA256:fixtureFingerprintOnly123456789";

    struct FakeRunner {
        answers: VecDeque<CommandResult>,
        calls: Vec<(String, Vec<OsString>)>,
    }

    impl FakeRunner {
        fn with_current_key() -> Self {
            Self {
                answers: VecDeque::from([
                    CommandResult {
                        success: true,
                        stdout: format!("# commentaire\n{KEY_LINE}\n").into_bytes(),
                        stderr: vec![],
                    },
                    CommandResult {
                        success: true,
                        stdout: format!("256 {FINGERPRINT} fixture (ED25519)\n").into_bytes(),
                        stderr: vec![],
                    },
                ]),
                calls: vec![],
            }
        }
    }

    impl HostKeyCommandRunner for FakeRunner {
        fn run(
            &mut self,
            program: &str,
            args: &[OsString],
            _stdin: &[u8],
        ) -> Result<CommandResult, super::HostIdentityError> {
            self.calls.push((program.to_owned(), args.to_vec()));
            Ok(self.answers.pop_front().expect("réponse factice"))
        }
    }

    fn profile(fingerprint: Option<&str>) -> ConnectionProfile {
        ConnectionProfile::Ssh {
            id: "production".into(),
            label: "Cartae.app".into(),
            host: "cartae.app".into(),
            port: 2222,
            user: "moi".into(),
            identity: SshIdentityRef::Agent,
            host_fingerprint: fingerprint.map(str::to_owned),
            capabilities: vec![ProfileCapability::Ui],
        }
    }

    #[test]
    fn hote_inconnu_cree_un_ticket_et_approbation_ecrit_le_known_hosts_applicatif() {
        let mut runner = FakeRunner::with_current_key();
        let result = check_host_identity(&profile(None), &mut runner).expect("scan");
        let HostIdentityStatus::AwaitingApproval(ticket) = result else {
            panic!("ticket attendu")
        };
        assert_eq!(runner.calls[0].0, "ssh-keyscan");
        assert_eq!(runner.calls[1].0, "ssh-keygen");
        let directory =
            std::env::temp_dir().join(format!("bridget-known-hosts-{}", Uuid::new_v4()));
        let mut profile = profile(None);
        approve_host_identity(&mut profile, ticket, &directory.join("known_hosts"))
            .expect("approbation");
        assert!(
            fs::read_to_string(directory.join("known_hosts"))
                .expect("lecture")
                .contains(KEY_LINE)
        );
        fs::remove_dir_all(directory).expect("nettoyage exact du test");
    }

    #[test]
    fn hote_approuve_reste_approuve_et_changement_bloque() {
        let mut runner = FakeRunner::with_current_key();
        assert_eq!(
            check_host_identity(&profile(Some(FINGERPRINT)), &mut runner).expect("approuvé"),
            HostIdentityStatus::Approved
        );
        let mut runner = FakeRunner::with_current_key();
        assert!(matches!(
            check_host_identity(
                &profile(Some("SHA256:ancienneEmpreinte123456")),
                &mut runner
            ),
            Ok(HostIdentityStatus::Changed { .. })
        ));
    }
}
