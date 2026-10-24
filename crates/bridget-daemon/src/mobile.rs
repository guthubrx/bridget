//! Accès humain mobile : identité d'appareil distincte des identités d'agent.
pub(crate) mod client;
pub(crate) mod http;
pub(crate) mod store;
pub(crate) mod notifications;

use bridget_transport::protocol::{
    CLIENT_CONTRACT_VERSION, ClientCapability, ConnectionRole, IdentityCredential,
    MobileAdministrationRequest as AdminRequest, MobileAdministrationResult as AdminResult,
};
use bridget_transport::{DaemonToWrapper, WrapperToDaemon};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::io::{IsTerminal, Read, Write};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    public_url: String,
    listen_addr: SocketAddr,
}

impl Config {
    fn validate(&self) -> Result<(), &'static str> {
        let host = self
            .public_url
            .strip_prefix("https://")
            .ok_or("invalid_public_url")?;
        if host.parse::<std::net::IpAddr>().is_ok()
            || host.len() > 253
            || !host.contains('.')
            || host.split('.').any(|part| {
                part.is_empty()
                    || part.len() > 63
                    || part.starts_with('-')
                    || part.ends_with('-')
                    || !part
                        .bytes()
                        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            })
            || !self.listen_addr.ip().is_loopback()
            || self.listen_addr.port() == 0
        {
            return Err("invalid_mobile_configuration");
        }
        Ok(())
    }

    fn load(root: &Path) -> Result<Option<Self>, &'static str> {
        let path = root.join("mobile.json");
        crate::environment::validate_state_file(&path, false)
            .map_err(|_| "mobile_configuration_unavailable")?;
        let mut file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(_) => return Err("mobile_configuration_unavailable"),
        };
        let mut bytes = Vec::new();
        Read::by_ref(&mut file)
            .take(4097)
            .read_to_end(&mut bytes)
            .map_err(|_| "mobile_configuration_unavailable")?;
        if bytes.len() > 4096 {
            return Err("mobile_configuration_unavailable");
        }
        let config: Self =
            serde_json::from_slice(&bytes).map_err(|_| "mobile_configuration_unavailable")?;
        config.validate()?;
        Ok(Some(config))
    }

    fn save(&self, root: &Path) -> Result<(), &'static str> {
        use std::os::unix::fs::OpenOptionsExt;
        self.validate()?;
        crate::environment::validate_existing_tree(root)
            .map_err(|_| "mobile_configuration_unavailable")?;
        crate::environment::ensure_private_directory(root)
            .map_err(|_| "mobile_configuration_unavailable")?;
        let path = root.join("mobile.json");
        crate::environment::validate_state_file(&path, false)
            .map_err(|_| "mobile_configuration_unavailable")?;
        let temp = root.join(format!(".mobile-{}.tmp", uuid::Uuid::new_v4()));
        let result = (|| {
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW)
                .open(&temp)?;
            file.write_all(&serde_json::to_vec(self).map_err(std::io::Error::other)?)?;
            file.sync_all()?;
            std::fs::rename(&temp, &path)?;
            std::fs::File::open(root)?.sync_all()
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(temp);
        }
        result.map_err(|_| "mobile_configuration_unavailable")
    }
}

/// O(D), D<=256 ; appelée seulement après contrôle du pair humain par le daemon.
pub(crate) fn administer(root: &Path, request: AdminRequest, now: i64) -> AdminResult {
    let result = (|| -> Result<AdminResult, &'static str> {
        if let AdminRequest::Configure {
            public_url,
            listen_addr,
        } = &request
        {
            let config = Config {
                public_url: public_url.clone(),
                listen_addr: listen_addr
                    .parse()
                    .map_err(|_| "invalid_mobile_configuration")?,
            };
            config.save(root)?;
            return Ok(AdminResult::Configuration {
                configured: true,
                public_url: Some(config.public_url),
                listen_addr: Some(config.listen_addr.to_string()),
            });
        }
        if let AdminRequest::ConfigureNotifications { account } = &request {
            Config::load(root)?.ok_or("mobile_not_configured")?;
            notifications::configure(root,account.expose_for_attestation().as_bytes())?;
            return Ok(AdminResult::NotificationsConfigured {configured:true});
        }
        let config = Config::load(root)?;
        if matches!(&request, AdminRequest::Status) {
            return Ok(AdminResult::Configuration {
                configured: config.is_some(),
                public_url: config.as_ref().map(|c| c.public_url.clone()),
                listen_addr: config.as_ref().map(|c| c.listen_addr.to_string()),
            });
        }
        let config = config.ok_or("mobile_not_configured")?;
        let mut devices =
            store::Store::open(&root.join("mobile-devices.db")).map_err(|e| e.code())?;
        match request {
            AdminRequest::Pair { scope } => {
                let scope: store::Scope =
                    serde_json::from_value(scope).map_err(|_| "invalid_scope")?;
                let grant = devices.issue_pairing(&scope, now).map_err(|e| e.code())?;
                Ok(AdminResult::Pairing {
                    pairing_id: grant.pairing_id,
                    credential: IdentityCredential::new(grant.credential),
                    expires_at: grant.expires_at,
                    public_url: config.public_url,
                })
            }
            AdminRequest::Devices => Ok(AdminResult::Devices {
                items: devices
                    .devices()
                    .map_err(|e| e.code())?
                    .into_iter()
                    .map(|device| serde_json::to_value(device).expect("device JSON"))
                    .collect(),
            }),
            AdminRequest::Revoke { device_id } => Ok(AdminResult::Revoked {
                found: devices.revoke(&device_id, now).map_err(|e| e.code())?,
                device_id,
            }),
            _ => Err("invalid_request"),
        }
    })();
    result.unwrap_or_else(|code| AdminResult::Error {
        code: code.to_owned(),
    })
}

fn admin_connection(socket: &Path, request: AdminRequest) -> Result<AdminResult, &'static str> {
    use crate::communication::client::DaemonConnection;
    let mut connection = DaemonConnection::connect(socket).map_err(|_| "daemon_unavailable")?;
    if !matches!(
        connection.exchange(&WrapperToDaemon::RoleHandshake {
            role: ConnectionRole::Client
        }),
        Ok(DaemonToWrapper::RoleAccepted {
            role: ConnectionRole::Client
        })
    ) {
        return Err("human_connection_refused");
    }
    if !matches!(connection.exchange(&WrapperToDaemon::ClientHello {
        contract_version:CLIENT_CONTRACT_VERSION,
        issuer_scope:crate::communication::issuer_scope("bridget-mobile-admin"),
        capabilities:vec![ClientCapability::MobileAdministrationV1],
    }),Ok(DaemonToWrapper::ClientWelcome {capabilities,..}) if capabilities==[ClientCapability::MobileAdministrationV1])
    {
        return Err("mobile_admin_unsupported");
    }
    match connection.exchange(&WrapperToDaemon::MobileAdministration { request }) {
        Ok(DaemonToWrapper::MobileAdministrationResult { result }) => Ok(result),
        _ => Err("mobile_admin_outcome_unknown"),
    }
}

fn parse_admin(args: &[String]) -> Result<AdminRequest, &'static str> {
    match args.first().map(String::as_str) {
        Some("status") if args.len() == 1 => Ok(AdminRequest::Status),
        Some("devices") if args.len() == 1 => Ok(AdminRequest::Devices),
        Some("revoke") if args.len() == 2 => Ok(AdminRequest::Revoke {
            device_id: args[1].clone(),
        }),
        Some("configure") if args.len() == 3 && args[1] == "--public-url" => {
            Ok(AdminRequest::Configure {
                public_url: args[2].clone(),
                listen_addr: "127.0.0.1:9467".into(),
            })
        }
        Some("configure-notifications") if args.len()==3 && args[1]=="--service-account-file" => {
            let path=PathBuf::from(&args[2]);
            if !path.is_absolute(){return Err("service_account_file_must_be_absolute");}
            crate::environment::validate_state_file(&path,false).map_err(|_|"notification_configuration_unavailable")?;
            let mut bytes=Vec::new();
            std::fs::File::open(path).and_then(|f|f.take(32*1024+1).read_to_end(&mut bytes)).map_err(|_|"notification_configuration_unavailable")?;
            if bytes.len()>32*1024{return Err("notification_configuration_unavailable");}
            let account=String::from_utf8(bytes).map_err(|_|"notification_configuration_unavailable")?;
            Ok(AdminRequest::ConfigureNotifications {account:IdentityCredential::new(account)})
        }
        Some("pair") if args.len() == 2 && args[1] == "--all" => Ok(AdminRequest::Pair {
            scope: json!({"all_machines":true,"machines":[],"all_projects":true,"projects":[],
              "operations":["read","message","task_read","task_continue","task_cancel","task_respond","devices_manage"]}),
        }),
        Some("pair") if args.len() == 3 && args[1] == "--scope-file" => {
            let path = PathBuf::from(&args[2]);
            if !path.is_absolute() {
                return Err("scope_file_must_be_absolute");
            }
            let mut file = std::fs::File::open(path).map_err(|_| "scope_file_unavailable")?;
            let mut bytes = Vec::new();
            Read::by_ref(&mut file)
                .take(16 * 1024 + 1)
                .read_to_end(&mut bytes)
                .map_err(|_| "scope_file_unavailable")?;
            if bytes.len() > 16 * 1024 {
                return Err("invalid_scope");
            }
            let scope: store::Scope =
                serde_json::from_slice(&bytes).map_err(|_| "invalid_scope")?;
            Ok(AdminRequest::Pair {
                scope: serde_json::to_value(scope).map_err(|_| "invalid_scope")?,
            })
        }
        _ => Err("invalid_mobile_command"),
    }
}

pub(crate) fn run(args: &[String]) {
    if args.is_empty()
        || matches!(
            args.first().map(String::as_str),
            Some("help" | "--help" | "-h")
        )
    {
        println!(
            "bridget mobile configure --public-url https://bridget.example.net\nbridget mobile pair --all | --scope-file /absolute/scope.json\nbridget mobile devices\nbridget mobile revoke DEVICE_UUID\nbridget mobile status\nbridget mobile serve\nbridget mobile configure-notifications --service-account-file /absolute/private-account.json"
        );
        return;
    }
    let result = (|| -> Result<(), &'static str> {
        if args == ["serve"] {
            let ns = crate::environment::Namespace::from_environment()
                .map_err(|_| "namespace_unavailable")?;
            return http::serve(&ns.root, &ns.socket);
        }
        let request = parse_admin(args)?;
        if !matches!(request, AdminRequest::Status)
            && !(std::io::stdin().is_terminal() && std::io::stdout().is_terminal())
        {
            return Err("human_terminal_required");
        }
        let namespace = crate::environment::Namespace::from_environment()
            .map_err(|_| "namespace_unavailable")?;
        let response = admin_connection(&namespace.socket, request)?;
        match response {
            AdminResult::Pairing {
                pairing_id,
                credential,
                expires_at,
                public_url,
            } => {
                let payload = json!({"version":1,"endpoint":public_url,"pairing_id":pairing_id,
                    "credential":credential.expose_for_attestation()})
                .to_string();
                let qr = qrcode::QrCode::new(payload.as_bytes())
                    .map_err(|_| "pairing_qr_unavailable")?;
                let rendered = qr
                    .render::<qrcode::render::unicode::Dense1x2>()
                    .dark_color(qrcode::render::unicode::Dense1x2::Light)
                    .light_color(qrcode::render::unicode::Dense1x2::Dark)
                    .quiet_zone(true)
                    .build();
                println!(
                    "Scannez ce QR dans Bridget. Il autorise le téléphone ; ne le partagez pas.\n{rendered}\nExpiration Unix : {expires_at}."
                );
            }
            AdminResult::Error { code } => {
                eprintln!("bridget mobile : {code}");
                std::process::exit(1);
            }
            other => println!(
                "{}",
                serde_json::to_string_pretty(&other).map_err(|_| "mobile_result_unavailable")?
            ),
        }
        Ok(())
    })();
    if let Err(code) = result {
        eprintln!("bridget mobile : {code}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mobile177_configuration_requires_https_and_loopback() {
        let mut c = Config {
            public_url: "https://bridget.example.net".into(),
            listen_addr: "127.0.0.1:9467".parse().unwrap(),
        };
        assert!(c.validate().is_ok());
        for url in [
            "http://bridget.example.net",
            "https://user@bridget.example.net",
            "https://bridget.example.net/?secret=x",
            "https://bridget.example.net/",
            "https://-bad.example.net",
        ] {
            c.public_url = url.into();
            assert!(c.validate().is_err());
        }
        c.public_url = "https://bridget.example.net".into();
        c.listen_addr = "0.0.0.0:9467".parse().unwrap();
        assert!(c.validate().is_err());
    }

    #[test]
    fn mobile177_pairing_requires_an_explicit_scope() {
        assert!(parse_admin(&["pair".into()]).is_err());
        assert!(matches!(
            parse_admin(&["pair".into(), "--all".into()]),
            Ok(AdminRequest::Pair { .. })
        ));
        assert!(parse_admin(&["pair".into(), "--all".into(), "--extra".into()]).is_err());
        assert!(parse_admin(&["revoke".into()]).is_err());
    }

    #[test]
    fn mobile177_pairing_result_debug_redacts_the_credential() {
        let result = AdminResult::Pairing {
            pairing_id: uuid::Uuid::new_v4().to_string(),
            credential: IdentityCredential::new("private-pairing-value".into()),
            expires_at: 100,
            public_url: "https://bridget.example.net".into(),
        };
        assert!(!format!("{result:?}").contains("private-pairing-value"));
        let encoded = serde_json::to_string(&result).unwrap();
        assert!(encoded.contains("private-pairing-value"));
    }
}
