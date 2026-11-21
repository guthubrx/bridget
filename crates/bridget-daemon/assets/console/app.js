import { apiRequest } from "/assets/api.js";
import { renderAccountsView } from "/assets/views/accounts.js";

const mediaDark = window.matchMedia ? window.matchMedia("(prefers-color-scheme: dark)") : null;
let state = { data: null, activeView: "all", themePref: document.documentElement.getAttribute("data-theme-pref") || "system" };

function resolveTheme(pref) {
  if (pref === "light" || pref === "dark") return pref;
  return mediaDark && mediaDark.matches ? "dark" : "light";
}

function applyTheme(pref) {
  state.themePref = pref;
  const resolved = resolveTheme(pref);
  document.documentElement.setAttribute("data-theme-pref", pref);
  document.documentElement.setAttribute("data-theme", resolved);
  for (const btn of document.querySelectorAll(".theme-opt")) {
    btn.setAttribute("aria-checked", String(btn.getAttribute("data-theme-value") === pref));
  }
  const lbl = document.getElementById("themeResolved");
  lbl.textContent = pref === "system"
    ? (resolved === "dark" ? "Système · Sombre actif" : "Système · Clair actif")
    : "";
}

function showFeedback(kind, icon, text) {
  const bar = document.getElementById("feedbackBar");
  bar.className = `feedback ${kind}`;
  bar.innerHTML = `<span><strong>${icon}</strong> ${text.replace(/[<>&]/g, (c) => ({ "<": "&lt;", ">": "&gt;", "&": "&amp;" }[c]))}</span>
    <button class="btn subtle" type="button" id="fbClose">Fermer</button>`;
  bar.classList.remove("hidden");
  document.getElementById("fbClose").onclick = () => bar.classList.add("hidden");
}

function openModal(title, bodyHtml, footButtons) {
  document.getElementById("modalTitle").textContent = title;
  document.getElementById("modalBody").innerHTML = bodyHtml;
  const foot = document.getElementById("modalFoot");
  foot.innerHTML = "";
  for (const cfg of footButtons) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = `btn${cfg.primary ? " primary" : ""}`;
    btn.textContent = cfg.label;
    btn.onclick = cfg.onClick;
    foot.appendChild(btn);
  }
  document.getElementById("modalBackdrop").classList.remove("hidden");
}

function closeModal() {
  document.getElementById("modalBackdrop").classList.add("hidden");
}

async function handleTogglePreview(conn) {
  const action = conn.enabled ? "disable" : "enable";
  const verb = action === "enable" ? "Activer" : "Désactiver";
  try {
    const preview = await apiRequest("/api/accounts/preview", {
      method: "POST",
      body: JSON.stringify({ connection_id: conn.connection_id, machine_id: conn.machine_id, action }),
    });
    const body = `<p>Vérifiez la portée exacte avant d'écrire dans le registre.</p>
      <dl class="kv">
        <dt>Action</dt><dd>${verb} <code>${conn.connection_id}</code></dd>
        <dt>Propriétaire</dt><dd>${preview.owner_badge}</dd>
        <dt>Registre</dt><dd><code>${preview.registry_path}</code></dd>
        <dt>Champs modifiés</dt><dd><code>routing_policy.connections[].enabled</code>, <code>routing_policy.revision</code></dd>
        <dt>Empreinte SHA-256</dt><dd><code>${preview.expected_sha256.slice(0, 16)}…</code></dd>
      </dl>`;
    openModal(`${verb} le compte`, body, [
      { label: "Annuler", onClick: closeModal },
      {
        label: `Confirmer (${verb.toLowerCase()})`,
        primary: true,
        onClick: async () => {
          closeModal();
          try {
            const res = await apiRequest("/api/accounts/apply", {
              method: "POST",
              body: JSON.stringify({ preview_id: preview.preview_id }),
            });
            showFeedback("ok", "✓", res.ui_state || "Enregistré et relu — prise en compte à confirmer");
            await loadStatus();
          } catch (e) {
            const st = e.payload && e.payload.ui_state ? e.payload.ui_state : `Refusé : ${e.message}`;
            showFeedback(e.status === 409 ? "warn" : "err", e.status === 409 ? "▲" : "✕", st);
          }
        },
      },
    ]);
  } catch (e) {
    const st = e.payload && e.payload.ui_state ? e.payload.ui_state : `Refusé : ${e.message}`;
    showFeedback("err", "✕", st);
  }
}

function handleShowCli(conn) {
  const cmd = conn.cli_hint || `bridget accounts list --registry ${conn.registry_path || "<registre>"}`;
  const body = `<p>Cette cible est en <strong>lecture seule</strong> dans ce premier lot (ou relève d'une action CLI dédiée).</p>
    <dl class="kv">
      <dt>Machine</dt><dd>${conn.machine_id}</dd>
      <dt>Propriétaire</dt><dd>${conn.owner_badge || "Propriétaire indéterminé · Lecture seule"}</dd>
      <dt>Registre</dt><dd><code>${conn.registry_path || "non attesté"}</code></dd>
    </dl>
    <div class="cmd-box"><code>${cmd.replace(/[<>&]/g, (c) => ({ "<": "&lt;", ">": "&gt;", "&": "&amp;" }[c]))}</code></div>`;
  openModal("Commande CLI de référence", body, [{ label: "Fermer", primary: true, onClick: closeModal }]);
}

function render() {
  if (!state.data) return;
  document.getElementById("hostBadge").textContent = `Hôte console : ${state.data.local_machine}`;
  renderAccountsView({
    data: state.data,
    activeView: state.activeView,
    onSelectView: (v) => { state.activeView = v; render(); },
    onTogglePreview: handleTogglePreview,
    onShowCli: handleShowCli,
  });
}

async function loadStatus() {
  const btn = document.getElementById("refreshBtn");
  btn.disabled = true;
  try {
    state.data = await apiRequest("/api/status");
    render();
  } catch (e) {
    showFeedback("err", "✕", `Refusé : ${e.message}`);
  } finally {
    btn.disabled = false;
  }
}

for (const btn of document.querySelectorAll(".theme-opt")) {
  btn.onclick = async () => {
    const value = btn.getAttribute("data-theme-value");
    applyTheme(value);
    try {
      await apiRequest("/api/ui-prefs", { method: "POST", body: JSON.stringify({ theme: value }) });
    } catch (e) {
      showFeedback("warn", "▲", `Thème appliqué mais non enregistré : ${e.message}`);
    }
  };
}
if (mediaDark) mediaDark.addEventListener("change", () => { if (state.themePref === "system") applyTheme("system"); });
document.getElementById("refreshBtn").onclick = loadStatus;
document.getElementById("modalCloseBtn").onclick = closeModal;
document.getElementById("cliHelpBtn").onclick = () => openModal("Actions CLI disponibles", `<p>Dans le Lot 1a-0, la console sécurise l'affichage multi-machines, les thèmes et l'activation/désactivation sur registre local autorisé. Utilisez la CLI pour :</p>
  <div class="cmd-box"><code>bridget accounts status --json<br>bridget accounts login --connection &lt;id&gt; --machine &lt;machine&gt; --registry &lt;registre&gt;<br>bridget accounts list --registry &lt;registre&gt;</code></div>`, [{ label: "Fermer", primary: true, onClick: closeModal }]);
document.addEventListener("keydown", (e) => { if (e.key === "Escape") closeModal(); });

applyTheme(state.themePref);
loadStatus();
