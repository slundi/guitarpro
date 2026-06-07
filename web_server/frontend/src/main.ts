import * as alphaTab from "@coderline/alphatab";

// ── DOM refs ──────────────────────────────────────────────────────────────────
const atContainer    = document.getElementById("alphatab")!;
const placeholder    = document.getElementById("placeholder")!;
const sidebar        = document.getElementById("sidebar")!;
const sidebarToggle  = document.getElementById("sidebar-toggle") as HTMLButtonElement;
const trackList      = document.getElementById("track-list")!;
const scoreTitle     = document.getElementById("score-title")!;
const fileInput      = document.getElementById("file-input") as HTMLInputElement;
const zoomSlider     = document.getElementById("zoom-slider") as HTMLInputElement;
const zoomValue      = document.getElementById("zoom-value")!;
const statusBar      = document.getElementById("status-bar")!;
const scoreContainer = document.getElementById("score-container")!;

// ── Persisted preferences ─────────────────────────────────────────────────────
const PREF_MODE   = "staveProfile";
const PREF_LAYOUT = "layoutMode";
const PREF_SCALE  = "scale";

const staveProfileMap: Record<string, alphaTab.StaveProfile> = {
  "notation-tab": alphaTab.StaveProfile.ScoreTab,
  "notation":     alphaTab.StaveProfile.Score,
  "tab":          alphaTab.StaveProfile.Tab,
};

const layoutModeMap: Record<string, alphaTab.LayoutMode> = {
  "page":       alphaTab.LayoutMode.Page,
  "horizontal": alphaTab.LayoutMode.Horizontal,
};

const initStaveProfile = staveProfileMap[localStorage.getItem(PREF_MODE) ?? ""] ?? alphaTab.StaveProfile.ScoreTab;
const initLayoutMode   = layoutModeMap[localStorage.getItem(PREF_LAYOUT) ?? ""] ?? alphaTab.LayoutMode.Page;
const initScale        = parseFloat(localStorage.getItem(PREF_SCALE) ?? "1");

// ── alphaTab initialisation ───────────────────────────────────────────────────
const api = new alphaTab.AlphaTabApi(atContainer, {
  player: { enablePlayer: false },
  display: {
    showMeasureNumbers: true,
    scale:              initScale,
    staveProfile:       initStaveProfile,
    layoutMode:         initLayoutMode,
  },
});

// ── Sync toolbar state from localStorage ──────────────────────────────────────
const savedMode   = localStorage.getItem(PREF_MODE)   ?? "notation-tab";
const savedLayout = localStorage.getItem(PREF_LAYOUT) ?? "page";
const savedPct    = Math.round(initScale * 100);

document.querySelectorAll<HTMLButtonElement>(".mode-btn").forEach((btn) =>
  btn.classList.toggle("active", btn.dataset.mode === savedMode)
);
document.querySelectorAll<HTMLButtonElement>(".layout-btn").forEach((btn) =>
  btn.classList.toggle("active", btn.dataset.layout === savedLayout)
);
zoomSlider.value      = String(savedPct);
zoomValue.textContent = `${savedPct}%`;

// ── File loading ──────────────────────────────────────────────────────────────
async function uploadFile(file: File): Promise<void> {
  const form = new FormData();
  form.append("file", file);

  const res = await fetch("/api/score/upload", { method: "POST", body: form });
  if (!res.ok) {
    const err = await res.json() as { error: string; detail: string };
    console.error(`Upload failed: ${err.error} — ${err.detail}`);
    return;
  }
  const { id } = await res.json() as { id: string };
  loadScore(id);
}

function loadScore(id: string): void {
  const url = new URL(location.href);
  url.searchParams.set("id", id);
  history.replaceState(null, "", url.toString());
  api.load(`/api/score/${id}/raw`);
}

// ── scoreLoaded ───────────────────────────────────────────────────────────────
api.scoreLoaded.on((score) => {
  placeholder.style.display = "none";

  const title = score.title || "Guitar Score Viewer";
  document.title = title;
  scoreTitle.textContent = title;

  // Open sidebar on first load so the user sees the track list.
  if (!sidebar.classList.contains("open")) {
    sidebar.classList.add("open");
    sidebarToggle.textContent = "‹ Tracks";
  }

  buildTrackList(score.tracks as Array<{ name: string }>);
});

// ── Track selector ────────────────────────────────────────────────────────────
let singleSelect = false;

function buildTrackList(tracks: Array<{ name: string }>): void {
  trackList.innerHTML = "";
  tracks.forEach((track, i) => {
    const label = document.createElement("label");
    label.className = "track-item";

    const cb = document.createElement("input");
    cb.type = "checkbox";
    cb.checked = true;
    cb.dataset.index = String(i);
    cb.addEventListener("change", () => applyTrackSelection(cb));

    const name = document.createElement("span");
    name.textContent = track.name;

    label.append(cb, name);
    trackList.appendChild(label);
  });
}

function applyTrackSelection(changed: HTMLInputElement): void {
  const score = api.score;
  if (!score) return;

  if (singleSelect && changed.checked) {
    // Behave like a radio: uncheck siblings.
    trackList
      .querySelectorAll<HTMLInputElement>("input[type=checkbox]")
      .forEach((cb) => { if (cb !== changed) cb.checked = false; });
  }

  const selected = Array.from(
    trackList.querySelectorAll<HTMLInputElement>("input[type=checkbox]:checked")
  ).map((cb) => parseInt(cb.dataset.index ?? "0"));

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  api.renderTracks((score as any).tracks.filter((_: unknown, i: number) => selected.includes(i)));
}

// Track-mode radio: "all" | "current"
document.querySelectorAll<HTMLInputElement>('input[name="track-mode"]').forEach((radio) => {
  radio.addEventListener("change", () => {
    singleSelect = radio.value === "current";
    if (radio.value === "all") {
      const score = api.score;
      trackList
        .querySelectorAll<HTMLInputElement>("input[type=checkbox]")
        .forEach((cb) => { cb.checked = true; });
      if (score) api.renderTracks((score as any).tracks);
    }
  });
});

// ── Sidebar toggle ────────────────────────────────────────────────────────────
sidebarToggle.addEventListener("click", () => {
  sidebar.classList.toggle("open");
  sidebarToggle.textContent = sidebar.classList.contains("open") ? "‹ Tracks" : "Tracks ›";
});

// ── File input ────────────────────────────────────────────────────────────────
fileInput.addEventListener("change", () => {
  const file = fileInput.files?.[0];
  if (file) void uploadFile(file);
  fileInput.value = ""; // allow re-selecting the same file
});

// ── Drag and drop ─────────────────────────────────────────────────────────────
scoreContainer.addEventListener("dragover", (e) => {
  e.preventDefault();
  scoreContainer.classList.add("drag-over");
});
scoreContainer.addEventListener("dragleave", () =>
  scoreContainer.classList.remove("drag-over")
);
scoreContainer.addEventListener("drop", (e) => {
  e.preventDefault();
  scoreContainer.classList.remove("drag-over");
  const file = e.dataTransfer?.files[0];
  if (file) void uploadFile(file);
});

// ── Rendering mode buttons ────────────────────────────────────────────────────
document.querySelectorAll<HTMLButtonElement>(".mode-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    const mode = btn.dataset.mode ?? "notation-tab";
    localStorage.setItem(PREF_MODE, mode);
    document.querySelectorAll(".mode-btn").forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");
    api.settings.display.staveProfile =
      staveProfileMap[mode] ?? alphaTab.StaveProfile.ScoreTab;
    api.updateSettings();
    api.render();
  });
});

// ── Layout buttons ────────────────────────────────────────────────────────────
document.querySelectorAll<HTMLButtonElement>(".layout-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    const layout = btn.dataset.layout ?? "page";
    localStorage.setItem(PREF_LAYOUT, layout);
    document.querySelectorAll(".layout-btn").forEach((b) => b.classList.remove("active"));
    btn.classList.add("active");
    api.settings.display.layoutMode =
      layoutModeMap[layout] ?? alphaTab.LayoutMode.Page;
    api.updateSettings();
    api.render();
  });
});

// ── Zoom ──────────────────────────────────────────────────────────────────────
zoomSlider.addEventListener("input", () => {
  const pct   = parseInt(zoomSlider.value);
  const scale = pct / 100;
  zoomValue.textContent = `${pct}%`;
  localStorage.setItem(PREF_SCALE, String(scale));
  api.settings.display.scale = scale;
  api.updateSettings();
  api.render();
});

// ── Print ─────────────────────────────────────────────────────────────────────
document.getElementById("print-btn")!.addEventListener("click", () => window.print());

// ── Measure cursor (status bar) ───────────────────────────────────────────────
api.beatMouseDown.on((beat) => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const bar = (beat as any)?.voice?.bar;
  if (bar !== undefined) {
    statusBar.textContent = `Measure ${(bar.index as number) + 1}`;
  }
});

// ── URL ?id= auto-load ────────────────────────────────────────────────────────
const urlId = new URLSearchParams(location.search).get("id");
if (urlId) loadScore(urlId);
