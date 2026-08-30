//! Cas d'usage fermés de la coque locale pour les profils.

use crate::profile::{ConnectionProfile, ProfileDraft};
use crate::profile_store::{ProfileStore, ProfileStoreError};
use std::fmt;
use uuid::Uuid;

#[derive(Debug)]
pub enum ProfileServiceError {
    Store(ProfileStoreError),
    NotFound(String),
    ConfirmationRequired,
}

impl fmt::Display for ProfileServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(error) => error.fmt(formatter),
            Self::NotFound(id) => write!(formatter, "Profil introuvable : {id}."),
            Self::ConfirmationRequired => {
                formatter.write_str("La suppression d'un profil exige une confirmation explicite.")
            }
        }
    }
}

impl std::error::Error for ProfileServiceError {}

impl From<ProfileStoreError> for ProfileServiceError {
    fn from(error: ProfileStoreError) -> Self {
        Self::Store(error)
    }
}

pub struct ProfileService {
    store: ProfileStore,
}

impl ProfileService {
    pub fn new(store: ProfileStore) -> Self {
        Self { store }
    }

    pub fn list(&self) -> Result<Vec<ConnectionProfile>, ProfileServiceError> {
        Ok(self.store.load()?.profiles)
    }

    pub fn save(
        &self,
        profile_id: Option<&str>,
        draft: ProfileDraft,
    ) -> Result<ConnectionProfile, ProfileServiceError> {
        let mut loaded = self.store.load()?;
        let id = profile_id
            .map(str::to_owned)
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let position = loaded
            .profiles
            .iter()
            .position(|profile| profile.id() == id);

        if profile_id.is_some() && position.is_none() {
            return Err(ProfileServiceError::NotFound(id));
        }

        let mut replacement = draft.into_profile(id);
        if let Some(position) = position {
            replacement =
                preserve_trust_if_target_is_unchanged(&loaded.profiles[position], replacement);
            loaded.profiles[position] = replacement.clone();
        } else {
            loaded.profiles.push(replacement.clone());
        }
        self.store.save(&loaded.profiles)?;
        Ok(replacement)
    }

    pub fn delete(&self, profile_id: &str, confirmed: bool) -> Result<(), ProfileServiceError> {
        if !confirmed {
            return Err(ProfileServiceError::ConfirmationRequired);
        }
        if !self.store.remove(profile_id)? {
            return Err(ProfileServiceError::NotFound(profile_id.to_owned()));
        }
        Ok(())
    }

    pub fn replace(&self, profile: ConnectionProfile) -> Result<(), ProfileServiceError> {
        profile.validate().map_err(ProfileStoreError::from)?;
        let mut loaded = self.store.load()?;
        let position = loaded
            .profiles
            .iter()
            .position(|candidate| candidate.id() == profile.id())
            .ok_or_else(|| ProfileServiceError::NotFound(profile.id().to_owned()))?;
        loaded.profiles[position] = profile;
        self.store.save(&loaded.profiles)?;
        Ok(())
    }
}

fn preserve_trust_if_target_is_unchanged(
    previous: &ConnectionProfile,
    mut replacement: ConnectionProfile,
) -> ConnectionProfile {
    match (previous, &mut replacement) {
        (
            ConnectionProfile::Ssh {
                host: old_host,
                port: old_port,
                host_fingerprint: old_fingerprint,
                capabilities: old_capabilities,
                ..
            },
            ConnectionProfile::Ssh {
                host: new_host,
                port: new_port,
                host_fingerprint: new_fingerprint,
                capabilities: new_capabilities,
                ..
            },
        ) => {
            *new_capabilities = old_capabilities.clone();
            if old_host == new_host && old_port == new_port {
                *new_fingerprint = old_fingerprint.clone();
            }
        }
        (
            ConnectionProfile::Local {
                capabilities: old_capabilities,
                ..
            },
            ConnectionProfile::Local {
                capabilities: new_capabilities,
                ..
            },
        ) => *new_capabilities = old_capabilities.clone(),
        _ => {}
    }
    replacement
}

#[cfg(test)]
mod tests {
    use super::{ProfileService, ProfileServiceError};
    use crate::profile::{ProfileDraft, SshIdentityRef};
    use crate::profile_store::ProfileStore;
    use std::fs;
    use std::path::PathBuf;
    use uuid::Uuid;

    fn service() -> (ProfileService, PathBuf) {
        let directory =
            std::env::temp_dir().join(format!("bridget-desktop-service-{}", Uuid::new_v4()));
        fs::create_dir_all(&directory).expect("dossier temporaire");
        (
            ProfileService::new(ProfileStore::new(directory.join("profiles.json"))),
            directory,
        )
    }

    fn ssh_draft(host: &str) -> ProfileDraft {
        ProfileDraft::Ssh {
            label: "Cartae.app".into(),
            host: host.into(),
            port: 2222,
            user: "moi".into(),
            identity: SshIdentityRef::Agent,
        }
    }

    #[test]
    fn ajout_et_edition_restent_fermes_et_l_identite_changee_perd_sa_confiance() {
        let (service, directory) = service();
        let created = service
            .save(None, ssh_draft("cartae.app"))
            .expect("création");
        let id = created.id().to_owned();
        let updated = service
            .save(Some(&id), ssh_draft("autre.cartae.app"))
            .expect("édition");
        assert_eq!(updated.id(), id);
        if let crate::profile::ConnectionProfile::Ssh {
            host_fingerprint, ..
        } = updated
        {
            assert!(host_fingerprint.is_none());
        } else {
            panic!("profil SSH attendu");
        }
        assert_eq!(service.list().expect("liste").len(), 1);
        fs::remove_dir_all(directory).expect("nettoyage exact du test");
    }

    #[test]
    fn suppression_sans_confirmation_expresse_est_refusee() {
        let (service, directory) = service();
        let created = service
            .save(None, ssh_draft("cartae.app"))
            .expect("création");
        assert!(matches!(
            service.delete(created.id(), false),
            Err(ProfileServiceError::ConfirmationRequired)
        ));
        assert!(service.delete(created.id(), true).is_ok());
        fs::remove_dir_all(directory).expect("nettoyage exact du test");
    }
}
