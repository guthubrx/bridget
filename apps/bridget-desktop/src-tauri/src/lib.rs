//! Coque locale Bridget Desktop.
//!
//! Les modules de profil et de connexion sont ajoutés progressivement par la
//! SPEC-074. La première coque n'accorde des capabilities qu'à son webview
//! locale; les panneaux distants sont créés avec d'autres labels.

pub mod artifact_cache;
pub mod artifact_sandbox;
pub mod connection;
pub mod fleet;
pub mod host_identity;
pub mod panels;
pub mod preferences_store;
pub mod profile;
pub mod profile_service;
pub mod profile_store;
pub mod ssh;

#[cfg(target_os = "macos")]
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    use crate::connection::{
        ConnectionStatus, HttpRelayProbe, RemoteTransport, SshRemoteTransport,
        attention_request_path, attention_state_request_path, connect_remote, desktop_panel_url,
        discover_local_endpoint, fleet_projects_path, fleet_snapshot_path, mark_tunnel_lost,
    };
    use crate::fleet::{
        DesktopFleetSnapshotV1, FleetSourceInput, LOCAL_SOURCE_ID, LOCAL_SOURCE_LABEL, SourceKind,
        project_source, source_without_snapshot, unavailable_source,
    };
    use crate::host_identity::{
        HostIdentityStatus, HostIdentityTicket, SystemHostKeyCommandRunner, approve_host_identity,
        check_host_identity,
    };
    use crate::panels::{PanelRegistry, is_browser_target};
    use crate::preferences_store::{BrowserPanelStateV1, DesktopPreferences, PreferencesStore};
    use crate::profile::{ConnectionProfile, ProfileDraft};
    use crate::profile_service::ProfileService;
    use crate::profile_store::{ClientIdentityStore, ProfileStore};
    use std::collections::{HashMap, HashSet};
    use std::io::{Read, Write};
    use std::net::{Ipv4Addr, SocketAddr, TcpStream};
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::Duration;
    use tauri::webview::{PageLoadEvent, WebviewBuilder};
    use tauri::{Emitter, Manager, PhysicalPosition, PhysicalSize, State, WebviewUrl};
    use tauri_plugin_notification::NotificationExt;

    struct DesktopState {
        profiles: Mutex<ProfileService>,
        client_id: String,
        preferences: Mutex<PreferencesStore>,
        known_hosts: PathBuf,
        pending_host_tickets: Mutex<HashMap<String, HostIdentityTicket>>,
        sessions: Mutex<HashMap<String, ActiveConnection>>,
        panels: Mutex<PanelRegistry>,
        notified_attention_events: Mutex<HashSet<String>>,
    }

    const DESKTOP_SHELL_WIDTH: u32 = 200;
    const BROWSER_CHROME_HEIGHT: u32 = 52;
    const BROWSER_CHROME_LABEL: &str = "browser-chrome";

    struct ActiveConnection {
        session: crate::profile::ConnectionSession,
        transport: Option<SshRemoteTransport>,
        local_port: u16,
    }

    #[derive(Clone)]
    struct FleetReadTarget {
        input: FleetSourceInput,
        local_port: u16,
        endpoint: crate::profile::RelayEndpoint,
    }

    #[derive(Debug, serde::Deserialize)]
    struct AttentionResponse {
        events: Vec<AttentionEvent>,
    }

    #[derive(Debug, serde::Deserialize)]
    struct AttentionEvent {
        event_id: String,
        display_name: String,
        summary: String,
        attention_enabled: bool,
        seen: bool,
        native_notified: bool,
    }

    #[derive(serde::Serialize)]
    struct PanelView {
        label: String,
        source_id: String,
        open_panels: usize,
    }

    #[derive(serde::Serialize)]
    struct HostIdentityView {
        status: &'static str,
        fingerprint: Option<String>,
        ticket: Option<String>,
    }

    #[derive(serde::Serialize)]
    struct DesktopAboutView {
        version: &'static str,
        update_status: &'static str,
    }

    fn as_message(error: impl std::fmt::Display) -> String {
        error.to_string()
    }

    fn content_security_snapshot(preferences: &DesktopPreferences) -> serde_json::Value {
        serde_json::json!({
            "externalLinks": preferences.content_security.external_links,
            "fileReferences": preferences.content_security.file_references,
            "remoteImages": preferences.content_security.remote_images,
        })
    }

    fn content_security_script(preferences: &DesktopPreferences) -> Result<String, String> {
        let encoded =
            serde_json::to_string(&content_security_snapshot(preferences)).map_err(as_message)?;
        Ok(format!(
            "Object.defineProperty(window, '__BRIDGET_CONTENT_SECURITY__', {{ value: Object.freeze({encoded}), writable: false, configurable: false }}); Object.defineProperty(window, '__BRIDGET_DESKTOP_SHELL__', {{ value: true, writable: false, configurable: false }});"
        ))
    }

    fn approved_external_https_url(value: &str) -> Option<tauri::Url> {
        let url = value.parse::<tauri::Url>().ok()?;
        if url.scheme() != "https"
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return None;
        }
        Some(url)
    }

    fn reload_panels_after_preferences_save(app: &tauri::AppHandle, state: &DesktopState) {
        let labels = match state.panels.lock() {
            Ok(panels) => panels
                .panels()
                .map(|panel| panel.label.clone())
                .collect::<Vec<_>>(),
            Err(_) => return,
        };
        for label in labels {
            if let Some(webview) = app.get_webview(&label) {
                // Un événement DOM serait forgeable par le document relayé.
                // Le rechargement exécute à nouveau le script d'initialisation
                // Tauri avant les scripts du panneau, avec le snapshot local.
                let _ = webview.eval("window.location.reload();");
            }
        }
    }

    fn publish_connection_state(app: &tauri::AppHandle, status: &ConnectionStatus) {
        let _ = app.emit_to("main", "connection-state", status);
    }

    fn request_relay_json(
        local_port: u16,
        method: &str,
        path: &str,
        body: Option<&[u8]>,
    ) -> Result<Vec<u8>, ()> {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, local_port));
        let timeout = Duration::from_secs(2);
        let mut stream = TcpStream::connect_timeout(&address, timeout).map_err(|_| ())?;
        stream.set_read_timeout(Some(timeout)).map_err(|_| ())?;
        let content_length = body.map_or(0, <[u8]>::len);
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {content_length}\r\n\r\n"
        );
        stream.write_all(request.as_bytes()).map_err(|_| ())?;
        if let Some(body) = body {
            stream.write_all(body).map_err(|_| ())?;
        }
        let mut response = Vec::new();
        stream.read_to_end(&mut response).map_err(|_| ())?;
        let header_end = response
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|index| index + 4)
            .ok_or(())?;
        if !response.starts_with(b"HTTP/1.1 2") {
            return Err(());
        }
        Ok(response[header_end..].to_vec())
    }

    fn fleet_connection_state(state: &crate::profile::ConnectionState) -> String {
        match state {
            crate::profile::ConnectionState::Disconnected => "disconnected",
            crate::profile::ConnectionState::ConnectingSsh => "connecting",
            crate::profile::ConnectionState::AwaitingHostApproval => "awaiting_host_approval",
            crate::profile::ConnectionState::OpeningTunnel => "opening_tunnel",
            crate::profile::ConnectionState::CheckingRelay => "checking_relay",
            crate::profile::ConnectionState::Connected => "connected",
            crate::profile::ConnectionState::Reconnecting => "reconnecting",
            crate::profile::ConnectionState::Failed => "failed",
            crate::profile::ConnectionState::Closed => "closed",
        }
        .to_owned()
    }

    fn read_fleet_target(target: FleetReadTarget) -> crate::fleet::SourceProjection {
        let snapshot_path = fleet_snapshot_path(&target.endpoint);
        let snapshot = match request_relay_json(target.local_port, "GET", &snapshot_path, None) {
            Ok(body) => body,
            Err(_) => {
                return unavailable_source(
                    target.input,
                    "La lecture de cette source est indisponible.",
                );
            }
        };
        let projects_path = fleet_projects_path(&target.endpoint);
        let projects = match request_relay_json(target.local_port, "GET", &projects_path, None) {
            Ok(body) => body,
            Err(_) => {
                return source_without_snapshot(
                    target.input,
                    Some("La liste des projets de cette source est indisponible.".to_owned()),
                );
            }
        };
        match project_source(target.input.clone(), &snapshot, &projects) {
            Ok(projection) => projection,
            Err(error) => source_without_snapshot(
                FleetSourceInput {
                    source_id: target.input.source_id,
                    label: target.input.label,
                    kind: target.input.kind,
                    connection_state: "failed".to_owned(),
                },
                Some(error.to_string()),
            ),
        }
    }

    fn connected_remote_targets(
        profiles: &[ConnectionProfile],
        sessions: &HashMap<String, ActiveConnection>,
    ) -> (Vec<FleetReadTarget>, Vec<crate::fleet::SourceProjection>) {
        let mut targets = Vec::new();
        let mut inactive = Vec::new();
        for profile in profiles {
            let input = FleetSourceInput {
                source_id: profile.id().to_owned(),
                label: profile.label().to_owned(),
                kind: SourceKind::Ssh,
                connection_state: sessions
                    .get(profile.id())
                    .map(|active| fleet_connection_state(&active.session.state))
                    .unwrap_or_else(|| "disconnected".to_owned()),
            };
            if let Some((active, endpoint)) = sessions
                .get(profile.id())
                .filter(|active| active.session.state == crate::profile::ConnectionState::Connected)
                .and_then(|active| {
                    active
                        .session
                        .endpoint()
                        .cloned()
                        .map(|endpoint| (active, endpoint))
                })
            {
                targets.push(FleetReadTarget {
                    input,
                    local_port: active.local_port,
                    endpoint,
                });
                continue;
            }
            inactive.push(source_without_snapshot(input, None));
        }
        (targets, inactive)
    }

    fn read_attention(
        local_port: u16,
        endpoint: &crate::profile::RelayEndpoint,
        client_id: &str,
    ) -> Result<Vec<AttentionEvent>, ()> {
        let body = request_relay_json(
            local_port,
            "GET",
            &attention_request_path(endpoint, client_id),
            None,
        )?;
        serde_json::from_slice::<AttentionResponse>(&body)
            .map(|response| response.events)
            .map_err(|_| ())
    }

    fn mark_native_attention(
        local_port: u16,
        endpoint: &crate::profile::RelayEndpoint,
        client_id: &str,
        event_id: &str,
    ) {
        let body = serde_json::to_vec(&serde_json::json!({
            "version": 1,
            "client_id": client_id,
            "event_ids": [event_id],
            "action": "mark_native_notified",
        }));
        if let Ok(body) = body {
            let _ = request_relay_json(
                local_port,
                "POST",
                &attention_state_request_path(endpoint),
                Some(&body),
            );
        }
    }

    fn watch_attention(app: tauri::AppHandle, profile_id: String) {
        std::thread::spawn(move || {
            loop {
                let Some(state) = app.try_state::<DesktopState>() else {
                    return;
                };
                let connection = match state.sessions.lock() {
                    Ok(sessions) => sessions.get(&profile_id).and_then(|active| {
                        active
                            .session
                            .endpoint()
                            .cloned()
                            .map(|endpoint| (active.local_port, endpoint))
                    }),
                    Err(_) => return,
                };
                let Some((local_port, endpoint)) = connection else {
                    return;
                };
                if let Ok(events) = read_attention(local_port, &endpoint, &state.client_id) {
                    for event in events.into_iter().filter(|event| {
                        event.attention_enabled
                            && !event.seen
                            && !event.native_notified
                            && !event.event_id.is_empty()
                    }) {
                        let should_notify = state
                            .notified_attention_events
                            .lock()
                            .map(|mut ids| ids.insert(event.event_id.clone()))
                            .unwrap_or(false);
                        if should_notify
                            && app
                                .notification()
                                .builder()
                                .title(event.display_name)
                                .body(event.summary)
                                .show()
                                .is_ok()
                        {
                            mark_native_attention(
                                local_port,
                                &endpoint,
                                &state.client_id,
                                &event.event_id,
                            );
                        }
                    }
                }
                std::thread::sleep(Duration::from_secs(5));
            }
        });
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

    fn browser_data_store_identifier(generation: u64) -> [u8; 16] {
        let mut identifier = [0_u8; 16];
        identifier[..8].copy_from_slice(&generation.to_be_bytes());
        identifier[8..].copy_from_slice(b"bridget!");
        identifier
    }

    fn browser_preferences(state: &DesktopState) -> Result<BrowserPanelStateV1, String> {
        state
            .preferences
            .lock()
            .map_err(as_message)?
            .load()
            .map(|preferences| preferences.browser_panel)
            .map_err(as_message)
    }

    fn save_browser_preferences(
        state: &DesktopState,
        update: impl FnOnce(&mut BrowserPanelStateV1),
    ) -> Result<BrowserPanelStateV1, String> {
        let store = state.preferences.lock().map_err(as_message)?;
        let mut preferences = store.load().map_err(as_message)?;
        update(&mut preferences.browser_panel);
        store.save(preferences.clone()).map_err(as_message)?;
        Ok(preferences.browser_panel)
    }

    fn sync_browser_chrome_url(app: &tauri::AppHandle, url: &str) {
        let Ok(url) = serde_json::to_string(url) else {
            return;
        };
        if let Some(chrome) = app.get_webview(BROWSER_CHROME_LABEL) {
            let _ = chrome.eval(format!("window.__BRIDGET_BROWSER_SET_URL?.({url});"));
        }
    }

    fn create_browser_chrome(app: &tauri::AppHandle) -> Result<(), String> {
        if app.get_webview(BROWSER_CHROME_LABEL).is_some() {
            return Ok(());
        }
        let main = main_window(app)?;
        let navigation_app = app.clone();
        let child = WebviewBuilder::new(
            BROWSER_CHROME_LABEL,
            WebviewUrl::App("browser-chrome.html".into()),
        )
        .on_navigation(move |url| {
            if url.scheme() == "tauri" {
                return true;
            }
            if url.scheme() != "bridget-browser" {
                return false;
            }
            let action = url.host_str().unwrap_or_default();
            let Some(state) = navigation_app.try_state::<DesktopState>() else {
                return false;
            };
            match action {
                "open" => {
                    let target = url
                        .query_pairs()
                        .find_map(|(key, value)| (key == "url").then(|| value.into_owned()))
                        .and_then(|value| approved_external_https_url(&value));
                    if let Some(target) = target {
                        let _ = open_browser_surface(&navigation_app, &state, target.as_str());
                    }
                }
                "back" => {
                    if let Some(browser) = navigation_app.get_webview("browser-primary") {
                        let _ = browser.eval("history.back();");
                    }
                }
                "forward" => {
                    if let Some(browser) = navigation_app.get_webview("browser-primary") {
                        let _ = browser.eval("history.forward();");
                    }
                }
                "reload" => {
                    if let Some(browser) = navigation_app.get_webview("browser-primary") {
                        let _ = browser.reload();
                    }
                }
                "maximize" => {
                    let _ = save_browser_preferences(&state, |panel| {
                        panel.right_panel_visible = true;
                        panel.right_panel_maximized = !panel.right_panel_maximized;
                    });
                    let _ = arrange_panels(&navigation_app, &state);
                }
                _ => {}
            }
            false
        });
        main.add_child(
            child,
            PhysicalPosition::new(0_i32, 0_i32),
            PhysicalSize::new(1_u32, 1_u32),
        )
        .map(|_| ())
        .map_err(as_message)
    }

    fn create_browser_content(
        app: &tauri::AppHandle,
        label: String,
        initial_url: WebviewUrl,
        preferences: &BrowserPanelStateV1,
    ) -> Result<(), String> {
        let main = main_window(app)?;
        let page_load_app = app.clone();
        let child = WebviewBuilder::new(label, initial_url)
            .data_store_identifier(browser_data_store_identifier(
                preferences.browser_profile_generation,
            ))
            .on_navigation(|url| {
                (url.scheme() == "https" && url.host_str().is_some())
                    || (url.scheme() == "http" && url.host_str() == Some("127.0.0.1"))
                    || url.scheme() == "tauri"
            })
            .on_page_load(move |_webview, payload| {
                if payload.event() == PageLoadEvent::Finished {
                    sync_browser_chrome_url(&page_load_app, payload.url().as_str());
                }
            });
        main.add_child(
            child,
            PhysicalPosition::new(0_i32, 0_i32),
            PhysicalSize::new(1_u32, 1_u32),
        )
        .map(|_| ())
        .map_err(as_message)
    }

    fn open_browser_surface(
        app: &tauri::AppHandle,
        state: &DesktopState,
        target: &str,
    ) -> Result<(), String> {
        if !is_browser_target(target) || !target.starts_with("https://") {
            return Err("La cible Browser doit être une URL HTTPS approuvée.".to_owned());
        }
        let url = target
            .parse::<tauri::Url>()
            .map_err(|_| "URL Browser invalide.".to_owned())?;
        let browser = {
            let mut panels = state.panels.lock().map_err(as_message)?;
            panels.open_browser(target).map_err(as_message)?
        };
        let preferences =
            save_browser_preferences(state, |panel| panel.right_panel_visible = true)?;
        create_browser_chrome(app)?;
        if let Some(webview) = app.get_webview(&browser.label) {
            webview.navigate(url).map_err(as_message)?;
        } else {
            create_browser_content(app, browser.label, WebviewUrl::External(url), &preferences)?;
        }
        sync_browser_chrome_url(app, target);
        arrange_panels(app, state)
    }

    fn open_browser_home_surface(
        app: &tauri::AppHandle,
        state: &DesktopState,
    ) -> Result<(), String> {
        let browser = {
            let mut panels = state.panels.lock().map_err(as_message)?;
            if let Some(browser) = panels.browser().cloned() {
                browser
            } else {
                panels
                    .open_browser("bridget://browser-home")
                    .map_err(as_message)?
            }
        };
        let preferences =
            save_browser_preferences(state, |panel| panel.right_panel_visible = true)?;
        create_browser_chrome(app)?;
        if app.get_webview(&browser.label).is_none() {
            create_browser_content(
                app,
                browser.label,
                WebviewUrl::App("browser-home.html".into()),
                &preferences,
            )?;
        }
        sync_browser_chrome_url(app, "");
        arrange_panels(app, state)
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
            arrange_panels(app, state)?;
        }
        Ok(())
    }

    fn close_open_panels(app: &tauri::AppHandle, state: &DesktopState) -> Result<(), String> {
        let panels = {
            let mut panels = state.panels.lock().map_err(as_message)?;
            let labels = panels
                .panels()
                .map(|panel| panel.label.clone())
                .collect::<Vec<_>>();
            labels
                .into_iter()
                .filter_map(|label| panels.close(&label))
                .collect::<Vec<_>>()
        };
        for panel in panels {
            if let Some(webview) = app.get_webview(&panel.label) {
                webview.close().map_err(as_message)?;
            }
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

    fn arrange_panels(app: &tauri::AppHandle, state: &DesktopState) -> Result<(), String> {
        let main = main_window(app)?;
        let size = main.inner_size().map_err(as_message)?;
        let (relay, browser) = {
            let panels = state.panels.lock().map_err(as_message)?;
            (panels.panels().next().cloned(), panels.browser().cloned())
        };
        let Some(panel) = relay else {
            return Ok(());
        };
        let browser_preferences = browser_preferences(state)?;
        let browser_available_width = size.width.saturating_sub(DESKTOP_SHELL_WIDTH);
        let browser_width = if browser_preferences.right_panel_visible && browser.is_some() {
            if browser_preferences.right_panel_maximized {
                browser_available_width.max(1)
            } else {
                browser_available_width.clamp(320, 720)
            }
        } else {
            0
        };
        let relay_width = if browser_preferences.right_panel_maximized && browser_width > 0 {
            1
        } else {
            browser_available_width.saturating_sub(browser_width).max(1)
        };
        if let Some(webview) = app.get_webview(&panel.label) {
            webview
                .set_position(PhysicalPosition::new(DESKTOP_SHELL_WIDTH as i32, 0_i32))
                .map_err(as_message)?;
            webview
                .set_size(PhysicalSize::new(relay_width, size.height.max(1)))
                .map_err(as_message)?;
        }
        if let Some(browser) = browser {
            let browser_x = DESKTOP_SHELL_WIDTH.saturating_add(relay_width) as i32;
            let content_y = BROWSER_CHROME_HEIGHT.min(size.height) as i32;
            let content_height = size.height.saturating_sub(BROWSER_CHROME_HEIGHT).max(1);
            if let Some(chrome) = app.get_webview(BROWSER_CHROME_LABEL) {
                chrome
                    .set_position(PhysicalPosition::new(browser_x, 0_i32))
                    .map_err(as_message)?;
                chrome
                    .set_size(PhysicalSize::new(
                        browser_width.max(1),
                        BROWSER_CHROME_HEIGHT.min(size.height).max(1),
                    ))
                    .map_err(as_message)?;
            }
            if let Some(webview) = app.get_webview(&browser.label) {
                webview
                    .set_position(PhysicalPosition::new(browser_x, content_y))
                    .map_err(as_message)?;
                webview
                    .set_size(PhysicalSize::new(browser_width.max(1), content_height))
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

    /// Préférences du Mac uniquement. Elles ne sont jamais remises au tunnel
    /// SSH, ni sérialisées dans un profil de serveur.
    #[tauri::command(rename_all = "snake_case")]
    fn preferences_get(state: State<'_, DesktopState>) -> Result<DesktopPreferences, String> {
        state
            .preferences
            .lock()
            .map_err(as_message)?
            .load()
            .map_err(as_message)
    }

    #[tauri::command(rename_all = "snake_case")]
    fn preferences_save(
        app: tauri::AppHandle,
        state: State<'_, DesktopState>,
        preferences: DesktopPreferences,
    ) -> Result<DesktopPreferences, String> {
        state
            .preferences
            .lock()
            .map_err(as_message)?
            .save(preferences.clone())
            .map_err(as_message)?;
        reload_panels_after_preferences_save(&app, &state);
        arrange_panels(&app, &state)?;
        Ok(preferences)
    }

    /// Agrège des copies de sessions déjà établies. Aucun verrou n'est conservé
    /// pendant les lectures HTTP des relais, afin qu'une source lente n'empêche
    /// ni une autre source ni la fermeture d'un tunnel.
    #[tauri::command(rename_all = "snake_case")]
    fn fleet_snapshot(state: State<'_, DesktopState>) -> Result<DesktopFleetSnapshotV1, String> {
        let profiles = state
            .profiles
            .lock()
            .map_err(as_message)?
            .list()
            .map_err(as_message)?;
        let (targets, mut projections) = {
            let sessions = state.sessions.lock().map_err(as_message)?;
            connected_remote_targets(&profiles, &sessions)
        };
        projections.extend(targets.into_iter().map(read_fleet_target));

        // La source locale n'existe dans la réponse que si le relais est
        // réellement joignable et que sa flotte peut être lue.
        if let Ok(endpoint) = discover_local_endpoint() {
            let local = read_fleet_target(FleetReadTarget {
                input: FleetSourceInput {
                    source_id: LOCAL_SOURCE_ID.to_owned(),
                    label: LOCAL_SOURCE_LABEL.to_owned(),
                    kind: SourceKind::Local,
                    connection_state: "connected".to_owned(),
                },
                local_port: endpoint.port,
                endpoint,
            });
            if local.source.error.is_none() {
                projections.push(local);
            }
        }
        Ok(DesktopFleetSnapshotV1::from_sources(projections))
    }

    #[tauri::command(rename_all = "snake_case")]
    fn desktop_about() -> DesktopAboutView {
        DesktopAboutView {
            version: env!("CARGO_PKG_VERSION"),
            update_status: "not_configured",
        }
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
        if let Some(mut transport) = state
            .sessions
            .lock()
            .map_err(as_message)?
            .remove(&profile_id)
            .and_then(|previous| previous.transport)
        {
            transport.close();
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
        watch_remote_tunnel(app.clone(), monitor_id);
        watch_attention(app, profile.id().to_owned());
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
        source_id: String,
        agent_name: Option<String>,
        project_id: Option<String>,
        desktop_action: Option<String>,
    ) -> Result<PanelView, String> {
        let (local_port, endpoint) = if source_id == LOCAL_SOURCE_ID {
            let endpoint = discover_local_endpoint()
                .map_err(|_| "Le relais de cet ordinateur est indisponible.".to_owned())?;
            let path = fleet_snapshot_path(&endpoint);
            request_relay_json(endpoint.port, "GET", &path, None)
                .map_err(|_| "Le relais de cet ordinateur ne répond pas.".to_owned())?;
            (endpoint.port, endpoint)
        } else {
            let sessions = state.sessions.lock().map_err(as_message)?;
            let active = sessions
                .get(&source_id)
                .ok_or_else(|| "Ce serveur n'est pas encore connecté.".to_owned())?;
            if active.session.state != crate::profile::ConnectionState::Connected {
                return Err("Ce serveur n'est pas encore connecté.".to_owned());
            }
            let endpoint = active
                .session
                .endpoint()
                .cloned()
                .ok_or_else(|| "Le relais connecté ne fournit pas d'endpoint.".to_owned())?;
            (active.local_port, endpoint)
        };
        let url = desktop_panel_url(
            local_port,
            &endpoint,
            &state.client_id,
            agent_name.as_deref(),
            project_id.as_deref(),
            desktop_action.as_deref(),
        );
        close_open_panels(&app, &state)?;
        let panel = {
            let mut panels = state.panels.lock().map_err(as_message)?;
            panels.open(source_id.clone(), url).map_err(as_message)?
        };
        let main = main_window(&app)?;
        let external_url = panel
            .url
            .parse()
            .map_err(|_| "L'URL du relais local est invalide.")?;
        let preferences = state
            .preferences
            .lock()
            .map_err(as_message)?
            .load()
            .map_err(as_message)?;
        let security_script = content_security_script(&preferences)?;
        let navigation_app = app.clone();
        let child = WebviewBuilder::new(panel.label.clone(), WebviewUrl::External(external_url))
            .initialization_script(security_script)
            .on_navigation(move |url| {
                if url.scheme() == "http" && url.host_str() == Some("127.0.0.1") {
                    return true;
                }
                if url.scheme() == "bridget-open" {
                    let destination = url
                        .query_pairs()
                        .find_map(|(key, value)| (key == "url").then(|| value.into_owned()))
                        .and_then(|value| approved_external_https_url(&value));
                    if let Some(destination) = destination
                        && let Some(state) = navigation_app.try_state::<DesktopState>()
                    {
                        let _ = open_browser_surface(&navigation_app, &state, destination.as_str());
                    }
                }
                if url.scheme() == "bridget-panel"
                    && let Some(state) = navigation_app.try_state::<DesktopState>()
                {
                    let action = url
                        .query_pairs()
                        .find_map(|(key, value)| (key == "action").then(|| value.into_owned()));
                    let _ = match action.as_deref() {
                        Some("toggle") => {
                            let missing_browser = state
                                .panels
                                .lock()
                                .map(|panels| panels.browser().is_none())
                                .unwrap_or(false);
                            if missing_browser {
                                let _ = open_browser_home_surface(&navigation_app, &state);
                            }
                            save_browser_preferences(&state, |panel| {
                                panel.right_panel_visible = if missing_browser {
                                    true
                                } else {
                                    !panel.right_panel_visible
                                };
                            })
                        }
                        Some("maximize") => save_browser_preferences(&state, |panel| {
                            panel.right_panel_visible = true;
                            panel.right_panel_maximized = !panel.right_panel_maximized;
                        }),
                        _ => return false,
                    };
                    let _ = arrange_panels(&navigation_app, &state);
                }
                false
            });
        if let Err(error) = main.add_child(
            child,
            PhysicalPosition::new(0_i32, 0_i32),
            PhysicalSize::new(1_u32, 1_u32),
        ) {
            state.panels.lock().map_err(as_message)?.close(&panel.label);
            return Err(as_message(error));
        }
        // Le choix de garder le Browser ouvert est une préférence durable,
        // alors que son WebView disparaît à l'arrêt du processus. On le
        // recrée donc avec la conversation plutôt que de laisser un état
        // « visible » sans surface réelle.
        if browser_preferences(&state)?.right_panel_visible {
            open_browser_home_surface(&app, &state)?;
        }
        arrange_panels(&app, &state)?;
        let panels = state.panels.lock().map_err(as_message)?;
        Ok(PanelView {
            label: panel.label,
            source_id,
            open_panels: panels.panels().count(),
        })
    }

    #[tauri::command(rename_all = "snake_case")]
    fn panel_close(
        app: tauri::AppHandle,
        state: State<'_, DesktopState>,
        source_id: String,
    ) -> Result<(), String> {
        close_panel_for_profile(&app, &state, &source_id)
    }

    /// Commande de la coque locale uniquement. Une page de conversation ne
    /// reçoit pas de capability IPC et ne peut donc pas l'invoquer elle-même.
    #[tauri::command(rename_all = "snake_case")]
    fn browser_open(
        app: tauri::AppHandle,
        state: State<'_, DesktopState>,
        target: String,
    ) -> Result<(), String> {
        open_browser_surface(&app, &state, &target)
    }

    #[tauri::command(rename_all = "snake_case")]
    fn browser_set_panel_state(
        app: tauri::AppHandle,
        state: State<'_, DesktopState>,
        visible: Option<bool>,
        maximized: Option<bool>,
        active_tab: Option<String>,
    ) -> Result<BrowserPanelStateV1, String> {
        let panel = save_browser_preferences(&state, |panel| {
            if let Some(visible) = visible {
                panel.right_panel_visible = visible;
            }
            if let Some(maximized) = maximized {
                panel.right_panel_maximized = maximized;
            }
            if let Some(active_tab) = active_tab {
                panel.active_tab = active_tab;
            }
        })?;
        arrange_panels(&app, &state)?;
        Ok(panel)
    }

    #[tauri::command(rename_all = "snake_case")]
    fn browser_clear_data(
        app: tauri::AppHandle,
        state: State<'_, DesktopState>,
    ) -> Result<BrowserPanelStateV1, String> {
        if let Some(webview) = app.get_webview("browser-primary") {
            webview.clear_all_browsing_data().map_err(as_message)?;
        }
        save_browser_preferences(&state, |panel| {
            panel.browser_profile_generation = panel.browser_profile_generation.saturating_add(1);
        })
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("Répertoire applicatif indisponible : {error}"))?;
            let client_id = ClientIdentityStore::new(app_data_dir.join("client-id.json"))
                .load_or_create()
                .map_err(as_message)?;
            app.manage(DesktopState {
                profiles: Mutex::new(ProfileService::new(ProfileStore::new(
                    app_data_dir.join("profiles.json"),
                ))),
                preferences: Mutex::new(PreferencesStore::new(
                    app_data_dir.join("preferences.json"),
                )),
                client_id,
                known_hosts: app_data_dir.join("known_hosts"),
                pending_host_tickets: Mutex::new(HashMap::new()),
                sessions: Mutex::new(HashMap::new()),
                panels: Mutex::new(PanelRegistry::default()),
                notified_attention_events: Mutex::new(HashSet::new()),
            });
            let handle = app.handle().clone();
            let main = main_window(&handle)?;
            main.show().map_err(as_message)?;
            main.set_focus().map_err(as_message)?;

            main.on_window_event(move |event| {
                if matches!(event, tauri::WindowEvent::Resized(_))
                    && let Some(state) = handle.try_state::<DesktopState>()
                {
                    let _ = arrange_panels(&handle, state.inner());
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            profiles_list,
            profile_save,
            profile_delete,
            preferences_get,
            preferences_save,
            fleet_snapshot,
            desktop_about,
            host_identity_check,
            host_identity_approve,
            connection_open,
            connection_close,
            panel_open,
            panel_close,
            browser_open,
            browser_set_panel_state,
            browser_clear_data
        ])
        .run(tauri::generate_context!())
        .expect("Bridget Desktop n'a pas pu démarrer");
}

#[cfg(not(target_os = "macos"))]
pub fn run() {
    panic!("Bridget Desktop doit être exécuté sur macOS");
}
