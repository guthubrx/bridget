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

  function sandboxFrameUrl(value, frameInstanceId) {
    const source = asText(value).trim();
    if (!source || !frameInstanceId) return "";
    // Le document parent est parfois présenté à WKWebView sous tauri://. Ce
    // schéma ne peut pas servir de base à `new URL` pour notre chemin HTTP
    // relatif, alors que `iframe.src` le résout correctement vers le relais.
    if (!/^\/v1\/artifacts\/sandbox\/frame\?/.test(source)) return "";
    return `${source}&frame_instance_id=${encodeURIComponent(frameInstanceId)}`;
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

  function render(documentRef, artifact, options = {}) {
    const payload = artifact && artifact.publication && artifact.publication.payload || {};
    const frameInstanceId = (root.crypto && root.crypto.randomUUID ? root.crypto.randomUUID() : `frame-${Date.now()}-${Math.random()}`);
    const section = documentRef.createElement("section"); section.className = "artifact-sandbox";
    const frameUrl = sandboxFrameUrl(options.frameUrl, frameInstanceId);
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
    const expand = documentRef.createElement("button"); expand.type = "button"; expand.textContent = "Agrandir";
    expand.addEventListener("click", () => { section.classList.toggle("artifact-sandbox--expanded"); expand.textContent = section.classList.contains("artifact-sandbox--expanded") ? "Réduire" : "Agrandir"; options.onExpand && options.onExpand(artifact); });
    const save = documentRef.createElement("button"); save.type = "button"; save.textContent = "Enregistrer comme nouvelle version";
    const cancel = documentRef.createElement("button"); cancel.type = "button"; cancel.textContent = "Annuler";
    let latestState = {};
    save.addEventListener("click", async () => {
      save.disabled = true; save.textContent = "Enregistrement…";
      try { const result = await (options.onSave && options.onSave(latestState)); save.textContent = result && result.version_ref ? "Nouvelle version enregistrée" : "Enregistrement indisponible"; }
      catch (cause) { save.textContent = (cause && cause.message) || "Enregistrement indisponible"; save.disabled = false; }
    });
    cancel.addEventListener("click", () => { latestState = {}; options.onCancel && options.onCancel(); cancel.textContent = "État temporaire annulé"; });
    actions.append(expand, save, cancel); section.append(iframe, error, actions);
    const onMessage = (event) => {
      if (event.source !== iframe.contentWindow || !validMessage(event.data, frameInstanceId)) { return; }
      const message = event.data;
      if (message.type === "sandbox.resize") iframe.style.height = `${message.height}px`;
      if (message.type === "sandbox.state") { latestState = message.ui_state; options.onState && options.onState(message.ui_state); }
      if (message.type === "sandbox.open_source") options.onOpenSource && options.onOpenSource(message.source_ref_id);
      if (message.type === "sandbox.error") { error.hidden = false; error.textContent = `Rendu dégradé (${message.code}) : ${message.message || "erreur déclarée"}.`; }
    };
    (options.window || root).addEventListener("message", onMessage);
    iframe.addEventListener("load", () => iframe.contentWindow.postMessage({ type: "sandbox.bootstrap", frame_instance_id: frameInstanceId, data: payload.data || null, limits: { max_height: MAX_HEIGHT, max_state_bytes: MAX_STATE_BYTES } }, "*"));
    return section;
  }
  return { MAX_HEIGHT, MAX_STATE_BYTES, sandboxFrameUrl, validMessage, render };
});
