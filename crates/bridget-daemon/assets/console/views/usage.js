function esc(value) {
  return String(value ?? "").replace(/[&<>"']/g, (ch) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  }[ch]));
}

function formatAge(observedAt, now) {
  if (!observedAt) return "jamais observé";
  const d = Math.max(0, (now || Math.floor(Date.now() / 1000)) - observedAt);
  if (d < 10) return "à l'instant";
  if (d < 60) return `il y a ${d}s`;
  if (d < 3600) return `il y a ${Math.floor(d / 60)} min`;
  return `il y a ${Math.floor(d / 3600)}h`;
}

function formatReset(resetsAt, now) {
  if (!resetsAt) return "";
  const ref = now || Math.floor(Date.now() / 1000);
  const diff = resetsAt - ref;
  if (diff <= 0) return "Réinitialisé";
  const hours = Math.floor(diff / 3600);
  const mins = Math.floor((diff % 3600) / 60);
  const rel = hours >= 24
    ? `dans ${Math.floor(hours / 24)}j ${hours % 24}h`
    : (hours > 0 ? `dans ${hours}h ${String(mins).padStart(2, "0")}m` : `dans ${mins} min`);
  try {
    const abs = new Date(resetsAt * 1000).toLocaleString("fr-FR", { dateStyle: "short", timeStyle: "short" });
    return `Reset ${rel} (${abs})`;
  } catch {
    return `Reset ${rel}`;
  }
}

function groupPassiveOrUnexposed(subs) {
  const active = [];
  const groups = new Map();
  for (const s of subs) {
    if ((s.source_state === "live_probe" || s.source_state === "observed" || s.source_state === "stale_reset") && s.windows && s.windows.length > 0) {
      active.push(s);
      continue;
    }
    const key = `${s.engine}::${s.source_state}`;
    if (!groups.has(key)) {
      const reason = s.engine === "claude"
        ? "Jauges 5h/7j mises à jour passivement dès la prochaine session Claude active (StatusLine / stream-json, 0 jeton)."
        : (s.source_state === "not_exposed_by_provider"
            ? "Compteur de quota restant non exposé en lecture libre par le fournisseur."
            : (s.detail || "Machine hors ligne ou sonde indisponible."));
      groups.set(key, {
        engine: s.engine,
        provider_label: s.provider_label || s.engine,
        source_state: s.source_state,
        badge: s.source_state === "not_exposed_by_provider"
          ? "Non exposé"
          : (s.engine === "claude" ? "En attente" : "Indisponible"),
        machines: new Set(),
        connections: 0,
        reason,
      });
    }
    const g = groups.get(key);
    const ms = (s.machines && s.machines.length) ? s.machines : [s.machine_id];
    for (const m of ms) if (m) g.machines.add(m);
    g.connections += (s.connection_ids && s.connection_ids.length) ? s.connection_ids.length : 1;
  }
  return { active, others: Array.from(groups.values()) };
}

export function renderUsageView(ctx) {
  const { data, filter, refreshing, refreshingId, onFilter, onRefresh } = ctx;
  const root = document.getElementById("accountsRoot");
  const banner = document.getElementById("alertBanner");
  if (banner) banner.classList.add("hidden");

  document.getElementById("viewTitle").textContent = "Quotas & Abonnements";
  if (!data) {
    document.getElementById("viewSummary").textContent = "Lecture des quotas restants (0 jeton LLM)…";
    root.innerHTML = `<div class="empty-box">Collecte des sondes gratuites (Codex, Antigravity) et des observations passives (Claude)…</div>`;
    return;
  }

  const subs = data.subscriptions || [];
  const filtered = filter && filter !== "all"
    ? subs.filter((s) => s.engine === filter)
    : subs;
  const sum = data.summary || { total_subscriptions: subs.length, with_live_quota: 0, not_exposed: 0, exhausted: 0 };

  document.getElementById("viewSummary").textContent =
    `${sum.with_live_quota} abonnement(s) actif(s) · mis à jour ${formatAge(data.generated_at, Math.floor(Date.now() / 1000))} (0 jeton LLM)`;

  const countFor = (eng) => eng === "all" ? subs.length : subs.filter((s) => s.engine === eng).length;
  const filters = [
    ["all", `Tous (${countFor("all")})`],
    ["codex", `Codex (${countFor("codex")})`],
    ["antigravity", `Antigravity (${countFor("antigravity")})`],
    ["claude", `Claude (${countFor("claude")})`],
    ["cursor", `Cursor (${countFor("cursor")})`],
  ];

  const filterBar = `<div class="quota-toolbar">
    <div class="filter-pills" role="tablist" aria-label="Filtrer par fournisseur">
      ${filters.map(([id, lbl]) => {
        const active = (filter || "all") === id ? " active" : "";
        return `<button class="filter-pill${active}" type="button" data-usage-filter="${esc(id)}">${esc(lbl)}</button>`;
      }).join("")}
    </div>
    <button class="btn primary" type="button" id="usageForceRefreshBtn" ${refreshing ? "disabled" : ""}>
      ${refreshing && !refreshingId ? '<span class="spinner"></span> Actualisation…' : "Actualiser tout (0 jeton)"}
    </button>
  </div>`;

  // Bandeau CPAMC : capacité agrégée par fournisseur + segments colorés par compte.
  const totals = (data.provider_totals || []).filter((t) => !filter || filter === "all" || t.engine === filter);
  const totalsHtml = totals.length === 0
    ? ""
    : `<div class="quota-totals-grid">${totals.map((t) => {
        const segs = (t.segments || []).map((seg) => {
          const pct = typeof seg.remaining_percent === "number" ? seg.remaining_percent : 0;
          const tone = seg.tone === "danger" ? "err" : (seg.tone === "warn" ? "warn" : "ok");
          return `<div class="seg-track" title="${esc(seg.label)} : ${pct}%"><div class="seg-fill ${tone}" style="width:${pct}%"></div></div>`;
        }).join("");
        const rst = formatReset(t.next_reset_at, data.generated_at);
        return `<section class="quota-total-card" data-usage-filter="${esc(t.engine)}" style="cursor:pointer">
          <div class="quota-total-head">
            <span class="row-label">${esc(t.label)}</span>
            <span class="row-sub">${t.credentials_count} compte${t.credentials_count > 1 ? "s" : ""}</span>
          </div>
          <div class="quota-total-val"><strong>${t.remaining_sum}%</strong> <span class="row-sub">sur ${t.capacity_max}%</span></div>
          <div class="seg-row">${segs}</div>
          <div class="quota-total-foot"><span>${esc(t.window_label)}</span><span>${esc(rst)}</span></div>
        </section>`;
      }).join("")}</div>`;

  // Alerte seuil bas / saturation.
  const alerts = (data.alerts || []).filter((a) => !filter || filter === "all" || a.engine === filter);
  const alertsHtml = alerts.length === 0
    ? ""
    : `<div class="quota-alert-bar"><strong>▲ Seuil bas :</strong> ${alerts.map((a) => {
        const rst = formatReset(a.resets_at, data.generated_at);
        return `<span>${esc(a.account_label)} (${esc(a.window_label)}) : <strong>${a.remaining_percent}%</strong>${rst ? ` · ${esc(rst)}` : ""}</span>`;
      }).join(" — ")}</div>`;

  const { active, others } = groupPassiveOrUnexposed(filtered);

  const activeHtml = active.length === 0
    ? ""
    : `<div class="quota-grid">${active.map((s) => {
        const showPlan = s.plan_type && !String(s.account_label || "").toLowerCase().includes(String(s.plan_type).toLowerCase());
        const plan = showPlan ? `<span class="badge guarded" style="text-transform:uppercase">${esc(s.plan_type)}</span>` : "";
        const age = formatAge(s.observed_at, data.generated_at);
        const machinesTxt = (s.machines && s.machines.length) ? s.machines.join(" · ") : s.machine_id;
        const stateCls = s.source_state === "live_probe" || s.source_state === "observed" ? "ok" : "warn";
        const stateText = s.source_state === "live_probe" ? "● Live" : (s.source_state === "observed" ? "● Passif" : "◌ Réinitialisé");
        const canRefresh = s.engine === "codex" || s.engine === "antigravity";
        const isBusy = refreshing && refreshingId === s.id;
        const windowsHtml = s.windows.map((w) => {
          if (w.status === "reset_elapsed") {
            return `<div class="quota-win">
              <div class="quota-win-head">
                <span>${esc(w.label)}</span>
                <span class="row-sub">Réinitialisée</span>
              </div>
              <div class="quota-track"><div class="quota-fill muted" style="width:0%"></div></div>
            </div>`;
          }
          const rst = formatReset(w.resets_at, data.generated_at);
          if (typeof w.used_percent !== "number" && w.status !== "rejected") {
            return `<div class="quota-win">
              <div class="quota-win-head">
                <span class="quota-win-label">${esc(w.label)}</span>
                <span class="quota-win-val ok"><strong>Disponible</strong> <span class="row-sub">(% non publié)</span></span>
              </div>
              ${rst ? `<div class="quota-win-foot">${esc(rst)}</div>` : ""}
            </div>`;
          }
          const used = typeof w.used_percent === "number" ? w.used_percent : (w.status === "rejected" ? 100 : 0);
          const rem = typeof w.remaining_percent === "number" ? w.remaining_percent : Math.max(0, 100 - used);
          const tone = w.status === "rejected" || used >= 90 ? "err" : (used >= 70 ? "warn" : "ok");
          const deltaHtml = typeof w.delta_1h === "number"
            ? ` <span class="row-sub">(${w.delta_1h > 0 ? `+${w.delta_1h}` : w.delta_1h}%/1h)</span>`
            : "";
          return `<div class="quota-win">
            <div class="quota-win-head">
              <span class="quota-win-label">${esc(w.label)}</span>
              <span class="quota-win-val ${tone}"><strong>${rem}%</strong> restant <span class="row-sub">(${used}% utilisé)</span>${deltaHtml}</span>
            </div>
            <div class="quota-track"><div class="quota-fill ${tone}" style="width:${rem}%"></div></div>
            ${rst ? `<div class="quota-win-foot">${esc(rst)}</div>` : ""}
          </div>`;
        }).join("");
        const resetsLine = typeof s.manual_resets_available === "number"
          ? `<div class="quota-meta-line">Resets manuels Codex : <strong>${s.manual_resets_available}</strong> disponible${s.manual_resets_available > 1 ? "s" : ""}</div>`
          : "";
        const creditsLine = s.credits_unlimited
          ? `<div class="quota-meta-line">Crédits : illimités</div>`
          : (s.credits_balance && s.credits_balance !== "0"
              ? `<div class="quota-meta-line">Crédits additionnels : ${esc(s.credits_balance)}</div>`
              : "");
        const conns = (s.connection_ids || []).join(", ");
        return `<section class="quota-card">
          <div class="quota-card-head">
            <div class="quota-card-meta">
              <div class="quota-card-title"><span>${esc(s.account_label)}</span> ${plan}</div>
              <div class="row-sub">${esc(s.provider_label)} · ${esc(machinesTxt)} · ${esc(age)}</div>
            </div>
            <span class="status-pill ${stateCls}" style="flex-shrink:0;white-space:nowrap"><span>${esc(stateText)}</span></span>
          </div>
          <div class="quota-windows">${windowsHtml}</div>
          ${resetsLine}
          ${creditsLine}
          <div class="quota-card-foot">
            <span class="row-sub" title="${esc(conns)}">${esc(s.source_label)}</span>
            ${canRefresh ? `<button class="btn subtle" style="padding:3px 9px;font-size:11.5px" type="button" data-refresh-sub="${esc(s.id)}" ${refreshing ? "disabled" : ""}>${isBusy ? '<span class="spinner"></span>' : "↻ Actualiser"}</button>` : ""}
          </div>
        </section>`;
      }).join("")}</div>`;

  const othersHtml = others.length === 0
    ? ""
    : `<section class="settings-card">
        <div class="row-label">Autres fournisseurs</div>
        <div class="settings-list">
          ${others.map((g) => {
            const cls = g.source_state === "not_exposed_by_provider" ? "muted" : (g.engine === "claude" ? "warn" : "muted");
            const hosts = Array.from(g.machines).sort().join(", ");
            const plural = g.connections > 1 ? "s" : "";
            return `<div class="quota-other-row">
              <div class="quota-other-head">
                <strong>${esc(g.provider_label)}</strong>
                <span class="status-pill ${cls}"><span>${esc(g.badge)}</span></span>
                <span class="row-sub">${esc(hosts)} (${g.connections} connexion${plural})</span>
              </div>
              <div class="row-sub">${esc(g.reason)}</div>
            </div>`;
          }).join("")}
        </div>
      </section>`;

  const bodyHtml = (!activeHtml && !othersHtml)
    ? `<div class="empty-box">Aucun abonnement trouvé pour ce filtre.</div>`
    : totalsHtml + alertsHtml + activeHtml + othersHtml;

  const infoNote = `<section class="settings-card">
    <div class="row-label">Comment ces jauges restent à jour à 0 jeton LLM</div>
    <p style="margin:0;color:var(--muted);font-size:12.5px;line-height:1.5">
      <strong>Codex</strong> (<code>account/rateLimits/read</code>, ~0,6 s) et <strong>Antigravity</strong> (<code>agy -p /usage --output-format json</code>, ~1,7 s) sont interrogés via leurs commandes officielles à 0 jeton, dédupliqués par abonnement et mis en cache 5 minutes.
      <strong>Claude</strong> remonte automatiquement ses fenêtres 5h / 7j via son hook officiel <code>statusLine</code> (y compris en session interactive) et <code>stream-json</code>.
      Depuis le terminal : <code>bridget ui</code> (ou <code>bridget ui usage</code>) ouvre cette console ; en conversation : <code>/bridget usage</code> ou <code>$bridget-usage</code>.
    </p>
  </section>`;

  root.innerHTML = filterBar + bodyHtml + infoNote;

  for (const b of root.querySelectorAll("[data-usage-filter]")) {
    b.onclick = () => onFilter(b.getAttribute("data-usage-filter"));
  }
  const forceBtn = document.getElementById("usageForceRefreshBtn");
  if (forceBtn) forceBtn.onclick = () => onRefresh(true, null);
  for (const rb of root.querySelectorAll("[data-refresh-sub]")) {
    rb.onclick = () => onRefresh(true, rb.getAttribute("data-refresh-sub"));
  }
}
