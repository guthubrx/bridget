import { vendorLogoSvg } from "/assets/logos.js";

function esc(value) {
  return String(value ?? "").replace(/[&<>"']/g, (ch) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  }[ch]));
}

export function renderMachinesView(ctx) {
  const { data, onAddSsh, onEditSsh, onRepair, onDeployClient, onBatchUpdate } = ctx;
  const root = document.getElementById("accountsRoot");
  const banner = document.getElementById("alertBanner");
  if (banner) banner.classList.add("hidden");

  document.getElementById("viewTitle").textContent = "Machines & Clients CLI";
  if (!data) {
    document.getElementById("viewSummary").textContent = "Analyse SSH et inventaire des clients CLI en cours…";
    root.innerHTML = `<div class="empty-box">Sonde des machines et des programmes (Claude, Codex, Antigravity, Cursor, Gemini)…</div>`;
    return;
  }

  const machines = data.machines || [];
  const online = machines.filter((m) => m.reachable).length;
  const outdatedAll = [];
  const installedAll = [];
  let upToDateCount = 0;
  for (const m of machines) {
    if (!m.reachable) continue;
    for (const c of (m.clients || [])) {
      if (!c.installed) continue;
      installedAll.push({ machine: m, client: c });
      if (c.update_status === "update_available") {
        outdatedAll.push({ machine: m, client: c });
      } else if (c.update_status === "up_to_date") {
        upToDateCount += 1;
      }
    }
  }
  const updatesPart = outdatedAll.length > 0
    ? ` · ${outdatedAll.length} mise(s) à jour CLI disponible(s)`
    : (upToDateCount > 0 ? " · CLI vérifiables à jour" : " · versions amont non vérifiées");
  document.getElementById("viewSummary").textContent =
    `${online}/${machines.length} machines joignables · Bridget local v${data.local_bridget_version || "?"}${updatesPart}`;

  const batchBar = installedAll.length > 0
    ? `<div class="batch-bar">
        <div class="batch-bar-meta">${outdatedAll.length > 0
          ? `<strong>${outdatedAll.length} mise(s) à jour publiée(s)</strong> prête(s) sur les machines joignables.`
          : (upToDateCount > 0
              ? "Tous les CLI avec version amont vérifiable (Codex, Claude, Gemini) sont à jour. Antigravity et Cursor ne publient pas de numéro amont vérifiable."
              : "Impossible de vérifier les dernières versions amont pour le moment (hors ligne ou registre npm indisponible).")}</div>
        <div class="batch-bar-actions">
          ${outdatedAll.length > 0
            ? `<button class="btn primary" type="button" id="batchUpdateOutdatedBtn">Tout mettre à jour (${outdatedAll.length})</button>`
            : ""}
          <button class="btn subtle" type="button" id="batchUpdateAllInstalledBtn">Tout vérifier / mettre à jour (${installedAll.length})</button>
        </div>
      </div>`
    : "";

  const blocks = machines.map((m) => {
    const dot = m.reachable ? (m.worker_state === "running" || m.is_local ? "ok" : "warn") : "err";
    const metaParts = [];
    if (m.is_local) metaParts.push("hôte console");
    if (m.os) metaParts.push(`${m.os} ${m.arch || ""}`.trim());
    if (m.bridget_version) metaParts.push(m.bridget_version);
    if (!m.reachable) metaParts.push(m.ssh_configured ? "injoignable en SSH" : "SSH non configuré");

    const workerBadge = m.is_local
      ? `<span class="badge guarded">Local</span>`
      : m.reachable
        ? `<span class="badge${m.worker_state === "running" ? " guarded" : ""}">Worker : ${m.worker_state === "running" ? "actif" : "inactif"}</span>`
        : `<span class="badge">Hors ligne</span>`;

    const machineOutdated = m.reachable
      ? (m.clients || []).filter((c) => c.installed && c.update_status === "update_available")
      : [];
    const machineUpdateBtn = machineOutdated.length > 0
      ? `<button class="btn primary" type="button" data-batch-machine="${esc(m.machine_id)}">Mettre à jour (${machineOutdated.length})</button>`
      : "";

    const repairs = (m.suggested_repairs || []).map((r, idx) =>
      `<button class="btn${r.action === "configure_ssh" || r.action === "fix_codex_link" || r.action === "macos_keychain_no_autolock" || r.action === "macos_keychain_unlock_guide" ? " primary" : ""}" type="button" data-repair-m="${esc(m.machine_id)}" data-repair-idx="${idx}">${esc(r.label)}</button>`
    ).join("");

    let macNote = "";
    if (!m.is_local && m.reachable && m.os === "Darwin") {
      if (m.macos_keychain_state === "locked") {
        macNote = `<div class="macos-note"><strong>macOS distant détecté · Trousseau verrouillé</strong>Sur un Mac utilisé à distance, macOS verrouille automatiquement le trousseau en veille, ce qui bloque les connexions <strong>Claude Code</strong> et <strong>Cursor CLI</strong> (Codex, Antigravity et Gemini ne sont pas concernés). Cliquez sur « Déverrouiller le trousseau macOS » pour copier la commande unique à lancer une fois.</div>`;
      } else if (m.macos_keychain_state === "autolock_enabled") {
        macNote = `<div class="macos-note"><strong>macOS distant détecté · Verrouillage auto actif</strong>Le trousseau est ouvert mais macOS le reverrouillera automatiquement en veille, ce qui coupera Claude Code et Cursor CLI. Vous pouvez désactiver ce verrouillage auto en un clic ci-dessous.</div>`;
      }
    }

    const sshBtn = !m.is_local
      ? `<button class="btn subtle" type="button" data-edit-ssh="${esc(m.machine_id)}">SSH</button>`
      : "";

    const cliRows = (m.clients || []).map((c) => {
      let st;
      let detail;
      let actionBtn;
      if (!c.installed) {
        st = `<span class="status-pill muted"><span aria-hidden="true">○</span><span>Absent</span></span>`;
        detail = c.install_summary;
        actionBtn = m.reachable
          ? `<button class="btn primary" type="button" data-deploy-m="${esc(m.machine_id)}" data-deploy-c="${esc(c.id)}" data-deploy-mode="install">Installer</button>`
          : `<span class="row-sub">Hors ligne</span>`;
      } else if (c.update_status === "update_available") {
        st = `<span class="status-pill warn"><span aria-hidden="true">▲</span><span>MàJ dispo</span></span>`;
        detail = `${c.version || "?"} -> ${c.latest_version}${c.path ? ` · ${c.path}` : ""}`;
        actionBtn = m.reachable
          ? `<button class="btn primary" type="button" data-deploy-m="${esc(m.machine_id)}" data-deploy-c="${esc(c.id)}" data-deploy-mode="update">Mettre à jour</button>`
          : `<span class="row-sub">Hors ligne</span>`;
      } else if (c.update_status === "up_to_date") {
        st = `<span class="status-pill ok"><span aria-hidden="true">●</span><span>À jour</span></span>`;
        detail = `${c.version || "à jour"}${c.path ? ` · ${c.path}` : ""}`;
        actionBtn = m.reachable
          ? `<button class="btn subtle" type="button" title="Relancer la commande de mise à jour officielle" data-deploy-m="${esc(m.machine_id)}" data-deploy-c="${esc(c.id)}" data-deploy-mode="update">↻</button>`
          : `<span class="row-sub">Hors ligne</span>`;
      } else {
        st = `<span class="status-pill ok"><span aria-hidden="true">●</span><span>Installé</span></span>`;
        detail = `${c.version || "version détectée"} (version amont non vérifiable)${c.path ? ` · ${c.path}` : ""}`;
        actionBtn = m.reachable
          ? `<button class="btn subtle" type="button" title="Vérifier et appliquer une éventuelle mise à jour officielle" data-deploy-m="${esc(m.machine_id)}" data-deploy-c="${esc(c.id)}" data-deploy-mode="update">Vérifier / MàJ</button>`
          : `<span class="row-sub">Hors ligne</span>`;
      }
      return `<div class="row">
        <div>
          <div class="row-label v-label-inline">${vendorLogoSvg(c.id, 16)}<span>${esc(c.label)}</span></div>
          <div class="row-sub mono">${esc(c.binary)}</div>
        </div>
        <div class="row-detail" style="grid-column: span 2" title="${esc(detail)}">${esc(detail)}</div>
        <div>${st}</div>
        <div class="row-actions">${actionBtn}</div>
      </div>`;
    }).join("");

    return `<section class="machine-block">
      <div class="machine-head">
        <div class="machine-title">
          <span class="dot ${dot}" aria-hidden="true"></span>
          <h2>${esc(m.machine_id)}</h2>
          <span class="machine-meta">${esc(metaParts.join(" · "))}</span>
        </div>
        <div class="badges">${machineUpdateBtn}${workerBadge}${sshBtn}</div>
      </div>
      ${macNote}
      ${repairs ? `<div class="repair-bar">${repairs}</div>` : ""}
      <div class="table-card">${cliRows}</div>
    </section>`;
  });

  root.innerHTML = blocks.length
    ? batchBar + blocks.join("")
    : `<div class="empty-box">Aucune machine détectée. Cliquez sur « + Ajouter une machine » pour en raccorder une.</div>`;

  const byId = new Map(machines.map((m) => [m.machine_id, m]));
  const batchOutBtn = document.getElementById("batchUpdateOutdatedBtn");
  if (batchOutBtn && onBatchUpdate) {
    batchOutBtn.onclick = () => onBatchUpdate(outdatedAll, `Mettre à jour ${outdatedAll.length} client(s) CLI en retard`);
  }
  const batchAllBtn = document.getElementById("batchUpdateAllInstalledBtn");
  if (batchAllBtn && onBatchUpdate) {
    batchAllBtn.onclick = () => onBatchUpdate(installedAll, `Vérifier / mettre à jour ${installedAll.length} client(s) CLI installés`);
  }
  for (const b of root.querySelectorAll("[data-batch-machine]")) {
    b.onclick = () => {
      const m = byId.get(b.getAttribute("data-batch-machine"));
      if (!m || !onBatchUpdate) return;
      const items = (m.clients || [])
        .filter((c) => c.installed && c.update_status === "update_available")
        .map((c) => ({ machine: m, client: c }));
      if (items.length) onBatchUpdate(items, `Mettre à jour ${m.machine_id} (${items.length} CLI)`);
    };
  }
  for (const b of root.querySelectorAll("[data-edit-ssh]")) {
    b.onclick = () => {
      const m = byId.get(b.getAttribute("data-edit-ssh"));
      if (m) onEditSsh(m);
    };
  }
  for (const b of root.querySelectorAll("[data-repair-m]")) {
    b.onclick = () => {
      const m = byId.get(b.getAttribute("data-repair-m"));
      const idx = Number(b.getAttribute("data-repair-idx"));
      const rep = m && m.suggested_repairs ? m.suggested_repairs[idx] : null;
      if (!m || !rep) return;
      if (rep.action === "configure_ssh") onEditSsh(m);
      else onRepair(m, rep);
    };
  }
  for (const b of root.querySelectorAll("[data-deploy-m]")) {
    b.onclick = () => {
      const m = byId.get(b.getAttribute("data-deploy-m"));
      const cid = b.getAttribute("data-deploy-c");
      const mode = b.getAttribute("data-deploy-mode") || "install";
      const client = m && m.clients ? m.clients.find((x) => x.id === cid) : null;
      if (m && client) onDeployClient(m, client, mode);
    };
  }
  const addBtn = document.getElementById("primaryActionBtn");
  if (addBtn) addBtn.onclick = () => onAddSsh();
}
