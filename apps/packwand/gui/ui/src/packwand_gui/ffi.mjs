let eventSource = null;

export async function requestJson(method, url, body, onSuccess, onFailure) {
  try {
    const options = { method, headers: { Accept: "application/json" } };
    if (body) {
      options.headers["Content-Type"] = "application/json";
      options.body = body;
    }
    const response = await fetch(url, options);
    const text = await response.text();
    if (!response.ok) {
      onFailure(text || `HTTP ${response.status}`);
      return;
    }
    onSuccess(text || "{}");
  } catch (error) {
    onFailure(String(error));
  }
}

export function prettyJson(raw) {
  try {
    return JSON.stringify(JSON.parse(raw), null, 2) + "\n";
  } catch {
    return raw;
  }
}

export function currentHash() {
  return window.location.hash.replace(/^#/, "");
}

export function setViewHash(view) {
  const hash = `#${view}`;
  if (window.location.hash !== hash) {
    window.history.pushState(null, "", hash);
  }
}

export function watchViewHash(onChange) {
  const notify = () => onChange(currentHash());
  window.addEventListener("popstate", notify);
  window.addEventListener("hashchange", notify);
}

// IDE editor bridge (IDE.md §4): cursor introspection for completion and a
// single shared debounce timer for buffer checks.

export function textareaCursor(id, cb) {
  const el = document.getElementById(id);
  cb(el && typeof el.selectionStart === "number" ? el.selectionStart : 0);
}

let checkTimer = null;

export function scheduleCheck(delayMs, cb) {
  if (checkTimer) clearTimeout(checkTimer);
  checkTimer = setTimeout(() => {
    checkTimer = null;
    cb();
  }, delayMs);
}

export async function copyText(value) {
  try {
    await navigator.clipboard.writeText(value);
  } catch (error) {
    console.error("Could not copy Packwand text", error);
  }
}

// Launcher (boot-a-pack) bridge: talks to the Tauri desktop shell directly
// via window.__TAURI__ (tauri.conf.json sets withGlobalTauri: true), not the
// packwand HTTP API — this is the in-process Rust core path (packwandrs.md),
// with no Go sidecar involved. Unavailable when running in a plain browser.

function requireTauri(onError) {
  if (!window.__TAURI__) {
    onError("the launcher requires the Packwand desktop app (Tauri), not a browser");
    return null;
  }
  return window.__TAURI__;
}

export function bootPack(packDir, dock, onSessionId, onError) {
  const tauri = requireTauri(onError);
  if (!tauri) return;
  tauri.core
    .invoke("launcher_boot", { packDir, dock })
    .then(onSessionId)
    .catch((error) => onError(String(error)));
}

export function listPackInstances(onSuccess, onError) {
  const tauri = requireTauri(onError); if (!tauri) return;
  tauri.core.invoke("launcher_list_pack_instances").then((items) => onSuccess(JSON.stringify(items))).catch((error) => onError(String(error)));
}

export function deletePackInstance(instanceId, onDone, onError) {
  const tauri = requireTauri(onError); if (!tauri) return;
  tauri.core.invoke("launcher_delete_pack_instance", { instanceId }).then(() => onDone()).catch((error) => onError(String(error)));
}

export function cancelBoot(sessionId, onDone, onError) {
  const tauri = requireTauri(onError);
  if (!tauri) return;
  tauri.core
    .invoke("launcher_cancel", { sessionId })
    .then(() => onDone())
    .catch((error) => onError(String(error)));
}

let launcherListenersStarted = false;

export function watchLauncher(onEvent, onProgress) {
  if (launcherListenersStarted || !window.__TAURI__) return;
  launcherListenersStarted = true;
  const tauri = window.__TAURI__;
  tauri.event.listen("launcher://event", (e) => onEvent(JSON.stringify(e.payload)));
  tauri.event.listen("launcher://progress", (e) => onProgress(JSON.stringify(e.payload)));
}

// Real Microsoft account sign-in (packwand-msa) — same window.__TAURI__
// bridge as the launcher above, not the packwand HTTP API.

export function authLogin(onDone, onError) {
  const tauri = requireTauri(onError);
  if (!tauri) return;
  tauri.core.invoke("auth_login").then(onDone).catch((error) => onError(String(error)));
}

export function authLogout(onDone, onError) {
  const tauri = requireTauri(onError);
  if (!tauri) return;
  tauri.core.invoke("auth_logout").then(onDone).catch((error) => onError(String(error)));
}

export function authStatus(onStatus, onError) {
  const tauri = requireTauri(onError);
  if (!tauri) return;
  tauri.core
    .invoke("auth_status")
    .then((status) => onStatus(JSON.stringify(status)))
    .catch((error) => onError(String(error)));
}

let authListenerStarted = false;

export function watchAuthEvents(onEvent) {
  if (authListenerStarted || !window.__TAURI__) return;
  authListenerStarted = true;
  window.__TAURI__.event.listen("auth://event", (e) => onEvent(JSON.stringify(e.payload)));
}

export function watchJob(id, onLine, onDone) {
  if (!id) {
    onDone("failed", "The Packwand API returned an empty job ID.");
    return;
  }
  if (eventSource) eventSource.close();

  const source = new EventSource(`/api/v1/jobs/${encodeURIComponent(id)}/events`);
  eventSource = source;
  source.onmessage = (event) => {
    try {
      onLine(JSON.parse(event.data));
    } catch {
      onLine(event.data);
    }
  };
  source.onerror = async () => {
    source.close();
    if (eventSource === source) eventSource = null;
    try {
      const response = await fetch(`/api/v1/jobs/${encodeURIComponent(id)}`);
      if (!response.ok) throw new Error(await response.text());
      const job = await response.json();
      onDone(job.status || "completed", job.error || "");
    } catch (error) {
      onDone("failed", String(error));
    }
  };
}
