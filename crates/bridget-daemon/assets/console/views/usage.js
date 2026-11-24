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
        ? "Jauges 5h/7j mises à jour passivement dès la prochaine session Claude active (0 jeton)."
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
  const { data, filter, refreshing, onFilter, onRefresh } = ctx;
  const root = document.getElementById("accountsRoot");
  const banner = document.getElementById("alertBanner");
  if (banner) banner.classList.add("hidden");

  document.getElementById("viewTitle").textContent = "Quotas & Abonnements";
  if (!data) {
    document.getElementById("viewSummary").textContent = "Lecture des quotas restants (0 jeton LLM)…";
    root.innerHTML = `<div class="empty-box">Collecte des sondes gratuites (Codex) et des observations passives (Claude)…</div>`;
    return;
  }

  const subs = data.subscriptions || [];
  const filtered = filter && filter !== "all"
    ? subs.filter((s) => s.engine === filter)
    : subs;
  const sum = data.summary || { total_subscriptions: subs.length, with_live_quota: 0, not_exposed: 0, exhausted: 0 };

  document.getElementById("viewSummary").textContent =
    `${sum.with_live_quota} jauge(s) active(s) · mis à jour ${formatAge(data.generated_at, Math.floor(Date.now() / 1000))} (0 jeton LLM)`;

  const filters = [
    ["all", "Tous"],
    ["codex", "Codex (OpenAI)"],
    ["claude", "Claude (Anthropic)"],
    ["antigravity", "Antigravity (Google)"],
    ["cursor", "Cursor"],
  ];

  const filterBar = `<div class="quota-toolbar">
    <div class="filter-pills" role="tablist" aria-label="Filtrer par fournisseur">
      ${filters.map(([id, lbl]) => {
        const active = (filter || "all") === id ? " active" : "";
        return `<button class="filter-pill${active}" type="button" data-usage-filter="${esc(id)}">${esc(lbl)}</button>`;
      }).join("")}
    </div>
    <button class="btn primary" type="button" id="usageForceRefreshBtn" ${refreshing ? "disabled" : ""}>
      ${refreshing ? '<span class="spinner"></span> Actualisation…' : "Actualiser maintenant (0 jeton)"}
    </button>
  </div>`;

  const { active, others } = groupPassiveOrUnexposed(filtered);

  const activeHtml = active.length === 0
    ? ""
    : `<div class="quota-grid">${active.map((s) => {
        const plan = s.plan_type ? `<span class="badge guarded" style="text-transform:uppercase">${esc(s.plan_type)}</span>` : "";
        const age = formatAge(s.observed_at, data.generated_at);
        const machinesTxt = (s.machines && s.machines.length) ? s.machines.join(" · ") : s.machine_id;
        const stateCls = s.source_state === "live_probe" || s.source_state === "observed" ? "ok" : "warn";
        const stateText = s.source_state === "live_probe" ? "● Live" : (s.source_state === "observed" ? "● Passif" : "◌ Réinitialisé");
        const windowsHtml = s.windows.map((w) => {
          if (w.status === "reset_elapsed") {
            return `<div class="quota-win">
              <div class="quota-win-head">
                <span>Fenêtre ${esc(w.label)}</span>
                <span class="row-sub">Réinitialisée</span>
              </div>
              <div class="quota-track"><div class="quota-fill muted" style="width:0%"></div></div>
            </div>`;
          }
          const used = typeof w.used_percent === "number" ? w.used_percent : (w.status === "rejected" ? 100 : 0);
          const rem = typeof w.remaining_percent === "number" ? w.remaining_percent : Math.max(0, 100 - used);
          const tone = w.status === "rejected" || used >= 90 ? "err" : (used >= 70 ? "warn" : "ok");
          const rst = formatReset(w.resets_at, data.generated_at);
          return `<div class="quota-win">
            <div class="quota-win-head">
              <span class="quota-win-label">Fenêtre ${esc(w.label)}</span>
              <span class="quota-win-val ${tone}"><strong>${rem}%</strong> restant <span class="row-sub">(${used}% utilisé)</span></span>
            </div>
            <div class="quota-track"><div class="quota-fill ${tone}" style="width:${rem}%"></div></div>
            ${rst ? `<div class="quota-win-foot">${esc(rst)}</div>` : ""}
          </div>`;
        }).join("");
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
          ${creditsLine}
          <div class="quota-card-foot">
            <span class="row-sub" title="${esc(conns)}">${esc(s.source_label)}</span>
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
    : activeHtml + othersHtml;

  const infoNote = `<section class="settings-card">
    <div class="row-label">Comment ces jauges restent à jour sans coûter de jetons</div>
    <p style="margin:0;color:var(--muted);font-size:12.5px;line-height:1.5">
      <strong>Codex (OpenAI)</strong> expose un appel JSON-RPC gratuit (<code>account/rateLimits/read</code>, ~0,6 s, 0 jeton) dédupliqué par abonnement et mis en cache 5 minutes.
      <strong>Claude (Anthropic)</strong> publie automatiquement ses fenêtres 5h / 7j pendant les sessions actives (<code>rate_limit_event</code> et StatusLine), mémorisées passivement dans <code>usage-quotas.json</code>.
      <strong>Antigravity</strong> et <strong>Cursor</strong> n'exposent pas leur quota restant en lecture libre ; Bridget l'indique honnêtement sans fabriquer de pourcentage.
      En session, utilisez <code>/bridget usage [claude|codex|agy|cursor]</code>.
    </p>
  </section>`;

  root.innerHTML = filterBar + bodyHtml + infoNote;

  for (const b of root.querySelectorAll("[data-usage-filter]")) {
    b.onclick = () => onFilter(b.getAttribute("data-usage-filter"));
  }
  const forceBtn = document.getElementById("usageForceRefreshBtn");
  if (forceBtn) forceBtn.onclick = () => onRefresh(true);
}
