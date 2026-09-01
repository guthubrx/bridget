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
  const CSP = "default-src 'none'; connect-src 'none'; frame-src 'none'; child-src 'none'; form-action 'none'; base-uri 'none'; object-src 'none'; media-src 'none'; img-src data: blob:; style-src 'unsafe-inline'; script-src 'unsafe-inline'";

  function asText(value, fallback = "") { return typeof value === "string" ? value : fallback; }
  function utf8Length(value) { return new TextEncoder().encode(JSON.stringify(value)).length; }
  function escapeAttribute(value) { return String(value).replaceAll("&", "&amp;").replaceAll('"', "&quot;").replaceAll("<", "&lt;"); }
  function escapedBase64(value) { return btoa(unescape(encodeURIComponent(asText(value)))); }

  function srcdoc(html, bootstrap) {
    const init = escapedBase64(JSON.stringify(bootstrap || {}));
    const prelude = `<meta charset="utf-8"><meta http-equiv="Content-Security-Policy" content="${escapeAttribute(CSP)}"><script>const __b=JSON.parse(decodeURIComponent(escape(atob('${init}'))));parent.postMessage({type:'sandbox.ready',frame_instance_id:__b.frame_instance_id},'*');window.addEventListener('message',e=>{if(e.data&&e.data.type==='sandbox.bootstrap')window.dispatchEvent(new CustomEvent('bridget-sandbox-bootstrap',{detail:e.data}));if(e.data&&e.data.type==='sandbox.close')window.close()});</script>`;
    // WebKit laisse l'iframe blanche lorsque le document complet est injecté
    // par document.write pendant le parsing de srcdoc. Le document reste donc
    // dans son propre flux de parsing, avec le CSP et le protocole ajoutés au
    // début de son <head>.
    const documentHtml = asText(html).replace(/^\s*<!doctype[^>]*>/i, "");
    if (/<head\b[^>]*>/i.test(documentHtml)) {
      return documentHtml.replace(/<head\b[^>]*>/i, (head) => `${head}${prelude}`);
    }
    return `<!doctype html><html><head>${prelude}</head><body>${documentHtml}</body></html>`;
  }

  function sandboxDocumentUrl(html, bootstrap) {
    return `data:text/html;charset=utf-8;base64,${escapedBase64(srcdoc(html, bootstrap))}`;
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
    const html = asText(payload.html);
    const frameInstanceId = (root.crypto && root.crypto.randomUUID ? root.crypto.randomUUID() : `frame-${Date.now()}-${Math.random()}`);
    const section = documentRef.createElement("section"); section.className = "artifact-sandbox";
    if (!html) { section.textContent = "Le contenu HTML canonique est indisponible. Consultez le manifeste ou demandez une restauration par Bridget."; section.classList.add("artifact-sandbox--error"); return section; }
    const iframe = documentRef.createElement("iframe");
    iframe.className = "artifact-sandbox__frame";
    iframe.setAttribute("sandbox", "allow-scripts");
    iframe.setAttribute("referrerpolicy", "no-referrer");
    iframe.title = asText(artifact.publication && artifact.publication.title, "Artefact HTML Bridget");
    // WKWebView monte parfois une iframe srcdoc vide dans une WebView Tauri,
    // malgré une source valide. Une navigation data: garde le document dans
    // une origine opaque, sans réseau, et évite ce chemin WebKit.
    iframe.src = sandboxDocumentUrl(html, { frame_instance_id: frameInstanceId });
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
  return { CSP, MAX_HEIGHT, MAX_STATE_BYTES, srcdoc, sandboxDocumentUrl, validMessage, render };
});
