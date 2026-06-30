const $ = (id) => document.getElementById(id);

let eventSource = null;
let currentView = "overview";
const typeRank = { modpack: 0, resourcepack: 1, datapack: 2 };

export async function fetchHealth(done) {
  const response = await fetch("/api/health");
  const json = await response.json();
  setText("packwandVersion", `packwand ${json.version || ""}`.trim());
  done(json.root || "");
}

export async function fetchProjects(done) {
  const response = await fetch("/api/projects");
  const json = await response.json();
  done((json.projects || []).slice().sort(compareProjects));
}

export function projectCount(projects) {
  return projects.length;
}

export function projectAt(projects, index) {
  return projects[index] || {};
}

export function projectString(project, field) {
  const value = project[field];
  return value == null ? "" : String(value);
}

export function projectBool(project, field) {
  return Boolean(project[field]);
}

export function variantCount(project) {
  return Array.isArray(project.variants) ? project.variants.length : 0;
}

export function variantAt(project, index) {
  return project.variants?.[index] || {};
}

export function variantString(variant, field) {
  const value = variant[field];
  return value == null ? "" : String(value);
}

export function subdirCount(project) {
  return Array.isArray(project.subdirs) ? project.subdirs.length : 0;
}

export function subdirAt(project, index) {
  return project.subdirs?.[index] || {};
}

export function subdirString(subdir, field) {
  const value = subdir[field];
  return value == null ? "" : String(value);
}

export function subdirInt(subdir, field) {
  const value = Number(subdir[field]);
  return Number.isFinite(value) ? value : 0;
}

export function subdirBool(subdir, field) {
  return Boolean(subdir[field]);
}

export function setText(id, value) {
  const node = $(id);
  if (node) node.textContent = value;
}

export function setHtml(id, value) {
  const node = $(id);
  if (node) node.innerHTML = value;
}

export function setValue(id, value) {
  const node = $(id);
  if (node) node.value = value;
}

export function setProjectIcon(projectID) {
  const icon = $("projectIcon");
  if (!icon) return;
  icon.hidden = true;
  icon.onload = () => {
    icon.hidden = false;
  };
  icon.onerror = () => {
    if (!icon.src.endsWith("/lucy.svg")) {
      icon.src = "/lucy.svg";
      icon.hidden = false;
      return;
    }
    icon.hidden = true;
  };
  icon.src = `/api/project-icon/${encodeURIComponent(projectID)}`;
}

export function selectValue(id) {
  const node = $(id);
  return node?.value || "";
}

export function onClick(id, handler) {
  const node = $(id);
  if (node) node.onclick = () => handler();
}

export function onSelect(id, handler) {
  const node = $(id);
  if (node) node.onchange = () => handler(node.value);
}

export function onActionButtons(handler) {
  document.querySelectorAll("[data-action]").forEach((button) => {
    button.addEventListener("click", () => {
      handler(button.dataset.action || "", button.dataset.dryRun === "true");
    });
  });
}

export function onSubdirActionButtons(handler) {
  document.querySelectorAll("[data-subdir-action]").forEach((button) => {
    button.addEventListener("click", () => {
      handler(button.dataset.subdirAction || "");
    });
  });
}

export function onModButtons(handler) {
  document.querySelectorAll("[data-mod-action]").forEach((button) => {
    button.addEventListener("click", () => {
      handler(button.dataset.modAction || "", button.dataset.slug || "");
    });
  });
}

export function modSlugInput() {
  return $("modSlugInput")?.value?.trim() || "";
}

export async function fetchMods(subdir, done) {
  if (!subdir) {
    done([]);
    return;
  }
  const response = await fetch(`/api/mods?subdir=${encodeURIComponent(subdir)}`);
  if (!response.ok) {
    appendLog(await response.text());
    done([]);
    return;
  }
  done(await response.json());
}

export function modCount(mods) {
  return mods.length;
}

export function modAt(mods, index) {
  return mods[index] || {};
}

export function modString(mod, field) {
  const value = mod[field];
  return value == null ? "" : String(value);
}

export function modBool(mod, field) {
  return Boolean(mod[field]);
}

export async function startAction(name, subdir, slug, dryRun, done) {
  const response = await fetch("/api/actions", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ action: name, subdir, slug, dry_run: dryRun }),
  });
  if (!response.ok) {
    appendLog(await response.text());
    return;
  }
  const json = await response.json();
  done(json.job_id || "");
}

export function watchJob(id) {
  if (!id) return;
  if (eventSource) eventSource.close();
  setText("jobStatus", "running");
  eventSource = new EventSource(`/api/jobs/${id}/events`);
  eventSource.onmessage = (event) => {
    appendLog(JSON.parse(event.data));
  };
  eventSource.onerror = async () => {
    eventSource.close();
    eventSource = null;
    try {
      const response = await fetch(`/api/jobs/${id}`);
      const job = await response.json();
      setText("jobStatus", job.status || "done");
      if (job.error) appendLog(job.error);
    } catch (error) {
      appendLog(String(error));
    }
  };
}

export function appendLog(line) {
  const pane = $("logPane");
  if (!pane) return;
  pane.textContent += `${line}\n`;
  pane.scrollTop = pane.scrollHeight;
}

export async function copyText(text) {
  await navigator.clipboard.writeText(text);
}

export function innerText(id) {
  return $(id)?.innerText || "";
}

export function setupViews() {
  document.querySelectorAll(".nav-btn").forEach((button) => {
    button.addEventListener("click", () => {
      selectView(button.dataset.view || "overview", true);
    });
  });
  selectView(location.hash.replace(/^#/, "") || "overview");
  window.addEventListener("hashchange", () => {
    selectView(location.hash.replace(/^#/, "") || "overview");
  });
}

function selectView(view, writeHash = false) {
  const valid = new Set(["overview", "exports", "mods", "changelog", "logs", "settings"]);
  currentView = valid.has(view) ? view : "overview";
  document.querySelectorAll(".nav-btn").forEach((button) => {
    button.classList.toggle("active", button.dataset.view === currentView);
  });
  document.querySelectorAll("[data-view-section]").forEach((section) => {
    const views = (section.dataset.viewSection || "").split(/\s+/);
    section.hidden = !views.includes(currentView);
  });
  if (writeHash && location.hash !== `#${currentView}`) {
    history.pushState(null, "", `#${currentView}`);
  }
}

function compareProjects(a, b) {
  const rank = (typeRank[a.type] ?? 99) - (typeRank[b.type] ?? 99);
  if (rank !== 0) return rank;
  return String(a.id || "").localeCompare(String(b.id || ""));
}
