import { apiRequest } from "/assets/api.js";
import { renderAccountsView } from "/assets/views/accounts.js";

const mediaDark = window.matchMedia ? window.matchMedia("(prefers-color-scheme: dark)") : null;
let state = { data: null, activeView: "all", themePref: document.documentElement.getAttribute("data-theme-pref") || "system" };
let loginCtx = { sessionId: null, pollTimer: null, offset: 0, authUrl: null, popup: null, running: false };

function esc(v) {
  return String(v ?? "").replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" }[c]));
}

function closeLoginPopup() {
  if (loginCtx.popup && !loginCtx.popup.closed) { try { loginCtx.popup.close(); } catch {} }
  loginCtx.popup = null;
}

function stopLoginPolling() {
  if (loginCtx.pollTimer) { clearInterval(loginCtx.pollTimer); loginCtx.pollTimer = null; }
}

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
  stopLoginPolling();
  closeLoginPopup();
  if (loginCtx.running && loginCtx.sessionId) {
    const sid = loginCtx.sessionId;
    loginCtx.running = false;
    apiRequest("/api/login/cancel", { method: "POST", body: JSON.stringify({ session_id: sid }) }).catch(() => {});
  }
  loginCtx.sessionId = null;
  document.getElementById("modalBackdrop").classList.add("hidden");
}

function openAuthPopup(url) {
  const w = 620, h = 760;
  const left = Math.max(40, Math.round((screen.width - w) / 2));
  const top = Math.max(40, Math.round((screen.height - h) / 2));
  const p = window.open(url, "bridget_oauth_popup", `width=${w},height=${h},left=${left},top=${top},noopener=no`);
  if (p) { loginCtx.popup = p; try { p.focus(); } catch {} }
  return p;
}

async function handleLogin(conn, logoutFirst, enableOnSuccess) {
  stopLoginPolling();
  closeLoginPopup();
  loginCtx = { sessionId: null, pollTimer: null, offset: 0, authUrl: null, popup: null, running: true };
  const popup = openAuthPopup("about:blank");
  const title = `${logoutFirst ? "Changer de compte" : "Connexion"} · ${conn.account_label || conn.engine} (${conn.machine_id})`;
  const body = `<div id="loginBanner" class="login-banner"><strong>Préparation de la session…</strong>Démarrage de la connexion sur ${esc(conn.machine_id)}.</div>
    <div id="loginUrlRow" class="login-actions hidden"><button class="btn primary" type="button" id="openAuthUrlBtn">Ouvrir la page de connexion</button></div>
    <div id="loginCodeBox" class="code-row hidden">
      <input id="loginCodeInput" type="text" placeholder="Collez le code OAuth si la détection automatique ne l'a pas envoyé" autocomplete="off" />
      <button class="btn primary" type="button" id="sendCodeBtn">Envoyer</button>
    </div>
    <details id="loginLogDetails" class="tech-log"><summary>Journal technique</summary><pre id="loginLog" class="log-pre"></pre></details>`;
  openModal(title, body, [
    { label: "Annuler", onClick: closeModal },
  ]);

  const setBanner = (cls, heading, sub) => {
    const el = document.getElementById("loginBanner");
    if (!el) return;
    el.className = `login-banner${cls ? ` ${cls}` : ""}`;
    el.innerHTML = `<strong>${esc(heading)}</strong>${esc(sub)}`;
  };
  const setFootDone = () => {
    const foot = document.getElementById("modalFoot");
    if (!foot) return;
    foot.innerHTML = "";
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "btn primary";
    btn.textContent = "Fermer";
    btn.onclick = closeModal;
    foot.appendChild(btn);
  };

  try {
    const started = await apiRequest("/api/login/start", {
      method: "POST",
      body: JSON.stringify({
        connection_id: conn.connection_id,
        machine_id: conn.machine_id,
        logout_first: Boolean(logoutFirst),
        enable_on_success: Boolean(enableOnSuccess),
        open_in_browser: !popup,
      }),
    });
    loginCtx.sessionId = started.session_id;
    const openBtn = document.getElementById("openAuthUrlBtn");
    if (openBtn) openBtn.onclick = () => { if (loginCtx.authUrl) openAuthPopup(loginCtx.authUrl); };
    const sendBtn = document.getElementById("sendCodeBtn");
    const inputEl = document.getElementById("loginCodeInput");
    const sendCode = async () => {
      if (!loginCtx.sessionId || !inputEl || !inputEl.value.trim()) return;
      const text = inputEl.value;
      inputEl.value = "";
      try {
        await apiRequest("/api/login/input", { method: "POST", body: JSON.stringify({ session_id: loginCtx.sessionId, text }) });
        setBanner("", "Code envoyé…", "Validation en cours sur la machine cible.");
      } catch (e) { setBanner("err", "Envoi impossible", e.message); }
    };
    if (sendBtn) sendBtn.onclick = sendCode;
    if (inputEl) inputEl.onkeydown = (ev) => { if (ev.key === "Enter") sendCode(); };

    const poll = async () => {
      if (!loginCtx.sessionId) return;
      try {
        const res = await apiRequest(`/api/login/poll?session=${encodeURIComponent(loginCtx.sessionId)}&offset=${loginCtx.offset}`);
        loginCtx.offset = res.next_offset || loginCtx.offset;
        if (res.chunk) {
          const pre = document.getElementById("loginLog");
          if (pre) { pre.textContent += res.chunk; pre.scrollTop = pre.scrollHeight; }
        }
        if (res.auth_url && res.auth_url !== loginCtx.authUrl) {
          loginCtx.authUrl = res.auth_url;
          if (loginCtx.popup && !loginCtx.popup.closed) {
            try { loginCtx.popup.location.href = loginCtx.authUrl; loginCtx.popup.focus(); } catch {}
          }
          const urlRow = document.getElementById("loginUrlRow");
          if (urlRow) urlRow.classList.remove("hidden");
          const needsCode = res.engine !== "codex";
          const codeBox = document.getElementById("loginCodeBox");
          if (codeBox) codeBox.classList.toggle("hidden", !needsCode);
          setBanner(
            "",
            needsCode ? "Validez dans la fenêtre ouverte puis cliquez « Copier »" : "Validez dans la fenêtre ouverte",
            needsCode ? "Dès que vous copiez le code, Bridget l'envoie automatiquement (ou collez-le ci-dessous)." : "La redirection locale est automatique pour Codex."
          );
        }
        if (res.running && res.code_auto_sent) {
          closeLoginPopup();
          setBanner("", "Code détecté et envoyé automatiquement", "Finalisation de la session sur le serveur…");
          const codeBox = document.getElementById("loginCodeBox");
          if (codeBox) codeBox.classList.add("hidden");
        }
        if (!res.running) {
          loginCtx.running = false;
          stopLoginPolling();
          closeLoginPopup();
          const urlRow = document.getElementById("loginUrlRow");
          const codeBox = document.getElementById("loginCodeBox");
          if (urlRow) urlRow.classList.add("hidden");
          if (codeBox) codeBox.classList.add("hidden");
          setFootDone();
          if (res.exit_code === 0) {
            const who = res.logged_in_as ? `Compte connecté : ${res.logged_in_as}.` : "Session enregistrée et prête à l'emploi.";
            const extra = res.auto_enabled ? " Le nouveau compte dédié a été activé." : "";
            setBanner("ok", "✓ Connexion réussie", who + extra);
          } else {
            setBanner("err", `✕ Connexion interrompue (code ${res.exit_code})`, "Ouvrez le journal technique ci-dessous pour voir le détail.");
            const det = document.getElementById("loginLogDetails");
            if (det) det.open = true;
          }
          loadStatus();
        }
      } catch {}
    };
    loginCtx.pollTimer = setInterval(poll, 800);
    poll();
  } catch (e) {
    loginCtx.running = false;
    closeLoginPopup();
    setBanner("err", "Impossible de démarrer la connexion", e.message);
    setFootDone();
  }
}

function handleOpenPrepare() {
  if (!state.data) return;
  const candidates = (state.data.connections || []).filter((c) => !c.managed && c.auth_state !== "machine_unreachable");
  if (!candidates.length) { showFeedback("warn", "▲", "Aucune machine joignable pour préparer un compte."); return; }
  const opts = candidates.map((c) =>
    `<option value="${esc(c.machine_id)}|${esc(c.connection_id)}">${esc(c.machine_id)} · ${esc(c.account_label || c.engine)} (${esc(c.connection_id)})</option>`
  ).join("");
  const body = `<p>Crée un compte isolé (profil dédié) puis lance immédiatement la connexion fournisseur.</p>
    <div class="field"><label for="prepSource">Machine et fournisseur</label><select id="prepSource">${opts}</select></div>
    <div class="field"><label for="prepName">Nom du compte (minuscules, chiffres, - ou _)</label><input id="prepName" type="text" placeholder="ex. perso, pro, backup" autocomplete="off" /></div>
    <div id="prepMsg" class="row-sub"></div>`;
  openModal("Ajouter un compte dédié", body, [
    { label: "Annuler", onClick: closeModal },
    {
      label: "Préparer et connecter",
      primary: true,
      onClick: async () => {
        const val = document.getElementById("prepSource").value;
        const [machineId, fromConn] = val.split("|");
        const newId = document.getElementById("prepName").value.trim().toLowerCase();
        const msgEl = document.getElementById("prepMsg");
        if (!newId) { msgEl.textContent = "Indiquez un nom court (ex. perso)."; return; }
        msgEl.textContent = `Préparation sur ${machineId}…`;
        try {
          const res = await apiRequest("/api/prepare", {
            method: "POST",
            body: JSON.stringify({ machine_id: machineId, from_connection_id: fromConn, new_id: newId }),
          });
          await loadStatus();
          handleLogin({ connection_id: res.connection_id, machine_id: res.machine_id, engine: res.engine, account_label: `${res.engine} (${newId})` }, false, true);
        } catch (e) { msgEl.textContent = e.message; }
      },
    },
  ]);
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
    onLogin: handleLogin,
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
document.getElementById("prepareBtn").onclick = handleOpenPrepare;
document.addEventListener("keydown", (e) => { if (e.key === "Escape") closeModal(); });

applyTheme(state.themePref);
loadStatus();
