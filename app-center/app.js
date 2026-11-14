(() => {
  "use strict";

  // The App Center is an unprivileged planner. No command is executed here.
  const $ = (id) => document.getElementById(id);
  const ui = {
    search: $("search"), platform: $("platform"), kind: $("kind"),
    category: $("category"), list: $("app-list"),
    visible: $("visible-count"), selected: $("selected-count"),
    preview: $("plan-preview"), status: $("status"),
    snapshot: $("snapshot-note"), refresh: $("refresh-repos")
  };
  const labels = {
    native_catalog: "Native Trellis catalog",
    external_preflight: "External · preflight required",
    review_required: "Φ source · review required",
    archived_review: "Archived · review only"
  };
  let apps = [];
  let selected = new Set();
  let visibleApps = [];
  let metadata = {};
  let importedObservations = new Map();

  const setStatus = (message) => { ui.status.textContent = message; };
  const safeUrl = (value) => {
    try {
      const parsed = new URL(value);
      return parsed.protocol === "https:" ? parsed.href : null;
    } catch (_) { return null; }
  };

  function makeLink(href, text) {
    const a = document.createElement("a");
    a.textContent = text;
    const validated = safeUrl(href);
    if (validated) {
      a.href = validated;
      a.target = "_blank";
      a.rel = "noopener noreferrer";
    } else {
      a.removeAttribute("href");
    }
    return a;
  }

  function updateCategories() {
    const last = ui.category.value;
    const categories = [...new Set(apps.map(a => a.category))].sort();
    ui.category.replaceChildren(new Option("All categories", "all"));
    for (const category of categories) ui.category.add(new Option(category, category));
    ui.category.value = categories.includes(last) ? last : "all";
  }

  function filtersMatch(app) {
    const query = ui.search.value.trim().toLowerCase();
    if (query && ![app.name, app.description, app.repo, app.package_id].filter(Boolean)
        .some(value => value.toLowerCase().includes(query))) return false;
    if (ui.kind.value !== "all" && ui.kind.value !== app.kind) return false;
    if (ui.category.value !== "all" && ui.category.value !== app.category) return false;
    if (ui.platform.value !== "all" && !(app.platforms || []).includes(ui.platform.value)) return false;
    return true;
  }

  function rowFor(app) {
    const row = document.createElement("label");
    row.className = "app-row";
    const input = document.createElement("input");
    input.type = "checkbox";
    input.checked = selected.has(app.id);
    input.setAttribute("aria-label", "Select " + app.name);
    input.addEventListener("change", () => {
      if (input.checked) selected.add(app.id);
      else selected.delete(app.id);
      renderPlan();
    });

    const content = document.createElement("div");
    const title = document.createElement("div");
    title.className = "app-title";
    title.textContent = app.name;
    const description = document.createElement("p");
    description.className = "app-desc";
    description.textContent = app.description || "No description provided.";
    const meta = document.createElement("div");
    meta.className = "app-meta";
    const sourceType = document.createElement("span");
    sourceType.textContent = app.kind === "phi_repo" ? "Public source only" :
      app.kind === "trellis" ? "Trellis spec" : app.kind === "winget" ? "WinGet ID" : "Flatpak ID";
    meta.append(sourceType);
    if (app.package_id) {
      const identifier = document.createElement("code");
      identifier.textContent = app.package_id;
      meta.append(identifier);
    }
    if (app.repo) {
      const repo = document.createElement("span");
      repo.textContent = app.repo;
      meta.append(repo);
    }
    if (app.source) meta.append(makeLink(app.source, "Source"));
    if (app.homepage) meta.append(makeLink(app.homepage, "Website"));
    content.append(title, description, meta);
    if (app.kind === "winget" && importedObservations.has(app.package_id)) {
      const observation = document.createElement("div");
      observation.className = "imported-observation";
      const status = importedObservations.get(app.package_id);
      observation.textContent = ({
        observed_in_winget_list: "WinGet list: observed (not a functionality check)",
        not_confirmed_in_winget_list: "WinGet list: not confirmed",
        unknown: "WinGet list: status unknown"
      })[status];
      content.append(observation);
    }

    const tag = document.createElement("span");
    tag.className = "tag " + (labels[app.status] ? app.status : "review_required");
    tag.textContent = labels[app.status] || "Needs review";
    row.append(input, content, tag);
    return row;
  }

  function renderCatalog() {
    visibleApps = apps.filter(filtersMatch).sort((a, b) =>
      a.category.localeCompare(b.category) || a.name.localeCompare(b.name));
    ui.list.replaceChildren();
    const fragment = document.createDocumentFragment();
    for (const app of visibleApps) fragment.append(rowFor(app));
    ui.list.append(fragment);
    ui.visible.textContent = visibleApps.length + " shown of " + apps.length + " entries";
    if (!visibleApps.length) {
      const empty = document.createElement("p");
      empty.className = "footnote";
      empty.textContent = "Nothing matches these filters.";
      ui.list.append(empty);
    }
    $("stat-total").textContent = apps.length;
    $("stat-native").textContent = apps.filter(a => a.kind === "trellis").length;
    $("stat-external").textContent = apps.filter(a => a.kind === "winget" || a.kind === "flatpak").length;
    $("stat-repos").textContent = apps.filter(a => a.kind === "phi_repo").length;
  }

  function planStep(app) {
    if (app.kind === "trellis") {
      return {
        action: "native_trellis_install_after_local_review",
        preflight: ["trellis --registry-root packages seed", "trellis info " + app.package_id],
        suggested_commands: ["trellis install " + app.package_id],
        note: "Requires Trellis installed, matching OS/architecture, and locally reviewed registry specs."
      };
    }
    if (app.kind === "winget") {
      return {
        action: "external_windows_preflight",
        preflight: ["winget show --id " + app.package_id + " --exact --source winget"],
        suggested_commands: ["winget install --id " + app.package_id + " --exact --source winget"],
        note: "Not a native Trellis package. Verify source, package identity, license, and any prompts on Windows."
      };
    }
    if (app.kind === "flatpak") {
      return {
        action: "external_linux_preflight",
        preflight: ["flatpak remote-info flathub " + app.package_id],
        suggested_commands: ["flatpak install flathub " + app.package_id],
        note: "Not a native Trellis package. Requires Flatpak and a reviewed Flathub remote."
      };
    }
    if (app.status === "archived_review") {
      return {
        action: "archived_source_review_only", preflight: [], suggested_commands: [],
        note: "Archived source; no installation should be inferred."
      };
    }
    return {
      action: "phios_compatibility_review_only",
      preflight: ["phi-app profile-github https://github.com/" + app.repo],
      suggested_commands: [],
      note: "Advisory profile only. Must pass license, compatibility, build, provenance, and explicit operator approval before any install."
    };
  }

  function buildPlan() {
    const items = apps.filter(a => selected.has(a.id)).sort((a, b) => a.id.localeCompare(b.id))
      .map(app => ({
        id: app.id, name: app.name, kind: app.kind, status: app.status,
        source: app.source, target_platforms: app.platforms,
        ...planStep(app)
      }));
    return {
      schema_version: "trellis.selection-plan.v0.1",
      created_at: new Date().toISOString(),
      approval: "not_granted",
      execution: "none",
      source_snapshot_at: metadata.public_snapshot_at,
      items
    };
  }

  function planText(plan) {
    if (!plan.items.length) return "Choose packages to see the plan.";
    const lines = [
      "TRELLIS APP CENTER · REVIEW PLAN",
      "Selection count: " + plan.items.length,
      "Nothing is installed or authorized by this file.",
      "Native-only preflight: trellis --registry-root packages batch --plan trellis-selection-plan.json",
      "Native-only apply: inspect the full approval hash, then use --apply --confirm HASH.",
      "WinGet-only Windows preflight: trellis winget-batch --plan trellis-selection-plan.json",
      "WinGet-only Windows status: trellis winget-status --plan trellis-selection-plan.json --output trellis-winget-status.json",
      "WinGet-only Windows apply: inspect approval hash, then add --apply --confirm HASH.",
      "Mixed plans are HELD. Export separate native and WinGet-only plans.",
      "Flatpak/Phi repos remain review-only; no adapter installs them.",
      ""
    ];
    for (const app of plan.items) {
      lines.push(app.name + " [" + app.kind + " / " + app.status + "]");
      lines.push("Next action: " + app.action);
      for (const command of app.preflight) lines.push("  Preflight: " + command);
      for (const command of app.suggested_commands) lines.push("  After review: " + command);
      lines.push("  " + app.note, "");
    }
    return lines.join("\n");
  }

  function renderPlan() {
    const plan = buildPlan();
    ui.selected.textContent = plan.items.length + " selected";
    ui.preview.textContent = planText(plan);
    const disabled = plan.items.length === 0;
    $("copy-plan").disabled = disabled;
    $("download-plan").disabled = disabled;
  }

  function refreshUI() {
    updateCategories();
    selected = new Set([...selected].filter(id => apps.some(a => a.id === id)));
    renderCatalog();
    renderPlan();
  }

  function guessedCategory(name) {
    if (/phi(os|stream|serve|pie|kernel|madaprime|vine|portal|bridge|operator|reponerve|safe)|agentora|trellis|fieldshield|phircq/i.test(name)) return "Φ Infrastructure";
    if (/btm|elsewhere|retro|blackout|shed|spark|xr|vr|genewrist|bounce|reel|cine|waveforge|pixelforge|memeforge|infinite-commons|paperghost|framefield|phicade|phimath|focusbridge/i.test(name)) return "Φ Creative & XR";
    return "Φ Repositories";
  }

  async function refreshPublicRepos() {
    ui.refresh.disabled = true;
    setStatus("Reading public repository pages from GitHub; no private account access...");
    try {
      const refreshed = [];
      let complete = false;
      for (let page = 1; page <= 10; page++) {
        const response = await fetch("https://api.github.com/users/MichaelWave369/repos?per_page=100&page=" + page + "&type=owner&sort=full_name", {
          headers: { Accept: "application/vnd.github+json" }
        });
        if (!response.ok) throw new Error("GitHub API HTTP " + response.status + ". Keeping prior snapshot.");
        const rows = await response.json();
        if (!Array.isArray(rows)) throw new Error("Unexpected GitHub response.");
        for (const repo of rows) {
          if (!repo || repo.private || repo.owner?.login !== "MichaelWave369" ||
              typeof repo.name !== "string" || !/^[A-Za-z0-9._-]+$/.test(repo.name)) continue;
          refreshed.push({
            id: "phi:" + repo.name, name: repo.name,
            description: (repo.description || "Public source repository; install readiness not assessed.").slice(0, 280),
            category: guessedCategory(repo.name),
            kind: "phi_repo", status: repo.archived ? "archived_review" : "review_required",
            platforms: ["phios"], repo: "MichaelWave369/" + repo.name,
            source: "https://github.com/MichaelWave369/" + repo.name,
            homepage: safeUrl(repo.homepage), archived: !!repo.archived
          });
        }
        if (rows.length < 100) { complete = true; break; }
      }
      if (!complete) throw new Error("Repository pagination was incomplete; original snapshot kept.");
      const unique = new Map(refreshed.map(a => [a.id.toLowerCase(), a]));
      apps = [...apps.filter(a => a.kind !== "phi_repo"), ...unique.values()];
      metadata.public_snapshot_at = new Date().toISOString();
      refreshUI();
      ui.snapshot.textContent = "Public GitHub refresh completed in this browser. This is an observed repository list, not install-readiness proof.";
      setStatus("Refreshed " + unique.size + " public repositories. All remain review-only.");
    } catch (error) {
      setStatus("Refresh unavailable: " + error.message);
    } finally {
      ui.refresh.disabled = false;
    }
  }

  async function importStatusReport() {
    const control = $("status-file");
    const file = control.files && control.files[0];
    if (!file) return;
    try {
      if (file.size > 256 * 1024) throw new Error("The status file exceeds 256 KiB.");
      const report = JSON.parse(await file.text());
      if (report.schema_version !== "trellis.winget-status.v0.1" ||
          report.backend !== "winget" || !Array.isArray(report.items) ||
          report.items.length > 500) {
        throw new Error("Unrecognized WinGet status report.");
      }
      const allowed = new Set(apps.filter(app => app.kind === "winget").map(app => app.package_id));
      const valid = new Set([
        "observed_in_winget_list", "not_confirmed_in_winget_list", "unknown"
      ]);
      const imported = new Map();
      for (const record of report.items) {
        if (!record || typeof record.package_id !== "string" ||
            !allowed.has(record.package_id) || !valid.has(record.observation) ||
            imported.has(record.package_id)) {
          throw new Error("Invalid or duplicate WinGet observation.");
        }
        imported.set(record.package_id, record.observation);
      }
      importedObservations = imported;
      $("clear-status").disabled = imported.size === 0;
      renderCatalog();
      setStatus("Loaded " + imported.size +
        " local WinGet observations. These do not prove application health.");
    } catch (error) {
      setStatus("Status import rejected: " + error.message);
    } finally {
      control.value = "";
    }
  }

  async function init() {
    try {
      const response = await fetch("./catalog.json");
      if (!response.ok) throw new Error("Catalog file HTTP " + response.status);
      const data = await response.json();
      if (data.schema_version !== "trellis.app-center.v0.1" || !Array.isArray(data.apps))
        throw new Error("Unsupported catalog schema.");
      metadata = data;
      const unique = new Map();
      for (const app of data.apps) {
        if (app && typeof app.id === "string" && typeof app.name === "string") unique.set(app.id, app);
      }
      apps = [...unique.values()];
      ui.snapshot.textContent = "Bundled public-repo snapshot: " + data.public_snapshot_at +
        ". It may be stale; Refresh requests only public GitHub metadata.";
      refreshUI();
      setStatus("Catalog loaded. Selection and planning are local to this page.");
    } catch (error) {
      setStatus("Could not load the catalog: " + error.message +
        ". Open with a local web server, for example: python -m http.server 8000");
    }
  }

  for (const input of [ui.search, ui.platform, ui.kind, ui.category]) {
    input.addEventListener("input", renderCatalog);
  }
  $("select-visible").addEventListener("click", () => {
    for (const app of visibleApps) selected.add(app.id);
    renderCatalog();
    renderPlan();
  });
  $("clear-selection").addEventListener("click", () => {
    selected.clear();
    renderCatalog();
    renderPlan();
  });
  ui.refresh.addEventListener("click", refreshPublicRepos);
  $("status-file").addEventListener("change", importStatusReport);
  $("clear-status").addEventListener("click", () => {
    importedObservations.clear();
    $("clear-status").disabled = true;
    renderCatalog();
    setStatus("Imported status observations cleared.");
  });
  $("download-plan").addEventListener("click", () => {
    const plan = buildPlan();
    if (!plan.items.length) return;
    const blob = new Blob([JSON.stringify(plan, null, 2) + "\n"], { type: "application/json" });
    const objectUrl = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = objectUrl;
    anchor.download = "trellis-selection-plan.json";
    document.body.append(anchor);
    anchor.click();
    anchor.remove();
    URL.revokeObjectURL(objectUrl);
    setStatus("Review plan exported. No installation was performed.");
  });
  $("copy-plan").addEventListener("click", async () => {
    const plan = buildPlan();
    if (!plan.items.length) return;
    try {
      await navigator.clipboard.writeText(planText(plan));
      setStatus("Review steps copied. Manually inspect them before use.");
    } catch (_) {
      setStatus("Clipboard permission unavailable; select and copy the plan from the text panel.");
    }
  });
  init();
})();
