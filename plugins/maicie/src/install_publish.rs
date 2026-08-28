//! Republication atomique du binaire Maicie vers le chemin installé.
//!
//! Propriété visée : jamais de greffe migré avec un binaire installé périmé.
//! La republication précède donc la migration. Après une erreur englobante,
//! l'ancien installé n'est restauré que si une nouvelle connexion atteste que
//! le schéma durable est encore antérieur ; l'incertitude conserve le neuf.
//! Le préflight CLI strict n'a lieu qu'après migration réussie — avant migrate,
//! la base peut encore être en N-1.
//!
//! Modèle : `scripts/install-k1.sh` (`atomic_publish_file` puis préflight de
//! la paire publiée). Toute publication qui échoue au préflight restaure la
//! sauvegarde — la lisibilité quotidienne du registre n'est pas un collatéral.

use crate::store::{SCHEMA_VERSION, StoreError};
use rusqlite::{Connection, OpenFlags};
use std::fs::{self, DirBuilder, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// Variable d'environnement qui force le chemin installé (tests, staging).
pub const INSTALL_BIN_ENV: &str = "MAICIE_INSTALL_BIN";

const INSTALLED_MODE: u32 = 0o755;
const MIGRATION_LOCK_MODE: u32 = 0o600;
const PRIVATE_DIRECTORY_MODE: u32 = 0o700;

/// Frontières observables du remplacement durable. L'observateur est réservé
/// aux crash-tests ; la production emprunte exactement le même chemin avec un
/// observateur vide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InstallPublishPhase {
    TemporaryFileSynced,
    DestinationRenamed,
    DestinationRemoved,
    ParentDirectorySynced,
}

#[derive(Debug)]
pub enum InstallPublishError {
    Io(io::Error),
    CurrentExe(io::Error),
    HomeAbsent,
    DatabaseNewerThanBinary {
        found: i64,
        supported: i64,
    },
    InstallPathContainsSymlink,
    InstallPathInvalid,
    PreflightFailed {
        status: Option<i32>,
        stderr: String,
    },
    PublishedMismatch,
    MigrationAlreadyInProgress,
    MigrationLockInvalid,
    BackupRestore(io::Error),
    MigrationRestoreFailed {
        migration: String,
        restoration: String,
    },
    MigrationStateUnknown {
        migration: String,
        inspection: String,
    },
}

impl std::fmt::Display for InstallPublishError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "republication installée : {error}"),
            Self::CurrentExe(error) => {
                write!(formatter, "binaire courant illisible : {error}")
            }
            Self::HomeAbsent => write!(
                formatter,
                "HOME absent : impossible de résoudre ~/.local/bin/maicie"
            ),
            Self::DatabaseNewerThanBinary { found, supported } => write!(
                formatter,
                "schéma SQLite {found} plus récent que le binaire (maximum {supported}) : migration impossible avant republication"
            ),
            Self::InstallPathContainsSymlink => write!(
                formatter,
                "le chemin installé contient un lien symbolique intermédiaire"
            ),
            Self::InstallPathInvalid => write!(
                formatter,
                "le chemin installé n'est pas une destination régulière indépendante"
            ),
            Self::PreflightFailed { status, stderr } => write!(
                formatter,
                "préflight de la paire publiée refusé (status={status:?}) : {stderr}"
            ),
            Self::PublishedMismatch => write!(
                formatter,
                "binaire publié différent du migrateur après publication atomique"
            ),
            Self::MigrationAlreadyInProgress => write!(
                formatter,
                "republication ou migration Maicie déjà en cours pour ce greffe"
            ),
            Self::MigrationLockInvalid => write!(
                formatter,
                "verrou de republication et migration Maicie invalide"
            ),
            Self::BackupRestore(error) => write!(
                formatter,
                "restauration de la sauvegarde du binaire installé impossible : {error}"
            ),
            Self::MigrationRestoreFailed {
                migration,
                restoration,
            } => write!(
                formatter,
                "migration échouée ({migration}) ; restauration de l'ancien binaire impossible ({restoration})"
            ),
            Self::MigrationStateUnknown {
                migration,
                inspection,
            } => write!(
                formatter,
                "migration échouée ({migration}) ; état durable du schéma illisible ({inspection}) : installé neuf conservé par sûreté, aucune restauration"
            ),
        }
    }
}

impl std::error::Error for InstallPublishError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishOutcome {
    pub install_bin: PathBuf,
    pub source_exe: PathBuf,
    pub published: bool,
    pub backup: Option<PathBuf>,
}

/// Preuve qu'un binaire indépendant a été publié pour cette base avant tout
/// consentement de migration. Ses champs privés empêchent un appelant de la
/// fabriquer sans passer par la republication réelle.
#[derive(Debug)]
pub struct PublishedMigration {
    outcome: PublishOutcome,
    database_path: PathBuf,
}

impl PublishedMigration {
    pub(crate) fn database_path(&self) -> &Path {
        &self.database_path
    }

    pub fn was_published(&self) -> bool {
        self.outcome.published
    }

    /// Ouvre et migre exclusivement la base liée à cette publication. La
    /// preuve est consommée : aucun appelant ne peut restaurer ensuite
    /// l'ancien binaire tout en conservant le droit de migrer.
    ///
    /// ```compile_fail
    /// use maicie::store::MaicieStore;
    /// let path = std::path::Path::new("/tmp/maicie.sqlite3");
    /// let _ = MaicieStore::open_and_migrate(path);
    /// ```
    pub fn open(self) -> Result<PublishedMigratedStore, PublishedMigrationOpenError> {
        let exclusion =
            MigrationExclusion::acquire(self.database_path(), &self.outcome.install_bin)
                .map_err(PublishedMigrationOpenError::Install)?;
        let installed_is_still_published =
            installed_matches_source(&self.outcome.source_exe, &self.outcome.install_bin)
                .map_err(PublishedMigrationOpenError::Install)?;
        if !installed_is_still_published {
            return Err(PublishedMigrationOpenError::Install(
                InstallPublishError::PublishedMismatch,
            ));
        }
        let store = match crate::store::MaicieStore::open_after_publication(&self, &exclusion) {
            Ok(store) => store,
            Err(error) => {
                reconcile_failed_migration_under_exclusion(self, &error, &exclusion)
                    .map_err(PublishedMigrationOpenError::Install)?;
                return Err(PublishedMigrationOpenError::Store(error));
            }
        };
        Ok(PublishedMigratedStore {
            store,
            publication: self,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailedMigrationRecovery {
    RestoredPreviousInstall,
    KeptPublishedInstall { durable_schema_version: i64 },
}

#[derive(Debug)]
pub enum PublishedMigrationOpenError {
    Store(StoreError),
    Install(InstallPublishError),
}

/// Store migré dont la publication correspondante reste portée jusqu'au
/// préflight. Le champ privé empêche de fabriquer cet état depuis le binaire.
pub struct PublishedMigratedStore {
    store: crate::store::MaicieStore,
    publication: PublishedMigration,
}

impl PublishedMigratedStore {
    pub fn verify_preflight(
        self,
        config_path: &Path,
    ) -> Result<crate::store::MaicieStore, InstallPublishError> {
        verify_published_preflight_after_migrate(&self.publication.outcome, config_path)?;
        Ok(self.store)
    }
}

/// Chemin du binaire installé : `MAICIE_INSTALL_BIN` ou `~/.local/bin/maicie`.
pub fn install_bin_path() -> Result<PathBuf, InstallPublishError> {
    if let Some(override_path) = std::env::var_os(INSTALL_BIN_ENV) {
        return Ok(PathBuf::from(override_path));
    }
    let home = std::env::var_os("HOME").ok_or(InstallPublishError::HomeAbsent)?;
    Ok(PathBuf::from(home).join(".local/bin/maicie"))
}

/// Republie `std::env::current_exe()` vers le chemin installé, puis préflighte.
pub fn republish_current_exe_and_preflight(
    config_path: &Path,
) -> Result<PublishOutcome, InstallPublishError> {
    let source = std::env::current_exe().map_err(InstallPublishError::CurrentExe)?;
    let install_bin = install_bin_path()?;
    republish_exe_and_preflight(&source, &install_bin, config_path)
}

/// Republie le migrateur **avant** migration consentie : pas de préflight CLI
/// strict (la base peut encore être en schéma N-1). Refuse seulement une base
/// plus récente que le binaire — migrer dans ce cas serait incohérent.
pub fn republish_current_exe_before_migrate(
    database_path: &Path,
) -> Result<PublishedMigration, InstallPublishError> {
    let source = std::env::current_exe().map_err(InstallPublishError::CurrentExe)?;
    let install_bin = install_bin_path()?;
    republish_exe_before_migrate(&source, &install_bin, database_path)
}

/// Cœur testable : republication pré-migrate sans préflight CLI strict.
/// Complexité : O(n), n étant la taille du binaire ; mémoire O(1).
pub fn republish_exe_before_migrate(
    source_exe: &Path,
    install_bin: &Path,
    database_path: &Path,
) -> Result<PublishedMigration, InstallPublishError> {
    republish_exe_before_migrate_with_publisher(
        source_exe,
        install_bin,
        database_path,
        atomic_publish_file,
    )
}

fn republish_exe_before_migrate_with_publisher(
    source_exe: &Path,
    install_bin: &Path,
    database_path: &Path,
    mut publish: impl FnMut(&Path, &Path) -> Result<(), InstallPublishError>,
) -> Result<PublishedMigration, InstallPublishError> {
    let _exclusion = MigrationExclusion::acquire(database_path, install_bin)?;
    assert_database_not_newer_than_binary(database_path)?;

    let source_exe = fs::canonicalize(source_exe).unwrap_or_else(|_| source_exe.to_path_buf());

    if installed_matches_source(&source_exe, install_bin)? {
        return Ok(PublishedMigration {
            outcome: PublishOutcome {
                install_bin: install_bin.to_path_buf(),
                source_exe,
                published: false,
                backup: None,
            },
            database_path: database_path.to_path_buf(),
        });
    }

    let backup = backup_if_readable(install_bin)?;

    publish(&source_exe, install_bin)?;

    if !installed_matches_source(&source_exe, install_bin)? {
        restore_backup_real(install_bin, backup.as_deref())?;
        return Err(InstallPublishError::PublishedMismatch);
    }

    Ok(PublishedMigration {
        outcome: PublishOutcome {
            install_bin: install_bin.to_path_buf(),
            source_exe,
            published: true,
            backup,
        },
        database_path: database_path.to_path_buf(),
    })
}

/// Préflight strict de la paire publiée après migration réussie.
pub fn verify_published_preflight_after_migrate(
    outcome: &PublishOutcome,
    config_path: &Path,
) -> Result<(), InstallPublishError> {
    run_published_preflight(&outcome.install_bin, config_path)
}

/// Restaure la sauvegarde de l'installé après échec de migration post-republish.
fn restore_publish_outcome_with_syncs(
    outcome: &PublishOutcome,
    sync_temporary_file: &mut impl FnMut(&File) -> io::Result<()>,
    sync_renamed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
    sync_removed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
) -> Result<(), InstallPublishError> {
    if !outcome.published {
        return Ok(());
    }
    restore_backup(
        &outcome.install_bin,
        outcome.backup.as_deref(),
        sync_temporary_file,
        sync_renamed_parent,
        sync_removed_parent,
    )
}

/// Après une erreur englobante d'ouverture, la restauration dépend du fait
/// durable et non de `Err` : seul un schéma encore antérieur autorise le retour
/// à l'ancien binaire. Si la relecture échoue, conserver le neuf est l'état
/// fail-closed déjà mesuré par le CLI (il refuse un schéma ancien avant écrit).
pub fn reconcile_failed_migration(
    publication: PublishedMigration,
    migration_error: &StoreError,
) -> Result<FailedMigrationRecovery, InstallPublishError> {
    let mut sync_temporary_file = File::sync_all;
    let mut sync_renamed_parent = sync_parent_directory;
    let mut sync_removed_parent = sync_parent_directory;
    reconcile_failed_migration_with_syncs(
        publication,
        migration_error,
        &mut sync_temporary_file,
        &mut sync_renamed_parent,
        &mut sync_removed_parent,
    )
}

fn reconcile_failed_migration_with_syncs(
    publication: PublishedMigration,
    migration_error: &StoreError,
    sync_temporary_file: &mut impl FnMut(&File) -> io::Result<()>,
    sync_renamed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
    sync_removed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
) -> Result<FailedMigrationRecovery, InstallPublishError> {
    let exclusion = MigrationExclusion::acquire(
        publication.database_path(),
        &publication.outcome.install_bin,
    )?;
    reconcile_failed_migration_under_exclusion_with_syncs(
        publication,
        migration_error,
        &exclusion,
        sync_temporary_file,
        sync_renamed_parent,
        sync_removed_parent,
    )
}

fn reconcile_failed_migration_under_exclusion(
    publication: PublishedMigration,
    migration_error: &StoreError,
    _exclusion: &MigrationExclusion,
) -> Result<FailedMigrationRecovery, InstallPublishError> {
    let mut sync_temporary_file = File::sync_all;
    let mut sync_renamed_parent = sync_parent_directory;
    let mut sync_removed_parent = sync_parent_directory;
    reconcile_failed_migration_under_exclusion_with_syncs(
        publication,
        migration_error,
        _exclusion,
        &mut sync_temporary_file,
        &mut sync_renamed_parent,
        &mut sync_removed_parent,
    )
}

fn reconcile_failed_migration_under_exclusion_with_syncs(
    publication: PublishedMigration,
    migration_error: &StoreError,
    _exclusion: &MigrationExclusion,
    sync_temporary_file: &mut impl FnMut(&File) -> io::Result<()>,
    sync_renamed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
    sync_removed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
) -> Result<FailedMigrationRecovery, InstallPublishError> {
    let durable_schema_version =
        read_durable_schema_version(publication.database_path()).map_err(|inspection| {
            InstallPublishError::MigrationStateUnknown {
                migration: migration_error.to_string(),
                inspection: inspection.to_string(),
            }
        })?;
    if durable_schema_version >= SCHEMA_VERSION {
        return Ok(FailedMigrationRecovery::KeptPublishedInstall {
            durable_schema_version,
        });
    }
    restore_publish_outcome_with_syncs(
        &publication.outcome,
        sync_temporary_file,
        sync_renamed_parent,
        sync_removed_parent,
    )
    .map_err(|restoration| InstallPublishError::MigrationRestoreFailed {
        migration: migration_error.to_string(),
        restoration: restoration.to_string(),
    })?;
    Ok(FailedMigrationRecovery::RestoredPreviousInstall)
}

/// Exclusion commune aux écritures de l'installé et au consentement de
/// migration. Les fichiers ne sont jamais supprimés : délier un verrou encore
/// tenu permettrait à un autre processus de verrouiller un nouvel inode aux
/// mêmes chemins et recréerait précisément la course que ce type ferme.
pub(crate) struct MigrationExclusion {
    _database: AdvisoryLock,
    _install: AdvisoryLock,
}

impl MigrationExclusion {
    fn acquire(database_path: &Path, install_bin: &Path) -> Result<Self, InstallPublishError> {
        // Ordre global obligatoire : greffe, puis installé. La republication
        // seule ne prend que le second ; aucun chemin ne les prend à l'envers.
        let database = AdvisoryLock::acquire(&migration_lock_path(database_path)?)?;
        let install = AdvisoryLock::acquire(&install_lock_path(install_bin)?)?;
        Ok(Self {
            _database: database,
            _install: install,
        })
    }
}

struct InstallExclusion {
    _install: AdvisoryLock,
}

impl InstallExclusion {
    fn acquire(install_bin: &Path) -> Result<Self, InstallPublishError> {
        Ok(Self {
            _install: AdvisoryLock::acquire(&install_lock_path(install_bin)?)?,
        })
    }
}

struct AdvisoryLock {
    file: File,
}

impl AdvisoryLock {
    fn acquire(lock_path: &Path) -> Result<Self, InstallPublishError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .mode(MIGRATION_LOCK_MODE)
            .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
            .open(lock_path)
            .map_err(InstallPublishError::Io)?;
        let metadata = file.metadata().map_err(InstallPublishError::Io)?;
        if !metadata.file_type().is_file()
            || metadata.permissions().mode() & 0o777 != MIGRATION_LOCK_MODE
        {
            return Err(InstallPublishError::MigrationLockInvalid);
        }

        loop {
            let result = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) };
            if result == 0 {
                return Ok(Self { file });
            }
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            if error.kind() == io::ErrorKind::WouldBlock {
                return Err(InstallPublishError::MigrationAlreadyInProgress);
            }
            return Err(InstallPublishError::Io(error));
        }
    }
}

impl Drop for AdvisoryLock {
    fn drop(&mut self) {
        let _ = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

fn migration_lock_path(database_path: &Path) -> Result<PathBuf, InstallPublishError> {
    if !database_path.is_absolute() {
        return Err(InstallPublishError::MigrationLockInvalid);
    }
    let parent = database_path
        .parent()
        .ok_or(InstallPublishError::MigrationLockInvalid)?;
    let database_name = database_path
        .file_name()
        .ok_or(InstallPublishError::MigrationLockInvalid)?;
    if !parent.exists() {
        DirBuilder::new()
            .recursive(true)
            .mode(PRIVATE_DIRECTORY_MODE)
            .create(parent)
            .map_err(InstallPublishError::Io)?;
    }
    let parent_metadata = fs::symlink_metadata(parent).map_err(InstallPublishError::Io)?;
    if !parent_metadata.file_type().is_dir()
        || parent_metadata.permissions().mode() & 0o777 != PRIVATE_DIRECTORY_MODE
    {
        return Err(InstallPublishError::MigrationLockInvalid);
    }
    let canonical_parent = fs::canonicalize(parent).map_err(InstallPublishError::Io)?;
    let mut lock_name = database_name.to_os_string();
    lock_name.push(".migration.lock");
    Ok(canonical_parent.join(lock_name))
}

fn install_lock_path(install_bin: &Path) -> Result<PathBuf, InstallPublishError> {
    let absolute_install = validate_install_parent(install_bin)?;
    let parent = absolute_install
        .parent()
        .ok_or(InstallPublishError::InstallPathInvalid)?;
    fs::create_dir_all(parent).map_err(InstallPublishError::Io)?;
    let absolute_install = validate_install_parent(&absolute_install)?;
    let canonical_parent = fs::canonicalize(
        absolute_install
            .parent()
            .ok_or(InstallPublishError::InstallPathInvalid)?,
    )
    .map_err(InstallPublishError::Io)?;
    let mut lock_name = absolute_install
        .file_name()
        .ok_or(InstallPublishError::InstallPathInvalid)?
        .to_os_string();
    lock_name.push(".install.lock");
    Ok(canonical_parent.join(lock_name))
}

fn read_durable_schema_version(database_path: &Path) -> Result<i64, InstallPublishError> {
    let connection = Connection::open_with_flags(database_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| InstallPublishError::Io(io::Error::other(error)))?;
    connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| InstallPublishError::Io(io::Error::other(error)))
}

fn assert_database_not_newer_than_binary(database_path: &Path) -> Result<(), InstallPublishError> {
    if !database_path.exists() {
        return Ok(());
    }
    let connection = Connection::open(database_path)
        .map_err(|error| InstallPublishError::Io(io::Error::other(error)))?;
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| InstallPublishError::Io(io::Error::other(error)))?;
    if version > SCHEMA_VERSION {
        return Err(InstallPublishError::DatabaseNewerThanBinary {
            found: version,
            supported: SCHEMA_VERSION,
        });
    }
    Ok(())
}

/// Cœur testable : source et destination explicites (jamais un `which`).
/// Complexité : O(n), n étant la taille du binaire ; mémoire O(1).
pub fn republish_exe_and_preflight(
    source_exe: &Path,
    install_bin: &Path,
    config_path: &Path,
) -> Result<PublishOutcome, InstallPublishError> {
    let _exclusion = InstallExclusion::acquire(install_bin)?;
    let source_exe = fs::canonicalize(source_exe).unwrap_or_else(|_| source_exe.to_path_buf());

    if installed_matches_source(&source_exe, install_bin)? {
        run_published_preflight(install_bin, config_path)?;
        return Ok(PublishOutcome {
            install_bin: install_bin.to_path_buf(),
            source_exe,
            published: false,
            backup: None,
        });
    }

    let backup = backup_if_readable(install_bin)?;

    // La publication atomique retire son temporaire en cas d'échec ; la
    // sauvegarde reste intacte pour l'humain.
    atomic_publish_file(&source_exe, install_bin)?;

    if !installed_matches_source(&source_exe, install_bin)? {
        restore_backup_real(install_bin, backup.as_deref())?;
        return Err(InstallPublishError::PublishedMismatch);
    }

    if let Err(error) = run_published_preflight(install_bin, config_path) {
        restore_backup_real(install_bin, backup.as_deref())?;
        return Err(error);
    }

    Ok(PublishOutcome {
        install_bin: install_bin.to_path_buf(),
        source_exe,
        published: true,
        backup,
    })
}

/// Une installation valide n'est pas seulement un flux d'octets identique :
/// son chemin doit désigner directement un fichier régulier exécutable. Deux
/// chemins distincts ne peuvent pas partager le même inode, sinon le chantier
/// resterait capable de modifier l'installé après la publication.
fn installed_matches_source(
    source_exe: &Path,
    install_bin: &Path,
) -> Result<bool, InstallPublishError> {
    let absolute_install = validate_install_parent(install_bin)?;
    let metadata = match fs::symlink_metadata(&absolute_install) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(InstallPublishError::Io(error)),
    };
    if !metadata.file_type().is_file() || metadata.permissions().mode() & 0o777 != INSTALLED_MODE {
        return Ok(false);
    }
    if !files_equal(source_exe, &absolute_install)? {
        return Ok(false);
    }

    let source_metadata = fs::metadata(source_exe).map_err(InstallPublishError::Io)?;
    let same_location = source_exe == absolute_install;
    let independent_inode =
        source_metadata.dev() != metadata.dev() || source_metadata.ino() != metadata.ino();
    Ok(same_location || independent_inode)
}

fn validate_install_parent(install_bin: &Path) -> Result<PathBuf, InstallPublishError> {
    if install_bin
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(InstallPublishError::InstallPathInvalid);
    }
    let absolute = if install_bin.is_absolute() {
        install_bin.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(InstallPublishError::Io)?
            .join(install_bin)
    };
    let parent = absolute
        .parent()
        .ok_or(InstallPublishError::InstallPathInvalid)?;
    for ancestor in parent.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(InstallPublishError::InstallPathContainsSymlink);
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(InstallPublishError::InstallPathInvalid);
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(InstallPublishError::Io(error)),
        }
    }
    Ok(absolute)
}

fn backup_if_readable(install_bin: &Path) -> Result<Option<PathBuf>, InstallPublishError> {
    match fs::metadata(install_bin) {
        Ok(metadata) if metadata.is_file() => backup_existing(install_bin).map(Some),
        Ok(_) => Err(InstallPublishError::InstallPathInvalid),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(InstallPublishError::Io(error)),
    }
}

fn files_equal(left: &Path, right: &Path) -> Result<bool, InstallPublishError> {
    let left_meta = fs::metadata(left).map_err(InstallPublishError::Io)?;
    let right_meta = fs::metadata(right).map_err(InstallPublishError::Io)?;
    if left_meta.len() != right_meta.len() {
        return Ok(false);
    }
    let mut left_file = File::open(left).map_err(InstallPublishError::Io)?;
    let mut right_file = File::open(right).map_err(InstallPublishError::Io)?;
    let mut left_buf = [0_u8; 64 * 1024];
    let mut right_buf = [0_u8; 64 * 1024];
    loop {
        let left_n = left_file
            .read(&mut left_buf)
            .map_err(InstallPublishError::Io)?;
        let right_n = right_file
            .read(&mut right_buf)
            .map_err(InstallPublishError::Io)?;
        if left_n != right_n || left_buf[..left_n] != right_buf[..right_n] {
            return Ok(false);
        }
        if left_n == 0 {
            return Ok(true);
        }
    }
}

fn backup_existing(install_bin: &Path) -> Result<PathBuf, InstallPublishError> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    let backup = install_bin.with_file_name(format!(
        "{}.avant-republish-{stamp}",
        install_bin
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("maicie")
    ));
    fs::copy(install_bin, &backup).map_err(InstallPublishError::Io)?;
    let permissions = fs::Permissions::from_mode(INSTALLED_MODE);
    fs::set_permissions(&backup, permissions).map_err(InstallPublishError::Io)?;
    Ok(backup)
}

fn restore_backup_real(
    install_bin: &Path,
    backup: Option<&Path>,
) -> Result<(), InstallPublishError> {
    restore_backup_observed(install_bin, backup, |_| Ok(()))
}

fn restore_backup(
    install_bin: &Path,
    backup: Option<&Path>,
    sync_temporary_file: &mut impl FnMut(&File) -> io::Result<()>,
    sync_renamed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
    sync_removed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
) -> Result<(), InstallPublishError> {
    restore_backup_observed_with_syncs(
        install_bin,
        backup,
        |_| Ok(()),
        sync_temporary_file,
        sync_renamed_parent,
        sync_removed_parent,
    )
}

fn restore_backup_observed(
    install_bin: &Path,
    backup: Option<&Path>,
    observer: impl FnMut(InstallPublishPhase) -> io::Result<()>,
) -> Result<(), InstallPublishError> {
    let mut sync_temporary_file = File::sync_all;
    let mut sync_renamed_parent = sync_parent_directory;
    let mut sync_removed_parent = sync_parent_directory;
    restore_backup_observed_with_syncs(
        install_bin,
        backup,
        observer,
        &mut sync_temporary_file,
        &mut sync_renamed_parent,
        &mut sync_removed_parent,
    )
}

fn restore_backup_observed_with_syncs(
    install_bin: &Path,
    backup: Option<&Path>,
    observer: impl FnMut(InstallPublishPhase) -> io::Result<()>,
    sync_temporary_file: &mut impl FnMut(&File) -> io::Result<()>,
    sync_renamed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
    sync_removed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
) -> Result<(), InstallPublishError> {
    match backup {
        Some(path) => atomic_publish_file_observed_with_syncs(
            path,
            install_bin,
            observer,
            sync_temporary_file,
            sync_renamed_parent,
        )
        .map_err(|error| match error {
            InstallPublishError::Io(error) => InstallPublishError::BackupRestore(error),
            other => InstallPublishError::BackupRestore(io::Error::other(other)),
        }),
        None => remove_installed_file_durable_with_sync(install_bin, observer, sync_removed_parent)
            .map_err(InstallPublishError::BackupRestore),
    }
}

fn atomic_publish_file(source: &Path, destination: &Path) -> Result<(), InstallPublishError> {
    atomic_publish_file_observed(source, destination, |_| Ok(()))
}

fn atomic_publish_file_observed(
    source: &Path,
    destination: &Path,
    observer: impl FnMut(InstallPublishPhase) -> io::Result<()>,
) -> Result<(), InstallPublishError> {
    let mut sync_temporary_file = File::sync_all;
    let mut sync_renamed_parent = sync_parent_directory;
    atomic_publish_file_observed_with_syncs(
        source,
        destination,
        observer,
        &mut sync_temporary_file,
        &mut sync_renamed_parent,
    )
}

fn atomic_publish_file_observed_with_syncs(
    source: &Path,
    destination: &Path,
    mut observer: impl FnMut(InstallPublishPhase) -> io::Result<()>,
    sync_temporary_file: &mut impl FnMut(&File) -> io::Result<()>,
    sync_renamed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
) -> Result<(), InstallPublishError> {
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(InstallPublishError::Io)?;
    }
    let parent = destination
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let temporary = tempfile_in(&parent)?;
    let result = (|| {
        let mut source_file = File::open(source).map_err(InstallPublishError::Io)?;
        let mut dest_file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(InstallPublishError::Io)?;
        io::copy(&mut source_file, &mut dest_file).map_err(InstallPublishError::Io)?;
        dest_file.flush().map_err(InstallPublishError::Io)?;
        let permissions = fs::Permissions::from_mode(INSTALLED_MODE);
        fs::set_permissions(&temporary, permissions).map_err(InstallPublishError::Io)?;
        sync_temporary_file(&dest_file).map_err(InstallPublishError::Io)?;
        observer(InstallPublishPhase::TemporaryFileSynced).map_err(InstallPublishError::Io)?;
        drop(dest_file);
        fs::rename(&temporary, destination).map_err(InstallPublishError::Io)?;
        observer(InstallPublishPhase::DestinationRenamed).map_err(InstallPublishError::Io)?;
        sync_renamed_parent(&parent).map_err(InstallPublishError::Io)?;
        observer(InstallPublishPhase::ParentDirectorySynced).map_err(InstallPublishError::Io)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn remove_installed_file_durable_with_sync(
    install_bin: &Path,
    mut observer: impl FnMut(InstallPublishPhase) -> io::Result<()>,
    sync_removed_parent: &mut impl FnMut(&Path) -> io::Result<()>,
) -> io::Result<()> {
    match fs::symlink_metadata(install_bin) {
        Ok(_) => fs::remove_file(install_bin)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    }
    observer(InstallPublishPhase::DestinationRemoved)?;
    let parent = install_bin.parent().unwrap_or_else(|| Path::new("."));
    sync_removed_parent(parent)?;
    observer(InstallPublishPhase::ParentDirectorySynced)
}

fn sync_parent_directory(parent: &Path) -> io::Result<()> {
    File::open(parent)?.sync_all()
}

fn tempfile_in(parent: &Path) -> Result<PathBuf, InstallPublishError> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    for attempt in 0..32 {
        let candidate = parent.join(format!(".maicie.publish.{stamp}.{attempt}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(InstallPublishError::Io(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "impossible d'allouer un temporaire de publication Maicie",
    )))
}

fn run_published_preflight(
    install_bin: &Path,
    config_path: &Path,
) -> Result<(), InstallPublishError> {
    let output = Command::new(install_bin)
        .args([
            "preflight",
            "--config",
            &config_path.display().to_string(),
            "--json",
        ])
        .output()
        .map_err(InstallPublishError::Io)?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let detail = if !stderr.is_empty() { stderr } else { stdout };
    Err(InstallPublishError::PreflightFailed {
        status: output.status.code(),
        stderr: detail,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::os::fd::RawFd;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::process::CommandExt;
    use std::process::Stdio;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    const CRASH_CHILD_ENV: &str = "MAICIE_M4_CRASH_CHILD";
    const CRASH_OPERATION_ENV: &str = "MAICIE_M4_CRASH_OPERATION";
    const CRASH_SOURCE_ENV: &str = "MAICIE_M4_CRASH_SOURCE";
    const CRASH_INSTALL_ENV: &str = "MAICIE_M4_CRASH_INSTALL";
    const BARRIER_READY_FD: RawFd = 112;
    const BARRIER_RELEASE_FD: RawFd = 113;

    fn assert_publish_io(error: InstallPublishError, marker: &str) {
        match error {
            InstallPublishError::Io(error) => assert_eq!(error.to_string(), marker),
            other => panic!("erreur de publication inattendue : {other}"),
        }
    }

    fn assert_restore_io(error: InstallPublishError, marker: &str) {
        match error {
            InstallPublishError::BackupRestore(error) => assert_eq!(error.to_string(), marker),
            other => panic!("erreur de restauration inattendue : {other}"),
        }
    }

    fn assert_migration_restore_io(error: InstallPublishError, marker: &str) {
        match error {
            InstallPublishError::MigrationRestoreFailed { restoration, .. } => {
                assert!(restoration.contains(marker), "restauration={restoration}");
            }
            other => panic!("résultat de réconciliation inattendu : {other}"),
        }
    }

    struct BarrierChild {
        child: std::process::Child,
        ready: RawFd,
        release: RawFd,
    }

    impl BarrierChild {
        fn wait_phase(&self, expected: InstallPublishPhase) {
            let mut poll = libc::pollfd {
                fd: self.ready,
                events: libc::POLLIN,
                revents: 0,
            };
            assert_eq!(unsafe { libc::poll(&mut poll, 1, 5_000) }, 1);
            let mut byte = 0_u8;
            assert_eq!(
                unsafe { libc::read(self.ready, (&mut byte as *mut u8).cast(), 1) },
                1
            );
            assert_eq!(byte, phase_byte(expected));
        }

        fn release_phase(&self) {
            assert_eq!(
                unsafe { libc::write(self.release, [b'R'].as_ptr().cast(), 1) },
                1
            );
        }
    }

    impl Drop for BarrierChild {
        fn drop(&mut self) {
            if self.child.try_wait().ok().flatten().is_none() {
                let _ = unsafe { libc::kill(self.child.id() as libc::pid_t, libc::SIGTERM) };
                let _ = self.child.wait();
            }
            unsafe {
                libc::close(self.ready);
                libc::close(self.release);
            }
        }
    }

    fn phase_byte(phase: InstallPublishPhase) -> u8 {
        match phase {
            InstallPublishPhase::TemporaryFileSynced => b'F',
            InstallPublishPhase::DestinationRenamed => b'R',
            InstallPublishPhase::DestinationRemoved => b'X',
            InstallPublishPhase::ParentDirectorySynced => b'D',
        }
    }

    fn spawn_crash_writer(operation: &str, source: &Path, install: &Path) -> BarrierChild {
        let current_exe = std::env::current_exe().unwrap();
        assert_ne!(
            current_exe.file_name().and_then(|name| name.to_str()),
            Some("firefox")
        );
        let mut ready_pipe = [-1; 2];
        let mut release_pipe = [-1; 2];
        assert_eq!(unsafe { libc::pipe(ready_pipe.as_mut_ptr()) }, 0);
        assert_eq!(unsafe { libc::pipe(release_pipe.as_mut_ptr()) }, 0);

        let mut command = Command::new(current_exe);
        command
            .arg("--exact")
            .arg("install_publish::tests::durable_publish_crash_child")
            .arg("--ignored")
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CRASH_CHILD_ENV, "1")
            .env(CRASH_OPERATION_ENV, operation)
            .env(CRASH_SOURCE_ENV, source)
            .env(CRASH_INSTALL_ENV, install)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        unsafe {
            command.pre_exec(move || {
                if libc::dup2(ready_pipe[1], BARRIER_READY_FD) < 0
                    || libc::dup2(release_pipe[0], BARRIER_RELEASE_FD) < 0
                {
                    return Err(io::Error::last_os_error());
                }
                for fd in ready_pipe.into_iter().chain(release_pipe) {
                    if fd != BARRIER_READY_FD && fd != BARRIER_RELEASE_FD {
                        libc::close(fd);
                    }
                }
                Ok(())
            });
        }
        let child = command.spawn().unwrap();
        unsafe {
            libc::close(ready_pipe[1]);
            libc::close(release_pipe[0]);
        }
        BarrierChild {
            child,
            ready: ready_pipe[0],
            release: release_pipe[1],
        }
    }

    fn terminate_test_child(process: &mut BarrierChild) {
        assert!(process.child.try_wait().unwrap().is_none());
        assert_eq!(
            unsafe { libc::kill(process.child.id() as libc::pid_t, libc::SIGTERM) },
            0
        );
        assert!(!process.child.wait().unwrap().success());
    }

    fn unique_root(label: &str) -> PathBuf {
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "maicie-install-publish-{label}-{}-{n}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
        root
    }

    fn write_script(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }

    #[test]
    fn republication_remplace_un_installe_divergent_et_conserve_une_sauvegarde() {
        let root = unique_root("replace");
        let source = root.join("migrateur");
        let install = root.join("bin").join("maicie");
        let config = root.join("config.json");
        fs::write(&config, b"{}").unwrap();
        write_script(
            &source,
            "#!/bin/sh\ncase \"$1\" in preflight) echo ok; exit 0;; *) exit 1;; esac\n",
        );
        write_script(&install, "#!/bin/sh\necho OLD\nexit 1\n");

        let outcome = republish_exe_and_preflight(&source, &install, &config).unwrap();

        assert!(outcome.published);
        let backup = outcome.backup.expect("sauvegarde obligatoire");
        assert_eq!(
            fs::read(&install).unwrap(),
            fs::read(&source).unwrap(),
            "l'installé doit être le migrateur"
        );
        assert!(
            String::from_utf8_lossy(&fs::read(&backup).unwrap()).contains("OLD"),
            "la sauvegarde doit conserver l'ancien installé"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn republication_avant_migration_sans_preflight_cli_strict() {
        let root = unique_root("before-migrate");
        let source = root.join("migrateur");
        let install = root.join("bin").join("maicie");
        let database = root.join("maicie.sqlite3");
        {
            let store = crate::store::MaicieStore::open(&database).unwrap();
            assert_eq!(
                store.schema_version().unwrap(),
                crate::store::SCHEMA_VERSION
            );
        }
        let connection = Connection::open(&database).unwrap();
        connection
            .pragma_update(None, "user_version", crate::store::SCHEMA_VERSION - 1)
            .unwrap();
        drop(connection);
        write_script(
            &source,
            "#!/bin/sh\ncase \"$1\" in preflight) echo refuse; exit 7;; *) exit 0;; esac\n",
        );
        write_script(&install, "#!/bin/sh\necho OLD\nexit 0\n");

        let outcome = republish_exe_before_migrate(&source, &install, &database).unwrap();
        assert!(outcome.was_published());
        assert_eq!(fs::read(&install).unwrap(), fs::read(&source).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn preflight_refuse_restaure_la_sauvegarde_de_l_installe() {
        let root = unique_root("restore");
        let source = root.join("migrateur-refuse");
        let install = root.join("bin").join("maicie");
        let config = root.join("config.json");
        fs::write(&config, b"{}").unwrap();
        write_script(
            &source,
            "#!/bin/sh\ncase \"$1\" in preflight) echo refuse; exit 7;; *) exit 1;; esac\n",
        );
        let ancien = "#!/bin/sh\necho ANCIEN-INTACT\nexit 0\n";
        write_script(&install, ancien);

        let error = republish_exe_and_preflight(&source, &install, &config).unwrap_err();
        assert!(
            matches!(error, InstallPublishError::PreflightFailed { .. }),
            "le préflight doit refuser, reçu={error}"
        );
        assert_eq!(
            fs::read_to_string(&install).unwrap(),
            ancien,
            "l'installé doit être restauré après un préflight refusé"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn mode_wal_reellement_indisponible_apres_schema_committe_conserve_le_neuf() {
        let root = unique_root("wal-apres-commit");
        let database = root.join("maicie.sqlite3");
        let source = root.join("migrateur");
        let install = root.join("bin").join("maicie");
        drop(crate::store::MaicieStore::open(&database).unwrap());
        write_script(&source, "#!/bin/sh\necho NEUF\n");
        write_script(&install, "#!/bin/sh\necho ANCIEN\n");
        let publication = republish_exe_before_migrate(&source, &install, &database).unwrap();

        let memory = Connection::open_in_memory().unwrap();
        let wal_error = crate::store::set_wal_mode(&memory)
            .expect_err("SQLite mémoire doit répondre memory, jamais wal");
        assert!(matches!(
            wal_error,
            StoreError::JournalModeUnavailable { ref actual } if actual == "memory"
        ));

        let recovery = reconcile_failed_migration(publication, &wal_error).unwrap();
        assert_eq!(
            recovery,
            FailedMigrationRecovery::KeptPublishedInstall {
                durable_schema_version: SCHEMA_VERSION,
            }
        );
        assert_eq!(fs::read(&install).unwrap(), fs::read(&source).unwrap());
        assert_ne!(
            fs::metadata(&install).unwrap().ino(),
            fs::metadata(&source).unwrap().ino()
        );
        let version: i64 = Connection::open(&database)
            .unwrap()
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        fs::remove_dir_all(root).unwrap();
    }

    fn assert_crash_frontiers(operation: &str, label: &str, initial: &str, replacement: &str) {
        let before_root = unique_root(&format!("{label}-before-rename"));
        let before_source = before_root.join("source");
        let before_install = before_root.join("bin/maicie");
        write_script(&before_source, replacement);
        write_script(&before_install, initial);
        let mut before = spawn_crash_writer(operation, &before_source, &before_install);
        before.wait_phase(InstallPublishPhase::TemporaryFileSynced);
        terminate_test_child(&mut before);
        assert_eq!(
            fs::read_to_string(&before_install).unwrap(),
            initial,
            "un crash avant rename doit conserver l'installé précédent"
        );
        drop(before);
        fs::remove_dir_all(before_root).unwrap();

        let after_root = unique_root(&format!("{label}-after-rename"));
        let after_source = after_root.join("source");
        let after_install = after_root.join("bin/maicie");
        write_script(&after_source, replacement);
        write_script(&after_install, initial);
        let mut after = spawn_crash_writer(operation, &after_source, &after_install);
        after.wait_phase(InstallPublishPhase::TemporaryFileSynced);
        after.release_phase();
        after.wait_phase(InstallPublishPhase::DestinationRenamed);
        terminate_test_child(&mut after);
        assert_eq!(
            fs::read_to_string(&after_install).unwrap(),
            replacement,
            "un crash après rename doit exposer un fichier complet"
        );
        drop(after);
        fs::remove_dir_all(after_root).unwrap();
    }

    #[test]
    fn crash_aux_frontieres_conserve_un_installe_complet_en_publication_et_restauration() {
        assert_crash_frontiers(
            "publish",
            "publish",
            "#!/bin/sh\necho ANCIEN\n",
            "#!/bin/sh\necho NEUF\n",
        );
        assert_crash_frontiers(
            "restore",
            "restore",
            "#!/bin/sh\necho NEUF\n",
            "#!/bin/sh\necho ANCIEN\n",
        );
    }

    #[test]
    fn succes_publication_et_restauration_franchit_le_sync_du_parent() {
        let root = unique_root("phases-success");
        let source = root.join("source");
        let backup = root.join("backup");
        let install = root.join("bin/maicie");
        write_script(&source, "#!/bin/sh\necho NEUF\n");
        write_script(&backup, "#!/bin/sh\necho ANCIEN\n");
        write_script(&install, "#!/bin/sh\necho INITIAL\n");

        let mut publish_phases = Vec::new();
        atomic_publish_file_observed(&source, &install, |phase| {
            publish_phases.push(phase);
            Ok(())
        })
        .unwrap();
        assert_eq!(
            publish_phases,
            [
                InstallPublishPhase::TemporaryFileSynced,
                InstallPublishPhase::DestinationRenamed,
                InstallPublishPhase::ParentDirectorySynced,
            ]
        );

        let mut restore_phases = Vec::new();
        restore_backup_observed(&install, Some(&backup), |phase| {
            restore_phases.push(phase);
            Ok(())
        })
        .unwrap();
        assert_eq!(restore_phases, publish_phases);
        assert_eq!(fs::read(&install).unwrap(), fs::read(&backup).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn restauration_vers_absence_synchronise_la_suppression() {
        let root = unique_root("restore-absence");
        let install = root.join("bin/maicie");
        write_script(&install, "#!/bin/sh\necho NEUF\n");
        let mut phases = Vec::new();

        restore_backup_observed(&install, None, |phase| {
            phases.push(phase);
            Ok(())
        })
        .unwrap();

        assert!(!install.exists());
        assert_eq!(
            phases,
            [
                InstallPublishPhase::DestinationRemoved,
                InstallPublishPhase::ParentDirectorySynced,
            ]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn erreur_sync_temporaire_est_propagee_par_la_publication() {
        let root = unique_root("erreur-sync-temporaire");
        let source = root.join("source");
        let install = root.join("bin/maicie");
        write_script(&source, "#!/bin/sh\necho NEUF\n");
        write_script(&install, "#!/bin/sh\necho ANCIEN\n");
        let mut fail_temporary = |_: &File| Err(io::Error::other("m4-sync-temporary-file"));
        let mut sync_parent = sync_parent_directory;

        let result = atomic_publish_file_observed_with_syncs(
            &source,
            &install,
            |_| Ok(()),
            &mut fail_temporary,
            &mut sync_parent,
        );
        let installed = fs::read_to_string(&install).unwrap();
        fs::remove_dir_all(root).unwrap();

        assert_publish_io(
            result.expect_err("l'erreur du sync fichier doit remonter"),
            "m4-sync-temporary-file",
        );
        assert_eq!(installed, "#!/bin/sh\necho ANCIEN\n");
    }

    #[test]
    fn erreur_sync_parent_est_propagee_par_la_restauration() {
        let root = unique_root("erreur-sync-parent-restauration");
        let backup = root.join("backup");
        let install = root.join("bin/maicie");
        write_script(&backup, "#!/bin/sh\necho ANCIEN\n");
        write_script(&install, "#!/bin/sh\necho NEUF\n");
        let mut sync_temporary = File::sync_all;
        let mut fail_renamed_parent = |_: &Path| Err(io::Error::other("m4-sync-renamed-parent"));
        let mut sync_removed_parent = sync_parent_directory;

        let result = restore_backup_observed_with_syncs(
            &install,
            Some(&backup),
            |_| Ok(()),
            &mut sync_temporary,
            &mut fail_renamed_parent,
            &mut sync_removed_parent,
        );
        let installed = fs::read_to_string(&install).unwrap();
        fs::remove_dir_all(root).unwrap();

        assert_restore_io(
            result.expect_err("l'erreur du sync parent après rename doit remonter"),
            "m4-sync-renamed-parent",
        );
        assert_eq!(installed, "#!/bin/sh\necho ANCIEN\n");
    }

    #[test]
    fn erreur_sync_parent_est_propagee_par_la_suppression() {
        let root = unique_root("erreur-sync-parent-suppression");
        let install = root.join("bin/maicie");
        write_script(&install, "#!/bin/sh\necho NEUF\n");
        let mut sync_temporary = File::sync_all;
        let mut sync_renamed_parent = sync_parent_directory;
        let mut fail_removed_parent = |_: &Path| Err(io::Error::other("m4-sync-removed-parent"));

        let result = restore_backup_observed_with_syncs(
            &install,
            None,
            |_| Ok(()),
            &mut sync_temporary,
            &mut sync_renamed_parent,
            &mut fail_removed_parent,
        );
        let install_exists = install.exists();
        fs::remove_dir_all(root).unwrap();

        assert_restore_io(
            result.expect_err("l'erreur du sync parent après suppression doit remonter"),
            "m4-sync-removed-parent",
        );
        assert!(!install_exists);
    }

    #[test]
    fn erreur_sync_parent_interdit_le_consentement_de_migration() {
        let root = unique_root("consentement-sync-parent");
        let source = root.join("source");
        let install = root.join("bin/maicie");
        let database = root.join("maicie.sqlite3");
        drop(crate::store::MaicieStore::open(&database).unwrap());
        write_script(&source, "#!/bin/sh\necho NEUF\n");
        write_script(&install, "#!/bin/sh\necho ANCIEN\n");
        let mut publish = |source: &Path, destination: &Path| {
            let mut sync_temporary = File::sync_all;
            let mut fail_parent =
                |_: &Path| Err(io::Error::other("m4-consent-sync-renamed-parent"));
            atomic_publish_file_observed_with_syncs(
                source,
                destination,
                |_| Ok(()),
                &mut sync_temporary,
                &mut fail_parent,
            )
        };

        let result =
            republish_exe_before_migrate_with_publisher(&source, &install, &database, &mut publish);
        let installed = fs::read_to_string(&install).unwrap();
        fs::remove_dir_all(root).unwrap();

        assert_publish_io(
            result.expect_err("aucun PublishedMigration après erreur de sync"),
            "m4-consent-sync-renamed-parent",
        );
        assert_eq!(installed, "#!/bin/sh\necho NEUF\n");
    }

    #[test]
    fn erreur_sync_restauration_avec_sauvegarde_devient_migration_restore_failed() {
        let root = unique_root("reconcile-sync-backup");
        let source = root.join("source");
        let backup = root.join("backup");
        let install = root.join("bin/maicie");
        let database = root.join("maicie.sqlite3");
        drop(crate::store::MaicieStore::open(&database).unwrap());
        let connection = Connection::open(&database).unwrap();
        connection
            .pragma_update(None, "user_version", SCHEMA_VERSION - 1)
            .unwrap();
        drop(connection);
        write_script(&source, "#!/bin/sh\necho NEUF\n");
        write_script(&backup, "#!/bin/sh\necho ANCIEN\n");
        write_script(&install, "#!/bin/sh\necho NEUF\n");
        let publication = PublishedMigration {
            outcome: PublishOutcome {
                install_bin: install.clone(),
                source_exe: source,
                published: true,
                backup: Some(backup),
            },
            database_path: database,
        };
        let migration_error = StoreError::JournalModeUnavailable {
            actual: "memory".to_string(),
        };
        let mut sync_temporary = File::sync_all;
        let mut fail_parent = |_: &Path| Err(io::Error::other("m4-reconcile-sync-renamed-parent"));
        let mut sync_removed_parent = sync_parent_directory;

        let result = reconcile_failed_migration_with_syncs(
            publication,
            &migration_error,
            &mut sync_temporary,
            &mut fail_parent,
            &mut sync_removed_parent,
        );
        let installed = fs::read_to_string(&install).unwrap();
        fs::remove_dir_all(root).unwrap();

        assert_migration_restore_io(
            result.expect_err("une restauration non durable ne peut pas être déclarée réussie"),
            "m4-reconcile-sync-renamed-parent",
        );
        assert_eq!(installed, "#!/bin/sh\necho ANCIEN\n");
    }

    #[test]
    fn erreur_sync_restauration_vers_absence_devient_migration_restore_failed() {
        let root = unique_root("reconcile-sync-absence");
        let source = root.join("source");
        let install = root.join("bin/maicie");
        let database = root.join("maicie.sqlite3");
        drop(crate::store::MaicieStore::open(&database).unwrap());
        let connection = Connection::open(&database).unwrap();
        connection
            .pragma_update(None, "user_version", SCHEMA_VERSION - 1)
            .unwrap();
        drop(connection);
        write_script(&source, "#!/bin/sh\necho NEUF\n");
        write_script(&install, "#!/bin/sh\necho NEUF\n");
        let publication = PublishedMigration {
            outcome: PublishOutcome {
                install_bin: install.clone(),
                source_exe: source,
                published: true,
                backup: None,
            },
            database_path: database,
        };
        let migration_error = StoreError::JournalModeUnavailable {
            actual: "memory".to_string(),
        };
        let mut sync_temporary = File::sync_all;
        let mut sync_renamed_parent = sync_parent_directory;
        let mut fail_removed_parent =
            |_: &Path| Err(io::Error::other("m4-reconcile-sync-removed-parent"));

        let result = reconcile_failed_migration_with_syncs(
            publication,
            &migration_error,
            &mut sync_temporary,
            &mut sync_renamed_parent,
            &mut fail_removed_parent,
        );
        let install_exists = install.exists();
        fs::remove_dir_all(root).unwrap();

        assert_migration_restore_io(
            result.expect_err("une suppression non durable ne peut pas être déclarée réussie"),
            "m4-reconcile-sync-removed-parent",
        );
        assert!(!install_exists);
    }

    #[test]
    #[ignore = "sous-processus contrôlé par les crash-tests M4"]
    fn durable_publish_crash_child() {
        if std::env::var(CRASH_CHILD_ENV).as_deref() != Ok("1") {
            return;
        }
        let operation = std::env::var(CRASH_OPERATION_ENV).unwrap();
        let source = PathBuf::from(std::env::var_os(CRASH_SOURCE_ENV).unwrap());
        let install = PathBuf::from(std::env::var_os(CRASH_INSTALL_ENV).unwrap());
        let observer = |phase: InstallPublishPhase| {
            let byte = phase_byte(phase);
            if unsafe { libc::write(BARRIER_READY_FD, (&byte as *const u8).cast(), 1) } != 1 {
                return Err(io::Error::last_os_error());
            }
            let mut release = 0_u8;
            if unsafe { libc::read(BARRIER_RELEASE_FD, (&mut release as *mut u8).cast(), 1) } != 1 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "barrière de crash fermée",
                ));
            }
            Ok(())
        };
        match operation.as_str() {
            "publish" => atomic_publish_file_observed(&source, &install, observer).unwrap(),
            "restore" => {
                restore_backup_observed(&install, Some(&source), observer).unwrap();
            }
            other => panic!("opération de crash inconnue : {other}"),
        }
    }

    #[test]
    fn publication_et_restauration_portent_les_syncs_durables() {
        let source = include_str!("install_publish.rs");
        let publish = source
            .split("fn atomic_publish_file_observed_with_syncs(")
            .nth(1)
            .and_then(|tail| {
                tail.split("\nfn remove_installed_file_durable_with_sync(")
                    .next()
            })
            .expect("corps de la publication atomique");
        let create_new = [".", "create_new(true)"].concat();
        let chmod = ["fs::", "set_permissions(&temporary, permissions)"].concat();
        let file_sync = ["sync_temporary_", "file(&dest_file)"].concat();
        let rename = ["fs::", "rename(&temporary, destination)"].concat();
        let parent_sync = ["sync_renamed_", "parent(&parent)"].concat();
        let create_new_at = publish
            .find(&create_new)
            .expect("le temporaire doit être créé sans fenêtre TOCTOU");
        let chmod_at = publish
            .find(&chmod)
            .expect("le mode installé doit être posé avant synchronisation");
        let file_sync_at = publish
            .find(&file_sync)
            .expect("le temporaire doit être synchronisé avant publication");
        let rename_at = publish
            .find(&rename)
            .expect("le renommage atomique doit rester explicite");
        let parent_sync_at = publish
            .find(&parent_sync)
            .expect("le répertoire parent doit être synchronisé après renommage");
        assert!(
            create_new_at < chmod_at
                && chmod_at < file_sync_at
                && file_sync_at < rename_at
                && rename_at < parent_sync_at
        );

        let restore = source
            .split("fn restore_backup_observed(")
            .nth(1)
            .and_then(|tail| tail.split("\nfn atomic_publish_file(").next())
            .expect("corps de la restauration");
        let durable_publish = ["atomic_publish_file_observed_", "with_syncs("].concat();
        let durable_remove = ["remove_installed_file_durable_", "with_sync("].concat();
        assert!(
            restore.contains(&durable_publish),
            "restaurer une sauvegarde doit reprendre le remplacement durable"
        );
        assert!(
            restore.contains(&durable_remove),
            "restaurer une absence doit synchroniser la suppression dans le parent"
        );

        let remove = source
            .split("fn remove_installed_file_durable_with_sync(")
            .nth(1)
            .and_then(|tail| tail.split("\nfn sync_parent_directory(").next())
            .expect("corps de la restauration vers l'absence");
        let remove_file = ["fs::", "remove_file(install_bin)"].concat();
        let remove_at = remove
            .find(&remove_file)
            .expect("suppression de l'installé");
        let remove_parent_sync = ["sync_removed_", "parent(parent)"].concat();
        let sync_at = remove
            .find(&remove_parent_sync)
            .expect("la suppression doit être synchronisée dans son parent");
        assert!(remove_at < sync_at);

        let sync_parent = source
            .split("fn sync_parent_directory(")
            .nth(1)
            .and_then(|tail| tail.split("\nfn tempfile_in(").next())
            .expect("primitive de synchronisation du parent");
        let actual_sync = ["File::open(parent)?.", "sync_all()"].concat();
        assert!(sync_parent.contains(&actual_sync));

        let publish_wrapper = source
            .split("fn atomic_publish_file_observed(")
            .nth(1)
            .and_then(|tail| {
                tail.split("\nfn atomic_publish_file_observed_with_syncs(")
                    .next()
            })
            .expect("raccord de publication vers les opérations réelles");
        assert!(publish_wrapper.contains("let mut sync_temporary_file = File::sync_all;"));
        assert!(publish_wrapper.contains("let mut sync_renamed_parent = sync_parent_directory;"));

        assert!(restore.contains("let mut sync_temporary_file = File::sync_all;"));
        assert!(restore.contains("let mut sync_renamed_parent = sync_parent_directory;"));
        assert!(restore.contains("let mut sync_removed_parent = sync_parent_directory;"));

        let consent_wrapper = source
            .split("pub fn republish_exe_before_migrate(")
            .nth(1)
            .and_then(|tail| {
                tail.split("\nfn republish_exe_before_migrate_with_publisher(")
                    .next()
            })
            .expect("raccord du consentement vers la publication injectée");
        let consent_worker = ["republish_exe_before_migrate_", "with_publisher("].concat();
        assert!(consent_wrapper.contains(&consent_worker));
        assert!(consent_wrapper.contains("atomic_publish_file,"));

        let reconcile_wrapper = source
            .split("pub fn reconcile_failed_migration(")
            .nth(1)
            .and_then(|tail| {
                tail.split("\nfn reconcile_failed_migration_with_syncs(")
                    .next()
            })
            .expect("raccord de réconciliation vers les syncs réels");
        assert!(reconcile_wrapper.contains("let mut sync_temporary_file = File::sync_all;"));
        assert!(reconcile_wrapper.contains("let mut sync_renamed_parent = sync_parent_directory;"));
        assert!(reconcile_wrapper.contains("let mut sync_removed_parent = sync_parent_directory;"));

        let under_exclusion = source
            .split("fn reconcile_failed_migration_under_exclusion(")
            .nth(1)
            .and_then(|tail| {
                tail.split("\nfn reconcile_failed_migration_under_exclusion_with_syncs(")
                    .next()
            })
            .expect("raccord sous exclusion vers les syncs réels");
        assert!(under_exclusion.contains("let mut sync_temporary_file = File::sync_all;"));
        assert!(under_exclusion.contains("let mut sync_renamed_parent = sync_parent_directory;"));
        assert!(under_exclusion.contains("let mut sync_removed_parent = sync_parent_directory;"));
    }
}
