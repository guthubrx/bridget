function esc(value) {
  return String(value ?? "").replace(/[&<>"']/g, (ch) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  }[ch]));
}

export function renderMachinesView(ctx) {
  const { data, onAddSsh, onEditSsh, onRepair, onDeployClient } = ctx;
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
  document.getElementById("viewSummary").textContent =
    `${online}/${machines.length} machines joignables · Bridget local v${data.local_bridget_version || "?"}`;

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
      const st = c.installed
        ? `<span class="status-pill ok"><span aria-hidden="true">●</span><span>Installé</span></span>`
        : `<span class="status-pill muted"><span aria-hidden="true">○</span><span>Absent</span></span>`;
      const detail = c.installed
        ? `${c.version || "version détectée"}${c.path ? ` · ${c.path}` : ""}`
        : c.install_summary;
      const actionBtn = m.reachable
        ? (c.installed
            ? `<button class="btn subtle" type="button" data-deploy-m="${esc(m.machine_id)}" data-deploy-c="${esc(c.id)}" data-deploy-mode="update">Mettre à jour</button>`
            : `<button class="btn primary" type="button" data-deploy-m="${esc(m.machine_id)}" data-deploy-c="${esc(c.id)}" data-deploy-mode="install">Installer</button>`)
        : `<span class="row-sub">Hors ligne</span>`;
      return `<div class="row">
        <div>
          <div class="row-label">${esc(c.label)}</div>
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
        <div class="badges">${workerBadge}${sshBtn}</div>
      </div>
      ${macNote}
      ${repairs ? `<div class="repair-bar">${repairs}</div>` : ""}
      <div class="table-card">${cliRows}</div>
    </section>`;
  });

  root.innerHTML = blocks.length
    ? blocks.join("")
    : `<div class="empty-box">Aucune machine détectée. Cliquez sur « + Ajouter une machine » pour en raccorder une.</div>`;

  const byId = new Map(machines.map((m) => [m.machine_id, m]));
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
