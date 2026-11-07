//! Comptes isolés : une connexion du registre, aucun coffre de credentials.
//! La CLI humaine applique un aperçu lié au SHA du registre. Les mutations
//! partagent le verrou du préparateur de projets et conservent leur sauvegarde.
use crate::cli_table::table_println as println;
use crate::registry::AgentRegistry;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, &'static str>;
const MAX_ACCOUNTS: usize = 32;
const MAX_BACKUPS: usize = 32;

struct Change {
    next: Value,
    view: Value,
    create: Option<PathBuf>,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn label(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 48
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"_-".contains(&b))
        && id.as_bytes()[0].is_ascii_alphanumeric()
}
fn account_root(registry: &Path, id: &str) -> Result<PathBuf> {
    if !label(id) {
        return Err("account_id_invalid");
    }
    Ok(registry
        .parent()
        .ok_or("account_registry_unsafe")?
        .join("provider-accounts")
        .join(id))
}

/// O(connexions + octets du registre), borné par les limites du registre.
/// Aucun fichier d'authentification n'est lu, pas même pour annoncer un login.
fn prepare_change(
    raw: &str,
    path: &Path,
    scope: &str,
    action: &str,
    id: &str,
    from: Option<&str>,
) -> Result<Change> {
    let registry = AgentRegistry::from_json(raw, path).map_err(|_| "account_registry_invalid")?;
    let policy = registry
        .routing_policy()
        .ok_or("account_routing_unavailable")?;
    let mut next: Value = serde_json::from_str(raw).map_err(|_| "account_registry_invalid")?;
    let connection_id = format!("account-{id}");
    let root = account_root(path, id)?;
    let current = policy
        .connections
        .iter()
        .find(|c| c.connection_id == connection_id);
    if action == "prepare" {
        if current.is_some() {
            return Err("account_already_prepared");
        }
        if next["agents"]
            .as_object()
            .into_iter()
            .flat_map(|agents| agents.keys())
            .filter(|id| id.starts_with("account-"))
            .count()
            >= MAX_ACCOUNTS
        {
            return Err("account_limit");
        }
        let source = policy
            .connections
            .iter()
            .find(|c| Some(c.connection_id.as_str()) == from)
            .ok_or("account_source_unknown")?;
        if source.machine_id != scope && source.machine_id != "local" {
            return Err("account_remote_prepare_required");
        }
        let mut definition = registry
            .get(&source.agent_type)
            .map_err(|_| "account_source_unknown")?
            .clone();
        if !safe_template(&definition, &source.engine) {
            return Err("account_template_needs_review");
        }
        let variable = match (source.engine.as_str(), definition.protocol.as_str()) {
            ("codex", "codex_app_server") => "CODEX_HOME",
            ("claude", "claude_stream_json") => "CLAUDE_CONFIG_DIR",
            _ => return Err("account_adapter_unavailable"),
        };
        if root.exists() || root.symlink_metadata().is_ok() {
            return Err("account_directory_unregistered");
        }
        let profile = root.join(&source.engine);
        const AUTH_SELECTION: &[&str] = &[
            "OPENAI_BASE_URL",
            "ANTHROPIC_BASE_URL",
            "ANTHROPIC_BEDROCK_BASE_URL",
            "ANTHROPIC_VERTEX_BASE_URL",
            "OPENAI_API_KEY",
            "CODEX_API_KEY",
            "OPENAI_ACCESS_TOKEN",
            "OPENAI_IDENTITY_TOKEN_FILE",
            "OPENAI_FEDERATION_RULE_ID",
            "OPENAI_ORGANIZATION_ID",
            "ANTHROPIC_API_KEY",
            "ANTHROPIC_AUTH_TOKEN",
            "CLAUDE_CODE_OAUTH_TOKEN",
            "ANTHROPIC_PROFILE",
            "ANTHROPIC_FEDERATION_RULE_ID",
            "ANTHROPIC_ORGANIZATION_ID",
            "ANTHROPIC_IDENTITY_TOKEN_FILE",
            "ANTHROPIC_CONFIG_DIR",
            "CLAUDE_CODE_USE_BEDROCK",
            "CLAUDE_CODE_USE_VERTEX",
            "CLAUDE_CODE_USE_FOUNDRY",
        ];
        definition
            .pass_env
            .retain(|v| v != variable && !AUTH_SELECTION.contains(&v.as_str()));
        definition.capabilities.account_profiles = true;
        if source.engine == "codex" {
            definition.codex_home = Some(profile.to_string_lossy().into_owned());
            for name in ["OPENAI_API_KEY", "CODEX_API_KEY"] {
                if !definition.forbidden_env.iter().any(|v| v == name) {
                    definition.forbidden_env.push(name.into());
                }
            }
        } else {
            definition.claude_config_dir = Some(profile.to_string_lossy().into_owned());
        }
        let mut connection = source.clone();
        connection.connection_id = connection_id.clone();
        connection.agent_type = connection_id.clone();
        connection.display_name = Some(id.into());
        connection.enabled = false;
        let mut models = policy
            .model_catalog
            .get(&source.connection_id)
            .cloned()
            .unwrap_or_default();
        for model in &mut models {
            model.provenance = crate::registry::RoutingModelProvenance::Declared;
            model.observed_at = None;
        }
        next.as_object_mut()
            .ok_or("account_registry_invalid")?
            .entry("agents")
            .or_insert_with(|| json!({}));
        next["agents"]
            .as_object_mut()
            .ok_or("account_registry_invalid")?
            .insert(
                connection_id.clone(),
                serde_json::to_value(&definition).map_err(|_| "account_registry_invalid")?,
            );
        next["routing_policy"]["connections"]
            .as_array_mut()
            .ok_or("account_registry_invalid")?
            .push(serde_json::to_value(&connection).map_err(|_| "account_registry_invalid")?);
        next["routing_policy"]["model_catalog"]
            .as_object_mut()
            .ok_or("account_registry_invalid")?
            .insert(
                connection_id.clone(),
                serde_json::to_value(&models).map_err(|_| "account_registry_invalid")?,
            );
        advance(&mut next)?;
        validate_next(&next, path)?;
        let login_args = if source.engine == "codex" {
            vec!["login", "--device-auth"]
        } else {
            vec!["auth", "login"]
        };
        let mut login_environment=json!({(variable):profile});
        if source.engine=="claude" {
            login_environment["ANTHROPIC_CONFIG_DIR"]=json!(profile.join("anthropic"));
        }
        return Ok(Change {
            next,
            create: Some(profile.clone()),
            view: json!({
                "action":action,"connection_id":connection_id,"machine_id":source.machine_id,
                "engine":source.engine,"account_directory":profile,"enabled":false,"auth":"not_verified",
                "login":{"program":definition.command,"args":login_args,"environment":login_environment,
                    "clear_environment":AUTH_SELECTION},
                "login_scope":if source.engine=="claude" {"claude_and_anthropic_isolated"} else {"native_codex_profile"},
                "next_action":if source.engine=="claude" {
                    "Connecter le compte voulu avec la commande et les deux dossiers indiqués, puis activer cette connexion avec accounts enable. CLAUDE_CONFIG_DIR isole Claude et ANTHROPIC_CONFIG_DIR isole aussi les profils Console. Ne pas lancer la connexion sans cet environnement. Aucun login ni changement du compte principal n'a été lancé."
                } else {
                    "Effectuer la connexion officielle Codex avec la commande et l'environnement indiqués. Vérifier le compte choisi, puis activer cette connexion avec accounts enable. Aucun login ni changement du compte principal n'a été lancé."
                }
            }),
        });
    }
    let connection = current.ok_or("account_unknown")?;
    if connection.machine_id != scope && connection.machine_id != "local" {
        return Err("account_remote_prepare_required");
    }
    let definition = registry
        .get(&connection.agent_type)
        .map_err(|_| "account_unknown")?;
    let profile = match connection.engine.as_str() {
        "codex" => definition.codex_home.as_deref(),
        "claude" => definition.claude_config_dir.as_deref(),
        _ => None,
    }
    .ok_or("account_not_managed")?;
    if Path::new(profile) != root.join(&connection.engine) || connection.agent_type != connection_id
    {
        return Err("account_not_managed");
    }
    if action == "enable" {
        crate::lifecycle::validate_provider_profile_directory(profile, &connection.engine)
            .map_err(|_| "account_directory_unavailable")?;
    }
    let rows = next["routing_policy"]["connections"]
        .as_array_mut()
        .ok_or("account_registry_invalid")?;
    if action == "remove" {
        if policy.connections.iter().any(|other| {
            other.connection_id != connection_id && other.agent_type == connection.agent_type
        }) {
            return Err("account_profile_shared");
        }
        rows.retain(|row| row["connection_id"] != connection_id);
        next["routing_policy"]["model_catalog"]
            .as_object_mut()
            .ok_or("account_registry_invalid")?
            .remove(&connection_id);
        // Les tâches gardent leur définition figée. Le registre courant ne
        // propose plus cet alias, y compris au lancement direct.
        next["agents"]
            .as_object_mut()
            .ok_or("account_registry_invalid")?
            .remove(&connection.agent_type);
    } else if action == "enable" || action == "disable" {
        rows.iter_mut()
            .find(|row| row["connection_id"] == connection_id)
            .ok_or("account_unknown")?["enabled"] = json!(action == "enable");
    } else {
        return Err("account_arguments_invalid");
    }
    advance(&mut next)?;
    validate_next(&next, path)?;
    Ok(Change {
        next,
        create: None,
        view: json!({
            "action":action,"connection_id":connection_id,"machine_id":connection.machine_id,
            "auth":"not_verified","files":"preserved","existing_tasks":"unchanged",
            "next_action":if action=="remove" {
                "La connexion disparaît du catalogue courant ; ses anciens faits ne donnent plus un droit d'admission. Conserver le dossier et les tâches déjà admises. Publier la même déclaration aux composants concernés avant de considérer le retrait distribué comme terminé."
            } else { "Relire le catalogue courant. Ce réglage ne prouve ni la connexion au compte ni l'accès à un modèle. Les missions déjà admises conservent leur propre état." }
        }),
    })
}

/// Ne jamais copier une map de réglages libres ou un argument de secret depuis
/// le compte principal. Le profil conserve ses restrictions ou refuse en bloc.
fn safe_template(definition: &crate::registry::AgentDefinition, engine: &str) -> bool {
    if Path::new(&definition.command)
        .file_name()
        .and_then(|s| s.to_str())
        != Some(engine)
    {
        return false;
    }
    let mut args = definition.args.iter().map(String::as_str);
    while let Some(arg) = args.next() {
        match (engine, arg) {
            ("codex", "app-server" | "--dangerously-bypass-approvals-and-sandbox") => {}
            ("codex", "-c" | "--config") => {
                let Some((key, value)) = args.next().and_then(|s| s.split_once('=')) else {
                    return false;
                };
                if !matches!(
                    key.trim(),
                    "model"
                        | "model_reasoning_effort"
                        | "sandbox_mode"
                        | "approval_policy"
                        | "approvals_reviewer"
                ) || value.len() > 256
                    || value.chars().any(char::is_control)
                {
                    return false;
                }
                // Ces seules valeurs ne transportent ni table libre ni contenu
                // d'un fichier de profil. Les réglages de droits sont conservés.
                if !(value.trim().starts_with('"')
                    && serde_json::from_str::<String>(value.trim()).is_ok())
                {
                    return false;
                }
            }
            ("codex", "--sandbox" | "--ask-for-approval") => {
                if !args.next().is_some_and(|value| {
                    matches!(
                        value,
                        "read-only"
                            | "workspace-write"
                            | "danger-full-access"
                            | "never"
                            | "on-request"
                            | "on-failure"
                            | "untrusted"
                    )
                }) {
                    return false;
                }
            }
            (
                "claude",
                "--verbose"
                | "--strict-mcp-config"
                | "--dangerously-skip-permissions"
                | "--allow-dangerously-skip-permissions"
                | "--no-session-persistence",
            ) => {}
            ("claude", "--permission-mode") => {
                if !args.next().is_some_and(|value| {
                    matches!(
                        value,
                        "default" | "acceptEdits" | "plan" | "bypassPermissions" | "dontAsk"
                    )
                }) {
                    return false;
                }
            }
            ("claude", "--output-format") => {
                if !args
                    .next()
                    .is_some_and(|value| matches!(value, "text" | "json" | "stream-json"))
                {
                    return false;
                }
            }
            (_, "--model" | "-m") => {
                if !args
                    .next()
                    .is_some_and(|value| definition.capabilities.models.contains_key(value))
                {
                    return false;
                }
            }
            _ => return false,
        }
    }
    true
}

fn advance(value: &mut Value) -> Result<()> {
    let revision = value["routing_policy"]["revision"]
        .as_u64()
        .ok_or("account_registry_invalid")?;
    value["routing_policy"]["revision"] = json!(
        revision
            .checked_add(1)
            .ok_or("account_revision_exhausted")?
    );
    Ok(())
}
fn validate_next(value: &Value, path: &Path) -> Result<()> {
    AgentRegistry::from_json(&value.to_string(), path)
        .map(|_| ())
        .map_err(|_| "account_registry_invalid")
}

fn private_dir(path: &Path) -> Result<()> {
    if path.symlink_metadata().is_err() {
        fs::DirBuilder::new()
            .mode(0o700)
            .create(path)
            .map_err(|_| "account_directory_unavailable")?;
    }
    crate::lifecycle::validate_provider_profile_directory(&path.to_string_lossy(), "fournisseur")
        .map_err(|_| "account_directory_unavailable")
}

/// Une écriture humaine, sauvegarde préalable, même verrou que prepare-project.
/// Un éditeur extérieur doit respecter ce verrou ; le CAS refuse toute version
/// déjà divergente. Il ne prétend pas verrouiller les outils étrangers.
fn apply(
    path: &Path,
    expected: &str,
    action: &str,
    id: &str,
    from: Option<&str>,
    scope: &str,
) -> Result<Value> {
    let parent = path.parent().ok_or("account_registry_unsafe")?;
    private_dir(parent)?;
    let lock = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(parent.join("project-preparation.lock"))
        .map_err(|_| "account_registry_lock_unavailable")?;
    let m = lock
        .metadata()
        .map_err(|_| "account_registry_lock_unavailable")?;
    if !m.is_file()
        || m.nlink() != 1
        || m.uid() != unsafe { libc::geteuid() }
        || m.mode() & 0o077 != 0
    {
        return Err("account_registry_lock_unavailable");
    }
    // Attente bornée : une préparation brève ou un fork→exec voisin qui hérite
    // un instant du descripteur ne doit pas refuser l'opération. Au-delà, le
    // refus reste explicite. O(5 s / 20 ms) tentatives au plus.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        if std::io::Error::last_os_error().raw_os_error() != Some(libc::EWOULDBLOCK)
            || std::time::Instant::now() >= deadline
        {
            return Err("account_registry_busy");
        }
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
    let original =
        crate::registry::read_private_registry(path).map_err(|_| "account_registry_unsafe")?;
    if digest(original.as_bytes()) != expected {
        return Err("account_registry_changed");
    }
    let change = prepare_change(&original, path, scope, action, id, from)?;
    let backups = parent.join("account-registry-backups");
    private_dir(&backups)?;
    let count = fs::read_dir(&backups)
        .map_err(|_| "account_backup_unavailable")?
        .count();
    let backup = backups.join(format!("{expected}.json"));
    if backup.symlink_metadata().is_ok() {
        let saved=crate::registry::read_private_registry(&backup)
            .map_err(|_|"account_backup_unavailable")?;
        if saved!=original { return Err("account_backup_unavailable"); }
    } else {
        if count >= MAX_BACKUPS { return Err("account_backup_capacity"); }
        write_new(&backup, original.as_bytes())?;
    }
    File::open(&backups)
        .and_then(|f| f.sync_all())
        .map_err(|_| "account_backup_unavailable")?;
    if let Some(profile) = &change.create {
        let accounts = parent.join("provider-accounts");
        private_dir(&accounts)?;
        let root = profile.parent().ok_or("account_directory_unavailable")?;
        if root.symlink_metadata().is_ok() {
            return Err("account_directory_unregistered");
        }
        private_dir(root)?;
        private_dir(profile)?;
        if change.view["engine"]=="claude" {
            private_dir(&profile.join("anthropic"))?;
        }
        File::open(profile)
            .and_then(|f| f.sync_all())
            .map_err(|_| "account_directory_unavailable")?;
        File::open(root)
            .and_then(|f| f.sync_all())
            .map_err(|_| "account_directory_unavailable")?;
        File::open(&accounts)
            .and_then(|f| f.sync_all())
            .map_err(|_| "account_directory_unavailable")?;
    }
    let temporary = parent.join(format!(".accounts-{}.tmp", uuid::Uuid::new_v4()));
    let bytes = serde_json::to_vec_pretty(&change.next).map_err(|_| "account_registry_invalid")?;
    write_new(&temporary, &bytes)?;
    let current =
        crate::registry::read_private_registry(path).map_err(|_| "account_registry_unsafe")?;
    if current != original {
        return Err("account_registry_changed");
    }
    fs::rename(&temporary, path).map_err(|_| "account_registry_publish_failed")?;
    File::open(parent)
        .and_then(|f| f.sync_all())
        .map_err(|_| "account_registry_publish_unconfirmed")?;
    let mut view = change.view;
    view["status"] = json!("applied_local");
    view["registry_sha256"] = json!(digest(&bytes));
    view["backup"] = json!(backup);
    view["fleet_publication"] = json!("not_attested");
    Ok(view)
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
        .map_err(|_| "account_private_write_failed")?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "account_private_write_failed")
}

pub(crate) fn run(args: &[String]) {
    match execute(args) {
        Ok(view) => println!("{view}"),
        Err(code) => {
            println!(
                "{}",
                json!({"status":"refused","code":code,"scope":"this_account_operation",
                "next_action": guidance(code),"other_work":"Les autres travaux autorisés suivent leur propre état."})
            );
            crate::cli_table::exit(1);
        }
    }
}
fn guidance(code: &str) -> &'static str {
    match code {
        "account_registry_changed" => {
            "Relire l'aperçu du même compte. Le registre a changé ; aucune publication n'est autorisée avec l'ancien SHA. Conserver tout dossier ou fichier déjà préparé."
        }
        "account_profile_shared" => {
            "Cette définition sert encore une autre connexion. Demander au propriétaire quelles connexions retirer ensemble ; aucune définition ou donnée n'a été supprimée."
        }
        "account_template_needs_review" => {
            "Choisir une connexion utilisant directement le programme officiel, ou faire préparer un profil propre par son responsable. Bridget ne copie pas les réglages libres d'un lanceur ni un secret du compte principal."
        }
        "account_remote_prepare_required" => {
            "Préparer ce compte sur la machine déclarée par sa connexion, dans son registre worker. Aucun dossier du serveur n'a été créé sur cet ordinateur."
        }
        "account_directory_unregistered" => {
            "Conserver ce dossier. Une préparation antérieure ou un autre compte peut l'utiliser ; demander au responsable de vérifier sa provenance et les tâches concernées avant de choisir une suite."
        }
        "account_human_terminal_required" => {
            "L'application du changement appartient à l'utilisateur dans son terminal. L'aperçu reste lisible sans appliquer ni lancer de connexion."
        }
        "account_backup_capacity" => {
            "Faire examiner les sauvegardes de ce registre par son propriétaire avant une nouvelle modification. Aucun historique n'a été supprimé automatiquement."
        }
        "account_registry_publish_unconfirmed" => {
            "Relire le registre et le catalogue avant toute autre action. Le changement a été renommé mais sa durabilité finale n'est pas confirmée ; ne pas recréer le compte ni lancer une mission pour le tester."
        }
        "account_already_prepared" => {
            "Ce nom désigne déjà une connexion. Lire accounts list et son état au lieu de préparer un doublon."
        }
        "account_arguments_invalid" => {
            "Utiliser accounts list, ou accounts prepare --from CONNEXION --id NOM. Pour appliquer l'aperçu : reprendre son SHA avec --expected SHA --apply. enable, disable et remove utilisent --id NOM. --registry CHEMIN choisit un registre privé local."
        }
        _ => {
            "Conserver la connexion et ses données. Faire vérifier ce code sur le registre et le profil locaux par le responsable ; ne pas modifier le compte principal, les droits ou le programme pour forcer l'opération."
        }
    }
}
fn execute(args: &[String]) -> Result<Value> {
    let action = args
        .first()
        .map(String::as_str)
        .ok_or("account_arguments_invalid")?;
    let mut registry_path =
        crate::registry::config_path().map_err(|_| "account_registry_unsafe")?;
    let mut id = None;
    let mut from = None;
    let mut expected = None;
    let mut commit = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--apply" => {
                if commit {
                    return Err("account_arguments_invalid");
                };
                commit = true;
                index += 1;
            }
            "--json" => {
                index += 1;
            }
            key @ ("--registry" | "--id" | "--from" | "--expected") => {
                let value = args
                    .get(index + 1)
                    .ok_or("account_arguments_invalid")?
                    .clone();
                match key {
                    "--registry" => registry_path = PathBuf::from(value),
                    "--id" => id = Some(value),
                    "--from" => from = Some(value),
                    _ => expected = Some(value),
                }
                index += 2;
            }
            _ => return Err("account_arguments_invalid"),
        }
    }
    if !registry_path.is_absolute()
        || registry_path
            .components()
            .any(|p| matches!(p, std::path::Component::ParentDir))
    {
        return Err("account_registry_unsafe");
    }
    let raw = crate::registry::read_private_registry(&registry_path)
        .map_err(|_| "account_registry_unsafe")?;
    let registry =
        AgentRegistry::from_json(&raw, &registry_path).map_err(|_| "account_registry_invalid")?;
    let scope = crate::client_compatibility::machine_scope(
        &registry,
        &registry_path,
        &crate::build_info::local_host(),
    )
    .map_err(|_| "account_machine_scope_unavailable")?;
    if action == "list" {
        if commit || id.is_some() || from.is_some() || expected.is_some() {
            return Err("account_arguments_invalid");
        }
        let policy = registry
            .routing_policy()
            .ok_or("account_routing_unavailable")?;
        let accounts:Vec<_>=policy.connections.iter().filter(|c|c.machine_id==scope||c.machine_id=="local")
            .map(|c|json!({"connection_id":c.connection_id,"engine":c.engine,"machine_id":c.machine_id,
                "enabled":c.enabled,"auth":"not_verified","managed":c.connection_id.starts_with("account-")})).collect();
        return Ok(
            json!({"status":"read","machine_id":scope,"accounts":accounts,"registry_sha256":digest(raw.as_bytes())}),
        );
    }
    let id = id.ok_or("account_arguments_invalid")?;
    if !matches!(action, "prepare" | "enable" | "disable" | "remove")
        || (action != "prepare" && from.is_some())
    {
        return Err("account_arguments_invalid");
    }
    if commit {
        if !crate::cli::require_interactive_terminal("accounts") {
            return Err("account_human_terminal_required");
        }
        let expected = expected.ok_or("account_arguments_invalid")?;
        if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("account_arguments_invalid");
        }
        apply(
            &registry_path,
            &expected,
            action,
            &id,
            from.as_deref(),
            &scope,
        )
    } else {
        if expected.is_some() {
            return Err("account_arguments_invalid");
        }
        let change = prepare_change(&raw, &registry_path, &scope, action, &id, from.as_deref())?;
        let mut view = change.view;
        view["status"] = json!("preview");
        view["expected"] = json!(digest(raw.as_bytes()));
        view["files_created"] = json!(false);
        view["provider_started"] = json!(false);
        Ok(view)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::ffi::OsString;

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = std::env::temp_dir()
                .canonicalize()
                .unwrap()
                .join(format!("bridget-accounts-{}", uuid::Uuid::new_v4()));
            fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
            Self(p)
        }
        fn path(&self) -> PathBuf {
            self.0.join("agents.json")
        }
        fn registry(&self) -> String {
            json!({"agents":{"native-test":{
                "command":"codex","args":["app-server"],"protocol":"codex_app_server",
                "pass_env":["CODEX_HOME"],"capabilities":{"execution_paths":["codex_app_server"],
                    "models":{"m000":{"efforts":["low"]}}}}},
                "routing_policy":{"version":2,"revision":1,"enabled":true,
                    "connections":[{"connection_id":"main","machine_id":"local","engine":"codex","agent_type":"native-test","enabled":true}],
                    "model_catalog":{"main":[{"model":"m000","supported_efforts":["low"],"default_effort":"low","provenance":"declared","observed_at":null}]},
                    "machine_policies":{"local":{"enabled":true}}}}).to_string()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn spec243_verrou_bref_attendu_puis_occupation_durable_refusee() {
        use std::thread;
        use std::time::{Duration, Instant};
        let f = Fixture::new();
        let raw = f.registry();
        write_new(&f.path(), raw.as_bytes()).unwrap();
        let hold = |duration: Duration| {
            let lock = OpenOptions::new().read(true).write(true).create(true).truncate(false)
                .mode(0o600).open(f.0.join("project-preparation.lock")).unwrap();
            assert_eq!(unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX) }, 0);
            thread::spawn(move || { thread::sleep(duration); drop(lock); })
        };
        // Détention brève (préparation courte, fork→exec voisin) : attendue.
        let brief = hold(Duration::from_millis(300));
        let started = Instant::now();
        apply(&f.path(), &digest(raw.as_bytes()), "prepare", "work", Some("main"), "local").unwrap();
        assert!(started.elapsed() >= Duration::from_millis(250));
        brief.join().unwrap();
        // Occupation durable : refus explicite après la borne, registre inchangé.
        let current = crate::registry::read_private_registry(&f.path()).unwrap();
        let durable = hold(Duration::from_secs(7));
        assert_eq!(
            apply(&f.path(), &digest(current.as_bytes()), "remove", "work", None, "local"),
            Err("account_registry_busy")
        );
        assert_eq!(crate::registry::read_private_registry(&f.path()).unwrap(), current);
        durable.join().unwrap();
    }

    #[test]
    fn spec221_c06_prepare_pin_and_remove_keep_main_login_and_saved_profile() {
        let f = Fixture::new();
        let raw = f.registry();
        write_new(&f.path(), raw.as_bytes()).unwrap();
        let main = f.0.join("principal");
        private_dir(&main).unwrap();
        let auth = main.join("auth.json");
        write_new(&auth, b"MAIN_SYNTHETIC").unwrap();
        let preview =
            prepare_change(&raw, &f.path(), "local", "prepare", "work", Some("main")).unwrap();
        assert!(!f.0.join("provider-accounts").exists());
        assert_eq!(preview.view["auth"], "not_verified");
        assert_eq!(
            preview.next["routing_policy"]["connections"][0],
            serde_json::from_str::<Value>(&raw).unwrap()["routing_policy"]["connections"][0]
        );
        assert!(
            preview.view["login"]["clear_environment"]
                .as_array()
                .unwrap()
                .iter()
                .any(|v| v == "OPENAI_API_KEY")
        );
        let done = apply(
            &f.path(),
            &digest(raw.as_bytes()),
            "prepare",
            "work",
            Some("main"),
            "local",
        )
        .unwrap();
        assert_eq!(done["status"], "applied_local");
        let content = crate::registry::read_private_registry(&f.path()).unwrap();
        let registry = AgentRegistry::from_json(&content, f.path()).unwrap();
        let child = registry.get("account-work").unwrap();
        let profile = f.0.join("provider-accounts/work/codex");
        assert_eq!(child.codex_home.as_deref(), profile.to_str());
        assert!(!child.pass_env.iter().any(|v| v == "CODEX_HOME"));
        let source = BTreeMap::from([
            ("HOME".into(), OsString::from(&f.0)),
            ("PATH".into(), OsString::from("/usr/bin:/bin")),
            ("CODEX_HOME".into(), main.as_os_str().to_owned()),
        ]);
        let before = source.clone();
        let env = crate::lifecycle::build_environment(child, &source).unwrap();
        assert_eq!(env["CODEX_HOME"], profile.as_os_str());
        assert_eq!(env["HOME"], source["HOME"]);
        assert_eq!(source, before);
        assert_eq!(fs::read(&auth).unwrap(), b"MAIN_SYNTHETIC");
        assert!(!profile.join("auth.json").exists());
        let enabled = apply(
            &f.path(),
            &digest(content.as_bytes()),
            "enable",
            "work",
            None,
            "local",
        )
        .unwrap();
        assert_eq!(enabled["auth"], "not_verified");
        let active = crate::registry::read_private_registry(&f.path()).unwrap();
        apply(
            &f.path(),
            &digest(active.as_bytes()),
            "remove",
            "work",
            None,
            "local",
        )
        .unwrap();
        let removed = crate::registry::read_private_registry(&f.path()).unwrap();
        let parsed = AgentRegistry::from_json(&removed, f.path()).unwrap();
        assert!(
            !parsed
                .routing_policy()
                .unwrap()
                .connections
                .iter()
                .any(|c| c.connection_id == "account-work")
        );
        assert!(
            !parsed
                .routing_policy()
                .unwrap()
                .model_catalog
                .contains_key("account-work")
        );
        assert!(profile.is_dir());
        assert!(parsed.get("account-work").is_err());
        // Forme exacte du lecteur antérieur : matrice fermée à trois champs.
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        #[allow(dead_code)]
        struct LegacyCapabilities {
            #[serde(default)] execution_paths:Vec<String>,
            #[serde(default)] models:serde_json::Value,
            #[serde(default)] observed:Option<serde_json::Value>,
        }
        let guarded=serde_json::to_value(&child.capabilities).unwrap();
        assert!(guarded["account_profiles"]==true);
        assert!(serde_json::from_value::<LegacyCapabilities>(guarded).is_err());
        assert!(serde_json::from_value::<LegacyCapabilities>(json!({"execution_paths":["acp"]})).is_ok());

        assert!(
            AgentRegistry::from_resolved(
                "account-work",
                &registry.resolved_definition("account-work").unwrap()
            )
            .is_ok()
        );
        assert_eq!(fs::read(&auth).unwrap(), b"MAIN_SYNTHETIC");
    }

    #[test]
    fn spec221_c06_claude_native_profile_isolates_both_stores_without_settings_override() {
        use std::os::unix::fs::PermissionsExt;
        let f=Fixture::new();
        let program=f.0.join("claude");
        // Même fixture binaire que native149_capture : un script shell est
        // volontairement opaque et ne doit pas servir de faux témoin natif.
        fs::copy("/bin/echo",&program).unwrap();
        fs::set_permissions(&program,fs::Permissions::from_mode(0o700)).unwrap();
        let program_before=fs::read(&program).unwrap();
        let mut value:Value=serde_json::from_str(&f.registry()).unwrap();
        value["agents"]["native-test"]["command"]=json!(program);
        value["agents"]["native-test"]["args"]=json!([]);
        value["agents"]["native-test"]["protocol"]=json!("claude_stream_json");
        value["agents"]["native-test"]["capabilities"]["execution_paths"]=json!(["claude_stream_json"]);
        value["routing_policy"]["connections"][0]["engine"]=json!("claude");
        let raw=value.to_string();
        write_new(&f.path(),raw.as_bytes()).unwrap();
        let done=apply(&f.path(),&digest(raw.as_bytes()),"prepare","work",Some("main"),"local").unwrap();
        assert_eq!(done["login"]["args"],json!(["auth","login"]));
        let profile=f.0.join("provider-accounts/work/claude");
        assert_eq!(done["login"]["environment"]["ANTHROPIC_CONFIG_DIR"],json!(profile.join("anthropic")));
        let content=crate::registry::read_private_registry(&f.path()).unwrap();
        let registry=AgentRegistry::from_json(&content,f.path()).unwrap();
        let definition=registry.get("account-work").unwrap();
        assert!(definition.args.is_empty());
        let source=BTreeMap::from([("HOME".into(),f.0.as_os_str().to_owned()),
            ("PATH".into(),OsString::from("/usr/bin:/bin")),
            ("ANTHROPIC_CONFIG_DIR".into(),OsString::from("/principal"))]);
        let env=crate::lifecycle::build_environment(definition,&source).unwrap();
        assert_eq!(env["CLAUDE_CONFIG_DIR"],profile.as_os_str());
        assert_eq!(env["ANTHROPIC_CONFIG_DIR"],profile.join("anthropic").as_os_str());
        let context=crate::native_permissions::capture_context_with_env(definition,&f.0,&env).unwrap();
        assert!(context.get("settings_overrides").is_none());
        let mut args=definition.args.clone();
        crate::native_permissions::apply_child_arguments("claude_stream_json",
            &json!({"kind":"claude","permission_mode":"bypassPermissions",
                "tools":{"type":"preset","preset":"claude_code"}}),&mut args).unwrap();
        assert!(!args.iter().any(|a|a=="--settings"));
        assert_eq!(fs::read(&program).unwrap(),program_before);
    }

    #[test]
    fn spec221_c06_stale_preview_unknown_directory_and_secret_template_refuse_before_write() {
        let f = Fixture::new();
        let raw = f.registry();
        write_new(&f.path(), raw.as_bytes()).unwrap();
        assert_eq!(
            apply(
                &f.path(),
                &"0".repeat(64),
                "prepare",
                "work",
                Some("main"),
                "local"
            )
            .unwrap_err(),
            "account_registry_changed"
        );
        assert_eq!(fs::read_to_string(f.path()).unwrap(), raw);
        assert!(!f.0.join("provider-accounts").exists());
        let mut value: Value = serde_json::from_str(&raw).unwrap();
        value["agents"]["native-test"]["args"] = json!([
            "-c",
            r#"model_providers.private.api_key="SYNTHETIC""#,
            "app-server"
        ]);
        assert_eq!(
            prepare_change(
                &value.to_string(),
                &f.path(),
                "local",
                "prepare",
                "work",
                Some("main")
            )
            .err(),
            Some("account_template_needs_review")
        );
        value["agents"]["native-test"]["args"] = json!(["app-server"]);
        value["routing_policy"]["connections"][0]["machine_id"] = json!("other");
        value["routing_policy"]["connections"][0]["enabled"] = json!(false);
        assert_eq!(
            prepare_change(
                &value.to_string(),
                &f.path(),
                "local",
                "prepare",
                "work",
                Some("main")
            )
            .err(),
            Some("account_remote_prepare_required")
        );
        assert_eq!(
            prepare_change(
                &raw,
                &f.path(),
                "local",
                "prepare",
                "../wrong",
                Some("main")
            )
            .err(),
            Some("account_id_invalid")
        );
        private_dir(&f.0.join("provider-accounts")).unwrap();
        private_dir(&f.0.join("provider-accounts/work")).unwrap();
        assert_eq!(
            prepare_change(&raw, &f.path(), "local", "prepare", "work", Some("main")).err(),
            Some("account_directory_unregistered")
        );
        assert_eq!(fs::read_to_string(f.path()).unwrap(), raw);
    }

    #[test]
    fn spec221_c06_profile_digest_and_missing_directory_cannot_fall_back_to_main() {
        let f = Fixture::new();
        let profile = f.0.join("isolated");
        private_dir(&profile).unwrap();
        let value = json!({"agents":{"private-codex":{"command":"codex","protocol":"codex_app_server","codex_home":profile,"capabilities":{"account_profiles":true}}}});
        let registry = AgentRegistry::from_json(&value.to_string(), f.path()).unwrap();
        let resolved = registry.resolved_definition("private-codex").unwrap();
        let loaded = AgentRegistry::from_resolved("private-codex", &resolved).unwrap();
        assert_eq!(
            loaded.get("private-codex").unwrap().codex_home,
            resolved.codex_home
        );
        let mut changed = resolved.clone();
        changed.codex_home = None;
        assert!(AgentRegistry::from_resolved("private-codex", &changed).is_err());
        let source = BTreeMap::from([("HOME".into(), OsString::from(&f.0))]);
        fs::remove_dir(&profile).unwrap();
        assert!(
            crate::lifecycle::build_environment(registry.get("private-codex").unwrap(), &source)
                .is_err()
        );
        let mut inherited = value.clone();
        inherited["agents"]["private-codex"]["pass_env"] = json!(["CODEX_HOME"]);
        assert!(AgentRegistry::from_json(&inherited.to_string(), f.path()).is_err());
        let plain = AgentRegistry::from_json("{}", &f.path())
            .unwrap()
            .resolved_definition("codex")
            .unwrap();
        assert!(
            serde_json::to_value(plain)
                .unwrap()
                .get("codex_home")
                .is_none()
        );
    }
}
