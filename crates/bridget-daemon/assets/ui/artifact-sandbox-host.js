/* Runtime hôte des artefacts HTML Bridget.
 * Le document affiché est opaque et sans accès réseau. Cette surface ne reçoit
 * ni le jeton du relais ni un bridge Tauri : seul le parent local valide le
 * protocole minuscule ci-dessous. */
(function bootstrapArtifactSandbox(root, factory) {
  const api = factory(root);
  if (typeof module === "object" && module.exports) module.exports = api;
  if (root) root.BridgetArtifactSandbox = api;
})(typeof globalThis !== "undefined" ? globalThis : null, function artifactSandboxFactory(root) {
  "use strict";
  const MAX_HEIGHT = 1200;
  const MAX_STATE_BYTES = 128 * 1024;
  const MAX_ERROR_BYTES = 2048;

  function asText(value, fallback = "") { return typeof value === "string" ? value : fallback; }
  function utf8Length(value) { return new TextEncoder().encode(JSON.stringify(value)).length; }
  function normalizedTheme(value) { return value === "light" ? "light" : "dark"; }

  function sandboxFrameUrl(value, frameInstanceId, theme) {
    const source = asText(value).trim();
    if (!source || !frameInstanceId) return "";
    // Le document parent est parfois présenté à WKWebView sous tauri://. Ce
    // schéma ne peut pas servir de base à `new URL` pour notre chemin HTTP
    // relatif, alors que `iframe.src` le résout correctement vers le relais.
    if (!/^\/v1\/artifacts\/sandbox\/frame\?/.test(source)) return "";
    return `${source}&frame_instance_id=${encodeURIComponent(frameInstanceId)}&theme=${normalizedTheme(theme)}`;
  }

  function validMessage(value, instanceId) {
    if (!value || typeof value !== "object" || value.frame_instance_id !== instanceId) return false;
    if (value.type === "sandbox.ready") return true;
    if (value.type === "sandbox.resize") return Number.isInteger(value.height) && value.height >= 0 && value.height <= MAX_HEIGHT;
    if (value.type === "sandbox.state") return utf8Length(value.ui_state) <= MAX_STATE_BYTES && !hasUrl(value.ui_state);
    if (value.type === "sandbox.open_source") return typeof value.source_ref_id === "string" && /^[A-Za-z0-9._:-]{1,160}$/.test(value.source_ref_id);
    if (value.type === "sandbox.error") return typeof value.code === "string" && /^[a-z_]{1,80}$/.test(value.code) && asText(value.message).length <= MAX_ERROR_BYTES;
    return false;
  }
  function hasUrl(value) {
    if (typeof value === "string") return /^(?:https?|file):/i.test(value);
    if (Array.isArray(value)) return value.some(hasUrl);
    if (value && typeof value === "object") return Object.entries(value).some(([key, nested]) => /url/i.test(key) || hasUrl(nested));
    return false;
  }

  const ICON_PATHS = {
    expand: ["M8 3H3v5", "m3 3 6 6", "M16 3h5v5", "m21 3-6 6", "M3 16v5h5", "m3 21 6-6", "M21 16v5h-5", "m21 21-6-6"],
    reduce: ["m9 3-6 6", "M3 3h5v5", "m15 3 6 6", "M21 3h-5v5", "m9 21-6-6", "M3 21h5v-5", "m15 21 6-6", "M21 21h-5v-5"],
    save: ["M5 3h12l3 3v15H5z", "M8 3v6h8V3", "M8 21v-7h8v7"],
    cancel: ["M3 7v5h5", "M3 12a8 8 0 1 0 2.3-5.7L3 7"],
  };

  function setButtonIcon(button, icon, label) {
    const svg = button.ownerDocument.createElementNS("http://www.w3.org/2000/svg", "svg");
    svg.setAttribute("viewBox", "0 0 24 24");
    svg.setAttribute("fill", "none");
    svg.setAttribute("stroke", "currentColor");
    svg.setAttribute("stroke-width", "1.9");
    svg.setAttribute("stroke-linecap", "round");
    svg.setAttribute("stroke-linejoin", "round");
    for (const pathData of ICON_PATHS[icon] || []) {
      const path = button.ownerDocument.createElementNS("http://www.w3.org/2000/svg", "path");
      path.setAttribute("d", pathData);
      svg.append(path);
    }
    button.replaceChildren(svg);
    button.setAttribute("aria-label", label);
    button.title = label;
  }

  function actionButton(documentRef, icon, label) {
    const button = documentRef.createElement("button");
    button.type = "button";
    button.className = "artifact-sandbox__action";
    setButtonIcon(button, icon, label);
    return button;
  }

  function render(documentRef, artifact, options = {}) {
    const payload = artifact && artifact.publication && artifact.publication.payload || {};
    const frameInstanceId = (root.crypto && root.crypto.randomUUID ? root.crypto.randomUUID() : `frame-${Date.now()}-${Math.random()}`);
    const section = documentRef.createElement("section"); section.className = "artifact-sandbox";
    let theme = normalizedTheme(options.theme);
    const frameUrl = sandboxFrameUrl(options.frameUrl, frameInstanceId, theme);
    if (!frameUrl) { section.textContent = "La page sandboxée est indisponible. Rechargez l’artefact via Bridget."; section.classList.add("artifact-sandbox--error"); return section; }
    const iframe = documentRef.createElement("iframe");
    iframe.className = "artifact-sandbox__frame";
    iframe.setAttribute("sandbox", "allow-scripts");
    iframe.setAttribute("referrerpolicy", "no-referrer");
    iframe.title = asText(artifact.publication && artifact.publication.title, "Artefact HTML Bridget");
    // La page est servie par le relais via un ticket court. Cela évite le
    // chemin data:/srcdoc qui ne peint pas toujours dans WKWebView/Tauri.
    iframe.src = frameUrl;
    iframe.style.height = `${Math.min(MAX_HEIGHT, Number(payload.inline_height_hint) || 360)}px`;
    const error = documentRef.createElement("p"); error.className = "artifact-sandbox__error"; error.hidden = true;
    const actions = documentRef.createElement("div"); actions.className = "artifact-sandbox__actions";
    const expand = actionButton(documentRef, "expand", "Agrandir");
    let returnAnchor = null;
    expand.addEventListener("click", () => {
      const isExpanded = section.classList.toggle("artifact-sandbox--expanded");
      if (isExpanded) {
        returnAnchor = documentRef.createComment("bridget-artifact-position");
        section.parentNode.insertBefore(returnAnchor, section);
        documentRef.body.append(section);
      } else if (returnAnchor && returnAnchor.parentNode) {
        returnAnchor.parentNode.insertBefore(section, returnAnchor);
        returnAnchor.remove();
        returnAnchor = null;
      }
      setButtonIcon(expand, isExpanded ? "reduce" : "expand", isExpanded ? "Réduire" : "Agrandir");
      options.onExpand && options.onExpand(artifact);
    });
    const save = actionButton(documentRef, "save", "Enregistrer comme nouvelle version");
    const cancel = actionButton(documentRef, "cancel", "Rétablir le dernier état enregistré");
    let savedState = payload && payload.ui_state && typeof payload.ui_state === "object" ? payload.ui_state : null;
    let latestState = savedState || {};
    save.addEventListener("click", async () => {
      save.disabled = true; save.title = "Enregistrement…";
      try {
        const result = await (options.onSave && options.onSave(latestState));
        if (!result || !result.version_ref) throw new Error("Enregistrement indisponible");
        savedState = latestState;
        setButtonIcon(save, "save", "Nouvelle version enregistrée");
        save.disabled = false;
      } catch (cause) {
        save.title = (cause && cause.message) || "Enregistrement indisponible";
        save.disabled = false;
      }
    });
    cancel.addEventListener("click", () => {
      const restore = savedState || null;
      iframe.contentWindow.postMessage({ type: "sandbox.restore", frame_instance_id: frameInstanceId, ui_state: restore }, "*");
      latestState = restore || {};
      cancel.title = restore ? "Dernier état enregistré rétabli" : "Valeurs initiales rétablies";
      options.onCancel && options.onCancel();
    });
    actions.append(expand, save, cancel); section.append(actions, iframe, error);
    const onMessage = (event) => {
      if (event.source !== iframe.contentWindow || !validMessage(event.data, frameInstanceId)) { return; }
      const message = event.data;
      if (message.type === "sandbox.resize") iframe.style.height = `${message.height}px`;
      if (message.type === "sandbox.state") { latestState = message.ui_state; options.onState && options.onState(message.ui_state); }
      if (message.type === "sandbox.open_source") options.onOpenSource && options.onOpenSource(message.source_ref_id);
      if (message.type === "sandbox.error") { error.hidden = false; error.textContent = `Rendu dégradé (${message.code}) : ${message.message || "erreur déclarée"}.`; }
    };
    const parentWindow = options.window || root;
    const sendTheme = () => iframe.contentWindow.postMessage({ type: "sandbox.theme", frame_instance_id: frameInstanceId, theme }, "*");
    const onThemeChange = (event) => {
      theme = normalizedTheme(event && event.detail && event.detail.theme);
      sendTheme();
    };
    parentWindow.addEventListener("message", onMessage);
    parentWindow.addEventListener("bridget-theme-changed", onThemeChange);
    iframe.addEventListener("load", () => {
      iframe.contentWindow.postMessage({ type: "sandbox.bootstrap", frame_instance_id: frameInstanceId, data: payload.data || null, ui_state: savedState, limits: { max_height: MAX_HEIGHT, max_state_bytes: MAX_STATE_BYTES } }, "*");
      sendTheme();
    });
    return section;
  }
  return { MAX_HEIGHT, MAX_STATE_BYTES, normalizedTheme, sandboxFrameUrl, validMessage, render };
});
