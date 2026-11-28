// Logos vectoriels des moteurs pour la console Bridget (Quotas, Comptes, Machines & CLI).

let gradSeq = 0;

export function vendorLogoSvg(engine, size = 16) {
  const key = String(engine || "").toLowerCase();
  const s = Number(size) || 16;
  if (key === "antigravity" || key === "agy") {
    gradSeq += 1;
    const gid = `agyG${gradSeq}`;
    return `<svg class="v-logo" width="${s}" height="${s}" viewBox="0 0 24 24" fill="none" aria-hidden="true">`
      + `<defs><linearGradient id="${gid}" x1="3" y1="20" x2="21" y2="4" gradientUnits="userSpaceOnUse">`
      + `<stop offset="0%" stop-color="#3B82F6"/>`
      + `<stop offset="38%" stop-color="#22C55E"/>`
      + `<stop offset="68%" stop-color="#F59E0B"/>`
      + `<stop offset="100%" stop-color="#EF4444"/>`
      + `</linearGradient></defs>`
      + `<path d="M4.2 19.6C6.4 17.5 8.1 12.1 10.1 6.7C11.1 4.1 12.9 4.1 13.9 6.7C15.9 12.1 17.6 17.5 19.8 19.6C20.5 20.3 19.7 21.1 18.7 20.7C16.3 19.7 14.7 15.8 12 15.8C9.3 15.8 7.7 19.7 5.3 20.7C4.3 21.1 3.5 20.3 4.2 19.6Z" fill="url(#${gid})"/>`
      + `</svg>`;
  }
  if (key === "codex" || key === "openai") {
    return `<svg class="v-logo" width="${s}" height="${s}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.75" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">`
      + `<circle cx="12" cy="12" r="2.6"/>`
      + `<path d="M12 3.2c2.2 0 4 1.5 4.4 3.6 1.9.4 3.4 2.1 3.4 4.2 0 1.4-.7 2.7-1.7 3.5.3 2.1-.7 4.2-2.6 5.2-1.8 1-4 .8-5.5-.4-2 .4-4.1-.5-5.1-2.4-1-1.8-.8-4 .4-5.5-.4-2 .5-4.1 2.4-5.1C8.8 4 10.4 3.2 12 3.2Z"/>`
      + `<path d="M12 9.4V4.5M14.3 10.7l4.2-2.4M14.3 13.3l4.2 2.4M12 14.6v4.9M9.7 13.3l-4.2 2.4M9.7 10.7L5.5 8.3"/>`
      + `</svg>`;
  }
  if (key === "claude" || key === "anthropic") {
    return `<svg class="v-logo" width="${s}" height="${s}" viewBox="0 0 24 24" fill="none" aria-hidden="true">`
      + `<g stroke="#C96442" stroke-width="2.35" stroke-linecap="round">`
      + `<line x1="12" y1="3" x2="12" y2="21"/>`
      + `<line x1="3" y1="12" x2="21" y2="12"/>`
      + `<line x1="5.6" y1="5.6" x2="18.4" y2="18.4"/>`
      + `<line x1="18.4" y1="5.6" x2="5.6" y2="18.4"/>`
      + `<line x1="8.4" y1="3.7" x2="15.6" y2="20.3"/>`
      + `<line x1="20.3" y1="8.4" x2="3.7" y2="15.6"/>`
      + `</g><circle cx="12" cy="12" r="3.1" fill="#C96442"/>`
      + `</svg>`;
  }
  if (key === "cursor") {
    return `<svg class="v-logo" width="${s}" height="${s}" viewBox="0 0 24 24" fill="none" aria-hidden="true">`
      + `<polygon points="12,2.5 20.5,7.3 20.5,16.7 12,21.5 3.5,16.7 3.5,7.3" stroke="currentColor" stroke-width="1.6" fill="currentColor" fill-opacity="0.12"/>`
      + `<polygon points="12,2.5 20.5,7.3 12,12 3.5,7.3" fill="currentColor" fill-opacity="0.35"/>`
      + `<polyline points="12,12 12,21.5" stroke="currentColor" stroke-width="1.6"/>`
      + `</svg>`;
  }
  if (key === "gemini") {
    gradSeq += 1;
    const gid = `gemG${gradSeq}`;
    return `<svg class="v-logo" width="${s}" height="${s}" viewBox="0 0 24 24" fill="none" aria-hidden="true">`
      + `<defs><linearGradient id="${gid}" x1="3" y1="3" x2="21" y2="21" gradientUnits="userSpaceOnUse">`
      + `<stop offset="0%" stop-color="#4F8DF7"/><stop offset="100%" stop-color="#A855F7"/>`
      + `</linearGradient></defs>`
      + `<path d="M12 2.5C12.8 8.1 15.9 11.2 21.5 12C15.9 12.8 12.8 15.9 12 21.5C11.2 15.9 8.1 12.8 2.5 12C8.1 11.2 11.2 8.1 12 2.5Z" fill="url(#${gid})"/>`
      + `</svg>`;
  }
  if (key === "glm") {
    return `<svg class="v-logo" width="${s}" height="${s}" viewBox="0 0 24 24" fill="none" aria-hidden="true">`
      + `<rect x="2.5" y="2.5" width="19" height="19" rx="5" fill="#0EA5E9" fill-opacity="0.18" stroke="#0EA5E9" stroke-width="1.6"/>`
      + `<text x="12" y="15.3" text-anchor="middle" font-size="8.5" font-weight="700" font-family="ui-sans-serif,system-ui,sans-serif" fill="#0EA5E9">Z</text>`
      + `</svg>`;
  }
  return "";
}
