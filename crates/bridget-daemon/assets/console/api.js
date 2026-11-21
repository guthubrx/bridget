const TOKEN_KEY = "bridget_console_token";

export function getToken() {
  try {
    return sessionStorage.getItem(TOKEN_KEY) || "";
  } catch {
    return "";
  }
}

export async function apiRequest(path, options = {}) {
  const token = getToken();
  if (!token) {
    const err = new Error("Session locale absente ou expirée : rouvrez la console depuis bridget accounts ui.");
    err.status = 401;
    throw err;
  }
  const headers = {
    "x-bridget-token": token,
    ...(options.body ? { "content-type": "application/json" } : {}),
    ...(options.headers || {}),
  };
  const resp = await fetch(path, {
    ...options,
    credentials: "same-origin",
    headers,
  });
  let payload = null;
  try {
    payload = await resp.json();
  } catch {
    payload = null;
  }
  if (!resp.ok) {
    const msg =
      (payload && (payload.message || payload.error || payload.next_action)) ||
      (resp.status === 401
        ? "Session expirée ou non autorisée : relancez bridget accounts ui."
        : `Requête refusée (${resp.status}).`);
    const err = new Error(msg);
    err.status = resp.status;
    err.payload = payload;
    throw err;
  }
  return payload;
}
