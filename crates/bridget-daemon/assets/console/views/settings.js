function esc(value) {
  return String(value ?? "").replace(/[&<>"']/g, (ch) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  }[ch]));
}

function formatTs(ts) {
  if (!ts) return "Jamais";
  try {
    return new Date(ts * 1000).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" });
  } catch {
    return String(ts);
  }
}

export function renderSettingsView(ctx) {
  const {
    data,
    updatesData,
    onSaveAutomanaged,
    onSyncFleet,
    onRoutingPreview,
    onUpdatesGlobal,
    onUpdatesHostMode,
    onUpdatesCheck,
    onUpdatesInstall,
  } = ctx;
  const root = document.getElementById("accountsRoot");
  const banner = document.getElementById("alertBanner");
  if (banner) banner.classList.add("hidden");

  document.getElementById("viewTitle").textContent = "Paramètres Bridget";
  if (!data) {
    document.getElementById("viewSummary").textContent = "Chargement des réglages Bridget…";
    root.innerHTML = `<div class="empty-box">Lecture de la configuration locale et du registre…</div>`;
    return;
  }

  document.getElementById("viewSummary").textContent =
    `Bridget v${data.bridget_version || "?"} (${data.bridget_build_id || "local"}) · hôte ${data.local_machine || "local"}`;

  const auto = data.automanaged || {};
  const cfg = auto.config || { cli_updates: true, model_sync: true, interval_secs: 21600 };
  const st = auto.state || {};
  const routing = data.routing || {};
  const rp = routing.routing_policy;
  const writable = Boolean(routing.writable);

  const freqOpts = [
    [1800, "Toutes les 30 minutes"],
    [3600, "Toutes les heures"],
    [21600, "Toutes les 6 heures (recommandé)"],
    [43200, "Toutes les 12 heures"],
    [86400, "Une fois par jour"],
  ];
  if (!freqOpts.some(([v]) => v === cfg.interval_secs)) {
    freqOpts.push([cfg.interval_secs, `${Math.round(cfg.interval_secs / 60)} minutes (actuel)`]);
  }

  const autoHtml = `<section class="settings-card">
    <div class="machine-head">
      <div class="machine-title">
        <h2>Automatismes de la flotte</h2>
        <span class="machine-meta">Dernière synchro : ${esc(formatTs(st.last_run_at))}</span>
      </div>
      <div class="badges">
        <button class="btn" type="button" id="syncFleetBtn">Synchroniser maintenant</button>
      </div>
    </div>
    <div class="settings-list">
      <label class="setting-row">
        <div>
          <div class="row-label">Mises à jour automatiques des clients CLI</div>
          <div class="row-sub">Met à jour Claude, Codex, Antigravity et Cursor via leurs commandes officielles sans couper les tâches.</div>
        </div>
        <input type="checkbox" id="optCliUpdates" ${cfg.cli_updates ? "checked" : ""} />
      </label>
      <label class="setting-row">
        <div>
          <div class="row-label">Synchronisation automatique des modèles</div>
          <div class="row-sub">Ajoute automatiquement les nouveaux modèles publiés dans les registres (strictement additif).</div>
        </div>
        <input type="checkbox" id="optModelSync" ${cfg.model_sync ? "checked" : ""} />
      </label>
      <div class="setting-row">
        <div>
          <div class="row-label">Fréquence de vérification</div>
          <div class="row-sub">Appliquée localement dans <code>automanaged.json</code> (droits 0600) et propagée aux machines joignables.</div>
        </div>
        <select id="optInterval" class="compact-select">
          ${freqOpts.map(([v, lbl]) => `<option value="${v}"${v === cfg.interval_secs ? " selected" : ""}>${esc(lbl)}</option>`).join("")}
        </select>
      </div>
    </div>
    <div class="settings-foot">
      <button class="btn primary" type="button" id="saveAutoBtn">Enregistrer et propager</button>
    </div>
  </section>`;

  let routingHtml = "";
  if (rp && typeof rp === "object") {
    const mPolicies = rp.machine_policies || {};
    const mRows = Object.entries(mPolicies).map(([mid, pol]) => {
      const enabled = pol.enabled !== false;
      const cap = Number(pol.compilation_capacity || 1);
      return `<div class="row">
        <div>
          <div class="row-label">${esc(mid)}</div>
          <div class="row-sub">${enabled ? "Machine active au routage" : "Machine en pause"}</div>
        </div>
        <div class="row-detail">Capacité de compilation : <strong>${cap}</strong></div>
        <div class="row-detail">${writable ? `<select class="compact-select" data-cap-m="${esc(mid)}">${[1, 2, 3, 4, 6, 8].map((n) => `<option value="${n}"${n === cap ? " selected" : ""}>${n} tâche(s)</option>`).join("")}</select>` : ""}</div>
        <div><span class="status-pill ${enabled ? "ok" : "muted"}"><span>${enabled ? "● Active" : "○ Pause"}</span></span></div>
        <div class="row-actions">
          ${writable ? `<button class="btn${enabled ? "" : " primary"}" type="button" data-toggle-m="${esc(mid)}" data-next="${enabled ? "false" : "true"}">${enabled ? "Mettre en pause" : "Activer"}</button>` : `<span class="row-sub">Lecture seule</span>`}
        </div>
      </div>`;
    }).join("");

    const cRows = (rp.connections || []).map((c) => {
      const enabled = c.enabled !== false;
      return `<div class="row">
        <div>
          <div class="row-label">${esc(c.display_name || c.connection_id)}</div>
          <div class="row-sub mono">${esc(c.connection_id)}</div>
        </div>
        <div class="row-detail">${esc(c.machine_id)} · ${esc(c.engine)}</div>
        <div class="row-detail mono">${esc(c.agent_type)}</div>
        <div><span class="status-pill ${enabled ? "ok" : "muted"}"><span>${enabled ? "● Activée" : "○ Désactivée"}</span></span></div>
        <div class="row-actions">
          ${writable ? `<button class="btn subtle" type="button" data-toggle-c="${esc(c.connection_id)}" data-next="${enabled ? "false" : "true"}">${enabled ? "Désactiver" : "Activer"}</button>` : `<span class="row-sub">Lecture seule</span>`}
        </div>
      </div>`;
    }).join("");

    const globalEnabled = rp.enabled !== false;
    routingHtml = `<section class="settings-card">
      <div class="machine-head">
        <div class="machine-title">
          <h2>Politique de routage (${esc(routing.owner_badge || "Registre")})</h2>
          <span class="machine-meta">Révision ${esc(rp.revision || 1)} · <code>${esc(routing.registry_path || "")}</code></span>
        </div>
        <div class="badges">
          <span class="badge${writable ? " guarded" : ""}">${writable ? "Modifiable avec aperçu" : "Lecture seule"}</span>
          ${writable ? `<button class="btn" type="button" id="toggleGlobalRouting">${globalEnabled ? "Mettre le routage en pause" : "Réactiver le routage"}</button>` : ""}
        </div>
      </div>
      <div class="table-card">${mRows}</div>
      <details class="tech-log" style="margin-top:12px">
        <summary>Connexions déclarées (${(rp.connections || []).length})</summary>
        <div class="table-card" style="margin:8px 0 0;border-left:0;padding-left:0">${cRows}</div>
      </details>
    </section>`;
  } else {
    routingHtml = `<div class="empty-box">Politique de routage non disponible sur ce registre local.</div>`;
  }

  const macHelpHtml = `<section class="settings-card">
    <div class="machine-head">
      <div class="machine-title">
        <h2>Machines macOS utilisées à distance</h2>
      </div>
    </div>
    <p style="margin:0;color:var(--muted);font-size:13px;line-height:1.5">
      Sur un Mac contrôlé en SSH (comme un MacBook fermé), macOS verrouille automatiquement le trousseau en veille, ce qui bloque les connexions <strong>Claude Code</strong> et <strong>Cursor CLI</strong> (Codex, Antigravity et Gemini ne sont pas concernés).
      L'onglet <strong>Machines &amp; CLI</strong> détecte automatiquement cet état et vous propose soit de désactiver le verrouillage auto en un clic, soit la commande SSH unique de déverrouillage.
    </p>
  </section>`;

  let fleetUpdatesHtml = "";
  if (updatesData && updatesData.hosts && typeof updatesData.hosts === "object") {
    const globalAuto = updatesData.automatic !== false;
    const hostEntries = Object.entries(updatesData.hosts).sort(([a], [b]) => a.localeCompare(b));
    let availVersion = null;
    for (const [, h] of hostEntries) {
      if (h && h.available) { availVersion = h.available; break; }
    }
    const uRows = hostEntries.map(([mid, h]) => {
      const stage = (h && h.stage) || "not_reported";
      const installed = (h && h.installed) || "—";
      const avail = (h && h.available) || null;
      const modeVal = h && h.update_mode_explicit
        ? (h.update_mode || "auto")
        : "default";
      const effectiveLabel = (h && h.update_mode === "manual") ? "Manuel" : "Automatique";
      const pillCls = stage === "up_to_date" || stage === "active"
        ? "ok"
        : (stage === "available" || stage === "waiting_idle" || stage === "downloading" || stage === "ready" ? "warn" : (stage === "error" || stage === "rolled_back" ? "err" : "muted"));
      const stageLabels = {
        up_to_date: "● À jour",
        active: "● Activée",
        available: "▲ Disponible",
        downloading: "◌ Téléchargement",
        ready: "◌ Prête",
        installing: "◌ Activation",
        waiting_idle: "◌ En attente de repos",
        disabled: "○ Manuel",
        rolled_back: "✕ Repli effectué",
        error: "✕ Erreur",
        stale: "◌ Rapport ancien",
        not_reported: "○ Non observée",
      };
      const stTxt = stageLabels[stage] || stage;
      const verDetail = avail ? `v${installed} → v${avail}` : `v${installed}`;
      return `<div class="row">
        <div>
          <div class="row-label">${esc(mid)}</div>
          <div class="row-sub">Effectif : ${esc(effectiveLabel)} · ${esc(formatTs(h && h.observed_at))}</div>
        </div>
        <div class="row-detail mono">${esc(verDetail)}</div>
        <div class="row-detail">${esc(h && h.report_freshness === "fresh" ? "Rapport récent" : (h && h.report_freshness === "stale" ? "Rapport ancien" : "Non observé"))}</div>
        <div><span class="status-pill ${pillCls}"><span>${esc(stTxt)}</span></span></div>
        <div class="row-actions">
          <select class="compact-select" data-upd-host="${esc(mid)}" aria-label="Mode de mise à jour pour ${esc(mid)}">
            <option value="auto"${modeVal === "auto" ? " selected" : ""}>Automatique</option>
            <option value="manual"${modeVal === "manual" ? " selected" : ""}>Manuel</option>
            <option value="default"${modeVal === "default" ? " selected" : ""}>Défaut (${globalAuto ? "auto" : "manuel"})</option>
          </select>
        </div>
      </div>`;
    }).join("");

    fleetUpdatesHtml = `<section class="settings-card">
      <div class="machine-head">
        <div class="machine-title">
          <h2>Mises à jour Bridget de la flotte</h2>
          <span class="machine-meta">Source : GitHub Releases · vérification gén. ${esc(updatesData.check_generation || 0)}</span>
        </div>
        <div class="badges">
          ${availVersion ? `<button class="btn primary" type="button" id="updatesInstallBtn" data-ver="${esc(availVersion)}">Installer v${esc(availVersion)}</button>` : ""}
          <button class="btn" type="button" id="updatesCheckBtn">Vérifier maintenant</button>
        </div>
      </div>
      <div class="settings-list">
        <label class="setting-row">
          <div>
            <div class="row-label">Mode par défaut de la flotte : automatique</div>
            <div class="row-sub">Les serveurs sans réglage propre téléchargent, vérifient (SHA-256 + compatibilité) et redémarrent leurs services Bridget (avec repli automatique en cas d'échec).</div>
          </div>
          <input type="checkbox" id="optFleetAutoDefault" ${globalAuto ? "checked" : ""} />
        </label>
      </div>
      <div class="table-card">${uRows}</div>
    </section>`;
  } else {
    fleetUpdatesHtml = `<section class="settings-card">
      <div class="machine-head">
        <div class="machine-title"><h2>Mises à jour Bridget de la flotte</h2></div>
        <div class="badges"><button class="btn" type="button" id="updatesCheckBtn">Actualiser</button></div>
      </div>
      <div class="row-sub">Statut de mise à jour fédéré non disponible pour l'instant (vérifiez la connexion au daemon Bridget).</div>
    </section>`;
  }

  root.innerHTML = fleetUpdatesHtml + autoHtml + routingHtml + macHelpHtml;

  const fleetDefaultCb = document.getElementById("optFleetAutoDefault");
  if (fleetDefaultCb) {
    fleetDefaultCb.onchange = () => onUpdatesGlobal(fleetDefaultCb.checked);
  }
  const updCheckBtn = document.getElementById("updatesCheckBtn");
  if (updCheckBtn) updCheckBtn.onclick = () => onUpdatesCheck();
  const updInstBtn = document.getElementById("updatesInstallBtn");
  if (updInstBtn) updInstBtn.onclick = () => onUpdatesInstall(updInstBtn.getAttribute("data-ver"));
  for (const sel of root.querySelectorAll("[data-upd-host]")) {
    sel.onchange = () => onUpdatesHostMode(sel.getAttribute("data-upd-host"), sel.value);
  }

  document.getElementById("saveAutoBtn").onclick = () => {
    onSaveAutomanaged({
      cli_updates: document.getElementById("optCliUpdates").checked,
      model_sync: document.getElementById("optModelSync").checked,
      interval_secs: Number(document.getElementById("optInterval").value),
      propagate_remotes: true,
    });
  };
  document.getElementById("syncFleetBtn").onclick = () => onSyncFleet();

  const globalBtn = document.getElementById("toggleGlobalRouting");
  if (globalBtn && rp) {
    globalBtn.onclick = () => onRoutingPreview({ kind: "set_global_enabled", enabled: !(rp.enabled !== false) });
  }
  for (const b of root.querySelectorAll("[data-toggle-m]")) {
    b.onclick = () => onRoutingPreview({
      kind: "set_machine_enabled",
      machine_id: b.getAttribute("data-toggle-m"),
      enabled: b.getAttribute("data-next") === "true",
    });
  }
  for (const sel of root.querySelectorAll("[data-cap-m]")) {
    sel.onchange = () => onRoutingPreview({
      kind: "set_machine_capacity",
      machine_id: sel.getAttribute("data-cap-m"),
      capacity: Number(sel.value),
    });
  }
  for (const b of root.querySelectorAll("[data-toggle-c]")) {
    b.onclick = () => onRoutingPreview({
      kind: "set_connection_enabled",
      connection_id: b.getAttribute("data-toggle-c"),
      enabled: b.getAttribute("data-next") === "true",
    });
  }
}
