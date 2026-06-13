import * as alphaTab from "@coderline/alphatab";

// ── DOM refs ──────────────────────────────────────────────────────────────────
const atContainer       = document.getElementById("alphatab")!;
const placeholder       = document.getElementById("placeholder")!;
const sidebar           = document.getElementById("sidebar")!;
const sidebarToggle     = document.getElementById("sidebar-toggle") as HTMLButtonElement;
const trackList         = document.getElementById("track-list")!;
const scoreTitle        = document.getElementById("score-title")!;
const fileInput         = document.getElementById("file-input") as HTMLInputElement;
const zoomSlider        = document.getElementById("zoom-slider") as HTMLInputElement;
const zoomValue         = document.getElementById("zoom-value")!;
const statusBar         = document.getElementById("status-bar")!;
const scoreContainer    = document.getElementById("score-container")!;
const repeatsDivider    = document.getElementById("repeats-divider")!;
const repeatsLabel      = document.getElementById("repeats-label")!;
const repeatsInfo       = document.getElementById("repeats-info")!;
const expandSeqBtn      = document.getElementById("expand-sequence-btn") as HTMLButtonElement;
const sequenceList      = document.getElementById("sequence-list")!;
const repeatsBtn        = document.getElementById("repeats-btn") as HTMLButtonElement;
const formBtn           = document.getElementById("form-btn") as HTMLButtonElement;
const formLegend        = document.getElementById("form-legend")!;
const formDivider       = document.getElementById("form-divider")!;
const formSidebarLabel  = document.getElementById("form-sidebar-label")!;
const formTrackWrap     = document.getElementById("form-track-select-wrap")!;
const formTrackSelect   = document.getElementById("form-track-select") as HTMLSelectElement;
const formInfo          = document.getElementById("form-info")!;

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
    scale:        initScale,
    staveProfile: initStaveProfile,
    layoutMode:   initLayoutMode,
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

// ── Analysis state ────────────────────────────────────────────────────────────

interface RepeatsBlock {
  open_bar: number;
  close_bar: number;
  total_plays: number;
  volta_bars: Array<{ bar: number; endings: number[] }>;
}

interface RepeatsData {
  written_measures: number;
  sounding_measures: number;
  sounding_includes_jumps: boolean;
  navigation_events: Array<{
    bar: number;
    repeat_open: boolean;
    repeat_close?: number;
    volta?: number[];
    direction?: string;
    marker?: string;
  }>;
  repeat_blocks: RepeatsBlock[];
  play_sequence: Array<{ bar: number; pass: number }>;
  simile_runs: Array<{
    track: string;
    bars: string;
    source_bars: string;
    kind: string;
  }>;
}

// Palette for repeat blocks (semi-transparent for overlay, opaque for labels).
const BLOCK_COLORS = [
  "#e74c3c", "#3498db", "#2ecc71", "#e67e22", "#9b59b6",
  "#1abc9c", "#e91e63", "#ff9800", "#00bcd4", "#8bc34a",
];

let repeatsData: RepeatsData | null = null;
let repeatsVisible = false;
let currentScoreId: string | null = null;
let sequenceExpanded = false;

async function fetchRepeats(id: string): Promise<void> {
  try {
    const res = await fetch(`/api/score/${id}/analysis/repeats`);
    if (!res.ok) return;
    repeatsData = await res.json() as RepeatsData;
    renderRepeatsSidebar();
    if (repeatsVisible) drawRepeatsOverlay();
  } catch {
    // silently ignore fetch errors
  }
}

function renderRepeatsSidebar(): void {
  if (!repeatsData) return;

  const blocks = repeatsData.repeat_blocks;
  const hasSomething = blocks.length > 0 || repeatsData.sounding_measures !== repeatsData.written_measures;

  repeatsDivider.style.display = "";
  repeatsLabel.style.display = "";
  repeatsInfo.innerHTML = "";

  if (blocks.length === 0) {
    const p = document.createElement("p");
    p.style.cssText = "padding:0 10px 6px;font-size:0.78rem;color:#666;";
    p.textContent = "(no repeat barlines)";
    repeatsInfo.appendChild(p);
  } else {
    const summary = document.createElement("p");
    summary.style.cssText = "padding:0 10px 4px;font-size:0.74rem;color:#777;";
    summary.textContent =
      `${blocks.length} block${blocks.length !== 1 ? "s" : ""} · ` +
      `${repeatsData.sounding_measures} sounding bars` +
      (repeatsData.sounding_includes_jumps ? " (excl. jumps)" : "");
    repeatsInfo.appendChild(summary);

    for (const [i, block] of blocks.entries()) {
      const color = BLOCK_COLORS[i % BLOCK_COLORS.length];
      const item = document.createElement("div");
      item.className = "repeat-block-item";

      const swatch = document.createElement("span");
      swatch.className = "repeat-swatch";
      swatch.style.background = color;

      const label = document.createElement("span");
      label.textContent = `Bars ${block.open_bar}–${block.close_bar} ×${block.total_plays}`;
      if (block.volta_bars.length > 0) {
        const voltaDesc = block.volta_bars.map(v => `[${v.endings.join(",")}] bar ${v.bar}`).join(", ");
        label.title = voltaDesc;
        label.textContent += ` (${block.volta_bars.length} volta)`;
      }

      item.append(swatch, label);
      repeatsInfo.appendChild(item);
    }
  }

  if (hasSomething && repeatsData.play_sequence.length > 0) {
    expandSeqBtn.style.display = "";
  }
}

function drawRepeatsOverlay(): void {
  removeRepeatsOverlay();
  if (!repeatsData || !repeatsData.repeat_blocks.length) return;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const boundsLookup = (api as any).renderer?.boundsLookup;
  if (!boundsLookup) return;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const masterBars: any[] = boundsLookup.masterBars ?? [];
  if (!masterBars.length) return;

  // Build 0-based measure index → bounds map
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const boundsMap = new Map<number, any>();
  for (const mb of masterBars) {
    const idx: number = mb.masterBar?.index ?? mb.index ?? -1;
    if (idx >= 0) boundsMap.set(idx, mb);
  }

  const svgNS = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(svgNS, "svg");
  svg.id = "repeats-overlay";

  // Cover the full rendered height
  let maxY = 0;
  let maxX = 0;
  for (const mb of masterBars) {
    const rb = mb.realBounds;
    if (rb) {
      maxY = Math.max(maxY, rb.y + rb.h);
      maxX = Math.max(maxX, rb.x + rb.w);
    }
  }
  svg.setAttribute("width", String(maxX + 20));
  svg.setAttribute("height", String(maxY + 20));

  for (const [i, block] of repeatsData.repeat_blocks.entries()) {
    const color = BLOCK_COLORS[i % BLOCK_COLORS.length];
    // open_bar and close_bar are 1-based; convert to 0-based index
    const openBounds = boundsMap.get(block.open_bar - 1);
    const closeBounds = boundsMap.get(block.close_bar - 1);
    if (!openBounds || !closeBounds) continue;

    const ob = openBounds.realBounds;
    const cb = closeBounds.realBounds;
    if (!ob || !cb) continue;

    const g = document.createElementNS(svgNS, "g");
    g.setAttribute("data-block", String(i));

    // Left bracket  |—  at open bar
    appendBracket(g, svgNS, ob.x - 1, ob.y + 4, ob.x + 7, ob.x - 1, ob.y + ob.h - 4, color, "left");

    // Right bracket  —|  at close bar, with ×N label
    const rx = cb.x + cb.w + 1;
    appendBracket(g, svgNS, rx, cb.y + 4, rx - 7, rx, cb.y + cb.h - 4, color, "right");

    // "×N" label
    const text = document.createElementNS(svgNS, "text");
    text.setAttribute("x", String(rx - 2));
    text.setAttribute("y", String(cb.y + 1));
    text.setAttribute("fill", color);
    text.setAttribute("font-size", "10");
    text.setAttribute("font-weight", "bold");
    text.setAttribute("text-anchor", "end");
    text.setAttribute("dominant-baseline", "hanging");
    text.textContent = `×${block.total_plays}`;
    g.appendChild(text);

    // Volta bar markers
    for (const vb of block.volta_bars) {
      const vBounds = boundsMap.get(vb.bar - 1);
      if (!vBounds?.realBounds) continue;
      const vb2 = vBounds.realBounds;
      const rect = document.createElementNS(svgNS, "rect");
      rect.setAttribute("x", String(vb2.x));
      rect.setAttribute("y", String(vb2.y));
      rect.setAttribute("width", String(vb2.w));
      rect.setAttribute("height", "3");
      rect.setAttribute("fill", color);
      rect.setAttribute("opacity", "0.5");
      g.appendChild(rect);
      const vLabel = document.createElementNS(svgNS, "text");
      vLabel.setAttribute("x", String(vb2.x + 3));
      vLabel.setAttribute("y", String(vb2.y + 1));
      vLabel.setAttribute("fill", color);
      vLabel.setAttribute("font-size", "9");
      vLabel.setAttribute("dominant-baseline", "hanging");
      vLabel.textContent = vb.endings.join(",") + ".";
      g.appendChild(vLabel);
    }

    svg.appendChild(g);
  }

  atContainer.style.position = "relative";
  atContainer.appendChild(svg);
}

function appendBracket(
  parent: SVGElement,
  ns: string,
  x: number,
  topY: number,
  tickX: number,
  vertX: number,
  botY: number,
  color: string,
  _side: "left" | "right",
): void {
  const vert = document.createElementNS(ns, "line");
  vert.setAttribute("x1", String(vertX));
  vert.setAttribute("y1", String(topY));
  vert.setAttribute("x2", String(vertX));
  vert.setAttribute("y2", String(botY));
  vert.setAttribute("stroke", color);
  vert.setAttribute("stroke-width", "2");

  const top = document.createElementNS(ns, "line");
  top.setAttribute("x1", String(x));
  top.setAttribute("y1", String(topY));
  top.setAttribute("x2", String(tickX));
  top.setAttribute("y2", String(topY));
  top.setAttribute("stroke", color);
  top.setAttribute("stroke-width", "2");

  const bot = document.createElementNS(ns, "line");
  bot.setAttribute("x1", String(x));
  bot.setAttribute("y1", String(botY));
  bot.setAttribute("x2", String(tickX));
  bot.setAttribute("y2", String(botY));
  bot.setAttribute("stroke", color);
  bot.setAttribute("stroke-width", "2");

  parent.append(vert, top, bot);
}

function removeRepeatsOverlay(): void {
  document.getElementById("repeats-overlay")?.remove();
}

function renderSequence(): void {
  if (!repeatsData) return;
  const seq = repeatsData.play_sequence;
  if (!seq.length) { sequenceList.textContent = "(empty)"; return; }

  const hasMultiPass = seq.some(b => b.pass > 1);

  // Compact into ranges
  const parts: string[] = [];
  let i = 0;
  while (i < seq.length) {
    const pass = seq[i].pass;
    let end = i;
    while (
      end + 1 < seq.length &&
      seq[end + 1].pass === pass &&
      seq[end + 1].bar === seq[end].bar + 1
    ) end++;
    const first = seq[i].bar;
    const last = seq[end].bar;
    const passStr = hasMultiPass ? ` (p${pass})` : "";
    parts.push(first === last ? `${first}${passStr}` : `${first}–${last}${passStr}`);
    i = end + 1;
  }

  sequenceList.textContent = parts.join(", ");
}

// ── Form analysis state ───────────────────────────────────────────────────────

interface FormSection {
  label: string;
  bar_start: number;
  bar_end: number;
  name?: string;
}

interface FormTrackData {
  name: string;
  form: string;
  sections: FormSection[];
}

interface FormData {
  tracks: FormTrackData[];
}

// Base-letter → color (consistent across variants A, A', A'')
const FORM_COLORS = [
  '#3498db', '#e74c3c', '#2ecc71', '#e67e22',
  '#9b59b6', '#1abc9c', '#e91e63', '#ff9800',
  '#00bcd4', '#8bc34a', '#795548', '#607d8b',
];

function formColor(label: string): string {
  const base = label.charCodeAt(0) - 65; // 'A' = 0
  return FORM_COLORS[Math.max(0, base) % FORM_COLORS.length];
}

let formData: FormData | null = null;
let formVisible = false;
let activeFormTrackIdx = 0;

async function fetchForm(id: string): Promise<void> {
  try {
    const res = await fetch(`/api/score/${id}/analysis/form`);
    if (!res.ok) return;
    formData = await res.json() as FormData;
    renderFormSidebar();
    if (formVisible) {
      renderFormLegend();
      drawFormOverlay();
    }
  } catch {
    // silently ignore
  }
}

function activeFormTrack(): FormTrackData | null {
  if (!formData || !formData.tracks.length) return null;
  return formData.tracks[activeFormTrackIdx] ?? formData.tracks[0];
}

function renderFormSidebar(): void {
  if (!formData || !formData.tracks.length) return;

  formDivider.style.display = "";
  formSidebarLabel.style.display = "";

  // Track selector (only when more than one track)
  if (formData.tracks.length > 1) {
    formTrackSelect.innerHTML = "";
    for (const [i, t] of formData.tracks.entries()) {
      const opt = document.createElement("option");
      opt.value = String(i);
      opt.textContent = t.name;
      formTrackSelect.appendChild(opt);
    }
    formTrackWrap.style.display = "";
  }

  renderFormInfo();
}

function renderFormInfo(): void {
  const track = activeFormTrack();
  formInfo.innerHTML = "";
  if (!track) return;

  const summary = document.createElement("p");
  summary.style.cssText = "padding:0 10px 4px;font-size:0.74rem;color:#777;";
  summary.textContent = track.form;
  formInfo.appendChild(summary);

  // Deduplicate: one item per unique label (first occurrence)
  const seen = new Set<string>();
  for (const sec of track.sections) {
    if (seen.has(sec.label)) continue;
    seen.add(sec.label);

    const color = formColor(sec.label);
    const item = document.createElement("div");
    item.className = "form-section-item";

    const swatch = document.createElement("span");
    swatch.className = "form-swatch";
    swatch.style.background = color;

    const occurrences = track.sections.filter(s => s.label === sec.label);
    const rangeStr = occurrences.map(s => `${s.bar_start}–${s.bar_end}`).join(", ");

    const lbl = document.createElement("span");
    lbl.title = rangeStr;
    lbl.textContent = `${sec.label}  ${rangeStr}`;

    item.append(swatch, lbl);
    formInfo.appendChild(item);
  }
}

function renderFormLegend(): void {
  const track = activeFormTrack();
  formLegend.innerHTML = "";
  if (!track) {
    formLegend.style.display = "none";
    return;
  }

  formLegend.style.display = "";
  for (const sec of track.sections) {
    const color = formColor(sec.label);
    const badge = document.createElement("span");
    badge.className = "form-badge";
    badge.style.background = color;

    const lbl = document.createElement("span");
    lbl.textContent = sec.name ? `${sec.name} (${sec.label})` : sec.label;

    const bars = document.createElement("span");
    bars.className = "badge-bars";
    bars.textContent = `${sec.bar_start}–${sec.bar_end}`;

    badge.append(lbl, bars);
    formLegend.appendChild(badge);
  }
}

// Band = one colored rectangle on one rendered line
interface FormBand {
  x: number;
  y: number;
  w: number;
  h: number;
  label: string;
  firstOfSection: boolean;
}

function buildFormBands(
  sections: FormSection[],
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  boundsMap: Map<number, any>,
): FormBand[] {
  const bands: FormBand[] = [];

  for (const sec of sections) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    let cur: { x: number; y: number; w: number; h: number } | null = null;
    let isFirstBand = true;

    for (let idx = sec.bar_start - 1; idx < sec.bar_end; idx++) {
      const mb = boundsMap.get(idx);
      if (!mb) continue;
      const rb = mb.realBounds;
      if (!rb) continue;

      const lineBreak: boolean = mb.isFirstOfLine === true && idx !== sec.bar_start - 1;

      if (cur === null || lineBreak) {
        if (cur !== null) {
          bands.push({ ...cur, label: sec.label, firstOfSection: isFirstBand });
          isFirstBand = false;
        }
        cur = { x: rb.x, y: rb.y, w: rb.w, h: rb.h };
      } else {
        const right = Math.max(cur.x + cur.w, rb.x + rb.w);
        const bottom = Math.max(cur.y + cur.h, rb.y + rb.h);
        cur.w = right - cur.x;
        cur.h = bottom - cur.y;
      }
    }

    if (cur !== null) {
      bands.push({ ...cur, label: sec.label, firstOfSection: isFirstBand });
    }
  }

  return bands;
}

function drawFormOverlay(): void {
  removeFormOverlay();
  const track = activeFormTrack();
  if (!track || !track.sections.length) return;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const boundsLookup = (api as any).renderer?.boundsLookup;
  if (!boundsLookup) return;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const masterBars: any[] = boundsLookup.masterBars ?? [];
  if (!masterBars.length) return;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const boundsMap = new Map<number, any>();
  for (const mb of masterBars) {
    const idx: number = mb.masterBar?.index ?? mb.index ?? -1;
    if (idx >= 0) boundsMap.set(idx, mb);
  }

  let maxY = 0, maxX = 0;
  for (const mb of masterBars) {
    const rb = mb.realBounds;
    if (rb) { maxY = Math.max(maxY, rb.y + rb.h); maxX = Math.max(maxX, rb.x + rb.w); }
  }

  const svgNS = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(svgNS, "svg");
  svg.id = "form-overlay";
  svg.setAttribute("width", String(maxX + 20));
  svg.setAttribute("height", String(maxY + 20));

  const bands = buildFormBands(track.sections, boundsMap);

  for (const band of bands) {
    const color = formColor(band.label);
    const hex = color;

    // Translucent fill band
    const rect = document.createElementNS(svgNS, "rect");
    rect.setAttribute("x", String(band.x));
    rect.setAttribute("y", String(band.y));
    rect.setAttribute("width", String(band.w));
    rect.setAttribute("height", String(band.h));
    rect.setAttribute("fill", hex);
    rect.setAttribute("opacity", "0.12");
    svg.appendChild(rect);

    // Top border line to mark the section
    const line = document.createElementNS(svgNS, "line");
    line.setAttribute("x1", String(band.x));
    line.setAttribute("y1", String(band.y));
    line.setAttribute("x2", String(band.x + band.w));
    line.setAttribute("y2", String(band.y));
    line.setAttribute("stroke", hex);
    line.setAttribute("stroke-width", "2");
    line.setAttribute("opacity", "0.7");
    svg.appendChild(line);

    // Label badge on the first band of each section only
    if (band.firstOfSection) {
      const badgeH = 14;
      const badgePad = 5;

      const labelText = band.label;
      const badgeW = labelText.length * 7 + badgePad * 2;

      const badgeRect = document.createElementNS(svgNS, "rect");
      badgeRect.setAttribute("x", String(band.x + 2));
      badgeRect.setAttribute("y", String(band.y + 2));
      badgeRect.setAttribute("width", String(badgeW));
      badgeRect.setAttribute("height", String(badgeH));
      badgeRect.setAttribute("rx", "2");
      badgeRect.setAttribute("fill", hex);
      badgeRect.setAttribute("opacity", "0.85");
      svg.appendChild(badgeRect);

      const text = document.createElementNS(svgNS, "text");
      text.setAttribute("x", String(band.x + 2 + badgePad));
      text.setAttribute("y", String(band.y + 2 + badgeH - 3));
      text.setAttribute("fill", "#fff");
      text.setAttribute("font-size", "10");
      text.setAttribute("font-weight", "bold");
      text.setAttribute("font-family", "system-ui,sans-serif");
      text.textContent = labelText;
      svg.appendChild(text);
    }
  }

  atContainer.style.position = "relative";
  atContainer.appendChild(svg);
}

function removeFormOverlay(): void {
  document.getElementById("form-overlay")?.remove();
}

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
  currentScoreId = id;
  // Reset repeats state
  repeatsData = null;
  removeRepeatsOverlay();
  repeatsInfo.innerHTML = "";
  expandSeqBtn.style.display = "none";
  sequenceList.style.display = "none";
  sequenceExpanded = false;
  repeatsDivider.style.display = "none";
  repeatsLabel.style.display = "none";
  // Reset form state
  formData = null;
  activeFormTrackIdx = 0;
  removeFormOverlay();
  formLegend.style.display = "none";
  formLegend.innerHTML = "";
  formInfo.innerHTML = "";
  formTrackWrap.style.display = "none";
  formDivider.style.display = "none";
  formSidebarLabel.style.display = "none";
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

  if (currentScoreId) {
    void fetchRepeats(currentScoreId);
    void fetchForm(currentScoreId);
  }
});

// ── postRenderFinished — redraw overlays after each render ───────────────────
api.postRenderFinished.on(() => {
  if (repeatsVisible) drawRepeatsOverlay();
  if (formVisible) drawFormOverlay();
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

// ── Repeats toggle ────────────────────────────────────────────────────────────
repeatsBtn.addEventListener("click", () => {
  repeatsVisible = !repeatsVisible;
  repeatsBtn.classList.toggle("active", repeatsVisible);
  if (repeatsVisible) {
    drawRepeatsOverlay();
  } else {
    removeRepeatsOverlay();
  }
});

// ── Form toggle ───────────────────────────────────────────────────────────────
formBtn.addEventListener("click", () => {
  formVisible = !formVisible;
  formBtn.classList.toggle("active", formVisible);
  if (formVisible) {
    renderFormLegend();
    drawFormOverlay();
  } else {
    formLegend.style.display = "none";
    removeFormOverlay();
  }
});

// ── Form track selector ───────────────────────────────────────────────────────
formTrackSelect.addEventListener("change", () => {
  activeFormTrackIdx = parseInt(formTrackSelect.value);
  renderFormInfo();
  if (formVisible) {
    renderFormLegend();
    drawFormOverlay();
  }
});

// ── Expand sequence ───────────────────────────────────────────────────────────
expandSeqBtn.addEventListener("click", () => {
  sequenceExpanded = !sequenceExpanded;
  if (sequenceExpanded) {
    renderSequence();
    sequenceList.style.display = "";
    expandSeqBtn.textContent = "Hide sequence";
  } else {
    sequenceList.style.display = "none";
    expandSeqBtn.textContent = "Show sequence";
  }
});

// ── URL ?id= auto-load ────────────────────────────────────────────────────────
const urlId = new URLSearchParams(location.search).get("id");
if (urlId) loadScore(urlId);
