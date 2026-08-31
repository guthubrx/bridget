import "./fleet-presentation.js";
const presentation = globalThis.FleetPresentation;

const elements = {
  status: document.querySelector("#app-status"),
  sources: document.querySelector("#fleet-sources"),
  global: document.querySelector("#fleet-global"),
  filters: document.querySelector("#fleet-filters"),
  groups: document.querySelector("#fleet-groups"),
  fleetCount: document.querySelector("#fleet-count"),
  sortChips: document.querySelector("#sort-chips"),
  sortField: document.querySelector("#sort-field"),
  addSort: document.querySelector("#add-sort"),
  createProject: document.querySelector("#create-project"),
  importProject: document.querySelector("#import-project"),
  manageServers: document.querySelector("#manage-servers"),
  settingsLauncher: document.querySelector("#settings-launcher"),
  settingsDialog: document.querySelector("#settings-dialog"),
  serverList: document.querySelector("#server-list"),
  addServer: document.querySelector("#add-server"),
  profileDialog: document.querySelector("#profile-dialog"),
  profileForm: document.querySelector("#profile-form"),
  profileTitle: document.querySelector("#profile-dialog-title"),
  profileId: document.querySelector("#profile-id"),
  profileLabel: document.querySelector("#profile-label"),
  profileHost: document.querySelector("#profile-host"),
  profilePort: document.querySelector("#profile-port"),
  profileUser: document.querySelector("#profile-user"),
  profileIdentityFile: document.querySelector("#profile-identity-file"),
  identityFileField: document.querySelector("#identity-file-field"),
  profileError: document.querySelector("#form-error"),
  closeProfile: document.querySelector("#close-dialog"),
  cancelProfile: document.querySelector("#cancel-profile"),
  hostDialog: document.querySelector("#host-identity-dialog"),
  hostServer: document.querySelector("#host-identity-server"),
  hostFingerprint: document.querySelector("#host-identity-fingerprint"),
  showPreferences: document.querySelector("#show-preferences"),
  preferencesForm: document.querySelector("#preferences-form"),
  closePreferences: document.querySelector("#close-preferences"),
  cancelPreferences: document.querySelector("#cancel-preferences"),
  preferencesDisplayName: document.querySelector("#preferences-display-name"),
  preferencesColorScheme: document.querySelector("#preferences-color-scheme"),
  preferencesTimezone: document.querySelector("#preferences-timezone"),
  preferencesFontSize: document.querySelector("#preferences-font-size"),
  preferencesExternalLinks: document.querySelector("#preferences-external-links"),
  preferencesFileReferences: document.querySelector("#preferences-file-references"),
  preferencesRemoteImages: document.querySelector("#preferences-remote-images"),
  preferencesError: document.querySelector("#preferences-error"),
  targetDialog: document.querySelector("#target-dialog"),
  targetChoices: document.querySelector("#target-choices"),
  targetTitle: document.querySelector("#target-title"),
  closeTarget: document.querySelector("#close-target"),
};

let profiles = [];
let snapshot = { sources: [], agents: [] };
let preferences = null;
let pendingProjectAction = null;
const filters = { sourceIds: new Set(), projectIds: new Set() };
const connectionStates = new Map();

function invoke(command, payload = {}) {
  const api = window.__TAURI__?.core?.invoke;
  return api
    ? api(command, payload)
    : Promise.reject(new Error("Bridget Desktop doit être ouvert depuis l'application."));
}

function announce(message) {
  elements.status.textContent = message;
}

function clear(node) {
  node.replaceChildren();
}

function button(label, className = "", onClick = null) {
  const node = document.createElement("button");
  node.type = "button";
  node.className = className;
  node.textContent = label;
  if (onClick) node.addEventListener("click", onClick);
  return node;
}

function profileOrigin(profile) {
  return `SSH ${profile.user}@${profile.host}:${profile.port}`;
}

function selectedIdentitySource() {
  return document.querySelector("input[name='identity-source']:checked")?.value || "agent";
}

function syncIdentityField() {
  const isFile = selectedIdentitySource() === "file";
  elements.identityFileField.hidden = !isFile;
  elements.profileIdentityFile.required = isFile;
}

function sourceById(sourceId) {
  return snapshot.sources.find((source) => source.source_id === sourceId);
}

function sourceLabel(sourceId) {
  return sourceById(sourceId)?.label || sourceId;
}

function projectById(projectId) {
  for (const source of snapshot.sources) {
    const project = (source.projects || []).find((candidate) => candidate.project_id === projectId);
    if (project) return { ...project, source };
  }
  return null;
}

function effectivePinned(agent) {
  const pinned = new Set(preferences?.pinned_agent_keys || []);
  const unpinned = new Set(preferences?.unpinned_coordinator_keys || []);
  return presentation.isPinned(agent, pinned, unpinned);
}

function applyPreferences() {
  if (!preferences) return;
  document.documentElement.dataset.colorScheme = preferences.color_scheme || "system";
  document.documentElement.style.fontSize = `${preferences.font_size_px || 16}px`;
}

async function savePreferences() {
  preferences = await invoke("preferences_save", { preferences });
  applyPreferences();
}

function renderFilters() {
  clear(elements.filters);
  const addChip = (label, title, remove) => {
    const chip = button(`${label} ×`, "filter-chip", remove);
    chip.title = title;
    elements.filters.append(chip);
  };
  for (const sourceId of filters.sourceIds) {
    addChip(sourceLabel(sourceId), "Retirer ce filtre source", () => {
      filters.sourceIds.delete(sourceId);
      render();
    });
  }
  for (const projectId of filters.projectIds) {
    const project = projectById(projectId);
    addChip(project?.display_name || projectId, "Retirer ce filtre projet", () => {
      filters.projectIds.delete(projectId);
      render();
    });
  }
}

function renderSources() {
  clear(elements.sources);
  elements.global.setAttribute("aria-current", String(filters.sourceIds.size === 0 && filters.projectIds.size === 0));
  for (const source of snapshot.sources) {
    const sourceItem = document.createElement("li");
    sourceItem.className = "source-item";
    const sourceButton = button(source.label, "source-button", () => {
      if (filters.sourceIds.has(source.source_id)) filters.sourceIds.delete(source.source_id);
      else filters.sourceIds.add(source.source_id);
      render();
    });
    sourceButton.setAttribute("aria-pressed", String(filters.sourceIds.has(source.source_id)));
    const state = document.createElement("small");
    state.textContent = source.error || source.connection_state;
    sourceItem.append(sourceButton, state);
    const projects = document.createElement("ul");
    projects.className = "source-projects";
    for (const project of source.projects || []) {
      const projectButton = button(project.display_name, "project-button", () => {
        filters.sourceIds.add(source.source_id);
        filters.projectIds.add(project.project_id);
        render();
      });
      projectButton.setAttribute("aria-pressed", String(filters.projectIds.has(project.project_id)));
      const projectItem = document.createElement("li");
      projectItem.append(projectButton);
      projects.append(projectItem);
    }
    if (projects.childElementCount) sourceItem.append(projects);
    elements.sources.append(sourceItem);
  }
}

function renderSortChips() {
  clear(elements.sortChips);
  const labels = { source: "Source", state: "État", project: "Projet", activity: "Activité récente", name: "Nom" };
  (preferences?.sort_criteria || []).forEach((criterion, index) => {
    const chip = document.createElement("div");
    chip.className = "sort-chip";
    chip.append(document.createTextNode(`${labels[criterion.field]} ${criterion.direction === "asc" ? "↑" : "↓"}`));
    chip.append(
      button("Inverser", "quiet", async () => {
        criterion.direction = criterion.direction === "asc" ? "desc" : "asc";
        await savePreferences();
        render();
      }),
      button("←", "quiet", async () => {
        if (index === 0) return;
        [preferences.sort_criteria[index - 1], preferences.sort_criteria[index]] =
          [preferences.sort_criteria[index], preferences.sort_criteria[index - 1]];
        await savePreferences();
        render();
      }),
      button("→", "quiet", async () => {
        if (index === preferences.sort_criteria.length - 1) return;
        [preferences.sort_criteria[index + 1], preferences.sort_criteria[index]] =
          [preferences.sort_criteria[index], preferences.sort_criteria[index + 1]];
        await savePreferences();
        render();
      }),
      button("Retirer", "quiet", async () => {
        preferences.sort_criteria.splice(index, 1);
        await savePreferences();
        render();
      }),
    );
    elements.sortChips.append(chip);
  });
}

function renderAgentList(agents) {
  const list = document.createElement("ul");
  list.className = "fleet-agent-list";
  for (const agent of agents) {
    const item = document.createElement("li");
    const card = button("", "agent-card", () => void openAgent(agent));
    const name = document.createElement("strong");
    name.textContent = agent.display_name || agent.name;
    const origin = document.createElement("small");
    origin.textContent = `${agent.source_label} · ${agent.project_name || "Sans projet"}`;
    const details = document.createElement("span");
    details.textContent = agent.wait_state === "waiting" || (agent.alerts || []).length
      ? "À traiter"
      : agent.state || "Inconnu";
    card.append(name, origin, details);
    const pin = button(effectivePinned(agent) ? "★" : "☆", "pin-agent", () => void togglePin(agent));
    pin.setAttribute("aria-label", effectivePinned(agent) ? "Désépingler" : "Épingler");
    item.append(card, pin);
    list.append(item);
  }
  return list;
}

function renderGroup(group, collapsed, depth = 0) {
  const section = document.createElement("section");
  section.className = "fleet-group";
  section.style.paddingLeft = `${depth * 8}px`;
  const agents = presentation.flattenGroups([group]);
  const header = button(
    `${collapsed.has(group.key) ? "▸" : "▾"} ${group.label} (${agents.length})`,
    "group-header",
    async () => {
      const next = new Set(preferences.collapsed_group_keys || []);
      if (next.has(group.key)) next.delete(group.key); else next.add(group.key);
      preferences.collapsed_group_keys = [...next];
      await savePreferences();
      render();
    },
  );
  section.append(header);
  if (!collapsed.has(group.key)) {
    if (group.groups?.length) group.groups.forEach((child) => section.append(renderGroup(child, collapsed, depth + 1)));
    else section.append(renderAgentList(group.agents));
  }
  return section;
}

function renderGroups() {
  clear(elements.groups);
  const filtered = presentation.applyFilters(snapshot.agents || [], filters);
  const sorted = presentation.sortAgents(
    filtered,
    preferences?.sort_criteria || [],
    new Set(preferences?.pinned_agent_keys || []),
    new Set(preferences?.unpinned_coordinator_keys || []),
  );
  const groups = presentation.groupAgents(sorted, preferences?.sort_criteria || []);
  elements.fleetCount.textContent = `${sorted.length} agent${sorted.length > 1 ? "s" : ""}`;
  const collapsed = new Set(preferences?.collapsed_group_keys || []);
  groups.forEach((group) => elements.groups.append(renderGroup(group, collapsed)));
}

function render() {
  renderFilters();
  renderSources();
  renderSortChips();
  renderGroups();
}

async function refreshFleet() {
  try {
    snapshot = await invoke("fleet_snapshot");
    render();
  } catch (error) {
    announce(`Flotte indisponible : ${error.message || error}`);
  }
}

async function openAgent(agent) {
  try {
    await invoke("panel_open", { source_id: agent.source_id, agent_name: agent.name });
    announce(`${agent.display_name || agent.name} est ouvert depuis ${agent.source_label}.`);
  } catch (error) {
    announce(error.message || String(error));
  }
}

async function togglePin(agent) {
  const pinned = new Set(preferences.pinned_agent_keys || []);
  const unpinned = new Set(preferences.unpinned_coordinator_keys || []);
  if (effectivePinned(agent)) {
    pinned.delete(agent.key);
    if (agent.is_coordinator) unpinned.add(agent.key);
  } else {
    pinned.add(agent.key);
    unpinned.delete(agent.key);
  }
  preferences.pinned_agent_keys = [...pinned];
  preferences.unpinned_coordinator_keys = [...unpinned];
  await savePreferences();
  render();
}

function selectedTargetSources() {
  return snapshot.sources.filter((source) => source.connection_state === "connected" && !source.error);
}

function beginProject(action) {
  if (selectedTargetSources().length === 0) {
    announce("Aucune source connectée ne peut recevoir cette action.");
    return;
  }
  const selected = [...filters.sourceIds].filter((sourceId) => selectedTargetSources().some((source) => source.source_id === sourceId));
  if (selected.length === 1) return void openProjectAction(selected[0], action);
  pendingProjectAction = action;
  elements.targetTitle.textContent = action === "create_project" ? "Créer un projet sur…" : "Importer un projet depuis…";
  clear(elements.targetChoices);
  for (const source of selectedTargetSources()) {
    elements.targetChoices.append(button(source.label, "target-source", () => {
      elements.targetDialog.close();
      void openProjectAction(source.source_id, pendingProjectAction);
    }));
  }
  elements.targetDialog.showModal();
}

async function openProjectAction(sourceId, action) {
  try {
    await invoke("panel_open", { source_id: sourceId, desktop_action: action });
  } catch (error) {
    announce(error.message || String(error));
  }
}

function resetProfileForm(profile = null) {
  elements.profileForm.reset();
  elements.profileId.value = profile?.id || "";
  elements.profileTitle.textContent = profile ? "Modifier un serveur" : "Ajouter un serveur";
  elements.profileLabel.value = profile?.label || "";
  elements.profileHost.value = profile?.host || "";
  elements.profilePort.value = profile?.port || 22;
  elements.profileUser.value = profile?.user || "";
  const identity = profile?.identity?.source || "agent";
  document.querySelector(`input[name="identity-source"][value="${identity}"]`).checked = true;
  elements.profileIdentityFile.value = profile?.identity?.path || "";
  elements.profileError.hidden = true;
  syncIdentityField();
}

function openProfileDialog(profile = null) {
  resetProfileForm(profile);
  elements.profileDialog.showModal();
}

async function requestHostApproval(profile) {
  const identity = await invoke("host_identity_check", { profile_id: profile.id });
  if (identity.status === "approved") return;
  if (identity.status === "changed") throw new Error(`L'identité SSH de ${profile.label} a changé (${identity.fingerprint}).`);
  elements.hostServer.textContent = profileOrigin(profile);
  elements.hostFingerprint.textContent = identity.fingerprint || "";
  elements.hostDialog.returnValue = "cancel";
  elements.hostDialog.showModal();
  const accepted = await new Promise((resolve) => {
    elements.hostDialog.addEventListener("close", () => resolve(elements.hostDialog.returnValue === "approve"), { once: true });
  });
  if (!accepted) throw new Error("L'identité SSH n'a pas été approuvée.");
  await invoke("host_identity_approve", { profile_id: profile.id, ticket: identity.ticket });
}

async function connectProfile(profile) {
  try {
    await requestHostApproval(profile);
    const status = await invoke("connection_open", { profile_id: profile.id });
    connectionStates.set(profile.id, status);
    await refreshFleet();
  } catch (error) {
    announce(error.message || String(error));
  }
}

async function renderServers() {
  profiles = await invoke("profiles_list");
  clear(elements.serverList);
  for (const profile of profiles) {
    const item = document.createElement("li");
    const text = document.createElement("div");
    text.append(Object.assign(document.createElement("strong"), { textContent: profile.label }));
    text.append(Object.assign(document.createElement("small"), { textContent: profileOrigin(profile) }));
    const actions = document.createElement("div");
    const state = connectionStates.get(profile.id)?.state || "disconnected";
    actions.append(
      button(state === "connected" ? "Déconnecter" : "Connecter", "quiet", async () => {
        if (state === "connected") await invoke("connection_close", { profile_id: profile.id });
        else await connectProfile(profile);
        await renderServers();
        await refreshFleet();
      }),
      button("Modifier", "quiet", () => openProfileDialog(profile)),
      button("Retirer", "quiet", async () => {
        if (!window.confirm(`Retirer le serveur « ${profile.label} » ?`)) return;
        await invoke("profile_delete", { profile_id: profile.id, confirmed: true });
        await renderServers();
        await refreshFleet();
      }),
    );
    item.append(text, actions);
    elements.serverList.append(item);
  }
}

async function restoreManagedConnections() {
  profiles = await invoke("profiles_list");
  for (const profile of profiles) {
    if (!profile.host_fingerprint) continue;
    await connectProfile(profile);
  }
}

function closeSettingsLauncher() {
  elements.settingsLauncher.hidden = true;
  elements.showPreferences.setAttribute("aria-expanded", "false");
}

function selectSettingsSection(section) {
  const selected = ["general", "servers", "usage"].includes(section) ? section : "general";
  document.querySelectorAll("[data-settings-panel]").forEach((panel) => {
    panel.hidden = panel.dataset.settingsPanel !== selected;
  });
  document.querySelectorAll("[data-settings-section]").forEach((button) => {
    button.setAttribute("aria-current", String(button.dataset.settingsSection === selected));
  });
  return selected;
}

async function openSettings(section = "general") {
  try {
    closeSettingsLauncher();
    preferences = await invoke("preferences_get");
    elements.preferencesDisplayName.value = preferences.display_name || "";
    elements.preferencesColorScheme.value = preferences.color_scheme || "system";
    elements.preferencesTimezone.value = preferences.timezone || "system";
    elements.preferencesFontSize.value = String(preferences.font_size_px || 16);
    const content = preferences.content_security || {};
    elements.preferencesExternalLinks.checked = content.external_links === true;
    elements.preferencesFileReferences.checked = content.file_references === true;
    elements.preferencesRemoteImages.checked = content.remote_images === true;
    const selected = selectSettingsSection(section);
    if (selected === "servers") await renderServers();
    if (!elements.settingsDialog.open) elements.settingsDialog.showModal();
  } catch (error) {
    announce(error.message || String(error));
  }
}

elements.global.addEventListener("click", () => {
  filters.sourceIds.clear();
  filters.projectIds.clear();
  render();
});
elements.addSort.addEventListener("click", async () => {
  const field = elements.sortField.value;
  if (preferences.sort_criteria.some((criterion) => criterion.field === field)) return;
  preferences.sort_criteria.push({ field, direction: field === "activity" ? "desc" : "asc" });
  await savePreferences();
  render();
});
elements.createProject.addEventListener("click", () => beginProject("create_project"));
elements.importProject.addEventListener("click", () => beginProject("import_project"));
elements.manageServers.addEventListener("click", () => void openSettings("servers"));
elements.addServer.addEventListener("click", () => openProfileDialog());
elements.closeProfile.addEventListener("click", () => elements.profileDialog.close());
elements.cancelProfile.addEventListener("click", () => elements.profileDialog.close());
elements.closeTarget.addEventListener("click", () => elements.targetDialog.close());
document.querySelectorAll("input[name='identity-source']").forEach((input) => input.addEventListener("change", syncIdentityField));
elements.profileForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  const identity = selectedIdentitySource() === "file"
    ? { source: "file", path: elements.profileIdentityFile.value.trim() }
    : { source: "agent" };
  try {
    await invoke("profile_save", {
      profile_id: elements.profileId.value || null,
      draft: {
        kind: "ssh",
        label: elements.profileLabel.value.trim(),
        host: elements.profileHost.value.trim(),
        port: Number(elements.profilePort.value),
        user: elements.profileUser.value.trim(),
        identity,
      },
    });
    elements.profileDialog.close();
    await renderServers();
    await refreshFleet();
  } catch (error) {
    elements.profileError.textContent = error.message || String(error);
    elements.profileError.hidden = false;
  }
});
elements.showPreferences.addEventListener("click", (event) => {
  event.stopPropagation();
  const willOpen = elements.settingsLauncher.hidden;
  elements.settingsLauncher.hidden = !willOpen;
  elements.showPreferences.setAttribute("aria-expanded", String(willOpen));
});
document.querySelectorAll("[data-open-settings-section]").forEach((button) => {
  button.addEventListener("click", () => void openSettings(button.dataset.openSettingsSection));
});
document.querySelectorAll("[data-settings-section]").forEach((button) => {
  button.addEventListener("click", async () => {
    const selected = selectSettingsSection(button.dataset.settingsSection);
    if (selected === "servers") await renderServers();
  });
});
document.addEventListener("click", (event) => {
  if (!event.target.closest(".settings-launcher-wrap")) closeSettingsLauncher();
});
document.addEventListener("keydown", (event) => {
  if (event.key === "Escape" && !elements.settingsLauncher.hidden) closeSettingsLauncher();
});
elements.closePreferences.addEventListener("click", () => elements.settingsDialog.close());
elements.cancelPreferences.addEventListener("click", () => elements.settingsDialog.close());
elements.preferencesForm.addEventListener("submit", async (event) => {
  event.preventDefault();
  try {
    preferences = {
      ...preferences,
      display_name: elements.preferencesDisplayName.value.trim(),
      color_scheme: elements.preferencesColorScheme.value,
      timezone: elements.preferencesTimezone.value.trim() || "system",
      font_size_px: Number(elements.preferencesFontSize.value),
      content_security: {
        external_links: elements.preferencesExternalLinks.checked,
        file_references: elements.preferencesFileReferences.checked,
        remote_images: elements.preferencesRemoteImages.checked,
      },
    };
    await savePreferences();
    elements.settingsDialog.close();
    render();
  } catch (error) {
    elements.preferencesError.textContent = error.message || String(error);
    elements.preferencesError.hidden = false;
  }
});
window.__TAURI__?.event?.listen("connection-state", (event) => {
  if (event.payload?.profile_id) connectionStates.set(event.payload.profile_id, event.payload);
  void refreshFleet();
});

try {
  preferences = await invoke("preferences_get");
  applyPreferences();
  await refreshFleet();
  void restoreManagedConnections();
} catch (error) {
  announce(error.message || String(error));
}
window.setInterval(() => void refreshFleet(), 5000);
