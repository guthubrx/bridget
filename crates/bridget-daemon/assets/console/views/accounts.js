import { vendorLogoSvg } from "/assets/logos.js";

function esc(value) {
  return String(value ?? "").replace(/[&<>"']/g, (ch) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  }[ch]));
}

function statusMeta(conn) {
  const isDetected = String(conn.connection_id || "").startsWith("detected-");
  if (conn.auth_state === "machine_unreachable") return { cls: "err", icon: "✕", text: "Machine injoignable" };
  if (conn.auth_state === "authenticated" && !conn.enabled && !isDetected) return { cls: "muted", icon: "○", text: "Désactivé" };
  if (conn.auth_state === "authenticated") return { cls: "ok", icon: "●", text: "Connecté" };
  if (conn.auth_state === "unauthenticated") return { cls: "warn", icon: "▲", text: "À connecter" };
  if (conn.auth_state === "binary_missing") return { cls: "err", icon: "✕", text: "Programme absent" };
  return { cls: "muted", icon: "◌", text: "Non vérifié" };
}

function needsAttention(conn) {
  return conn.auth_state === "unauthenticated" || conn.auth_state === "binary_missing" || conn.auth_state === "machine_unreachable";
}

function machineDot(rows, reachable) {
  if (!reachable || rows.some((c) => c.auth_state === "machine_unreachable")) return "off";
  if (rows.some((c) => needsAttention(c))) return "warn";
  return rows.length > 0 ? "ok" : "off";
}

export function renderAccountsView(ctx) {
  const { data, activeView, onSelectView, onLogin, onTogglePreview, onShowCli } = ctx;
  const conns = data.connections || [];
  const known = data.known_machines || data.machines || [];
  const byMachine = new Map();
  for (const m of known) byMachine.set(m, []);
  for (const c of conns) {
    if (!byMachine.has(c.machine_id)) byMachine.set(c.machine_id, []);
    byMachine.get(c.machine_id).push(c);
  }

  const actionsList = conns.filter(needsAttention);
  document.getElementById("countAll").textContent = String(conns.length);
  const actEl = document.getElementById("countActions");
  actEl.textContent = String(actionsList.length);
  actEl.classList.toggle("warn", actionsList.length > 0);

  const navRoot = document.getElementById("machineNav");
  navRoot.innerHTML = Array.from(byMachine.entries()).map(([m, rows]) => {
    const okCount = rows.filter((r) => r.auth_state === "authenticated").length;
    const offline = rows.length === 0 || rows.every((r) => r.auth_state === "machine_unreachable");
    const right = offline && rows.length === 0
      ? "Aucun compte"
      : (offline ? "Hors ligne" : `${okCount}/${rows.length}`);
    const dot = machineDot(rows, !offline);
    const localTag = m === data.local_machine ? " (local)" : "";
    const active = activeView === `machine:${m}` ? " active" : "";
    return `<button class="nav-item${active}" type="button" data-view="machine:${esc(m)}">
      <span>${esc(m)}${esc(localTag)}</span>
      <span class="nav-right"><span>${esc(right)}</span><span class="dot ${dot}" aria-hidden="true"></span></span>
    </button>`;
  }).join("");

  for (const btn of document.querySelectorAll(".sidebar [data-view]")) {
    const view = btn.getAttribute("data-view");
    btn.classList.toggle("active", view === activeView);
    btn.onclick = () => onSelectView(view);
  }

  const authCount = conns.filter((c) => c.auth_state === "authenticated").length;
  const unauthCount = conns.filter((c) => c.auth_state === "unauthenticated").length;
  const offlineCount = conns.filter((c) => c.auth_state === "machine_unreachable").length;
  const titleEl = document.getElementById("viewTitle");
  if (activeView === "actions") titleEl.textContent = "Actions requises";
  else if (activeView.startsWith("machine:")) titleEl.textContent = activeView.slice(8);
  else titleEl.textContent = "Tous les comptes";
  document.getElementById("viewSummary").textContent =
    `${authCount} connectés · ${unauthCount} à connecter · ${offlineCount} hors ligne`;

  const banner = document.getElementById("alertBanner");
  if (unauthCount > 0 && activeView !== "actions") {
    const hosts = [...new Set(conns.filter((c) => c.auth_state === "unauthenticated").map((c) => c.machine_id))];
    banner.innerHTML = `<span><strong>▲ ${unauthCount} compte(s) nécessitent une connexion</strong> (${esc(hosts.join(", "))})</span>
      <button class="btn subtle" type="button" id="bannerFilterBtn">Voir</button>`;
    banner.classList.remove("hidden");
    const b = document.getElementById("bannerFilterBtn");
    if (b) b.onclick = () => onSelectView("actions");
  } else {
    banner.classList.add("hidden");
  }

  let machineKeys = Array.from(byMachine.keys());
  if (activeView.startsWith("machine:")) {
    const target = activeView.slice(8);
    machineKeys = machineKeys.filter((m) => m === target);
  }

  const root = document.getElementById("accountsRoot");
  const blocks = [];
  for (const m of machineKeys) {
    let rows = byMachine.get(m) || [];
    if (activeView === "actions") {
      rows = rows.filter(needsAttention);
      if (rows.length === 0) continue;
    }
    const okCount = rows.filter((r) => r.auth_state === "authenticated").length;
    const offline = rows.length === 0 || rows.every((r) => r.auth_state === "machine_unreachable");
    const mDot = machineDot(rows, !offline);
    const ownerBadges = [...new Set(rows.map((r) => r.owner_badge).filter(Boolean))];
    const writableAny = rows.some((r) => r.writable);
    const badgeHtml = (ownerBadges.length ? ownerBadges : ["Propriétaire indéterminé · Lecture seule"])
      .map((b) => `<span class="badge${writableAny ? " guarded" : ""}">${esc(b)}</span>`)
      .join("");
    const riskBadge = writableAny ? `<span class="badge guarded">Registre gardé</span>` : "";

    const rowsHtml = rows.length === 0
      ? `<div class="empty-box">Machine connue, aucun compte remonté ou machine injoignable.</div>`
      : `<div class="table-card">${rows.map((c, idx) => {
          const st = statusMeta(c);
          const extra = c.covered_connections && c.covered_connections.length > 1
            ? ` (+${c.covered_connections.length - 1})` : "";
          const key = `${m}::${c.connection_id}::${idx}`;
          const actions = [];
          if (c.auth_state === "unauthenticated") {
            actions.push(`<button class="btn primary" type="button" data-login="${esc(key)}">Connecter</button>`);
          } else if (c.auth_state === "authenticated") {
            actions.push(`<button class="btn subtle" type="button" data-relogin="${esc(key)}">Changer</button>`);
            if (c.writable && c.managed) {
              actions.push(`<button class="btn${c.enabled ? "" : " primary"}" type="button" data-toggle="${esc(key)}">${c.enabled ? "Désactiver" : "Activer"}</button>`);
            }
          } else {
            actions.push(`<button class="btn subtle" type="button" data-cli="${esc(key)}">Commande CLI</button>`);
          }
          const btnHtml = actions.join("");
          const isDetected = String(c.connection_id || "").startsWith("detected-");
          const subLabel = c.managed
            ? "Compte dédié (isolé)"
            : (isDetected ? "CLI installé (hors registre)" : "Compte principal");
          const logoEngine = (c.account_label || "").toLowerCase().startsWith("glm") ? "glm" : c.engine;
          return `<div class="row">
            <div>
              <div class="row-label v-label-inline">${vendorLogoSvg(logoEngine, 16)}<span>${esc(c.account_label || c.engine)}</span></div>
              <div class="row-sub">${subLabel}</div>
            </div>
            <div class="row-conn mono" title="${esc(c.connection_id)}">${esc(c.connection_id)}${esc(extra)}</div>
            <div class="row-detail" title="${esc(c.detail)}">${esc(c.detail)}</div>
            <div class="status-pill ${st.cls}"><span aria-hidden="true">${st.icon}</span><span>${esc(st.text)}</span></div>
            <div class="row-actions">${btnHtml}</div>
          </div>`;
        }).join("")}</div>`;

    blocks.push(`<section class="machine-block">
      <div class="machine-head">
        <div class="machine-title">
          <span class="dot ${mDot}" aria-hidden="true"></span>
          <h2>${esc(m)}</h2>
          <span class="machine-meta">${rows.length ? `${okCount}/${rows.length} connectés` : "0 compte"}${m === data.local_machine ? " · hôte console" : ""}</span>
        </div>
        <div class="badges">${riskBadge}${badgeHtml}</div>
      </div>
      ${rowsHtml}
    </section>`);
  }

  root.innerHTML = blocks.length
    ? blocks.join("")
    : `<div class="empty-box">Aucune action requise pour le moment.</div>`;

  const rowLookup = new Map();
  for (const m of machineKeys) {
    let rows = byMachine.get(m) || [];
    if (activeView === "actions") rows = rows.filter(needsAttention);
    rows.forEach((c, idx) => rowLookup.set(`${m}::${c.connection_id}::${idx}`, c));
  }
  for (const b of root.querySelectorAll("[data-login]")) {
    b.onclick = () => {
      const conn = rowLookup.get(b.getAttribute("data-login"));
      if (conn) onLogin(conn, false, false);
    };
  }
  for (const b of root.querySelectorAll("[data-relogin]")) {
    b.onclick = () => {
      const conn = rowLookup.get(b.getAttribute("data-relogin"));
      if (conn) onLogin(conn, true, false);
    };
  }
  for (const b of root.querySelectorAll("[data-toggle]")) {
    b.onclick = () => {
      const conn = rowLookup.get(b.getAttribute("data-toggle"));
      if (conn) onTogglePreview(conn);
    };
  }
  for (const b of root.querySelectorAll("[data-cli]")) {
    b.onclick = () => {
      const conn = rowLookup.get(b.getAttribute("data-cli"));
      if (conn) onShowCli(conn);
    };
  }
}
