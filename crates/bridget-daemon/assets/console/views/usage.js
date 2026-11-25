function esc(value) {
  return String(value ?? "").replace(/[&<>"']/g, (ch) => ({
    "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;",
  }[ch]));
}

function formatResetShort(resetsAt, now, withPrefix) {
  if (!resetsAt) return "";
  const diff = resetsAt - (now || Math.floor(Date.now() / 1000));
  if (diff <= 0) return "réinitialisé";
  const hours = Math.floor(diff / 3600);
  const mins = Math.floor((diff % 3600) / 60);
  const dur = hours >= 24
    ? `${Math.floor(hours / 24)}j ${hours % 24}h`
    : (hours > 0 ? `${hours}h ${String(mins).padStart(2, "0")}m` : `${mins} min`);
  return withPrefix ? `reset ${dur}` : dur;
}

function formatShortDate(epochSecs) {
  try {
    return new Date(epochSecs * 1000).toLocaleDateString("fr-FR", { day: "numeric", month: "short" });
  } catch {
    return "";
  }
}

function toneForRem(rem, status) {
  if (status === "rejected" || rem <= 10) return "err";
  if (rem <= 30) return "warn";
  return "ok";
}

function vendorSubs(subs, engine) {
  const all = subs.filter((s) => s.engine === engine);
  const active = all.filter((s) =>
    ["live_probe", "observed", "connected_no_quota", "stale_reset"].includes(s.source_state)
  );
  return active.length ? active : all;
}

function renderChips(sub) {
  const hosts = (sub.machines && sub.machines.length) ? sub.machines : (sub.machine_id ? [sub.machine_id] : []);
  return `<div class="u-chips">${hosts.map((m) => `<span class="u-chip">${esc(m)}</span>`).join("")}</div>`;
}

function renderResets(sub, now) {
  if ((sub.engine !== "codex" && sub.engine !== "claude") || typeof sub.manual_resets_available !== "number") {
    return `<div class="u-resets"><div class="u-muted">—</div></div>`;
  }
  const n = sub.manual_resets_available;
  const exps = (sub.manual_reset_expirations || []).filter((t) => t > now).slice(0, 2).map(formatShortDate).filter(Boolean);
  const subTxt = n > 0 && exps.length ? `exp. ${exps.join(", ")}` : "—";
  return `<div class="u-resets"><div><strong>${n}</strong> reset${n > 1 ? "s" : ""}</div><div class="u-rsub">${esc(subTxt)}</div></div>`;
}

function renderActs(sub, refreshing, refreshingId, withRefresh) {
  const isBusy = refreshing && refreshingId === sub.id;
  const refBtn = withRefresh
    ? `<button class="u-btn" type="button" title="Actualiser" data-refresh-sub="${esc(sub.id)}" ${refreshing ? "disabled" : ""}>${isBusy ? '<span class="spinner"></span>' : "↻"}</button>`
    : "";
  const disconnected = sub.logged_in === false;
  const lbl = disconnected ? "Connecter" : "Changer";
  const cls = disconnected ? "u-btn warn-b" : "u-btn";
  return `<div class="u-acts">${refBtn}<button class="${cls}" type="button" data-manage-sub="${esc(sub.id)}">${lbl}</button></div>`;
}

function renderMachineCell(machineId, engine, cell) {
  if (!cell) return `<td class="u-muted">—</td>`;
  if (cell.status === "offline") return `<td class="u-muted">Hors ligne</td>`;
  const dotCls = cell.tone === "danger" ? "err" : (cell.tone === "warn" ? "warn" : (cell.tone === "ok" ? "ok" : "off"));
  const line = cell.account
    ? esc(cell.account)
    : `<span class="u-muted">${esc(cell.summary || "Non connecté")}</span>`;
  const subParts = [];
  if (cell.account && cell.summary) {
    const isPct = cell.summary.endsWith("%") && !cell.summary.includes("/");
    subParts.push(isPct ? `<span class="u-${dotCls}">${esc(cell.summary)}</span>` : esc(cell.summary));
  }
  if (cell.extra) subParts.push(esc(cell.extra));
  const subHtml = subParts.length ? `<div class="u-csub">${subParts.join(" · ")}</div>` : "";
  const clickAttr = cell.connection_id
    ? ` class="u-mcell" data-mcell-machine="${esc(machineId)}" data-mcell-engine="${esc(engine)}" title="Cliquer pour connecter ou changer le compte sur ${esc(machineId)}"`
    : "";
  return `<td${clickAttr}><div class="u-cell"><div class="u-cline"><span class="dot ${dotCls}"></span>${line}</div>${subHtml}</div></td>`;
}

export function renderUsageView(ctx) {
  const { data, filter, refreshing, refreshingId, onFilter, onRefresh, onManageAccount, onManageCell } = ctx;
  const root = document.getElementById("accountsRoot");
  const banner = document.getElementById("alertBanner");
  if (banner) banner.classList.add("hidden");

  document.getElementById("viewTitle").textContent = "Quotas & Abonnements";
  if (!data) {
    document.getElementById("viewSummary").textContent = "Chargement…";
    root.innerHTML = `<div class="empty-box">Lecture des comptes et quotas…</div>`;
    return;
  }

  const now = data.generated_at || Math.floor(Date.now() / 1000);
  const subs = data.subscriptions || [];
  const activeFilter = filter || "all";
  document.getElementById("viewSummary").textContent = "";

  const filters = [
    ["all", "Tous"],
    ["codex", "Codex"],
    ["antigravity", "Antigravity"],
    ["claude", "Claude"],
    ["other", "Autres"],
  ];

  const toolbar = `<div class="quota-toolbar">
    <div class="filter-pills" role="tablist">
      ${filters.map(([id, lbl]) => `<button class="filter-pill${activeFilter === id ? " active" : ""}" type="button" data-usage-filter="${id}">${lbl}</button>`).join("")}
    </div>
    <button class="btn primary" type="button" id="usageForceRefreshBtn" ${refreshing ? "disabled" : ""}>
      ${refreshing && !refreshingId ? '<span class="spinner"></span> Actualisation…' : "↻ Actualiser"}
    </button>
  </div>`;

  const subById = new Map(subs.map((s) => [s.id, s]));
  const sections = [];

  if (activeFilter === "all" || activeFilter === "codex") {
    const list = vendorSubs(subs, "codex");
    if (list.length) {
      const tot = (data.provider_totals || []).find((t) => t.engine === "codex");
      const meta = tot
        ? `<span class="u-${tot.remaining_sum === 0 ? "err" : "ok"}">${tot.remaining_sum} %</span> / ${tot.capacity_max} % · ${tot.credentials_count} compte${tot.credentials_count > 1 ? "s" : ""}`
        : `${list.length} compte${list.length > 1 ? "s" : ""}`;
      const rows = list.map((s) => {
        const plan = s.plan_type ? `<span class="u-plan">${esc(s.plan_type)}</span>` : "";
        const w = (s.windows || []).find((x) => x.label === "7j") || (s.windows || [])[0];
        let gauge = `<div class="u-muted">${esc(s.source_label)}</div>`;
        if (w) {
          const rem = typeof w.remaining_percent === "number" ? w.remaining_percent : 0;
          const tone = toneForRem(rem, w.status);
          const left = (rem === 0 || w.status === "rejected")
            ? "épuisé"
            : (typeof w.delta_1h === "number" ? `${w.delta_1h > 0 ? `+${w.delta_1h}` : w.delta_1h} %/1h` : "");
          const right = formatResetShort(w.resets_at, now, true);
          gauge = `<div class="u-gauge">
            <div class="u-gtop"><span class="u-muted">${esc(w.label)}</span><span class="u-${tone} u-nw">${rem} %</span></div>
            <div class="u-track"><div class="u-fill ${tone}" style="width:${rem}%"></div></div>
            <div class="u-gsub"><span>${esc(left)}</span><span>${esc(right)}</span></div>
          </div>`;
        }
        return `<div class="u-row">
          <div class="u-who"><div><span class="u-email">${esc(s.account_label)}</span>${plan}</div>${renderChips(s)}</div>
          ${gauge}${renderResets(s, now)}${renderActs(s, refreshing, refreshingId, true)}
        </div>`;
      }).join("");
      sections.push(`<section class="u-vendor"><div class="u-vhead"><span class="u-vname">Codex</span><span class="u-vmeta">${meta}</span></div>${rows}</section>`);
    }
  }

  if (activeFilter === "all" || activeFilter === "antigravity") {
    const list = vendorSubs(subs, "antigravity");
    if (list.length) {
      const rows = list.map((s) => {
        const gauges = (s.windows || []).length
          ? `<div class="u-gauges-2">${s.windows.map((w) => {
              const rem = typeof w.remaining_percent === "number" ? w.remaining_percent : 0;
              const tone = toneForRem(rem, w.status);
              return `<div class="u-gauge">
                <div class="u-gtop"><span class="u-muted">${esc(w.label)}</span><span class="u-${tone} u-nw">${rem} %</span></div>
                <div class="u-track"><div class="u-fill ${tone}" style="width:${rem}%"></div></div>
                <div class="u-gsub"><span>${esc(formatResetShort(w.resets_at, now, false))}</span></div>
              </div>`;
            }).join("")}</div>`
          : `<div class="u-muted">${esc(s.source_label)}</div>`;
        return `<div class="u-row">
          <div class="u-who"><div><span class="u-email">${esc(s.account_label)}</span></div>${renderChips(s)}</div>
          ${gauges}${renderResets(s, now)}${renderActs(s, refreshing, refreshingId, true)}
        </div>`;
      }).join("");
      sections.push(`<section class="u-vendor"><div class="u-vhead"><span class="u-vname">Antigravity</span><span class="u-vmeta">${list.length} compte${list.length > 1 ? "s" : ""}</span></div>${rows}</section>`);
    }
  }

  if (activeFilter === "all" || activeFilter === "claude") {
    const list = vendorSubs(subs, "claude");
    if (list.length) {
      const tot = (data.provider_totals || []).find((t) => t.engine === "claude");
      const meta = tot
        ? `<span class="u-${tot.remaining_sum === 0 ? "err" : "ok"}">${tot.remaining_sum} %</span> / ${tot.capacity_max} % · ${tot.credentials_count} compte${tot.credentials_count > 1 ? "s" : ""}`
        : `${list.length} compte${list.length > 1 ? "s" : ""}`;
      const renderClaudeWin = (lbl, w) => {
        if (!w || w.status === "reset_elapsed") {
          return `<div class="u-gauge"><div class="u-gtop"><span class="u-muted">${lbl}</span><span class="u-muted u-nw">—</span></div><div class="u-track"></div><div class="u-gsub"><span>via StatusLine</span></div></div>`;
        }
        const sub = w.status === "idle" ? "inactif" : (formatResetShort(w.resets_at, now, true) || "via StatusLine");
        if (typeof w.remaining_percent === "number") {
          const tone = toneForRem(w.remaining_percent, w.status);
          return `<div class="u-gauge"><div class="u-gtop"><span class="u-muted">${lbl}</span><span class="u-${tone} u-nw">${w.remaining_percent} %</span></div><div class="u-track"><div class="u-fill ${tone}" style="width:${w.remaining_percent}%"></div></div><div class="u-gsub"><span>${esc(sub)}</span></div></div>`;
        }
        return `<div class="u-gauge"><div class="u-gtop"><span class="u-muted">${lbl}</span><span class="u-ok u-nw">Disponible</span></div><div class="u-track"><div class="u-fill neutral" style="width:100%"></div></div><div class="u-gsub"><span>${esc(sub)}</span></div></div>`;
      };
      const rows = list.map((s) => {
        const plan = s.plan_type ? `<span class="u-plan">${esc(s.plan_type)}</span>` : "";
        const w5 = (s.windows || []).find((w) => w.label === "5h");
        const w7 = (s.windows || []).find((w) => w.label === "7j");
        return `<div class="u-row">
          <div class="u-who"><div><span class="u-email">${esc(s.account_label)}</span>${plan}</div>${renderChips(s)}</div>
          <div class="u-gauges-2">${renderClaudeWin("5h", w5)}${renderClaudeWin("7j", w7)}</div>
          ${renderResets(s, now)}${renderActs(s, refreshing, refreshingId, true)}
        </div>`;
      }).join("");
      sections.push(`<section class="u-vendor"><div class="u-vhead"><span class="u-vname">Claude</span><span class="u-vmeta">${meta}</span></div>${rows}</section>`);
    }
  }

  if (activeFilter === "all" || activeFilter === "other") {
    const list = subs.filter((s) => s.engine === "glm" || s.engine === "cursor");
    if (list.length) {
      const rows = list.map((s) => {
        const stCls = s.logged_in === true ? "u-ok" : (s.logged_in === false ? "u-warn" : "u-muted");
        const stTxt = s.logged_in === true ? "Connecté" : (s.logged_in === false ? "Non connecté" : s.source_label);
        return `<div class="u-row">
          <div class="u-who"><div><span class="u-email">${esc(s.account_label)}</span></div>${renderChips(s)}</div>
          <div class="${stCls}">${esc(stTxt)}</div>
          ${renderResets(s, now)}${renderActs(s, refreshing, refreshingId, false)}
        </div>`;
      }).join("");
      sections.push(`<section class="u-vendor"><div class="u-vhead"><span class="u-vname">Autres</span></div>${rows}</section>`);
    }
  }

  const machines = activeFilter === "all" ? (data.machines || []) : [];
  if (machines.length) {
    const tbody = machines.map((m) => `<tr${m.online ? "" : ' class="u-off"'}>
      <td class="u-m">${esc(m.machine_id)}</td>
      ${renderMachineCell(m.machine_id, "codex", m.codex)}
      ${renderMachineCell(m.machine_id, "claude", m.claude)}
      ${renderMachineCell(m.machine_id, "antigravity", m.antigravity)}
      ${renderMachineCell(m.machine_id, "cursor", m.cursor)}
    </tr>`).join("");
    sections.push(`<section class="u-vendor"><div class="u-vhead"><span class="u-vname">Machines</span></div>
      <table class="u-table"><thead><tr><th>Machine</th><th>Codex</th><th>Claude</th><th>Antigravity</th><th>Cursor</th></tr></thead><tbody>${tbody}</tbody></table>
    </section>`);
  }

  root.innerHTML = toolbar + `<div class="u-wrap">${sections.length ? sections.join("") : '<div class="empty-box">Aucun abonnement pour ce filtre.</div>'}</div>`;

  for (const b of root.querySelectorAll("[data-usage-filter]")) {
    b.onclick = () => onFilter(b.getAttribute("data-usage-filter"));
  }
  const forceBtn = document.getElementById("usageForceRefreshBtn");
  if (forceBtn) forceBtn.onclick = () => onRefresh(true, null);
  for (const rb of root.querySelectorAll("[data-refresh-sub]")) {
    rb.onclick = () => onRefresh(true, rb.getAttribute("data-refresh-sub"));
  }
  for (const mb of root.querySelectorAll("[data-manage-sub]")) {
    mb.onclick = () => {
      const sub = subById.get(mb.getAttribute("data-manage-sub"));
      if (sub && onManageAccount) onManageAccount(sub);
    };
  }
  const byMachine = new Map(machines.map((m) => [m.machine_id, m]));
  for (const td of root.querySelectorAll("[data-mcell-machine]")) {
    td.onclick = () => {
      const mid = td.getAttribute("data-mcell-machine");
      const eng = td.getAttribute("data-mcell-engine");
      const row = byMachine.get(mid);
      if (row && row[eng] && onManageCell) onManageCell(mid, eng, row[eng]);
    };
  }
}
