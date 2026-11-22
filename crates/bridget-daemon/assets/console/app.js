import { apiRequest } from "/assets/api.js";
import { renderAccountsView } from "/assets/views/accounts.js";
import { renderMachinesView } from "/assets/views/machines.js";
import { renderSettingsView } from "/assets/views/settings.js";

const mediaDark = window.matchMedia ? window.matchMedia("(prefers-color-scheme: dark)") : null;
let state = {
  data: null,
  machinesData: null,
  settingsData: null,
  activeView: "all",
  themePref: document.documentElement.getAttribute("data-theme-pref") || "system",
};
let loginCtx = { sessionId: null, pollTimer: null, offset: 0, authUrl: null, popup: null, running: false };
let deployCtx = { jobId: null, pollTimer: null, offset: 0, running: false };

function stopDeployPolling() {
  if (deployCtx.pollTimer) { clearInterval(deployCtx.pollTimer); deployCtx.pollTimer = null; }
}

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
  stopDeployPolling();
  if (deployCtx.running && deployCtx.jobId) {
    const jid = deployCtx.jobId;
    deployCtx.running = false;
    apiRequest("/api/clients/deploy/cancel", { method: "POST", body: JSON.stringify({ job_id: jid }) }).catch(() => {});
  }
  deployCtx.jobId = null;
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

function handleOpenSshModal(existing) {
  const alias = existing ? existing.machine_id : "";
  const host = existing && existing.ssh_hostname ? existing.ssh_hostname : "";
  const user = existing && existing.ssh_user ? existing.ssh_user : "moi";
  const port = existing && existing.ssh_port ? existing.ssh_port : 22;
  const body = `<p>Ajoute ou met à jour l'entrée dans <code>~/.ssh/config</code> (droits 0600) puis teste immédiatement la connexion SSH.</p>
    <div class="field"><label for="sshAlias">Nom de la machine (alias Bridget)</label><input id="sshAlias" type="text" value="${esc(alias)}" placeholder="ex. macbook-m1, vps-gpu" autocomplete="off" /></div>
    <div class="field-inline">
      <div class="field"><label for="sshHost">Adresse (IP, nom .home ou Tailscale)</label><input id="sshHost" type="text" value="${esc(host)}" placeholder="ex. 192.168.1.51 ou macbook.home" autocomplete="off" /></div>
      <div class="field"><label for="sshPort">Port</label><input id="sshPort" type="number" value="${esc(port)}" min="1" max="65535" /></div>
    </div>
    <div class="field"><label for="sshUser">Utilisateur SSH</label><input id="sshUser" type="text" value="${esc(user)}" autocomplete="off" /></div>
    <label class="check-line"><input type="checkbox" id="sshIpv4" checked /> Forcer IPv4 (évite les adresses IPv6 obsolètes sur réseau local)</label>
    <div id="sshMsg" class="row-sub"></div>`;
  openModal(existing ? `Configurer SSH · ${alias}` : "Ajouter une machine", body, [
    { label: "Annuler", onClick: closeModal },
    {
      label: "Enregistrer et tester",
      primary: true,
      onClick: async () => {
        const msg = document.getElementById("sshMsg");
        msg.textContent = "Enregistrement dans ~/.ssh/config et test SSH…";
        try {
          const res = await apiRequest("/api/machines/ssh-add", {
            method: "POST",
            body: JSON.stringify({
              alias: document.getElementById("sshAlias").value.trim(),
              hostname: document.getElementById("sshHost").value.trim(),
              user: document.getElementById("sshUser").value.trim(),
              port: Number(document.getElementById("sshPort").value) || 22,
              ipv4_only: document.getElementById("sshIpv4").checked,
              accept_host_key: true,
            }),
          });
          if (res.status === "connected") {
            closeModal();
            showFeedback("ok", "✓", res.ui_state);
            loadMachines();
            loadStatus();
          } else {
            msg.innerHTML = `${esc(res.detail || res.ui_state)}${res.copy_key_hint ? `<div class="cmd-box" style="margin-top:8px"><code>${esc(res.copy_key_hint)}</code></div>` : ""}`;
            loadMachines();
          }
        } catch (e) { msg.textContent = e.message; }
      },
    },
  ]);
}

function handleRepairMachine(machine, repair) {
  if (repair.action === "macos_keychain_unlock_guide") {
    const cmd = `ssh -t ${machine.machine_id} 'security unlock-keychain ~/Library/Keychains/login.keychain-db && security set-keychain-settings ~/Library/Keychains/login.keychain-db'`;
    const body = `<p>Sur <strong>${esc(machine.machine_id)}</strong> (macOS), le trousseau est actuellement verrouillé. Par sécurité, Apple demande votre mot de passe de session une fois pour le déverrouiller — aucun mot de passe ne transite par cette page.</p>
      <p>Exécutez cette commande unique dans votre terminal : elle <strong>déverrouille le trousseau</strong> et <strong>désactive son verrouillage automatique</strong> jusqu'au prochain redémarrage du Mac.</p>
      <div class="cmd-box"><code>${esc(cmd)}</code></div>
      <div id="copyKcMsg" class="row-sub"></div>`;
    openModal(`Déverrouiller macOS · ${machine.machine_id}`, body, [
      { label: "Fermer", onClick: closeModal },
      {
        label: "Copier la commande",
        primary: true,
        onClick: async () => {
          try {
            await navigator.clipboard.writeText(cmd);
            const el = document.getElementById("copyKcMsg");
            if (el) el.textContent = "✓ Commande copiée. Collez-la dans un terminal puis cliquez sur Actualiser.";
          } catch {
            const el = document.getElementById("copyKcMsg");
            if (el) el.textContent = "Sélectionnez la commande ci-dessus et faites ⌘C.";
          }
        },
      },
    ]);
    return;
  }
  const body = `<p>Vérifiez l'action de maintenance avant de l'exécuter sur <strong>${esc(machine.machine_id)}</strong>.</p>
    <dl class="kv">
      <dt>Machine</dt><dd>${esc(machine.machine_id)} (${esc(machine.os)} ${esc(machine.arch)})</dd>
      <dt>Action</dt><dd>${esc(repair.label)}</dd>
      <dt>Effet</dt><dd>${esc(repair.detail)}</dd>
    </dl>
    <div id="repMsg" class="row-sub"></div>`;
  openModal(`Réparer · ${machine.machine_id}`, body, [
    { label: "Annuler", onClick: closeModal },
    {
      label: "Exécuter",
      primary: true,
      onClick: async () => {
        const msg = document.getElementById("repMsg");
        msg.textContent = "Opération en cours…";
        try {
          const res = await apiRequest("/api/machines/repair", {
            method: "POST",
            body: JSON.stringify({ machine_id: machine.machine_id, action: repair.action }),
          });
          closeModal();
          showFeedback("ok", "✓", res.ui_state);
          loadMachines();
          loadStatus();
        } catch (e) { msg.textContent = e.message; }
      },
    },
  ]);
}

function handleDeployClient(machine, client, mode) {
  stopDeployPolling();
  deployCtx = { jobId: null, pollTimer: null, offset: 0, running: false };
  const isUpdate = mode === "update";
  const verb = isUpdate ? "Mettre à jour" : "Installer";
  const cmdSummary = isUpdate ? client.update_summary : client.install_summary;
  const body = `<p>Déploiement officiel sur <strong>${esc(machine.machine_id)}</strong>. Aucun compte ni service Bridget existant ne sera modifié.</p>
    <dl class="kv">
      <dt>Client CLI</dt><dd>${esc(client.label)} (<code>${esc(client.binary)}</code>)</dd>
      <dt>Machine</dt><dd>${esc(machine.machine_id)}</dd>
      <dt>Méthode officielle</dt><dd>${esc(cmdSummary)}</dd>
      <dt>Connexion ensuite</dt><dd><code>${esc(client.login_hint)}</code> (ou bouton Connecter)</dd>
    </dl>
    <div id="depProgress" class="deploy-progress hidden">
      <div class="deploy-head">
        <span id="depStepText"><span class="spinner" aria-hidden="true"></span> Démarrage…</span>
        <span id="depTimer" class="deploy-timer mono">00:00</span>
      </div>
      <div class="step-pills">
        <div id="depPill1" class="step-pill active">1. Préparation</div>
        <div id="depPill2" class="step-pill">2. Installation</div>
        <div id="depPill3" class="step-pill">3. Vérification</div>
      </div>
      <div class="progress-bar"><div id="depFill" class="progress-fill running" style="width:15%"></div></div>
      <details id="depLogDetails" class="tech-log"><summary>Voir les commandes et la sortie en direct</summary><pre id="depLog" class="live-terminal"></pre></details>
    </div>
    <div id="depResult" class="login-banner hidden"></div>`;

  openModal(`${verb} ${client.label}`, body, [
    { label: "Annuler", onClick: closeModal },
    {
      label: `${verb} maintenant`,
      primary: true,
      onClick: async (ev) => {
        if (deployCtx.running) return;
        const btn = ev.currentTarget;
        btn.disabled = true;
        btn.innerHTML = `<span class="spinner" aria-hidden="true"></span> ${isUpdate ? "Mise à jour…" : "Installation…"}`;
        document.getElementById("depProgress").classList.remove("hidden");
        const setPills = (idx, failed, doneAll) => {
          for (let n = 1; n <= 3; n++) {
            const p = document.getElementById(`depPill${n}`);
            if (!p) continue;
            p.className = "step-pill";
            if (doneAll) p.classList.add("done");
            else if (failed && n === idx) p.classList.add("err");
            else if (n < idx) p.classList.add("done");
            else if (n === idx) p.classList.add("active");
          }
        };
        const fmtSec = (s) => `${String(Math.floor(s / 60)).padStart(2, "0")}:${String(s % 60).padStart(2, "0")}`;
        try {
          const started = await apiRequest("/api/clients/deploy", {
            method: "POST",
            body: JSON.stringify({ machine_id: machine.machine_id, client_id: client.id, mode }),
          });
          deployCtx.jobId = started.job_id;
          deployCtx.running = true;
          const poll = async () => {
            if (!deployCtx.jobId) return;
            try {
              const res = await apiRequest(`/api/clients/deploy/poll?job=${encodeURIComponent(deployCtx.jobId)}&offset=${deployCtx.offset}`);
              deployCtx.offset = res.next_offset || deployCtx.offset;
              const pre = document.getElementById("depLog");
              if (res.chunk && pre) { pre.textContent += res.chunk; pre.scrollTop = pre.scrollHeight; }
              const timer = document.getElementById("depTimer");
              if (timer) timer.textContent = fmtSec(res.elapsed_secs || 0);
              const fill = document.getElementById("depFill");
              if (fill) fill.style.width = `${res.percent || 20}%`;
              const stepTxt = document.getElementById("depStepText");
              if (res.running) {
                if (stepTxt) stepTxt.innerHTML = `<span class="spinner" aria-hidden="true"></span> ${esc(res.step_label || "En cours…")}`;
                setPills(res.step_index || 1, false, false);
              } else {
                deployCtx.running = false;
                stopDeployPolling();
                const ok = res.exit_code === 0;
                if (stepTxt) stepTxt.textContent = ok ? "✓ Terminé" : `✕ ${res.step_label || "Échec"}`;
                if (fill) { fill.classList.remove("running"); fill.classList.add(ok ? "ok" : "err"); fill.style.width = "100%"; }
                setPills(res.step_index || 2, !ok, ok);
                const resBox = document.getElementById("depResult");
                if (resBox) {
                  resBox.className = `login-banner ${ok ? "ok" : "err"}`;
                  resBox.innerHTML = ok
                    ? `<strong>✓ ${esc(res.ui_state)}</strong>Vous pouvez fermer cette fenêtre ou connecter le compte.`
                    : `<strong>✕ ${esc(res.ui_state)}</strong>Consultez le détail déplié ci-dessus pour voir la cause exacte.`;
                  resBox.classList.remove("hidden");
                }
                if (!ok) {
                  const det = document.getElementById("depLogDetails");
                  if (det) det.open = true;
                }
                const foot = document.getElementById("modalFoot");
                if (foot) {
                  foot.innerHTML = "";
                  const closeB = document.createElement("button");
                  closeB.type = "button";
                  closeB.className = "btn primary";
                  closeB.textContent = "Fermer";
                  closeB.onclick = closeModal;
                  foot.appendChild(closeB);
                }
                if (ok) showFeedback("ok", "✓", res.ui_state);
                loadMachines();
                loadStatus();
              }
            } catch {}
          };
          deployCtx.pollTimer = setInterval(poll, 450);
          poll();
        } catch (e) {
          btn.disabled = false;
          btn.textContent = `${verb} maintenant`;
          const resBox = document.getElementById("depResult");
          if (resBox) {
            resBox.className = "login-banner err";
            resBox.innerHTML = `<strong>✕ Impossible de démarrer</strong>${esc(e.message)}`;
            resBox.classList.remove("hidden");
          }
        }
      },
    },
  ]);
}

async function handleSaveAutomanaged(payload) {
  try {
    const res = await apiRequest("/api/settings/automanaged", {
      method: "POST",
      body: JSON.stringify(payload),
    });
    const okRemotes = (res.propagated || []).filter((r) => r.propagated).length;
    const totRemotes = (res.propagated || []).length;
    showFeedback("ok", "✓", `Paramètres enregistrés (${okRemotes}/${totRemotes} machines distantes synchronisées).`);
    loadSettings();
  } catch (e) { showFeedback("err", "✕", e.message); }
}

async function handleSyncFleet() {
  showFeedback("warn", "⏳", "Synchronisation des CLI et des modèles sur la flotte en cours…");
  try {
    const res = await apiRequest("/api/settings/sync", {
      method: "POST",
      body: JSON.stringify({ cli: true, models: true }),
    });
    const okRemotes = (res.remotes || []).filter((r) => r.status === "ok").length;
    const totRemotes = (res.remotes || []).length;
    showFeedback("ok", "✓", `Synchronisation terminée (local + ${okRemotes}/${totRemotes} machines distantes).`);
    loadSettings();
  } catch (e) { showFeedback("err", "✕", e.message); }
}

async function handleRoutingPreview(mutation) {
  try {
    const preview = await apiRequest("/api/settings/routing/preview", {
      method: "POST",
      body: JSON.stringify(mutation),
    });
    const body = `<p>Vérifiez la modification de routage avant écriture (sauvegarde automatique préalable).</p>
      <dl class="kv">
        <dt>Modification</dt><dd>${esc(preview.summary)}</dd>
        <dt>Registre</dt><dd><code>${esc(preview.registry_path)}</code></dd>
        <dt>Champs</dt><dd><code>${esc((preview.fields_touched || []).join(", "))}</code></dd>
        <dt>SHA-256</dt><dd><code>${esc((preview.expected_sha256 || "").slice(0, 16))}…</code></dd>
      </dl>`;
    openModal("Confirmer la modification", body, [
      { label: "Annuler", onClick: () => { closeModal(); loadSettings(); } },
      {
        label: "Appliquer",
        primary: true,
        onClick: async () => {
          closeModal();
          try {
            const res = await apiRequest("/api/settings/routing/apply", {
              method: "POST",
              body: JSON.stringify({ preview_id: preview.preview_id }),
            });
            showFeedback("ok", "✓", res.ui_state || "Enregistré et relu");
            loadSettings();
            loadStatus();
          } catch (e) {
            const st = e.payload && e.payload.ui_state ? e.payload.ui_state : `Refusé : ${e.message}`;
            showFeedback(e.status === 409 ? "warn" : "err", e.status === 409 ? "▲" : "✕", st);
            loadSettings();
          }
        },
      },
    ]);
  } catch (e) {
    const st = e.payload && e.payload.ui_state ? e.payload.ui_state : `Refusé : ${e.message}`;
    showFeedback("err", "✕", st);
    loadSettings();
  }
}

function updateTopbarAction() {
  const btn = document.getElementById("primaryActionBtn");
  if (!btn) return;
  if (state.activeView === "machines") {
    btn.textContent = "+ Ajouter une machine";
    btn.classList.remove("hidden");
    btn.onclick = () => handleOpenSshModal(null);
  } else if (state.activeView === "settings") {
    btn.classList.add("hidden");
  } else {
    btn.textContent = "+ Nouveau compte";
    btn.classList.remove("hidden");
    btn.onclick = handleOpenPrepare;
  }
}

function selectView(view) {
  state.activeView = view;
  for (const b of document.querySelectorAll(".sidebar [data-view]")) {
    b.classList.toggle("active", b.getAttribute("data-view") === view);
  }
  updateTopbarAction();
  render();
  if (view === "machines" && !state.machinesData) loadMachines();
  if (view === "settings" && !state.settingsData) loadSettings();
}

function render() {
  updateTopbarAction();
  if (state.data) {
    document.getElementById("hostBadge").textContent = `Hôte console : ${state.data.local_machine}`;
  }
  if (state.activeView === "machines") {
    renderMachinesView({
      data: state.machinesData,
      onAddSsh: () => handleOpenSshModal(null),
      onEditSsh: (m) => handleOpenSshModal(m),
      onRepair: handleRepairMachine,
      onDeployClient: handleDeployClient,
    });
    return;
  }
  if (state.activeView === "settings") {
    renderSettingsView({
      data: state.settingsData,
      onSaveAutomanaged: handleSaveAutomanaged,
      onSyncFleet: handleSyncFleet,
      onRoutingPreview: handleRoutingPreview,
    });
    return;
  }
  if (!state.data) return;
  renderAccountsView({
    data: state.data,
    activeView: state.activeView,
    onSelectView: selectView,
    onLogin: handleLogin,
    onTogglePreview: handleTogglePreview,
    onShowCli: handleShowCli,
  });
}

async function loadMachines() {
  const btn = document.getElementById("refreshBtn");
  btn.disabled = true;
  try {
    state.machinesData = await apiRequest("/api/machines");
    if (state.activeView === "machines") render();
  } catch (e) {
    showFeedback("err", "✕", `Machines : ${e.message}`);
  } finally {
    btn.disabled = false;
  }
}

async function loadSettings() {
  const btn = document.getElementById("refreshBtn");
  btn.disabled = true;
  try {
    state.settingsData = await apiRequest("/api/settings");
    if (state.activeView === "settings") render();
  } catch (e) {
    showFeedback("err", "✕", `Paramètres : ${e.message}`);
  } finally {
    btn.disabled = false;
  }
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

function refreshCurrentView() {
  if (state.activeView === "machines") return loadMachines();
  if (state.activeView === "settings") return loadSettings();
  return loadStatus();
}

for (const btn of document.querySelectorAll(".sidebar [data-view]")) {
  btn.onclick = () => selectView(btn.getAttribute("data-view"));
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
document.getElementById("refreshBtn").onclick = refreshCurrentView;
document.getElementById("modalCloseBtn").onclick = closeModal;
document.addEventListener("keydown", (e) => { if (e.key === "Escape") closeModal(); });

applyTheme(state.themePref);
updateTopbarAction();
loadStatus();
