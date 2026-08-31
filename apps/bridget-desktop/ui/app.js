const elements = {
  add: document.querySelector("#add-profile"), dialog: document.querySelector("#profile-dialog"), close: document.querySelector("#close-dialog"), cancel: document.querySelector("#cancel-profile"), form: document.querySelector("#profile-form"), list: document.querySelector("#profiles-list"), empty: document.querySelector("#empty-state"), status: document.querySelector("#app-status"), connectionStatus: document.querySelector("#connection-status"), showProfiles: document.querySelector("#show-profiles"), activePanels: document.querySelector("#active-panels"), loadError: document.querySelector("#load-error"), formError: document.querySelector("#form-error"), title: document.querySelector("#profile-dialog-title"), id: document.querySelector("#profile-id"), label: document.querySelector("#profile-label"), host: document.querySelector("#profile-host"), port: document.querySelector("#profile-port"), user: document.querySelector("#profile-user"), identityFileField: document.querySelector("#identity-file-field"), identityFile: document.querySelector("#profile-identity-file"), hostIdentityDialog: document.querySelector("#host-identity-dialog"), hostIdentityServer: document.querySelector("#host-identity-server"), hostIdentityFingerprint: document.querySelector("#host-identity-fingerprint"),
  showPreferences: document.querySelector("#show-preferences"), preferencesDialog: document.querySelector("#preferences-dialog"), preferencesForm: document.querySelector("#preferences-form"), closePreferences: document.querySelector("#close-preferences"), cancelPreferences: document.querySelector("#cancel-preferences"), preferencesDisplayName: document.querySelector("#preferences-display-name"), preferencesColorScheme: document.querySelector("#preferences-color-scheme"), preferencesTimezone: document.querySelector("#preferences-timezone"), preferencesFontSize: document.querySelector("#preferences-font-size"), preferencesExternalLinks: document.querySelector("#preferences-external-links"), preferencesFileReferences: document.querySelector("#preferences-file-references"), preferencesRemoteImages: document.querySelector("#preferences-remote-images"), preferencesError: document.querySelector("#preferences-error"), desktopVersion: document.querySelector("#desktop-version"), desktopUpdateStatus: document.querySelector("#desktop-update-status"),
};

let profiles = [];
const connectionStates = new Map();
const openPanelProfiles = new Set();
const restoredProfiles = new Set();

function invoke(command, payload = {}) { const api = window.__TAURI__?.core?.invoke; return api ? api(command, payload) : Promise.reject(new Error("Bridget Desktop doit être ouvert depuis l'application macOS.")); }
function announce(message) { elements.status.textContent = message; }
function visibleError(target, error) { target.textContent = error instanceof Error ? error.message : String(error); target.hidden = false; }
function clearError(target) { target.textContent = ""; target.hidden = true; }
function escapeHtml(value) { const element = document.createElement("span"); element.textContent = value; return element.innerHTML; }
function profileOrigin(profile) { return `SSH ${profile.user}@${profile.host}:${profile.port}`; }
function profileConnection(profile) { return connectionStates.get(profile.id)?.state ?? "disconnected"; }
function connectionDescription(profile) { const state = profileConnection(profile); if (state === "connected") return "Relais vérifié"; if (state === "reconnecting") return "Reconnexion du tunnel"; if (state === "failed") return "Tunnel interrompu - réessayez explicitement"; if (state === "checking_relay") return "Vérification du relais"; if (state === "opening_tunnel") return "Ouverture du tunnel SSH"; if (state === "connecting_ssh") return "Connexion SSH"; return "Prêt à connecter"; }

function renderActivePanels() {
  elements.activePanels.replaceChildren();
  for (const profileId of openPanelProfiles) {
    const profile = profiles.find((candidate) => candidate.id === profileId); if (!profile) continue;
    const badge = document.createElement("span"); badge.className = "active-panel-origin";
    badge.textContent = `${profile.label} - tunnel SSH`;
    elements.activePanels.append(badge);
  }
}

function renderProfiles() {
  elements.list.replaceChildren(); elements.empty.hidden = profiles.length !== 0;
  for (const profile of profiles) {
    const item = document.createElement("li"); item.className = "profile-card";
    const connected = profileConnection(profile) === "connected";
    const action = connected
      ? `<button type="button" data-action="open" data-profile-id="${escapeHtml(profile.id)}">Ouvrir le relais</button><button type="button" class="secondary" data-action="disconnect" data-profile-id="${escapeHtml(profile.id)}">Déconnecter</button>`
      : `<button type="button" data-action="connect" data-profile-id="${escapeHtml(profile.id)}">${profileConnection(profile) === "failed" ? "Réessayer" : "Connecter"}</button>`;
    item.innerHTML = `<div><p class="profile-kind">TUNNEL SSH GÉRÉ</p><h3>${escapeHtml(profile.label)}</h3><p class="profile-origin">${escapeHtml(profileOrigin(profile))}</p><p class="connection-badge" data-state="${escapeHtml(profileConnection(profile))}">${escapeHtml(connectionDescription(profile))}</p></div><div class="profile-actions">${action}<button type="button" class="secondary" data-action="edit" data-profile-id="${escapeHtml(profile.id)}">Modifier</button><button type="button" class="danger" data-action="delete" data-profile-id="${escapeHtml(profile.id)}">Retirer</button></div>`;
    elements.list.append(item);
  }
}

async function restoreManagedConnections() {
  for (const profile of profiles.slice(0, 2)) {
    if (!profile.host_fingerprint || restoredProfiles.has(profile.id) || profileConnection(profile) === "connected") continue;
    restoredProfiles.add(profile.id);
    await connectProfile(profile, true);
  }
}

async function refreshProfiles({ restore = true } = {}) {
  clearError(elements.loadError);
  try {
    profiles = await invoke("profiles_list"); renderProfiles();
    announce(`${profiles.length} profil${profiles.length > 1 ? "s" : ""} chargé${profiles.length > 1 ? "s" : ""}.`);
    if (restore) void restoreManagedConnections();
  } catch (error) { visibleError(elements.loadError, error); }
}

function selectedIdentitySource() { return document.querySelector("input[name='identity-source']:checked")?.value; }
function syncIdentityField() { const file = selectedIdentitySource() === "file"; elements.identityFileField.hidden = !file; elements.identityFile.required = file; }
function resetForm(profile = null) {
  elements.form.reset(); elements.id.value = profile?.id ?? ""; elements.title.textContent = profile ? "Modifier un serveur" : "Ajouter un serveur";
  elements.label.value = profile?.label ?? ""; elements.host.value = profile?.host ?? ""; elements.port.value = profile?.port ?? 22; elements.user.value = profile?.user ?? "";
  const source = profile?.identity?.source ?? "agent"; document.querySelector(`input[name='identity-source'][value='${source}']`).checked = true;
  elements.identityFile.value = profile?.identity?.path ?? ""; syncIdentityField(); clearError(elements.formError);
}
function openProfileDialog(profile = null) { resetForm(profile); elements.dialog.showModal(); window.requestAnimationFrame(() => elements.label.focus()); }
function closeProfileDialog() { elements.dialog.close(); }
function makeDraft() { const identity = selectedIdentitySource() === "file" ? { source: "file", path: elements.identityFile.value.trim() } : { source: "agent" }; return { kind: "ssh", label: elements.label.value.trim(), host: elements.host.value.trim(), port: Number(elements.port.value), user: elements.user.value.trim(), identity }; }

function requestHostIdentityApproval(profile, fingerprint) {
  elements.hostIdentityServer.textContent = profileOrigin(profile);
  elements.hostIdentityFingerprint.textContent = fingerprint;
  elements.hostIdentityDialog.returnValue = "cancel";
  elements.hostIdentityDialog.showModal();
  return new Promise((resolve) => {
    elements.hostIdentityDialog.addEventListener("close", () => resolve(elements.hostIdentityDialog.returnValue === "approve"), { once: true });
  });
}

async function saveProfile(event) {
  event.preventDefault(); clearError(elements.formError); if (!elements.form.reportValidity()) return;
  try {
    const saved = await invoke("profile_save", { profile_id: elements.id.value || null, draft: makeDraft() });
    closeProfileDialog(); await refreshProfiles({ restore: false }); announce("Profil enregistré.");
    const profile = profiles.find((candidate) => candidate.id === saved.id);
    if (profile) void connectProfile(profile);
  } catch (error) { visibleError(elements.formError, error); }
}

async function deleteProfile(profile) {
  if (!window.confirm(`Retirer le profil « ${profile.label} » ? Cette action ferme sa future connexion éventuelle.`)) return;
  try { await invoke("profile_delete", { profile_id: profile.id, confirmed: true }); await refreshProfiles({ restore: false }); announce(`Profil ${profile.label} retiré.`); } catch (error) { visibleError(elements.loadError, error); }
}

async function verifyHostIdentity(profile) {
  const identity = await invoke("host_identity_check", { profile_id: profile.id });
  if (identity.status === "approved") return;
  if (identity.status === "changed") throw new Error(`L'identité SSH de ${profile.label} a changé (${identity.fingerprint}). La connexion est bloquée pour votre sécurité.`);
  if (identity.status !== "awaiting_approval" || !identity.ticket) throw new Error("Le contrôle d'identité SSH n'a pas produit de décision exploitable.");
  const accepted = await requestHostIdentityApproval(profile, identity.fingerprint);
  if (!accepted) throw new Error("L'identité SSH n'a pas été approuvée. Aucune connexion n'a été ouverte.");
  await invoke("host_identity_approve", { profile_id: profile.id, ticket: identity.ticket });
}

function setConnectionMessage(message) { elements.connectionStatus.textContent = message; announce(message); }
async function openPanel(profile, view = null) {
  const panel = await invoke("panel_open", view ? { profile_id: profile.id, view } : { profile_id: profile.id });
  openPanelProfiles.clear(); openPanelProfiles.add(panel.profile_id); document.body.classList.add("panel-view"); elements.showProfiles.hidden = false; renderActivePanels();
  setConnectionMessage(view === "settings"
    ? `Les réglages de ${profile.label} sont affichés dans un panneau local isolé.`
    : `${profile.label} est affiché dans un panneau local isolé.`);
}

async function connectProfile(profile, automatic = false, initialView = null) {
  clearError(elements.loadError);
  try {
    setConnectionMessage(`${automatic ? "Restauration du tunnel" : "Connexion à"} ${profile.label}…`);
    await verifyHostIdentity(profile);
    const status = await invoke("connection_open", { profile_id: profile.id });
    connectionStates.set(profile.id, status); renderProfiles(); await openPanel(profile, initialView);
  } catch (error) {
    connectionStates.set(profile.id, { state: "failed" }); renderProfiles(); visibleError(elements.loadError, error);
    setConnectionMessage(`Connexion à ${profile.label} non établie.`);
  }
}

async function disconnectProfile(profile) {
  try {
    await invoke("connection_close", { profile_id: profile.id }); connectionStates.delete(profile.id); openPanelProfiles.delete(profile.id);
    if (openPanelProfiles.size === 0) document.body.classList.remove("panel-view"); renderActivePanels(); renderProfiles(); setConnectionMessage(`${profile.label} est déconnecté.`);
  } catch (error) { visibleError(elements.loadError, error); }
}

async function showProfiles() {
  try {
    for (const profileId of [...openPanelProfiles]) await invoke("panel_close", { profile_id: profileId });
    openPanelProfiles.clear(); document.body.classList.remove("panel-view"); renderActivePanels();
    setConnectionMessage("Les panneaux sont masqués. Les tunnels restent ouverts jusqu'à la fermeture de Bridget Desktop ou leur déconnexion explicite.");
  } catch (error) { visibleError(elements.loadError, error); }
}

function applyLocalPreferences(preferences) {
  document.documentElement.dataset.colorScheme = preferences.color_scheme;
  document.documentElement.style.fontSize = `${preferences.font_size_px}px`;
  document.querySelector(".product-name span").textContent = preferences.display_name || "client";
}

function fillPreferencesForm(preferences) {
  elements.preferencesDisplayName.value = preferences.display_name || "";
  elements.preferencesColorScheme.value = preferences.color_scheme || "system";
  elements.preferencesTimezone.value = preferences.timezone || "system";
  elements.preferencesFontSize.value = String(preferences.font_size_px || 16);
  const content = preferences.content_security || {};
  elements.preferencesExternalLinks.checked = content.external_links === true;
  elements.preferencesFileReferences.checked = content.file_references === true;
  elements.preferencesRemoteImages.checked = content.remote_images === true;
  clearError(elements.preferencesError);
}

function localPreferencesDraft() {
  const timezone = elements.preferencesTimezone.value.trim() || "system";
  if (timezone !== "system") {
    try { Intl.DateTimeFormat(undefined, { timeZone: timezone }); }
    catch (_error) { throw new Error("Le fuseau doit être un identifiant IANA, par exemple Europe/Paris."); }
  }
  return {
    display_name: elements.preferencesDisplayName.value.trim(),
    color_scheme: elements.preferencesColorScheme.value,
    timezone,
    font_size_px: Number(elements.preferencesFontSize.value),
    content_security: {
      external_links: elements.preferencesExternalLinks.checked,
      file_references: elements.preferencesFileReferences.checked,
      remote_images: elements.preferencesRemoteImages.checked,
    },
  };
}

async function openPreferences() {
  try {
    const [preferences, about] = await Promise.all([invoke("preferences_get"), invoke("desktop_about")]);
    fillPreferencesForm(preferences); applyLocalPreferences(preferences);
    elements.desktopVersion.textContent = `Version ${about.version}`;
    elements.desktopUpdateStatus.textContent = about.update_status === "not_configured"
      ? "Vérification de mise à jour non configurée."
      : "État de mise à jour indisponible.";
    elements.preferencesDialog.showModal();
    window.requestAnimationFrame(() => elements.preferencesDisplayName.focus());
  } catch (error) { visibleError(elements.loadError, error); }
}

function closePreferences() { elements.preferencesDialog.close(); }

async function savePreferences(event) {
  event.preventDefault(); clearError(elements.preferencesError);
  try {
    const saved = await invoke("preferences_save", { preferences: localPreferencesDraft() });
    applyLocalPreferences(saved); closePreferences(); announce("Réglages locaux enregistrés sur ce Mac.");
  } catch (error) { visibleError(elements.preferencesError, error); }
}

async function restoreLocalPreferences() {
  try { applyLocalPreferences(await invoke("preferences_get")); } catch (_error) { /* l'ouverture des serveurs reste disponible */ }
}

elements.add.addEventListener("click", () => openProfileDialog()); elements.close.addEventListener("click", closeProfileDialog); elements.cancel.addEventListener("click", closeProfileDialog); elements.showProfiles.addEventListener("click", () => void showProfiles()); document.querySelectorAll("input[name='identity-source']").forEach((input) => input.addEventListener("change", syncIdentityField)); elements.form.addEventListener("submit", saveProfile);
elements.showPreferences.addEventListener("click", () => void openPreferences()); elements.closePreferences.addEventListener("click", closePreferences); elements.cancelPreferences.addEventListener("click", closePreferences); elements.preferencesForm.addEventListener("submit", savePreferences);
elements.list.addEventListener("click", (event) => {
  const button = event.target.closest("button[data-action]");
  if (!button) return;
  const profile = profiles.find((candidate) => candidate.id === button.dataset.profileId);
  if (!profile) return;
  if (button.dataset.action === "edit") openProfileDialog(profile);
  if (button.dataset.action === "delete") void deleteProfile(profile);
  if (button.dataset.action === "connect") void connectProfile(profile);
  if (button.dataset.action === "open") {
    void openPanel(profile).catch((error) => visibleError(elements.loadError, error));
  }
  if (button.dataset.action === "disconnect") void disconnectProfile(profile);
});

const eventApi = window.__TAURI__?.event;
if (eventApi?.listen) void eventApi.listen("connection-state", (event) => {
  const state = event.payload; if (!state?.profile_id) return;
  connectionStates.set(state.profile_id, state); renderProfiles();
  const profile = profiles.find((candidate) => candidate.id === state.profile_id);
  if (profile && state.state === "failed") setConnectionMessage(`Le tunnel de ${profile.label} s'est arrêté. Réessayez explicitement si nécessaire.`);
});

void restoreLocalPreferences();
void refreshProfiles();
