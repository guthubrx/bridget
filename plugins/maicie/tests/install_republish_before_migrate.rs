//! Oracle : republication **avant** migration — jamais de greffe migré avec
//! un binaire installé périmé (piste B corrigée).
//!
//! Chemin d'échec exigé : si la republication échoue, la migration ne doit
//! pas avoir avancé le greffe. L'installé reste inchangé.

use maicie::install_publish::{
    INSTALL_BIN_ENV, InstallPublishError, reconcile_failed_migration, republish_exe_and_preflight,
    republish_exe_before_migrate,
};
use maicie::store::{SCHEMA_VERSION, StoreError};
use serde_json::{Value, json};
use std::ffi::CString;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt, symlink};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

struct FixtureRoot(PathBuf);

impl std::ops::Deref for FixtureRoot {
    type Target = Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Drop for FixtureRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

static NEXT_ROOT: AtomicU64 = AtomicU64::new(0);

// Fixture historique figée à v19. Les oracles exercent donc toutes les migrations ultérieures.
const FIXTURE_SCHEMA_VERSION: i64 = 19;

fn unique_root(label: &str) -> FixtureRoot {
    let n = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!(
        "maicie-republish-oracle-{label}-{}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    FixtureRoot(root)
}

fn write_config(root: &Path, database: &Path) -> PathBuf {
    let config = root.join("config.json");
    let payload = serde_json::json!({
        "version": 1,
        "bridget_socket": root.join("bridget.sock"),
        "database_path": database,
        "durations": {
            "short_secs": 30,
            "normal_secs": 300,
            "long_secs": 3_600
        },
        "profiles": []
    });
    fs::write(&config, serde_json::to_vec(&payload).unwrap()).unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    config
}

fn files_equal(left: &Path, right: &Path) -> bool {
    match (fs::read(left), fs::read(right)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

fn write_executable(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
}

fn database_user_version(database: &Path) -> i64 {
    let connection = rusqlite::Connection::open(database).unwrap();
    connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .unwrap()
}

fn start_local_daemon_identity(socket: &Path) -> thread::JoinHandle<()> {
    let socket = socket.to_owned();
    let _ = fs::remove_file(&socket);
    let local_host = bridget_core::local_host();
    assert!(
        bridget_core::host_is_attested(&local_host),
        "la fixture exige une identité locale attestée, host={local_host:?}"
    );
    let (ready_tx, ready_rx) = mpsc::channel();
    let server = thread::spawn(move || {
        let listener = UnixListener::bind(&socket).unwrap();
        ready_tx.send(()).unwrap();
        let (stream, _) = listener.accept().unwrap();
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        assert_eq!(
            read_json_line(&mut reader),
            json!({"type":"RoleHandshake","role":"client"})
        );
        write_json_line(&mut writer, json!({"type":"RoleAccepted","role":"client"}));
        assert_eq!(read_json_line(&mut reader)["type"], "ClientHello");
        write_json_line(
            &mut writer,
            json!({"type":"ClientWelcome","version":1,"horizon_secs":60,"issued_at_tolerance_secs":5,"capabilities":["send_idempotent","lookup"]}),
        );
        assert_eq!(
            read_json_line(&mut reader),
            json!({"type":"DaemonIdentityRequest"})
        );
        write_json_line(
            &mut writer,
            json!({"type":"DaemonIdentityReport","host":local_host,"db_path":"/var/lib/bridget/bridget.db"}),
        );
    });
    ready_rx.recv().unwrap();
    server
}
fn read_json_line(r: &mut BufReader<UnixStream>) -> Value {
    let mut s = String::new();
    r.read_line(&mut s).unwrap();
    serde_json::from_str(&s).unwrap()
}
fn write_json_line(w: &mut BufWriter<UnixStream>, v: Value) {
    serde_json::to_writer(&mut *w, &v).unwrap();
    w.write_all(b"\n").unwrap();
    w.flush().unwrap();
}

fn fabricate_divergence(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let database = root.join("maicie.sqlite3");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/migration-v19/base-v19-empty.sqlite3"),
        &database,
    )
    .unwrap();
    fs::set_permissions(&database, fs::Permissions::from_mode(0o600)).unwrap();
    let connection = rusqlite::Connection::open(&database).unwrap();
    let versions: Vec<i64> = {
        let mut statement = connection
            .prepare("SELECT version FROM schema_migrations ORDER BY version")
            .unwrap();
        statement
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert_eq!(versions, (1..=FIXTURE_SCHEMA_VERSION).collect::<Vec<i64>>());
    let ddl: String = connection
        .query_row(
            "SELECT sql FROM sqlite_master WHERE type='table' AND name='guichet_receptions'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(
        ddl.contains("delivery_report")
            && ddl.contains("mission_status")
            && ddl.contains("deadline_question")
    );

    let install_bin = root.join("installed").join("maicie");
    fs::create_dir_all(install_bin.parent().unwrap()).unwrap();
    fs::write(
        &install_bin,
        "#!/bin/sh\necho '{\"error\":{\"code\":\"store\",\"message\":\"schema SQLite perime (oracle)\"}}' >&2\nexit 6\n",
    )
    .unwrap();
    fs::set_permissions(&install_bin, fs::Permissions::from_mode(0o755)).unwrap();

    let config = write_config(root, &database);
    (database, config, install_bin)
}

#[test]
fn migrate_cli_repare_la_divergence_du_binaire_installe() {
    let root = unique_root("nominal");
    let (database, config, install_bin) = fabricate_divergence(&root);
    let migrateur = PathBuf::from(env!("CARGO_BIN_EXE_maicie"));
    assert!(
        !files_equal(&install_bin, &migrateur),
        "précondition : installé ≠ migrateur"
    );

    let daemon = start_local_daemon_identity(&root.join("bridget.sock"));
    let output = Command::new(&migrateur)
        .args(["migrate", "--config", &config.display().to_string()])
        .env(INSTALL_BIN_ENV, &install_bin)
        .env("TMPDIR", std::env::temp_dir())
        .output()
        .expect("lancer maicie migrate");
    assert!(
        output.status.success(),
        "migrate doit réussir: status={:?} stderr={} stdout={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout),
    );
    daemon.join().unwrap();

    assert!(
        files_equal(&install_bin, &migrateur),
        "après migrate, {} doit être octet-pour-octet {}",
        install_bin.display(),
        migrateur.display()
    );

    let backups: Vec<_> = fs::read_dir(install_bin.parent().unwrap())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("maicie.avant-republish-"))
        })
        .collect();
    assert_eq!(
        backups.len(),
        1,
        "une sauvegarde de l'ancien installé est obligatoire"
    );

    let preflight = Command::new(&install_bin)
        .args([
            "preflight",
            "--config",
            &config.display().to_string(),
            "--json",
        ])
        .output()
        .expect("préflight du binaire installé");
    assert!(
        preflight.status.success(),
        "le binaire installé doit préflighter la base migrée: {}",
        String::from_utf8_lossy(&preflight.stderr)
    );
    assert_eq!(database_user_version(&database), SCHEMA_VERSION);
}

#[test]
fn echec_reel_republication_avant_migration_ne_laisse_pas_greffe_en_avance() {
    // Le répertoire installé réellement non inscriptible reproduit la panne
    // qui avait laissé le greffe en avance sur le binaire du PATH.
    let root = unique_root("echec-republish");
    let (database, config, install_bin) = fabricate_divergence(&root);
    let migrateur = PathBuf::from(env!("CARGO_BIN_EXE_maicie"));
    let install_avant = fs::read(&install_bin).unwrap();
    assert_eq!(database_user_version(&database), FIXTURE_SCHEMA_VERSION);

    let install_dir = install_bin.parent().unwrap();
    fs::set_permissions(install_dir, fs::Permissions::from_mode(0o500)).unwrap();

    let daemon = start_local_daemon_identity(&root.join("bridget.sock"));
    let output = Command::new(&migrateur)
        .args(["migrate", "--config", &config.display().to_string()])
        .env(INSTALL_BIN_ENV, &install_bin)
        .env("TMPDIR", std::env::temp_dir())
        .output()
        .expect("lancer maicie migrate");
    fs::set_permissions(install_dir, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(
        !output.status.success(),
        "migrate doit échouer quand la republication est refusée"
    );
    daemon.join().unwrap();
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Permission denied"),
        "l'échec doit venir des permissions réelles, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        database_user_version(&database),
        FIXTURE_SCHEMA_VERSION,
        "le greffe ne doit pas être migré si la republication échoue avant migrate"
    );
    assert_eq!(
        fs::read(&install_bin).unwrap(),
        install_avant,
        "l'installé ne doit pas changer quand la republication échoue avant migrate"
    );
    assert!(
        !files_equal(&install_bin, &migrateur),
        "pas de divergence inversée : greffe en avance, installé en retard"
    );
}

#[test]
fn lien_installe_est_remplace_par_un_fichier_independant_avant_migration() {
    let root = unique_root("lien-final");
    let (database, _config, install_bin) = fabricate_divergence(&root);
    let chantier = root.join("chantier");
    let source = chantier.join("maicie");
    write_executable(&source, "#!/bin/sh\necho SOURCE-INDEPENDANTE\n");
    fs::remove_file(&install_bin).unwrap();
    symlink(&source, &install_bin).unwrap();

    let outcome = republish_exe_before_migrate(&source, &install_bin, &database).unwrap();

    assert!(
        outcome.was_published(),
        "le lien ne doit jamais valoir installation"
    );
    let metadata = fs::symlink_metadata(&install_bin).unwrap();
    assert!(
        metadata.file_type().is_file(),
        "l'installé doit être régulier"
    );
    assert!(
        !metadata.file_type().is_symlink(),
        "l'installé ne doit plus être un lien"
    );
    assert_eq!(
        metadata.permissions().mode() & 0o777,
        0o755,
        "le mode exécutable publié est contractuel"
    );
    assert_eq!(
        database_user_version(&database),
        FIXTURE_SCHEMA_VERSION,
        "la primitive de publication ne doit pas migrer le greffe"
    );

    fs::remove_dir_all(&chantier).unwrap();
    let execution = Command::new(&install_bin)
        .output()
        .expect("l'installé doit survivre à la suppression du chantier");
    assert!(execution.status.success());
    assert_eq!(
        String::from_utf8_lossy(&execution.stdout).trim(),
        "SOURCE-INDEPENDANTE"
    );
    assert_eq!(database_user_version(&database), FIXTURE_SCHEMA_VERSION);
}

#[test]
fn lien_intermediaire_vers_le_chantier_est_refuse_sans_migrer() {
    let root = unique_root("lien-intermediaire");
    let (database, _config, install_bin) = fabricate_divergence(&root);
    let source = root.join("source-maicie");
    write_executable(&source, "#!/bin/sh\necho SOURCE\n");
    let install_dir = install_bin.parent().unwrap();
    let chantier_dir = root.join("chantier-bin");
    fs::create_dir_all(&chantier_dir).unwrap();
    fs::rename(&install_bin, chantier_dir.join("maicie")).unwrap();
    fs::remove_dir(install_dir).unwrap();
    symlink(&chantier_dir, install_dir).unwrap();
    let install_avant = fs::read(&install_bin).unwrap();

    let error = republish_exe_before_migrate(&source, &install_bin, &database)
        .expect_err("un parent lié ne peut pas porter une installation indépendante");

    assert!(
        error.to_string().contains("lien symbolique"),
        "refus explicite attendu, reçu={error}"
    );
    assert_eq!(fs::read(&install_bin).unwrap(), install_avant);
    assert_eq!(database_user_version(&database), FIXTURE_SCHEMA_VERSION);
    fs::remove_file(install_dir).unwrap();
}

#[test]
fn memes_octets_avec_mode_invalide_sont_republies() {
    let root = unique_root("mode");
    let (database, _config, install_bin) = fabricate_divergence(&root);
    let source = root.join("source-maicie");
    write_executable(&source, "#!/bin/sh\necho MODE\n");
    fs::write(&install_bin, fs::read(&source).unwrap()).unwrap();
    fs::set_permissions(&install_bin, fs::Permissions::from_mode(0o600)).unwrap();

    let outcome = republish_exe_before_migrate(&source, &install_bin, &database).unwrap();

    assert!(outcome.was_published(), "les octets seuls ne suffisent pas");
    let mode = fs::symlink_metadata(&install_bin)
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(mode, 0o755);
    assert_eq!(database_user_version(&database), FIXTURE_SCHEMA_VERSION);
}

#[test]
fn binaire_neuf_sur_greffe_ancien_refuse_parlant_puis_reprend_la_migration() {
    let root = unique_root("fenetre-symetrique");
    let (database, config, install_bin) = fabricate_divergence(&root);
    let migrateur = PathBuf::from(env!("CARGO_BIN_EXE_maicie"));

    let outcome = republish_exe_before_migrate(&migrateur, &install_bin, &database).unwrap();
    assert!(outcome.was_published());
    assert!(files_equal(&install_bin, &migrateur));
    assert_eq!(database_user_version(&database), FIXTURE_SCHEMA_VERSION);
    let daemon = start_local_daemon_identity(&root.join("bridget.sock"));

    let refusal = Command::new(&install_bin)
        .args(["status", "--config", &config.display().to_string()])
        .output()
        .expect("ouvrir le greffe ancien avec le binaire neuf");
    assert!(!refusal.status.success());
    let stderr = String::from_utf8_lossy(&refusal.stderr);
    daemon.join().unwrap();
    let _ = fs::remove_file(root.join("bridget.sock"));
    assert!(
        stderr.contains("antérieur au binaire")
            && stderr.contains("relancer avec : maicie migrate --config"),
        "le refus doit rendre la reprise lisible, stderr={stderr}"
    );
    assert_eq!(database_user_version(&database), FIXTURE_SCHEMA_VERSION);

    let daemon = start_local_daemon_identity(&root.join("bridget.sock"));
    let reprise = Command::new(&install_bin)
        .args(["migrate", "--config", &config.display().to_string()])
        .env(INSTALL_BIN_ENV, &install_bin)
        .output()
        .expect("reprendre la migration avec le binaire installé");
    assert!(
        reprise.status.success(),
        "la fenêtre symétrique doit être récupérable: {}",
        String::from_utf8_lossy(&reprise.stderr)
    );
    daemon.join().unwrap();
    assert_eq!(database_user_version(&database), SCHEMA_VERSION);
    assert!(files_equal(&install_bin, &migrateur));
}

#[test]
fn echec_migration_et_restauration_est_signale_sans_masquer_les_deux_etats() {
    let root = unique_root("echec-restauration");
    let (database, config, install_bin) = fabricate_divergence(&root);
    let migrateur = PathBuf::from(env!("CARGO_BIN_EXE_maicie"));
    let install_avant = fs::read(&install_bin).unwrap();
    let install_dir = install_bin.parent().unwrap();
    let verrou = rusqlite::Connection::open(&database).unwrap();
    verrou.execute_batch("BEGIN IMMEDIATE").unwrap();

    let daemon = start_local_daemon_identity(&root.join("bridget.sock"));
    let child = Command::new(&migrateur)
        .args(["migrate", "--config", &config.display().to_string()])
        .env(INSTALL_BIN_ENV, &install_bin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("lancer migrate sous verrou SQLite");

    let deadline = Instant::now() + Duration::from_secs(2);
    while !files_equal(&install_bin, &migrateur) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        files_equal(&install_bin, &migrateur),
        "précondition : le neuf doit être publié avant le blocage de migration"
    );
    fs::set_permissions(install_dir, fs::Permissions::from_mode(0o500)).unwrap();

    let output = child.wait_with_output().expect("attendre l'échec réel");
    daemon.join().unwrap();
    fs::set_permissions(install_dir, fs::Permissions::from_mode(0o700)).unwrap();
    verrou.execute_batch("ROLLBACK").unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr)
            .contains("restauration de la sauvegarde du binaire installé impossible"),
        "l'échec de restauration ne doit pas être masqué par l'échec SQLite: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        database_user_version(&database),
        FIXTURE_SCHEMA_VERSION,
        "le verrou doit laisser le greffe à son ancienne version"
    );
    assert_ne!(
        fs::read(&install_bin).unwrap(),
        install_avant,
        "la restauration réellement refusée laisse le neuf installé"
    );
    assert!(files_equal(&install_bin, &migrateur));
}

#[test]
fn erreur_identite_apres_commit_conserve_le_binaire_neuf() {
    let root = unique_root("identite-apres-commit");
    let (database, config, install_bin) = fabricate_divergence(&root);
    let migrateur = PathBuf::from(env!("CARGO_BIN_EXE_maicie"));
    let ancien = fs::read(&install_bin).unwrap();
    let connection = rusqlite::Connection::open(&database).unwrap();
    connection
        .execute_batch(
            "DELETE FROM maicie_identity;
             CREATE TRIGGER refuse_identite_apres_commit
             BEFORE INSERT ON maicie_identity
             BEGIN SELECT RAISE(ABORT, 'identite post-commit refusee'); END;",
        )
        .unwrap();
    drop(connection);

    let daemon = start_local_daemon_identity(&root.join("bridget.sock"));
    let output = Command::new(&migrateur)
        .args(["migrate", "--config", &config.display().to_string()])
        .env(INSTALL_BIN_ENV, &install_bin)
        .env("TMPDIR", std::env::temp_dir())
        .output()
        .expect("lancer migrate jusqu'à l'identité post-commit");

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("identite post-commit refusee"),
        "l'échec doit naître dans la transaction d'identité: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    daemon.join().unwrap();
    assert_eq!(
        database_user_version(&database),
        SCHEMA_VERSION,
        "la migration doit être durable avant l'échec d'identité"
    );
    assert_ne!(fs::read(&install_bin).unwrap(), ancien);
    assert!(
        files_equal(&install_bin, &migrateur),
        "un échec après commit ne doit jamais ressusciter l'ancien installé"
    );
}

#[test]
fn relecture_durable_impossible_conserve_le_binaire_neuf() {
    let root = unique_root("relecture-impossible");
    let (database, _config, install_bin) = fabricate_divergence(&root);
    let migrateur = PathBuf::from(env!("CARGO_BIN_EXE_maicie"));
    let ancien = fs::read(&install_bin).unwrap();
    let publication = republish_exe_before_migrate(&migrateur, &install_bin, &database).unwrap();
    let sqlite_error = rusqlite::Connection::open_in_memory()
        .unwrap()
        .execute("SELECT * FROM table_absente", [])
        .unwrap_err();
    let migration_error = StoreError::Sql(sqlite_error);
    fs::set_permissions(&database, fs::Permissions::from_mode(0o000)).unwrap();

    let error = reconcile_failed_migration(publication, &migration_error)
        .expect_err("une relecture réellement impossible doit rester explicite");

    fs::set_permissions(&database, fs::Permissions::from_mode(0o600)).unwrap();
    assert!(matches!(
        error,
        InstallPublishError::MigrationStateUnknown { .. }
    ));
    let message = error.to_string();
    assert!(message.contains("état durable du schéma illisible"));
    assert!(message.contains("installé neuf conservé par sûreté"));
    assert_eq!(database_user_version(&database), FIXTURE_SCHEMA_VERSION);
    assert_ne!(fs::read(&install_bin).unwrap(), ancien);
    assert!(files_equal(&install_bin, &migrateur));
}

#[test]
fn exclusion_commune_interdit_commit_19_pendant_restauration_ancien() {
    let root = unique_root("toctou-fifo");
    let (database, config, install_bin) = fabricate_divergence(&root);
    let migrateur = PathBuf::from(env!("CARGO_BIN_EXE_maicie"));
    let ancien = fs::read(&install_bin).unwrap();

    // Même montage que la contre-épreuve du STOP : deux preuves valides du
    // même neuf existent avant que la première migration échoue.
    let premiere = republish_exe_before_migrate(&migrateur, &install_bin, &database).unwrap();
    let seconde = republish_exe_before_migrate(&migrateur, &install_bin, &database).unwrap();
    let backup = fs::read_dir(install_bin.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .contains("avant-republish")
        })
        .unwrap();
    fs::remove_file(&backup).unwrap();
    let fifo = CString::new(backup.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);

    let verrou_sqlite = rusqlite::Connection::open(&database).unwrap();
    verrou_sqlite.execute_batch("BEGIN IMMEDIATE").unwrap();
    let premier = thread::spawn(move || match premiere.open() {
        Err(maicie::install_publish::PublishedMigrationOpenError::Store(error)) => {
            error.to_string()
        }
        Err(error) => panic!("mauvaise erreur de première ouverture: {error:?}"),
        Ok(_) => panic!("la première migration devait rencontrer le verrou SQLite"),
    });

    // L'ouverture non bloquante du writer ne réussit que lorsque la première
    // réconciliation a lu v18 et attend réellement les octets de sauvegarde.
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut writer = loop {
        match OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(&backup)
        {
            Ok(file) => break file,
            Err(error)
                if error.raw_os_error() == Some(libc::ENXIO) && Instant::now() < deadline =>
            {
                thread::sleep(Duration::from_millis(10));
            }
            Err(error) => panic!("la restauration n'a pas atteint la FIFO: {error}"),
        }
    };

    verrou_sqlite.execute_batch("ROLLBACK").unwrap();
    let republication_concurrente =
        republish_exe_and_preflight(&migrateur, &install_bin, &config).unwrap_err();
    assert!(
        matches!(
            republication_concurrente,
            InstallPublishError::MigrationAlreadyInProgress
        ),
        "toute écriture concurrente de l'installé doit partager l'exclusion, reçu={republication_concurrente}"
    );
    let seconde_erreur = match seconde.open() {
        Err(maicie::install_publish::PublishedMigrationOpenError::Install(error)) => error,
        Err(error) => panic!("mauvaise erreur de seconde ouverture: {error:?}"),
        Ok(_) => panic!("un second migrateur ne doit pas franchir l'exclusion commune"),
    };
    assert!(
        matches!(
            seconde_erreur,
            InstallPublishError::MigrationAlreadyInProgress
        ),
        "le concurrent doit être refusé par le verrou commun, reçu={seconde_erreur}"
    );
    assert_eq!(
        database_user_version(&database),
        FIXTURE_SCHEMA_VERSION,
        "aucun commit ne peut s'intercaler pendant la restauration"
    );
    assert!(
        files_equal(&install_bin, &migrateur),
        "le refus concurrent doit laisser le binaire neuf installé"
    );

    writer.write_all(&ancien).unwrap();
    drop(writer);
    let premiere_erreur = premier.join().unwrap();
    assert!(premiere_erreur.contains("locked"), "{premiere_erreur}");
    assert_eq!(database_user_version(&database), FIXTURE_SCHEMA_VERSION);
    assert_eq!(fs::read(&install_bin).unwrap(), ancien);

    // Contrôle positif : après libération de l'exclusion, une nouvelle preuve
    // republie le neuf et la reprise normale peut bien committer v19.
    fs::remove_file(&backup).unwrap();
    let reprise = republish_exe_before_migrate(&migrateur, &install_bin, &database).unwrap();
    let migrated = reprise.open().unwrap();
    assert_eq!(database_user_version(&database), SCHEMA_VERSION);
    assert!(files_equal(&install_bin, &migrateur));
    drop(migrated);
}

#[test]
fn preuve_invalidee_avant_consommation_ne_peut_pas_migrer() {
    let root = unique_root("preuve-invalidee");
    let (database, _config, install_bin) = fabricate_divergence(&root);
    let migrateur = PathBuf::from(env!("CARGO_BIN_EXE_maicie"));
    let ancien = fs::read(&install_bin).unwrap();
    let publication = republish_exe_before_migrate(&migrateur, &install_bin, &database).unwrap();
    fs::write(&install_bin, &ancien).unwrap();
    fs::set_permissions(&install_bin, fs::Permissions::from_mode(0o755)).unwrap();

    let error = match publication.open() {
        Ok(_) => panic!("une preuve dont l'installé a changé ne doit plus ouvrir le greffe"),
        Err(error) => error,
    };

    assert!(matches!(
        error,
        maicie::install_publish::PublishedMigrationOpenError::Install(
            InstallPublishError::PublishedMismatch
        )
    ));
    assert_eq!(database_user_version(&database), FIXTURE_SCHEMA_VERSION);
    assert_eq!(fs::read(&install_bin).unwrap(), ancien);
}

#[test]
fn preuve_de_republication_lie_le_consentement_au_greffe() {
    let root = unique_root("preuve-republish");
    let (database, config, install_bin) = fabricate_divergence(&root);
    let migrateur = PathBuf::from(env!("CARGO_BIN_EXE_maicie"));
    let publication = republish_exe_before_migrate(&migrateur, &install_bin, &database).unwrap();

    let store = publication
        .open()
        .unwrap()
        .verify_preflight(&config)
        .unwrap();
    assert_eq!(store.schema_version().unwrap(), SCHEMA_VERSION);
    drop(store);

    assert_eq!(database_user_version(&database), SCHEMA_VERSION);
    assert!(files_equal(&install_bin, &migrateur));
}
