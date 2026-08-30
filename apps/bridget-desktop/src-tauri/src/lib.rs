//! Coque locale Bridget Desktop.
//!
//! Les modules de profil et de connexion sont ajoutés progressivement par la
//! SPEC-074. La première coque n'accorde des capabilities qu'à son webview
//! locale; les panneaux distants sont créés avec d'autres labels.

pub mod connection;
pub mod host_identity;
pub mod panels;
pub mod profile;
pub mod profile_service;
pub mod profile_store;
pub mod ssh;

#[cfg(target_os = "macos")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use crate::connection::{
        ConnectionStatus, HttpRelayProbe, RemoteTransport, SshRemoteTransport, connect_remote,
        mark_tunnel_lost, relay_url,
    };
    use crate::host_identity::{
        HostIdentityStatus, HostIdentityTicket, SystemHostKeyCommandRunner, approve_host_identity,
        check_host_identity,
    };
    use crate::panels::PanelRegistry;
    use crate::profile::{ConnectionProfile, ProfileDraft};
    use crate::profile_service::ProfileService;
    use crate::profile_store::ProfileStore;
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use tauri::webview::WebviewBuilder;
    use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewUrl};

    struct DesktopState {
        profiles: Mutex<ProfileService>,
        known_hosts: PathBuf,
        pending_host_tickets: Mutex<HashMap<String, HostIdentityTicket>>,
        sessions: Mutex<HashMap<String, ActiveConnection>>,
        panels: Mutex<PanelRegistry>,
    }

    struct ActiveConnection {
        session: crate::profile::ConnectionSession,
        transport: Option<SshRemoteTransport>,
        local_port: u16,
    }

    #[derive(serde::Serialize)]
    struct PanelView {
        label: String,
        profile_id: String,
        open_panels: usize,
    }

    #[derive(serde::Serialize)]
    struct HostIdentityView {
        status: &'static str,
        fingerprint: Option<String>,
        ticket: Option<String>,
    }

    fn as_message(error: impl std::fmt::Display) -> String {
        error.to_string()
    }

    fn publish_connection_state(app: &tauri::AppHandle, status: &ConnectionStatus) {
        let _ = app.emit_to("main", "connection-state", status);
    }

    fn watch_remote_tunnel(app: tauri::AppHandle, profile_id: String) {
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_secs(1));
                let Some(state) = app.try_state::<DesktopState>() else {
                    return;
                };
                let status = {
                    let mut sessions = match state.sessions.lock() {
                        Ok(sessions) => sessions,
                        Err(_) => return,
                    };
                    let Some(active) = sessions.get_mut(&profile_id) else {
                        return;
                    };
                    let running = active
                        .transport
                        .as_mut()
                        .and_then(|transport| transport.is_running().ok())
                        .unwrap_or(false);
                    if running {
                        None
                    } else {
                        if let Some(mut transport) = active.transport.take() {
                            transport.close();
                        }
                        Some(mark_tunnel_lost(&mut active.session))
                    }
                };
                if let Some(status) = status {
                    publish_connection_state(&app, &status);
                    return;
                }
            }
        });
    }

    fn close_panel_for_profile(
        app: &tauri::AppHandle,
        state: &DesktopState,
        profile_id: &str,
    ) -> Result<(), String> {
        let panel = {
            let mut panels = state.panels.lock().map_err(as_message)?;
            let label = panels
                .panels()
                .find(|panel| panel.profile_id == profile_id)
                .map(|panel| panel.label.clone());
            label.and_then(|label| panels.close(&label))
        };
        if let Some(panel) = panel {
            if let Some(webview) = app.get_webview(&panel.label) {
                webview.close().map_err(as_message)?;
            }
            let panels = state.panels.lock().map_err(as_message)?;
            arrange_panels(app, &panels)?;
        }
        Ok(())
    }

    fn close_connection_for_profile(
        app: &tauri::AppHandle,
        state: &DesktopState,
        profile_id: String,
    ) -> Result<ConnectionStatus, String> {
        close_panel_for_profile(app, state, &profile_id)?;
        let active = state
            .sessions
            .lock()
            .map_err(as_message)?
            .remove(&profile_id);
        if let Some(mut active) = active {
            if let Some(mut transport) = active.transport {
                transport.close();
            }
            let _ = crate::connection::transition(
                &mut active.session,
                crate::profile::ConnectionState::Closed,
            );
            return Ok(ConnectionStatus::from_session(&active.session));
        }
        Ok(ConnectionStatus {
            profile_id,
            state: crate::profile::ConnectionState::Closed,
            category: None,
        })
    }

    fn main_window(app: &tauri::AppHandle) -> Result<tauri::Window, String> {
        app.get_window("main")
            .ok_or_else(|| "La fenêtre Bridget Desktop est indisponible.".to_owned())
    }

    fn arrange_panels(app: &tauri::AppHandle, panels: &PanelRegistry) -> Result<(), String> {
        let main = main_window(app)?;
        let size = main.inner_size().map_err(as_message)?;
        let mut all = panels.panels().cloned().collect::<Vec<_>>();
        all.sort_by(|left, right| left.label.cmp(&right.label));
        if all.is_empty() {
            return Ok(());
        }
        let count = u32::try_from(all.len()).map_err(|_| "Trop de panneaux ouverts.")?;
        let top = 82_u32;
        let width = (size.width / count).max(1);
        let height = size.height.saturating_sub(top).max(1);
        for (index, panel) in all.into_iter().enumerate() {
            if let Some(webview) = app.get_webview(&panel.label) {
                webview
                    .set_position(PhysicalPosition::new(
                        (index as u32 * width) as i32,
                        top as i32,
                    ))
                    .map_err(as_message)?;
                webview
                    .set_size(PhysicalSize::new(width, height))
                    .map_err(as_message)?;
            }
        }
        Ok(())
    }

    #[tauri::command(rename_all = "snake_case")]
    fn profiles_list(state: State<'_, DesktopState>) -> Result<Vec<ConnectionProfile>, String> {
        state
            .profiles
            .lock()
            .map_err(as_message)?
            .list()
            .map_err(as_message)
    }

    #[tauri::command(rename_all = "snake_case")]
    fn profile_save(
        state: State<'_, DesktopState>,
        profile_id: Option<String>,
        draft: ProfileDraft,
    ) -> Result<ConnectionProfile, String> {
        state
            .profiles
            .lock()
            .map_err(as_message)?
            .save(profile_id.as_deref(), draft)
            .map_err(as_message)
    }

    #[tauri::command(rename_all = "snake_case")]
    fn profile_delete(
        app: tauri::AppHandle,
        state: State<'_, DesktopState>,
        profile_id: String,
        confirmed: bool,
    ) -> Result<(), String> {
        if confirmed {
            let _ = close_connection_for_profile(&app, &state, profile_id.clone())?;
        }
        state
            .profiles
            .lock()
            .map_err(as_message)?
            .delete(&profile_id, confirmed)
            .map_err(as_message)
    }

    #[tauri::command(rename_all = "snake_case")]
    fn host_identity_check(
        state: State<'_, DesktopState>,
        profile_id: String,
    ) -> Result<HostIdentityView, String> {
        let profile = state
            .profiles
            .lock()
            .map_err(as_message)?
            .list()
            .map_err(as_message)?
            .into_iter()
            .find(|candidate| candidate.id() == profile_id)
            .ok_or_else(|| "Profil introuvable.".to_owned())?;
        let mut runner = SystemHostKeyCommandRunner;
        match check_host_identity(&profile, &mut runner).map_err(as_message)? {
            HostIdentityStatus::Approved => Ok(HostIdentityView {
                status: "approved",
                fingerprint: None,
                ticket: None,
            }),
            HostIdentityStatus::Changed {
                expected: _,
                observed,
            } => Ok(HostIdentityView {
                status: "changed",
                fingerprint: Some(observed),
                ticket: None,
            }),
            HostIdentityStatus::AwaitingApproval(ticket) => {
                let ticket_id = uuid::Uuid::new_v4().to_string();
                state
                    .pending_host_tickets
                    .lock()
                    .map_err(as_message)?
                    .insert(ticket_id.clone(), ticket.clone());
                Ok(HostIdentityView {
                    status: "awaiting_approval",
                    fingerprint: Some(ticket.fingerprint),
                    ticket: Some(ticket_id),
                })
            }
        }
    }

    #[tauri::command(rename_all = "snake_case")]
    fn host_identity_approve(
        state: State<'_, DesktopState>,
        profile_id: String,
        ticket: String,
    ) -> Result<(), String> {
        let host_ticket = state
            .pending_host_tickets
            .lock()
            .map_err(as_message)?
            .remove(&ticket)
            .ok_or_else(|| "Le ticket d'approbation a expiré.".to_owned())?;
        if host_ticket.profile_id != profile_id {
            return Err("Le ticket ne correspond pas à ce profil.".into());
        }
        let service = state.profiles.lock().map_err(as_message)?;
        let mut profile = service
            .list()
            .map_err(as_message)?
            .into_iter()
            .find(|candidate| candidate.id() == profile_id)
            .ok_or_else(|| "Profil introuvable.".to_owned())?;
        approve_host_identity(&mut profile, host_ticket, &state.known_hosts).map_err(as_message)?;
        service.replace(profile).map_err(as_message)
    }

    #[tauri::command(rename_all = "snake_case")]
    fn connection_open(
        app: tauri::AppHandle,
        state: State<'_, DesktopState>,
        profile_id: String,
    ) -> Result<ConnectionStatus, String> {
        let profile = state
            .profiles
            .lock()
            .map_err(as_message)?
            .list()
            .map_err(as_message)?
            .into_iter()
            .find(|candidate| candidate.id() == profile_id)
            .ok_or_else(|| "Profil introuvable.".to_owned())?;
        close_panel_for_profile(&app, &state, &profile_id)?;
        if let Some(previous) = state
            .sessions
            .lock()
            .map_err(as_message)?
            .remove(&profile_id)
        {
            if let Some(mut transport) = previous.transport {
                transport.close();
            }
        }
        let mut transport = SshRemoteTransport::new(state.known_hosts.clone());
        let mut probe = HttpRelayProbe;
        let session = connect_remote(&profile, &mut transport, &mut probe).map_err(as_message)?;
        let local_port = transport
            .local_port()
            .ok_or_else(|| "Le tunnel SSH est indisponible.".to_owned())?;
        let status = ConnectionStatus::from_session(&session);
        let monitor_id = profile_id.clone();
        state.sessions.lock().map_err(as_message)?.insert(
            profile_id,
            ActiveConnection {
                session,
                transport: Some(transport),
                local_port,
            },
        );
        publish_connection_state(&app, &status);
        watch_remote_tunnel(app, monitor_id);
        Ok(status)
    }

    #[tauri::command(rename_all = "snake_case")]
    fn connection_close(
        app: tauri::AppHandle,
        state: State<'_, DesktopState>,
        profile_id: String,
    ) -> Result<ConnectionStatus, String> {
        let status = close_connection_for_profile(&app, &state, profile_id)?;
        publish_connection_state(&app, &status);
        Ok(status)
    }

    /// Ouvre un relais déjà prouvé dans un enfant WebView isolé. Le jeton ne
    /// traverse jamais le contrat IPC de la page de contrôle locale.
    #[tauri::command(rename_all = "snake_case")]
    fn panel_open(
        app: tauri::AppHandle,
        state: State<'_, DesktopState>,
        profile_id: String,
    ) -> Result<PanelView, String> {
        let url = {
            let sessions = state.sessions.lock().map_err(as_message)?;
            let active = sessions
                .get(&profile_id)
                .ok_or_else(|| "Ce serveur n'est pas encore connecté.".to_owned())?;
            let endpoint = active
                .session
                .endpoint()
                .ok_or_else(|| "Le relais connecté ne fournit pas d'endpoint.".to_owned())?;
            relay_url(active.local_port, endpoint)
        };
        let panel = {
            let mut panels = state.panels.lock().map_err(as_message)?;
            panels.open(profile_id.clone(), url).map_err(as_message)?
        };
        let main = main_window(&app)?;
        let external_url = panel
            .url
            .parse()
            .map_err(|_| "L'URL du relais local est invalide.")?;
        let child = WebviewBuilder::new(panel.label.clone(), WebviewUrl::External(external_url))
            .on_navigation(|url| url.scheme() == "http" && url.host_str() == Some("127.0.0.1"));
        if let Err(error) = main.add_child(
            child,
            PhysicalPosition::new(0_i32, 82_i32),
            PhysicalSize::new(1_u32, 1_u32),
        ) {
            state.panels.lock().map_err(as_message)?.close(&panel.label);
            return Err(as_message(error));
        }
        let panels = state.panels.lock().map_err(as_message)?;
        arrange_panels(&app, &panels)?;
        Ok(PanelView {
            label: panel.label,
            profile_id,
            open_panels: panels.panels().count(),
        })
    }

    #[tauri::command(rename_all = "snake_case")]
    fn panel_close(
        app: tauri::AppHandle,
        state: State<'_, DesktopState>,
        profile_id: String,
    ) -> Result<(), String> {
        close_panel_for_profile(&app, &state, &profile_id)
    }

    tauri::Builder::default()
        .setup(|app| {
            let path = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("Répertoire applicatif indisponible : {error}"))?
                .join("profiles.json");
            app.manage(DesktopState {
                profiles: Mutex::new(ProfileService::new(ProfileStore::new(path))),
                known_hosts: app
                    .path()
                    .app_data_dir()
                    .map_err(|error| format!("Répertoire applicatif indisponible : {error}"))?
                    .join("known_hosts"),
                pending_host_tickets: Mutex::new(HashMap::new()),
                sessions: Mutex::new(HashMap::new()),
                panels: Mutex::new(PanelRegistry::default()),
            });
            let handle = app.handle().clone();
            if let Some(main) = app.get_window("main") {
                main.on_window_event(move |event| {
                    if matches!(event, tauri::WindowEvent::Resized(_)) {
                        if let Some(state) = handle.try_state::<DesktopState>() {
                            if let Ok(panels) = state.panels.lock() {
                                let _ = arrange_panels(&handle, &panels);
                            }
                        }
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            profiles_list,
            profile_save,
            profile_delete,
            host_identity_check,
            host_identity_approve,
            connection_open,
            connection_close,
            panel_open,
            panel_close
        ])
        .run(tauri::generate_context!())
        .expect("Bridget Desktop n'a pas pu démarrer");
}

#[cfg(not(target_os = "macos"))]
pub fn run() {
    panic!("Bridget Desktop doit être exécuté sur macOS");
}
