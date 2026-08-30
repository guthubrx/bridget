const elements = {
  add: document.querySelector("#add-profile"), dialog: document.querySelector("#profile-dialog"), close: document.querySelector("#close-dialog"), cancel: document.querySelector("#cancel-profile"), form: document.querySelector("#profile-form"), list: document.querySelector("#profiles-list"), empty: document.querySelector("#empty-state"), status: document.querySelector("#app-status"), connectionStatus: document.querySelector("#connection-status"), showProfiles: document.querySelector("#show-profiles"), activePanels: document.querySelector("#active-panels"), loadError: document.querySelector("#load-error"), formError: document.querySelector("#form-error"), title: document.querySelector("#profile-dialog-title"), id: document.querySelector("#profile-id"), kind: document.querySelector("#profile-kind"), label: document.querySelector("#profile-label"), host: document.querySelector("#profile-host"), port: document.querySelector("#profile-port"), user: document.querySelector("#profile-user"), sshFields: document.querySelector("#ssh-fields"), directFields: document.querySelector("#direct-fields"), directHost: document.querySelector("#profile-direct-host"), directPort: document.querySelector("#profile-direct-port"), directTokenDialog: document.querySelector("#direct-token-dialog"), directTokenForm: document.querySelector("#direct-token-form"), directToken: document.querySelector("#direct-token"), cancelDirectToken: document.querySelector("#cancel-direct-token"), identityFileField: document.querySelector("#identity-file-field"), identityFile: document.querySelector("#profile-identity-file"),
};
let profiles = [];
const connectionStates = new Map();
const openPanelProfiles = new Set();
function invoke(command, payload = {}) { const api = window.__TAURI__?.core?.invoke; return api ? api(command, payload) : Promise.reject(new Error("Bridget Desktop doit être ouvert depuis l'application macOS.")); }
function announce(message) { elements.status.textContent = message; }
function visibleError(target, error) { target.textContent = error instanceof Error ? error.message : String(error); target.hidden = false; }
function clearError(target) { target.textContent = ""; target.hidden = true; }
function escapeHtml(value) { const element = document.createElement("span"); element.textContent = value; return element.innerHTML; }
function profileOrigin(profile) { return profile.kind === "ssh" ? `SSH ${profile.user}@${profile.host}:${profile.port}` : `Direct ${profile.host}:${profile.relay_port}`; }
function profileConnection(profile) { return connectionStates.get(profile.id)?.state ?? "disconnected"; }
function connectionDescription(profile) { const state = profileConnection(profile); if (state === "connected") return "Relais vérifié"; if (state === "reconnecting") return "Reconnexion du tunnel"; if (state === "failed") return "Tunnel interrompu - réessayez explicitement"; if (state === "checking_relay") return "Vérification du relais"; return "Non connecté"; }
function renderActivePanels() {
  elements.activePanels.replaceChildren();
  for (const profileId of openPanelProfiles) {
    const profile = profiles.find((candidate) => candidate.id === profileId); if (!profile) continue;
    const badge = document.createElement("span"); badge.className = "active-panel-origin";
    badge.textContent = `${profile.label} - ${profile.kind === "ssh" ? "SSH" : "direct"}`;
    elements.activePanels.append(badge);
  }
}
function renderProfiles() {
  elements.list.replaceChildren(); elements.empty.hidden = profiles.length !== 0;
  for (const profile of profiles) {
    const item = document.createElement("li"); item.className = "profile-card";
    const connected = profileConnection(profile) === "connected";
    const action = connected ? `<button type="button" data-action="open" data-profile-id="${escapeHtml(profile.id)}">Ouvrir le relais</button><button type="button" class="secondary" data-action="disconnect" data-profile-id="${escapeHtml(profile.id)}">Déconnecter</button>` : `<button type="button" data-action="connect" data-profile-id="${escapeHtml(profile.id)}">${profileConnection(profile) === "failed" ? "Réessayer" : "Connecter"}</button>`;
    item.innerHTML = `<div><p class="profile-kind">${profile.kind === "ssh" ? "SSH" : "DIRECT"}</p><h3>${escapeHtml(profile.label)}</h3><p class="profile-origin">${escapeHtml(profileOrigin(profile))}</p><p class="connection-badge" data-state="${escapeHtml(profileConnection(profile))}">${escapeHtml(connectionDescription(profile))}</p></div><div class="profile-actions">${action}<button type="button" class="secondary" data-action="edit" data-profile-id="${escapeHtml(profile.id)}">Modifier</button><button type="button" class="danger" data-action="delete" data-profile-id="${escapeHtml(profile.id)}">Retirer</button></div>`;
    elements.list.append(item);
  }
}
async function refreshProfiles() { clearError(elements.loadError); try { profiles = await invoke("profiles_list"); renderProfiles(); announce(`${profiles.length} profil${profiles.length > 1 ? "s" : ""} chargé${profiles.length > 1 ? "s" : ""}.`); } catch (error) { visibleError(elements.loadError, error); } }
function selectedIdentitySource() { return document.querySelector("input[name='identity-source']:checked")?.value; }
function syncFormKind() { const ssh = elements.kind.value === "ssh"; elements.sshFields.hidden = !ssh; elements.directFields.hidden = ssh; elements.host.required = ssh; elements.user.required = ssh; elements.port.required = ssh; elements.directHost.required = !ssh; elements.directPort.required = !ssh; const file = selectedIdentitySource() === "file"; elements.identityFileField.hidden = !ssh || !file; elements.identityFile.required = ssh && file; }
function resetForm(profile = null) { elements.form.reset(); elements.id.value = profile?.id ?? ""; elements.title.textContent = profile ? "Modifier un serveur" : "Ajouter un serveur"; elements.kind.value = profile?.kind ?? "ssh"; elements.label.value = profile?.label ?? ""; elements.host.value = profile?.host ?? ""; elements.port.value = profile?.port ?? 22; elements.user.value = profile?.user ?? ""; elements.directHost.value = profile?.host ?? "127.0.0.1"; elements.directPort.value = profile?.relay_port ?? 17888; const source = profile?.identity?.source ?? "agent"; document.querySelector(`input[name='identity-source'][value='${source}']`).checked = true; elements.identityFile.value = profile?.identity?.path ?? ""; syncFormKind(); clearError(elements.formError); }
function openProfileDialog(profile = null) { resetForm(profile); elements.dialog.showModal(); window.requestAnimationFrame(() => elements.label.focus()); }
function closeProfileDialog() { elements.dialog.close(); }
function makeDraft() { if (elements.kind.value === "local") return { kind: "local", label: elements.label.value.trim(), host: elements.directHost.value.trim(), relay_port: Number(elements.directPort.value) }; const identity = selectedIdentitySource() === "file" ? { source: "file", path: elements.identityFile.value.trim() } : { source: "agent" }; return { kind: "ssh", label: elements.label.value.trim(), host: elements.host.value.trim(), port: Number(elements.port.value), user: elements.user.value.trim(), identity }; }
async function saveProfile(event) { event.preventDefault(); clearError(elements.formError); if (!elements.form.reportValidity()) return; try { await invoke("profile_save", { profile_id: elements.id.value || null, draft: makeDraft() }); closeProfileDialog(); await refreshProfiles(); announce("Profil enregistré."); } catch (error) { visibleError(elements.formError, error); } }
async function deleteProfile(profile) { if (!window.confirm(`Retirer le profil « ${profile.label} » ? Cette action ferme sa future connexion éventuelle.`)) return; try { await invoke("profile_delete", { profile_id: profile.id, confirmed: true }); await refreshProfiles(); announce(`Profil ${profile.label} retiré.`); } catch (error) { visibleError(elements.loadError, error); } }
async function verifyHostIdentity(profile) {
  if (profile.kind !== "ssh") return;
  const identity = await invoke("host_identity_check", { profile_id: profile.id });
  if (identity.status === "approved") return;
  if (identity.status === "changed") throw new Error(`L'identité SSH de ${profile.label} a changé (${identity.fingerprint}). La connexion est bloquée pour votre sécurité.`);
  if (identity.status !== "awaiting_approval" || !identity.ticket) throw new Error("Le contrôle d'identité SSH n'a pas produit de décision exploitable.");
  const accepted = window.confirm(`Première connexion à ${profileOrigin(profile)}.\n\nEmpreinte présentée :\n${identity.fingerprint}\n\nN'acceptez que si cette empreinte vous a été communiquée par une source fiable.`);
  if (!accepted) throw new Error("L'identité SSH n'a pas été approuvée. Aucune connexion n'a été ouverte.");
  await invoke("host_identity_approve", { profile_id: profile.id, ticket: identity.ticket });
}
function setConnectionMessage(message) { elements.connectionStatus.textContent = message; announce(message); }
async function openPanel(profile) {
  const panel = await invoke("panel_open", { profile_id: profile.id });
  openPanelProfiles.add(panel.profile_id);
  document.body.classList.add("panel-view"); elements.showProfiles.hidden = false; renderActivePanels();
  setConnectionMessage(`${profile.label} est affiché dans un panneau local isolé.`);
}
function requestDirectToken() {
  elements.directToken.value = "";
  elements.directTokenDialog.showModal();
  return new Promise((resolve) => {
    const finish = (value) => { elements.directTokenDialog.close(); resolve(value); };
    elements.directTokenForm.onsubmit = (event) => { event.preventDefault(); if (elements.directTokenForm.reportValidity()) finish(elements.directToken.value); };
    elements.cancelDirectToken.onclick = () => finish(null);
    elements.directTokenDialog.oncancel = () => finish(null);
    window.requestAnimationFrame(() => elements.directToken.focus());
  });
}
async function connectProfile(profile) {
  clearError(elements.loadError);
  try {
    setConnectionMessage(`Vérification de ${profile.label}…`);
    await verifyHostIdentity(profile);
    const relayToken = profile.kind === "local" ? await requestDirectToken() : null;
    if (profile.kind === "local" && !relayToken) { setConnectionMessage(`Connexion à ${profile.label} annulée.`); return; }
    const status = await invoke("connection_open", { profile_id: profile.id, relay_token: relayToken });
    connectionStates.set(profile.id, status);
    renderProfiles();
    await openPanel(profile);
  } catch (error) {
    connectionStates.set(profile.id, { state: "failed" }); renderProfiles();
    visibleError(elements.loadError, error);
    setConnectionMessage(`Connexion à ${profile.label} non établie.`);
  }
}
async function disconnectProfile(profile) {
  try {
    await invoke("connection_close", { profile_id: profile.id });
    connectionStates.delete(profile.id); openPanelProfiles.delete(profile.id);
    if (openPanelProfiles.size === 0) { elements.showProfiles.hidden = true; document.body.classList.remove("panel-view"); }
    renderActivePanels();
    renderProfiles(); setConnectionMessage(`${profile.label} est déconnecté.`);
  } catch (error) { visibleError(elements.loadError, error); }
}
async function showProfiles() {
  try {
    for (const profileId of [...openPanelProfiles]) await invoke("panel_close", { profile_id: profileId });
    openPanelProfiles.clear(); elements.showProfiles.hidden = true; document.body.classList.remove("panel-view"); renderActivePanels();
    setConnectionMessage("Les panneaux sont masqués. Les connexions restent ouvertes jusqu'à leur déconnexion explicite.");
  } catch (error) { visibleError(elements.loadError, error); }
}
elements.add.addEventListener("click", () => openProfileDialog()); elements.close.addEventListener("click", closeProfileDialog); elements.cancel.addEventListener("click", closeProfileDialog); elements.showProfiles.addEventListener("click", () => void showProfiles()); elements.kind.addEventListener("change", syncFormKind); document.querySelectorAll("input[name='identity-source']").forEach((input) => input.addEventListener("change", syncFormKind)); elements.form.addEventListener("submit", saveProfile); elements.list.addEventListener("click", (event) => { const button = event.target.closest("button[data-action]"); if (!button) return; const profile = profiles.find((candidate) => candidate.id === button.dataset.profileId); if (!profile) return; if (button.dataset.action === "edit") openProfileDialog(profile); if (button.dataset.action === "delete") void deleteProfile(profile); if (button.dataset.action === "connect") void connectProfile(profile); if (button.dataset.action === "open") void openPanel(profile).catch((error) => visibleError(elements.loadError, error)); if (button.dataset.action === "disconnect") void disconnectProfile(profile); });
const eventApi = window.__TAURI__?.event;
if (eventApi?.listen) void eventApi.listen("connection-state", (event) => {
  const state = event.payload; if (!state?.profile_id) return;
  connectionStates.set(state.profile_id, state); renderProfiles();
  const profile = profiles.find((candidate) => candidate.id === state.profile_id);
  if (profile && state.state === "failed") setConnectionMessage(`Le tunnel de ${profile.label} s'est arrêté. Réessayez explicitement si nécessaire.`);
});
void refreshProfiles();
