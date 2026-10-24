//! Sonnette FCM minimale ; aucune commande, identité fournisseur ou donnée métier.
use super::{Config, store::Store};
use base64::{
    Engine,
    engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant},
};

const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const MAX_RESPONSE: u64 = 64 * 1024;
#[derive(Deserialize)]
struct Account {
    #[serde(rename = "type")]
    kind: String,
    project_id: String,
    client_email: String,
    private_key: String,
    token_uri: String,
}
impl Account {
    fn load(root: &Path) -> Result<Self, &'static str> {
        let path = root.join("mobile-fcm-account.json");
        crate::environment::validate_state_file(&path, false)
            .map_err(|_| "notification_configuration_unavailable")?;
        let mut bytes = Vec::new();
        std::fs::File::open(path)
            .and_then(|f| f.take(32 * 1024 + 1).read_to_end(&mut bytes))
            .map_err(|_| "notification_configuration_unavailable")?;
        if bytes.len() > 32 * 1024 {
            return Err("notification_configuration_unavailable");
        }
        Self::parse(&bytes)
    }
    fn parse(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() > 32 * 1024 {
            return Err("notification_configuration_unavailable");
        }
        let value: Self =
            serde_json::from_slice(bytes).map_err(|_| "notification_configuration_unavailable")?;
        if value.kind != "service_account"
            || value.token_uri != TOKEN_URL
            || !(6..=63).contains(&value.project_id.len())
            || !value
                .project_id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            || !value.client_email.ends_with(".iam.gserviceaccount.com")
            || value.client_email.len() > 320
            || value.client_email.chars().any(char::is_control)
        {
            return Err("notification_configuration_unavailable");
        }
        value.key()?;
        Ok(value)
    }
    fn key(&self) -> Result<ring::signature::RsaKeyPair, &'static str> {
        let pem = self
            .private_key
            .trim()
            .strip_prefix("-----BEGIN PRIVATE KEY-----")
            .and_then(|s| s.strip_suffix("-----END PRIVATE KEY-----"))
            .ok_or("notification_configuration_unavailable")?;
        let der = STANDARD
            .decode(pem.split_whitespace().collect::<String>())
            .map_err(|_| "notification_configuration_unavailable")?;
        ring::signature::RsaKeyPair::from_pkcs8(&der)
            .map_err(|_| "notification_configuration_unavailable")
    }
    fn assertion(&self, now: i64) -> Result<String, &'static str> {
        let head = URL_SAFE_NO_PAD.encode(br#"{"alg":"RS256","typ":"JWT"}"#);
        let claims = json!({"iss":self.client_email,"scope":"https://www.googleapis.com/auth/firebase.messaging",
            "aud":TOKEN_URL,"iat":now,"exp":now+3600});
        let input = format!("{head}.{}", URL_SAFE_NO_PAD.encode(claims.to_string()));
        let key = self.key()?;
        let mut signature = vec![0; key.public().modulus_len()];
        key.sign(
            &ring::signature::RSA_PKCS1_SHA256,
            &ring::rand::SystemRandom::new(),
            input.as_bytes(),
            &mut signature,
        )
        .map_err(|_| "notification_authentication_unavailable")?;
        Ok(format!("{input}.{}", URL_SAFE_NO_PAD.encode(signature)))
    }
}
/// Configuration privée transmise seulement par l’administration humaine locale.
pub(super) fn configure(root: &Path, bytes: &[u8]) -> Result<(), &'static str> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    Account::parse(bytes)?;
    crate::environment::validate_existing_tree(root)
        .map_err(|_| "notification_configuration_unavailable")?;
    let path = root.join("mobile-fcm-account.json");
    crate::environment::validate_state_file(&path, false)
        .map_err(|_| "notification_configuration_unavailable")?;
    let temp = root.join(format!(".mobile-fcm-{}", uuid::Uuid::new_v4()));
    let result = (|| -> std::io::Result<()> {
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&temp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        std::fs::rename(&temp, &path)?;
        std::fs::File::open(root)?.sync_all()
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result.map_err(|_| "notification_configuration_unavailable")
}
pub(super) fn configured(root: &Path) -> bool {
    Account::load(root).is_ok()
}

fn response(request: minreq::Request) -> Result<(i32, Value), &'static str> {
    let mut reply = request
        .with_timeout(10)
        .with_follow_redirects(false)
        .send_lazy()
        .map_err(|_| "notification_transport_unavailable")?;
    let status = reply.status_code;
    let mut bytes = Vec::new();
    Read::take(Read::by_ref(&mut reply), MAX_RESPONSE + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "notification_transport_unavailable")?;
    if bytes.len() as u64 > MAX_RESPONSE {
        return Err("notification_response_invalid");
    }
    Ok((
        status.into(),
        serde_json::from_slice(&bytes).map_err(|_| "notification_response_invalid")?,
    ))
}
fn access_token(account: &Account, now: i64) -> Result<(String, i64), &'static str> {
    let assertion = account.assertion(now)?;
    let (status,value)=response(minreq::post(TOKEN_URL).with_header("Content-Type","application/x-www-form-urlencoded")
        .with_body(format!("grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Ajwt-bearer&assertion={assertion}")))?;
    let token = value["access_token"]
        .as_str()
        .ok_or("notification_authentication_unavailable")?;
    let ttl = value["expires_in"]
        .as_i64()
        .ok_or("notification_authentication_unavailable")?;
    if status != 200
        || value["token_type"] != "Bearer"
        || !(1..=8192).contains(&token.len())
        || !token.bytes().all(|b| b.is_ascii_graphic())
        || !(60..=3600).contains(&ttl)
    {
        return Err("notification_authentication_unavailable");
    }
    Ok((token.into(), now + ttl - 60))
}
pub(crate) fn binding(public_url: &str, device_id: &str) -> String {
    format!("{:x}", Sha256::digest(format!("{public_url}\0{device_id}")))
}
fn payload(token: &str, public_url: &str, device_id: &str) -> Value {
    json!({"message":{"token":token,"data":{"version":"1","binding":binding(public_url,device_id)},
        "android":{"priority":"HIGH","ttl":"300s","collapse_key":"bridget-update"}}})
}
fn deliver(
    account: &Account,
    authorization: &str,
    token: &str,
    public_url: &str,
    device_id: &str,
) -> Result<bool, &'static str> {
    let (status, value) = response(
        minreq::post(format!(
            "https://fcm.googleapis.com/v1/projects/{}/messages:send",
            account.project_id
        ))
        .with_header("Authorization", format!("Bearer {authorization}"))
        .with_header("Content-Type", "application/json")
        .with_body(payload(token, public_url, device_id).to_string()),
    )?;
    if status == 200 {
        return Ok(true);
    }
    let unregistered = value
        .pointer("/error/details")
        .and_then(Value::as_array)
        .is_some_and(|details| {
            details.iter().any(|v| {
                v["@type"] == "type.googleapis.com/google.firebase.fcm.v1.FcmError"
                    && v["errorCode"] == "UNREGISTERED"
            })
        });
    if unregistered {
        return Ok(false);
    }
    Err("notification_delivery_unconfirmed")
}

/// File de taille 1 ; O(1) sous le verrou métier, aucun appel Google dans ce thread.
pub(crate) struct Notifier(mpsc::SyncSender<()>);
impl Notifier {
    pub fn start(root: &Path) -> Option<Self> {
        let account = Account::load(root).ok()?;
        let config = Config::load(root).ok()??;
        let (sender, receiver) = mpsc::sync_channel(1);
        let root = root.to_path_buf();
        std::thread::Builder::new()
            .name("bridget-mobile-notices".into())
            .spawn(move || run(root, config.public_url, account, receiver))
            .ok()?;
        Some(Self(sender))
    }
    pub fn changed(&self) {
        let _ = self.0.try_send(());
    }
}
fn run(root: PathBuf, url: String, account: Account, receiver: mpsc::Receiver<()>) {
    let mut credential: Option<(String, i64)> = None;
    let mut previous: Option<Instant> = None;
    while receiver.recv().is_ok() {
        if let Some(last) = previous {
            if let Some(delay) = Duration::from_secs(15).checked_sub(last.elapsed()) {
                std::thread::sleep(delay);
            }
        }
        while receiver.try_recv().is_ok() {}
        let now = super::http::unix_now();
        let targets = match Store::open(&root.join("mobile-devices.db"))
            .and_then(|s| s.notification_targets(now))
        {
            Ok(v) => v,
            Err(_) => continue,
        };
        if targets.is_empty() {
            continue;
        }
        if credential.as_ref().is_none_or(|(_, expiry)| *expiry <= now) {
            credential = match access_token(&account, now) {
                Ok(v) => Some(v),
                Err(code) => {
                    log::warn!("mobile_push stage=authorization code={code}");
                    previous = Some(Instant::now());
                    continue;
                }
            };
        }
        for (device, token) in targets {
            // Revalider juste avant Google ; une notification déjà remise reste une sonnette sans contenu.
            let current = Store::open(&root.join("mobile-devices.db"))
                .and_then(|s| s.notification_targets(super::http::unix_now()));
            if !current.is_ok_and(|rows| rows.contains(&(device.clone(), token.clone()))) {
                continue;
            }
            match deliver(
                &account,
                &credential.as_ref().unwrap().0,
                &token,
                &url,
                &device,
            ) {
                Ok(true) => {}
                Ok(false) => {
                    let _ = Store::open(&root.join("mobile-devices.db"))
                        .and_then(|s| s.remove_notification_token(&device, &token));
                }
                Err(code) => {
                    log::warn!("mobile_push stage=delivery code={code}");
                    credential = None;
                    break;
                }
            }
        }
        previous = Some(Instant::now());
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mobile177_push_has_no_business_content_or_implicit_authority() {
        let value = payload("receiver", "https://bridge.example.net", "device");
        assert_eq!(
            value["message"]["data"],
            json!({"version":"1","binding":binding("https://bridge.example.net","device")})
        );
        assert!(value["message"].get("notification").is_none());
        assert_ne!(
            binding("https://a.example.net", "device"),
            binding("https://b.example.net", "device")
        );
        assert_ne!(
            binding("https://a.example.net", "device"),
            binding("https://a.example.net", "other")
        );
        assert_eq!(value["message"]["android"]["ttl"], "300s");
    }
    #[test]
    fn mobile177_private_fcm_configuration_loads_and_signs_without_external_service(){
        use std::os::unix::fs::{DirBuilderExt,PermissionsExt};
        let root=std::env::temp_dir().canonicalize().unwrap().join(format!("bridget177-fcm-{}",uuid::Uuid::new_v4()));
        std::fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let key=std::process::Command::new("openssl").args(["genpkey","-algorithm","RSA","-pkeyopt","rsa_keygen_bits:2048"]).output().expect("openssl is required to generate a throwaway test key");
        assert!(key.status.success());
        let account=json!({"type":"service_account","project_id":"bridget-fixture","client_email":"fixture@bridget-fixture.iam.gserviceaccount.com",
            "private_key":String::from_utf8(key.stdout).unwrap(),"token_uri":TOKEN_URL});
        configure(&root,account.to_string().as_bytes()).unwrap();
        assert!(configured(&root),"a regular private account file must be readable");
        let loaded=Account::load(&root).unwrap();
        let jwt=loaded.assertion(100).unwrap();
        let parts=jwt.split('.').collect::<Vec<_>>();assert_eq!(parts.len(),3);
        let claims:Value=serde_json::from_slice(&URL_SAFE_NO_PAD.decode(parts[1]).unwrap()).unwrap();
        assert_eq!(claims["exp"],3700);assert_eq!(claims["aud"],TOKEN_URL);
        let key=loaded.key().unwrap();
        ring::signature::UnparsedPublicKey::new(&ring::signature::RSA_PKCS1_2048_8192_SHA256,key.public().as_ref())
            .verify(format!("{}.{}",parts[0],parts[1]).as_bytes(),&URL_SAFE_NO_PAD.decode(parts[2]).unwrap()).unwrap();
        let mut bad=account.clone();bad["token_uri"]=json!("https://other.example.net/token");
        assert!(configure(&root,bad.to_string().as_bytes()).is_err());
        assert!(configured(&root),"an invalid replacement must preserve the current account");
        std::fs::set_permissions(root.join("mobile-fcm-account.json"),std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(!configured(&root),"a readable-by-others account must be refused");
        std::fs::remove_dir_all(root).unwrap();
    }

}
