const state = {
  projects: [],
  current: null,
  eventSource: null,
  view: "overview",
  mods: [],
  changelog: "",
};

const $ = (id) => document.getElementById(id);
const typeRank = { modpack: 0, resourcepack: 1, datapack: 2 };

async function api(path, options) {
  const response = await fetch(path, options);
  if (!response.ok) {
    throw new Error(await response.text());
  }
  return response.json();
}

async function boot() {
  const health = await api("/api/health");
  $("repoRoot").textContent = health.root;
  $("packwandVersion").textContent = `packwand ${health.version || ""}`.trim();
  await loadProjects();
  bindActions();
  selectView(viewFromHash());
  window.addEventListener("hashchange", () => selectView(viewFromHash()));
}

async function loadProjects() {
  const index = await api("/api/projects");
  state.projects = (index.projects || []).slice().sort(compareProjects);
  const select = $("projectSelect");
  select.innerHTML = "";
  for (const project of state.projects) {
    const option = document.createElement("option");
    option.value = project.id;
    option.textContent = `${project.id} (${project.type})`;
    select.append(option);
  }
  select.onchange = () => selectProject(select.value);
  selectProject(state.projects[0]?.id);
}

function selectProject(id) {
  state.current = state.projects.find((project) => project.id === id) || null;
  renderProject();
}

function renderProject() {
  const project = state.current;
  if (!project) {
    return;
  }

  $("projectName").textContent = project.id;
  $("projectMeta").textContent = [
    project.name,
    project.type,
    project.version ? `v${project.version}` : "",
    project.mc_version ? `mc${project.mc_version}` : "",
    project.loader || "",
  ].filter(Boolean).join("  ");
  $("projectRole").textContent = project.role || "none";
  $("subdirCount").textContent = `${project.subdirs?.length || 0} subdir(s)`;
  renderProjectIcon(project);
  loadChangelog(project.id);
  loadManifest(project.id);

  const fields = [
    ["Name", project.name],
    ["Directory", project.dir],
    ["Manifest", project.manifest_path],
    ["Lifecycle", project.lifecycle || "active"],
    ["Auto Update", project.auto_update ? "enabled" : "disabled"],
    ["Modrinth", project.modrinth_id || "-"],
    ["CurseForge", project.curseforge_id || "-"],
    ["GitHub", project.github_id || "-"],
    ["Gitea", project.gitea_id || "-"],
    ["GitLab", project.gitlab_id || "-"],
  ];
  $("projectDetails").innerHTML = fields.map(([label, value]) => `
    <div class="detail search-item">
      <span>${escapeHtml(label)}</span>
      <strong title="${escapeHtml(String(value))}">${escapeHtml(String(value))}</strong>
    </div>
  `).join("");

  const subdirs = project.subdirs || [];
  $("subdirList").innerHTML = subdirs.map((subdir) => `
    <div class="row search-item">
      <div>
        <strong>${escapeHtml(subdir.key)}</strong>
        <span>${escapeHtml(subdir.path)}${subdir.mod_count ? ` - ${subdir.mod_count} mods` : ""}</span>
      </div>
      <span>${escapeHtml(subdir.platform || "content")}</span>
    </div>
  `).join("") || `<div class="row"><span>No subdirs indexed.</span></div>`;

  const subdirSelect = $("subdirSelect");
  subdirSelect.innerHTML = "";
  for (const subdir of subdirs) {
    const option = document.createElement("option");
    option.value = subdir.path;
    option.textContent = subdir.key;
    subdirSelect.append(option);
  }
  subdirSelect.onchange = () => loadMods();
  loadMods();

  const variants = project.variants || [];
  $("variantList").innerHTML = variants.map((variant) => `
    <div class="mini-row search-item">
      <strong>${escapeHtml(variant.id || variant.mc_version || "variant")}</strong>
      <span>${escapeHtml([variant.mc_version, variant.loader, variant.version].filter(Boolean).join(" / "))}</span>
    </div>
  `).join("") || `<span class="empty-note">No variants declared.</span>`;
  applySearch();
}

function renderProjectIcon(project) {
  const icon = $("projectIcon");
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
  icon.src = `/api/project-icon/${encodeURIComponent(project.id)}`;
}

function bindActions() {
  document.querySelectorAll(".nav-btn").forEach((button) => {
    button.addEventListener("click", () => {
      selectView(button.dataset.view || "overview", true);
    });
  });

  $("refreshProjects").onclick = async () => {
    const job = await startAction({ action: "packs-index" });
    await watchJob(job.job_id);
    await loadProjects();
  };

  $("validateAll").onclick = () => startAction({ action: "validate-all" }).then((job) => watchJob(job.job_id));

  document.querySelectorAll("[data-action]").forEach((button) => {
    button.addEventListener("click", async () => {
      const job = await startAction({
        action: button.dataset.action,
        dry_run: button.dataset.dryRun === "true",
      });
      watchJob(job.job_id);
    });
  });

  document.querySelectorAll("[data-subdir-action]").forEach((button) => {
    button.addEventListener("click", async () => {
      const subdir = $("subdirSelect").value;
      if (!subdir) {
        appendLog("No subdir selected.");
        return;
      }
      const job = await startAction({
        action: button.dataset.subdirAction,
        subdir,
      });
      watchJob(job.job_id, loadMods);
    });
  });

  $("copySummary").onclick = async () => {
    const text = $("changelogPreview").innerText;
    await navigator.clipboard.writeText(text);
    appendLog("Copied changelog summary.");
  };

  $("addModButton").onclick = async () => {
    const slug = $("modSlugInput").value.trim();
    const subdir = $("subdirSelect").value;
    if (!slug || !subdir) {
      appendLog("Select a subdir and enter a mod slug.");
      return;
    }
    const job = await startAction({ action: "add-mod", subdir, slug });
    watchJob(job.job_id, loadMods);
  };

  $("packSearch").oninput = () => applySearch();

  $("saveManifest").onclick = async () => {
    const project = state.current;
    if (!project) return;
    try {
      await api(`/api/projects/${encodeURIComponent(project.id)}/manifest`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ content: $("manifestEditor").value }),
      });
      appendLog(`Saved manifest for ${project.id}.`);
      await loadProjects();
    } catch (error) {
      appendLog(error.message || String(error));
    }
  };

  $("createPack").onclick = async () => {
    const payload = {
      id: $("newPackID").value.trim(),
      name: $("newPackName").value.trim(),
      type: $("newPackType").value,
      loader: $("newPackLoader").value.trim(),
      mc_version: $("newPackMC").value.trim(),
      version: $("newPackVersion").value.trim(),
      description: $("newPackDescription").value.trim(),
    };
    try {
      await api("/api/projects", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
      });
      appendLog(`Created pack ${payload.id}.`);
      await new Promise((resolve) => setTimeout(resolve, 500));
      await loadProjects();
      selectProject(payload.id);
    } catch (error) {
      appendLog(error.message || String(error));
    }
  };
}

function selectView(view, writeHash = false) {
  const valid = new Set(["overview", "exports", "mods", "changelog", "logs", "settings"]);
  state.view = valid.has(view) ? view : "overview";

  document.querySelectorAll(".nav-btn").forEach((button) => {
    button.classList.toggle("active", button.dataset.view === state.view);
  });

  document.querySelectorAll("[data-view-section]").forEach((section) => {
    const views = (section.dataset.viewSection || "").split(/\s+/);
    section.hidden = !views.includes(state.view);
  });

  if (writeHash && location.hash !== `#${state.view}`) {
    history.pushState(null, "", `#${state.view}`);
  }
  document.body.dataset.view = state.view;
  applySearch();
}

async function startAction(payload) {
  appendLog(`> ${payload.action}`);
  return api("/api/actions", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
}

async function watchJob(jobID, afterDone) {
  if (state.eventSource) {
    state.eventSource.close();
  }
  $("jobStatus").textContent = "running";
  const events = new EventSource(`/api/jobs/${jobID}/events`);
  state.eventSource = events;
  events.onmessage = (event) => appendLog(JSON.parse(event.data));
  events.onerror = async () => {
    events.close();
    const job = await api(`/api/jobs/${jobID}`);
    $("jobStatus").textContent = job.status;
    if (job.error) {
      appendLog(job.error);
    }
    if (afterDone) {
      afterDone();
    }
  };
}

async function loadMods() {
  const subdir = $("subdirSelect").value;
  if (!subdir) {
    $("modCount").textContent = "0 mods";
    $("modList").innerHTML = `<div class="row"><span>No subdir selected.</span></div>`;
    return;
  }
  try {
    state.mods = await api(`/api/mods?subdir=${encodeURIComponent(subdir)}`);
    renderMods();
  } catch (error) {
    appendLog(error.message || String(error));
  }
}

function renderMods() {
  const mods = state.mods || [];
  $("modCount").textContent = `${mods.length} mods`;
  $("modList").innerHTML = mods.map((mod) => `
      <div class="row search-item">
        <div>
          <strong>${escapeHtml(mod.name || mod.slug)}</strong>
          <span>${escapeHtml([mod.slug, mod.filename, mod.side, mod.platform].filter(Boolean).join(" / "))}</span>
        </div>
        <button class="icon-btn" data-mod-action="update-mod" data-slug="${escapeHtml(mod.slug)}">Update</button>
        <button class="icon-btn" data-mod-action="${mod.pin ? "unpin-mod" : "pin-mod"}" data-slug="${escapeHtml(mod.slug)}">${mod.pin ? "Unpin" : "Pin"}</button>
        <button class="icon-btn danger" data-mod-action="remove-mod" data-slug="${escapeHtml(mod.slug)}">Remove</button>
      </div>
    `).join("") || `<div class="row"><span>No mods found.</span></div>`;
  bindModButtons();
  applySearch();
}

async function loadChangelog(projectID) {
  try {
    const changelog = await api(`/api/projects/${encodeURIComponent(projectID)}/changelog`);
    state.changelog = changelog.content || "";
    $("changelogPreview").innerHTML = renderMarkdown(state.changelog || "No changelog.md found for this pack.");
    applySearch();
  } catch (error) {
    $("changelogPreview").innerHTML = `<p>${escapeHtml(error.message || String(error))}</p>`;
  }
}

async function loadManifest(projectID) {
  try {
    const manifest = await api(`/api/projects/${encodeURIComponent(projectID)}/manifest`);
    $("manifestEditor").value = manifest.content || "";
  } catch (error) {
    $("manifestEditor").value = error.message || String(error);
  }
}

function bindModButtons() {
  document.querySelectorAll("[data-mod-action]").forEach((button) => {
    button.addEventListener("click", async () => {
      const action = button.dataset.modAction;
      const slug = button.dataset.slug;
      const subdir = $("subdirSelect").value;
      const job = await startAction({ action, subdir, slug });
      watchJob(job.job_id, loadMods);
    });
  });
}

function appendLog(line) {
  const pane = $("logPane");
  pane.textContent += `${line}\n`;
  pane.scrollTop = pane.scrollHeight;
}

function applySearch() {
  const input = $("packSearch");
  if (!input) return;
  const query = input.value.trim().toLowerCase();
  let visible = 0;
  let total = 0;
  document.querySelectorAll(".search-item").forEach((node) => {
    total++;
    const match = query === "" || node.textContent.toLowerCase().includes(query);
    node.hidden = !match;
    if (match) visible++;
  });
  const status = $("searchStatus");
  status.hidden = query === "";
  status.textContent = `${visible}/${total}`;
}

function renderMarkdown(markdown) {
  const lines = markdown.split(/\r?\n/);
  const out = [];
  let inList = false;
  for (const line of lines) {
    if (/^\s*[-*]\s+/.test(line)) {
      if (!inList) {
        out.push("<ul>");
        inList = true;
      }
      out.push(`<li class="search-item">${inlineMarkdown(line.replace(/^\s*[-*]\s+/, ""))}</li>`);
      continue;
    }
    if (inList) {
      out.push("</ul>");
      inList = false;
    }
    if (/^###\s+/.test(line)) {
      out.push(`<h3 class="search-item">${inlineMarkdown(line.replace(/^###\s+/, ""))}</h3>`);
    } else if (/^##\s+/.test(line)) {
      out.push(`<h2 class="search-item">${inlineMarkdown(line.replace(/^##\s+/, ""))}</h2>`);
    } else if (/^#\s+/.test(line)) {
      out.push(`<h2 class="search-item">${inlineMarkdown(line.replace(/^#\s+/, ""))}</h2>`);
    } else if (line.trim() !== "") {
      out.push(`<p class="search-item">${inlineMarkdown(line)}</p>`);
    }
  }
  if (inList) out.push("</ul>");
  return out.join("");
}

function inlineMarkdown(value) {
  return escapeHtml(value).replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
}

function escapeHtml(value) {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function compareProjects(a, b) {
  const rank = (typeRank[a.type] ?? 99) - (typeRank[b.type] ?? 99);
  if (rank !== 0) {
    return rank;
  }
  return String(a.id || "").localeCompare(String(b.id || ""));
}

function viewFromHash() {
  return location.hash.replace(/^#/, "") || "overview";
}

boot().catch((error) => {
  appendLog(error.message || String(error));
});
