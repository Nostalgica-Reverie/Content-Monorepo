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

export async function copyText(value) {
  try {
    await navigator.clipboard.writeText(value);
  } catch (error) {
    console.error("Could not copy Packwand text", error);
  }
}

export function watchJob(id, onLine, onDone) {
  if (!id) {
    onDone("failed", "The Packwand API returned an empty job ID.");
    return;
  }
  if (eventSource) eventSource.close();

  const source = new EventSource(`/api/jobs/${encodeURIComponent(id)}/events`);
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
      const response = await fetch(`/api/jobs/${encodeURIComponent(id)}`);
      if (!response.ok) throw new Error(await response.text());
      const job = await response.json();
      onDone(job.status || "completed", job.error || "");
    } catch (error) {
      onDone("failed", String(error));
    }
  };
}
