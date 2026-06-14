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
const fingDivider       = document.getElementById("fing-divider")!;
const fingLabel         = document.getElementById("fing-label")!;
const fingInfo          = document.getElementById("fing-info")!;
const fingBtn           = document.getElementById("fing-btn") as HTMLButtonElement;
const markersDivider    = document.getElementById("markers-divider")!;
const markersLabel      = document.getElementById("markers-label")!;
const markersSearchWrap = document.getElementById("markers-search-wrap")!;
const markersSearch     = document.getElementById("markers-search") as HTMLInputElement;
const markersList       = document.getElementById("markers-list")!;

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
  core: { includeNoteBounds: true },
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

  // Simile runs
  if (repeatsData.simile_runs.length > 0) {
    const simHeading = document.createElement("p");
    simHeading.style.cssText = "padding:4px 10px 2px;font-size:0.74rem;color:#777;";
    simHeading.textContent =
      `${repeatsData.simile_runs.length} simile run${repeatsData.simile_runs.length !== 1 ? "s" : ""}:`;
    repeatsInfo.appendChild(simHeading);

    for (const run of repeatsData.simile_runs) {
      const item = document.createElement("div");
      item.className = "repeat-block-item";

      const glyph = document.createElement("span");
      glyph.style.cssText = "color:#c8a000;font-weight:bold;font-size:0.88rem;flex-shrink:0;";
      glyph.textContent = "%";

      const lbl = document.createElement("span");
      lbl.textContent = `${run.track}: bars ${run.bars}`;
      lbl.title = `source: bars ${run.source_bars} · ${run.kind}`;

      item.append(glyph, lbl);
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
  const masterBars: any[] = (boundsLookup.staffSystems ?? []).flatMap((ss: any) => ss.bars ?? []);
  if (!masterBars.length) return;

  // Build 0-based measure index → bounds map
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const boundsMap = new Map<number, any>();
  for (const mb of masterBars) {
    const idx: number = mb.masterBar?.index ?? -1;
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

  // ── Simile mark glyphs (%) ───────────────────────────────────────────────
  const drawnSimMeasures = new Set<number>();
  for (const run of repeatsData.simile_runs) {
    const [startStr, endStr] = run.bars.split("-");
    const barStart = parseInt(startStr);
    const barEnd = parseInt(endStr ?? startStr);
    if (isNaN(barStart)) continue;

    for (let bar = barStart; bar <= barEnd; bar++) {
      if (drawnSimMeasures.has(bar)) continue;
      drawnSimMeasures.add(bar);

      const mb = boundsMap.get(bar - 1);
      if (!mb?.realBounds) continue;
      const rb = mb.realBounds;

      const simText = document.createElementNS(svgNS, "text");
      simText.setAttribute("x", String(rb.x + rb.w / 2));
      simText.setAttribute("y", String(rb.y - 3));
      simText.setAttribute("fill", "#c8a000");
      simText.setAttribute("font-size", "12");
      simText.setAttribute("font-weight", "bold");
      simText.setAttribute("text-anchor", "middle");
      simText.setAttribute("dominant-baseline", "auto");
      simText.setAttribute("font-family", "system-ui,sans-serif");
      simText.textContent = "%";
      svg.appendChild(simText);
    }
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
    updateSectionNav();
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
  const masterBars: any[] = (boundsLookup.staffSystems ?? []).flatMap((ss: any) => ss.bars ?? []);
  if (!masterBars.length) return;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const boundsMap = new Map<number, any>();
  for (const mb of masterBars) {
    const idx: number = mb.masterBar?.index ?? -1;
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

// ── Fingering analysis state ──────────────────────────────────────────────────

interface FingAssignment {
  string: number;
  fret: number;
  finger: number;
  role: string;
  position_shift: boolean;
}

interface FingData {
  tracks: Array<{
    name: string;
    measures: Array<{ measure: number; assignments: FingAssignment[] }>;
  }>;
}

// index 0 unused; 1=index(blue), 2=middle(green), 3=ring(orange), 4=pinky(red)
const FING_COLORS = ["", "#3498db", "#2ecc71", "#e67e22", "#e74c3c"];
const FING_NAMES  = ["", "Index", "Middle", "Ring", "Pinky"];

let fingeringData: FingData | null = null;
let fingeringVisible = false;
// trackIdx → measureNum(1-based) → `string:fret` → assignment
let fingeringLookup: Map<number, Map<number, Map<string, FingAssignment>>> = new Map();

async function fetchFingering(id: string): Promise<void> {
  try {
    const res = await fetch(`/api/score/${id}/analysis/fingering`);
    if (!res.ok) return;
    fingeringData = await res.json() as FingData;
    buildFingeringLookup();
    renderFingInfo();
    if (fingeringVisible) drawFingeringOverlay();
  } catch {
    // silently ignore
  }
}

function buildFingeringLookup(): void {
  fingeringLookup.clear();
  if (!fingeringData) return;
  fingeringData.tracks.forEach((track, trackIdx) => {
    const measMap = new Map<number, Map<string, FingAssignment>>();
    for (const m of track.measures) {
      const noteMap = new Map<string, FingAssignment>();
      for (const a of m.assignments) {
        noteMap.set(`${a.string}:${a.fret}`, a);
      }
      measMap.set(m.measure, noteMap);
    }
    fingeringLookup.set(trackIdx, measMap);
  });
}

function renderFingInfo(): void {
  if (!fingeringData) return;
  const hasData = fingeringData.tracks.some(t => t.measures.length > 0);
  if (!hasData) return;

  fingDivider.style.display = "";
  fingLabel.style.display = "";
  fingInfo.innerHTML = "";

  for (let f = 1; f <= 4; f++) {
    const item = document.createElement("div");
    item.className = "form-section-item";
    const swatch = document.createElement("span");
    swatch.className = "form-swatch";
    swatch.style.background = FING_COLORS[f];
    const lbl = document.createElement("span");
    lbl.textContent = FING_NAMES[f];
    item.append(swatch, lbl);
    fingInfo.appendChild(item);
  }
}

function drawFingeringOverlay(): void {
  removeFingeringOverlay();
  if (!fingeringData) return;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const boundsLookup = (api as any).renderer?.boundsLookup;
  if (!boundsLookup) return;

  const svgNS = "http://www.w3.org/2000/svg";
  const svg = document.createElementNS(svgNS, "svg");
  svg.id = "fingering-overlay";

  let maxY = 0, maxX = 0;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const staffSystems: any[] = boundsLookup.staffSystems ?? [];
  for (const ss of staffSystems) {
    for (const mb of (ss.bars ?? []) as any[]) {
      const measureNum: number = (mb.masterBar?.index ?? 0) + 1;
      for (const barBounds of (mb.bars ?? []) as any[]) {
        const rb = barBounds.realBounds;
        if (rb) {
          maxY = Math.max(maxY, rb.y + rb.h);
          maxX = Math.max(maxX, rb.x + rb.w);
        }
        for (const beatBounds of (barBounds.beats ?? []) as any[]) {
          const notes: any[] = beatBounds.notes ?? [];
          for (const nb of notes) {
            const note = nb.note;
            if (!note) continue;
            const trackIdx: number = note.voice?.bar?.track?.index ?? 0;
            const string: number = note.string ?? 0;
            const fret: number = note.fret ?? 0;
            const assignment = fingeringLookup.get(trackIdx)?.get(measureNum)?.get(`${string}:${fret}`);
            if (!assignment) continue;

            const hb = nb.noteHeadBounds;
            if (!hb) continue;

            const cx = hb.x + hb.w / 2;
            const cy = hb.y + hb.h / 2;
            const r = 6;
            const color = FING_COLORS[assignment.finger] ?? "#888";
            maxY = Math.max(maxY, cy + r + 2);
            maxX = Math.max(maxX, cx + r + 2);

            const isBarre = assignment.role !== "single";

            const circle = document.createElementNS(svgNS, "circle");
            circle.setAttribute("cx", String(cx));
            circle.setAttribute("cy", String(cy));
            circle.setAttribute("r", String(r));
            circle.setAttribute("fill", color);
            circle.setAttribute("opacity", isBarre ? "0.55" : "0.80");
            if (isBarre) {
              circle.setAttribute("stroke", color);
              circle.setAttribute("stroke-width", "1.5");
            }
            svg.appendChild(circle);

            const text = document.createElementNS(svgNS, "text");
            text.setAttribute("x", String(cx));
            text.setAttribute("y", String(cy + 1));
            text.setAttribute("fill", "#fff");
            text.setAttribute("font-size", "8");
            text.setAttribute("font-weight", "bold");
            text.setAttribute("font-family", "system-ui,sans-serif");
            text.setAttribute("text-anchor", "middle");
            text.setAttribute("dominant-baseline", "middle");
            text.textContent = String(assignment.finger);
            svg.appendChild(text);
          }
        }
      }
    }
  }

  svg.setAttribute("width",  String(maxX + 20));
  svg.setAttribute("height", String(maxY + 20));
  atContainer.style.position = "relative";
  atContainer.appendChild(svg);
}

function removeFingeringOverlay(): void {
  document.getElementById("fingering-overlay")?.remove();
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
  updateSaveasLinks();
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
  // Reset markers state
  markersData = [];
  markersDivider.style.display = "none";
  markersLabel.style.display = "none";
  markersSearchWrap.style.display = "none";
  markersSearch.value = "";
  markersList.innerHTML = "";
  // Reset section nav
  currentMeasure = 1;
  updateSectionNav();
  // Reset fingering state
  fingeringData = null;
  fingeringLookup.clear();
  removeFingeringOverlay();
  fingInfo.innerHTML = "";
  fingDivider.style.display = "none";
  fingLabel.style.display = "none";
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
    void fetchFingering(currentScoreId);
    void fetchMarkers(currentScoreId);
  }
});

// ── postRenderFinished — redraw overlays after each render ───────────────────
api.postRenderFinished.on(() => {
  if (repeatsVisible) drawRepeatsOverlay();
  if (formVisible) drawFormOverlay();
  if (fingeringVisible) drawFingeringOverlay();
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
let currentMeasure = 1;

api.beatMouseDown.on((beat) => {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const bar = (beat as any)?.voice?.bar;
  if (bar !== undefined) {
    currentMeasure = (bar.index as number) + 1;
    statusBar.textContent = `Measure ${currentMeasure}`;
    updateSectionNav();
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
  updateSectionNav();
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

// ── Fingering toggle ──────────────────────────────────────────────────────────
fingBtn.addEventListener("click", () => {
  fingeringVisible = !fingeringVisible;
  fingBtn.classList.toggle("active", fingeringVisible);
  if (fingeringVisible) {
    drawFingeringOverlay();
  } else {
    removeFingeringOverlay();
  }
});

// ── Marker search ─────────────────────────────────────────────────────────────

interface MarkerInfo {
  measure: number;
  title: string;
}

let markersData: MarkerInfo[] = [];

async function fetchMarkers(id: string): Promise<void> {
  try {
    const res = await fetch(`/api/score/${id}/info`);
    if (!res.ok) return;
    const info = await res.json() as { markers?: MarkerInfo[] };
    markersData = info.markers ?? [];
    renderMarkersList("");
  } catch {
    // silently ignore
  }
}

function renderMarkersList(filter: string): void {
  markersList.innerHTML = "";

  if (!markersData.length) {
    markersDivider.style.display = "none";
    markersLabel.style.display = "none";
    markersSearchWrap.style.display = "none";
    return;
  }

  markersDivider.style.display = "";
  markersLabel.style.display = "";
  markersSearchWrap.style.display = "";

  const lf = filter.toLowerCase();
  const visible = lf
    ? markersData.filter(
        (m) => m.title.toLowerCase().includes(lf) || String(m.measure).includes(lf),
      )
    : markersData;

  if (!visible.length) {
    const p = document.createElement("p");
    p.style.cssText = "padding:4px 10px;font-size:0.74rem;color:#666;";
    p.textContent = "No match";
    markersList.appendChild(p);
    return;
  }

  for (const marker of visible) {
    const item = document.createElement("button");
    item.className = "marker-item";

    const bar = document.createElement("span");
    bar.className = "marker-bar";
    bar.textContent = String(marker.measure);

    const title = document.createElement("span");
    title.className = "marker-title";
    title.textContent = marker.title;
    title.title = marker.title;

    item.append(bar, title);
    item.addEventListener("click", () => jumpToMeasure(marker.measure));
    markersList.appendChild(item);
  }
}

markersSearch.addEventListener("input", () => renderMarkersList(markersSearch.value));

// ── Section navigation ────────────────────────────────────────────────────────
const sectionNav       = document.getElementById("section-nav")!;
const prevSectionBtn   = document.getElementById("prev-section-btn") as HTMLButtonElement;
const sectionIndicator = document.getElementById("section-indicator")!;
const nextSectionBtn   = document.getElementById("next-section-btn") as HTMLButtonElement;

function findSectionIndex(measure: number): number {
  const sections = activeFormTrack()?.sections ?? [];
  return sections.findIndex((s) => measure >= s.bar_start && measure <= s.bar_end);
}

function updateSectionNav(): void {
  const sections = activeFormTrack()?.sections ?? [];
  if (!sections.length) {
    sectionNav.classList.remove("visible");
    return;
  }
  sectionNav.classList.add("visible");

  const idx = findSectionIndex(currentMeasure);
  sectionIndicator.textContent = idx >= 0 ? sections[idx].label : "–";
  sectionIndicator.title = idx >= 0
    ? `${sections[idx].label}: bars ${sections[idx].bar_start}–${sections[idx].bar_end}`
    : "";

  prevSectionBtn.disabled = idx <= 0;
  nextSectionBtn.disabled = idx < 0 || idx >= sections.length - 1;
}

function prevSection(): void {
  const sections = activeFormTrack()?.sections ?? [];
  const idx = findSectionIndex(currentMeasure);
  if (idx > 0) jumpToMeasure(sections[idx - 1].bar_start);
}

function nextSection(): void {
  const sections = activeFormTrack()?.sections ?? [];
  const idx = findSectionIndex(currentMeasure);
  if (idx >= 0 && idx < sections.length - 1) jumpToMeasure(sections[idx + 1].bar_start);
}

prevSectionBtn.addEventListener("click", prevSection);
nextSectionBtn.addEventListener("click", nextSection);

// ── Jump-to-measure ───────────────────────────────────────────────────────────
const jumpDialog = document.getElementById("jump-dialog")!;
const jumpInput  = document.getElementById("jump-input") as HTMLInputElement;
const jumpHint   = document.getElementById("jump-hint")!;

function openJumpDialog(): void {
  const score = api.score as unknown as { masterBars?: unknown[] } | null;
  if (!score) return;
  const total = (score.masterBars ?? []).length;
  jumpInput.max   = String(total);
  jumpInput.value = "";
  jumpHint.textContent = `1–${total}  ·  Enter · Esc`;
  jumpDialog.classList.add("visible");
  jumpInput.focus();
}

function closeJumpDialog(): void {
  jumpDialog.classList.remove("visible");
}

function jumpToMeasure(n: number): void {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const score = api.score as any;
  if (!score) return;
  const masterBars: unknown[] = score.masterBars ?? [];
  if (n < 1 || n > masterBars.length) return;

  // Set tick position (scrolls cursor when player is active; no-op when disabled)
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const barTick = (masterBars[n - 1] as any)?.start;
  if (barTick !== undefined) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (api as any).tickPosition = barTick;
  }

  // Scroll the score container to the bar using rendered bounds
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const boundsLookup = (api as any).renderer?.boundsLookup;
  if (boundsLookup) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const allBars: any[] = (boundsLookup.staffSystems ?? []).flatMap((ss: any) => ss.bars ?? []);
    const target = allBars.find((mb) => (mb.masterBar?.index ?? -1) === n - 1);
    if (target?.realBounds) {
      scoreContainer.scrollTo({ top: Math.max(0, target.realBounds.y - 32), behavior: "smooth" });
    }
  }
}

jumpInput.addEventListener("keydown", (e) => {
  if (e.key === "Enter") {
    const n = parseInt(jumpInput.value, 10);
    if (!isNaN(n)) jumpToMeasure(n);
    closeJumpDialog();
    e.preventDefault();
  } else if (e.key === "Escape") {
    closeJumpDialog();
    e.preventDefault();
  }
});

// Close when clicking outside the dialog
jumpDialog.addEventListener("mousedown", (e) => e.stopPropagation());
document.addEventListener("mousedown", () => {
  if (jumpDialog.classList.contains("visible")) closeJumpDialog();
});

// Global keyboard shortcuts — skip when a text input is focused
document.addEventListener("keydown", (e) => {
  const tag = (document.activeElement as HTMLElement)?.tagName?.toLowerCase();
  if (tag === "input" || tag === "select" || tag === "textarea") return;
  if (e.ctrlKey || e.metaKey || e.altKey) return;

  if (e.key === "g" && api.score) {
    e.preventDefault();
    openJumpDialog();
  } else if (e.key === "[") {
    e.preventDefault();
    prevSection();
  } else if (e.key === "]") {
    e.preventDefault();
    nextSection();
  }
});

// ── Files modal ───────────────────────────────────────────────────────────────

const filesBtn         = document.getElementById("files-btn") as HTMLButtonElement;
const filesModal       = document.getElementById("files-modal")!;
const filesModalClose  = document.getElementById("files-modal-close") as HTMLButtonElement;
const breadcrumbs      = document.getElementById("breadcrumbs")!;
const fileList         = document.getElementById("file-list")!;
const dupDirInput      = document.getElementById("dup-dir-input") as HTMLInputElement;
const dupThresholdInput = document.getElementById("dup-threshold-input") as HTMLInputElement;
const dupThresholdVal  = document.getElementById("dup-threshold-val")!;
const dupRecursiveInput = document.getElementById("dup-recursive-input") as HTMLInputElement;
const dupScanBtn       = document.getElementById("dup-scan-btn") as HTMLButtonElement;
const dupProgress      = document.getElementById("dup-progress")!;
const dupResults       = document.getElementById("dup-results")!;

interface FileEntry { name: string; path: string; size: number; modified: number; is_dir: boolean; }
interface DupFile   { path: string; name: string; similarity: number; }
interface DupGroup  { files: DupFile[]; }
type SseMsg =
  | { type: "progress"; file: string; current: number; total: number }
  | { type: "result";   groups: DupGroup[] }
  | { type: "error";    message: string };

let dupAbortController: AbortController | null = null;

function openFilesModal(): void {
  filesModal.classList.add("visible");
  void browseDir("");
}

function closeFilesModal(): void {
  filesModal.classList.remove("visible");
  if (dupAbortController) {
    dupAbortController.abort();
    dupAbortController = null;
  }
}

async function browseDir(path: string): Promise<void> {
  try {
    const url = path ? `/api/files?path=${encodeURIComponent(path)}` : "/api/files";
    const res = await fetch(url);
    if (!res.ok) {
      fileList.textContent = "Error loading directory";
      return;
    }
    const data = await res.json() as { current: string; entries: FileEntry[] };
    renderBreadcrumbs(data.current);
    renderFileList(data.entries);
    dupDirInput.value = data.current;
  } catch {
    fileList.textContent = "Failed to fetch file list";
  }
}

function renderBreadcrumbs(current: string): void {
  breadcrumbs.innerHTML = "";
  const parts = current.split("/").filter((p) => p.length > 0);

  // Root segment
  const rootBtn = document.createElement("button");
  rootBtn.className = "breadcrumb-btn";
  rootBtn.textContent = "/";
  rootBtn.addEventListener("click", () => void browseDir("/"));
  breadcrumbs.appendChild(rootBtn);

  // Build up paths segment by segment
  let accumulated = "";
  for (let i = 0; i < parts.length; i++) {
    accumulated += "/" + parts[i];
    const capturedPath = accumulated;

    const sep = document.createElement("span");
    sep.className = "breadcrumb-sep";
    sep.textContent = "/";
    breadcrumbs.appendChild(sep);

    const btn = document.createElement("button");
    btn.className = "breadcrumb-btn";
    btn.textContent = parts[i];
    if (i === parts.length - 1) {
      btn.style.color = "#ccc";
      btn.style.cursor = "default";
    } else {
      btn.addEventListener("click", () => void browseDir(capturedPath));
    }
    breadcrumbs.appendChild(btn);
  }
}

function renderFileList(entries: FileEntry[]): void {
  fileList.innerHTML = "";
  if (entries.length === 0) {
    const p = document.createElement("p");
    p.style.cssText = "padding: 12px 14px; color: #555; font-size: 0.8rem;";
    p.textContent = "(empty directory)";
    fileList.appendChild(p);
    return;
  }

  for (const entry of entries) {
    const btn = document.createElement("button");
    btn.className = "file-entry";

    const icon = document.createElement("span");
    icon.className = "file-entry-icon";
    icon.textContent = entry.is_dir ? "📁" : "🎵";

    const name = document.createElement("span");
    name.className = "file-entry-name";
    name.textContent = entry.name;
    name.title = entry.path;

    btn.append(icon, name);

    if (entry.is_dir) {
      btn.addEventListener("click", () => void browseDir(entry.path));
    } else {
      btn.addEventListener("click", () => void openFilePath(entry.path));
    }

    fileList.appendChild(btn);
  }
}

async function openFilePath(path: string): Promise<void> {
  try {
    const res = await fetch("/api/score/open", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ path }),
    });
    if (!res.ok) {
      const err = await res.json() as { error: string; detail: string };
      console.error(`Open failed: ${err.error} — ${err.detail}`);
      return;
    }
    const { id } = await res.json() as { id: string };
    loadScore(id);
    closeFilesModal();
  } catch {
    console.error("Failed to open file");
  }
}

async function runDupScan(): Promise<void> {
  if (dupAbortController) {
    dupAbortController.abort();
  }
  dupAbortController = new AbortController();

  dupProgress.textContent = "Starting scan…";
  dupResults.innerHTML = "";
  dupScanBtn.disabled = true;

  const body = {
    dir: dupDirInput.value,
    threshold: parseFloat(dupThresholdInput.value),
    recursive: dupRecursiveInput.checked,
  };

  try {
    const res = await fetch("/api/duplicates", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      signal: dupAbortController.signal,
    });

    if (!res.ok) {
      const err = await res.json() as { error: string; detail: string };
      dupProgress.textContent = `Error: ${err.error}`;
      return;
    }

    const reader = res.body!.getReader();
    const decoder = new TextDecoder();
    let buffer = "";

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      const messages = buffer.split("\n\n");
      buffer = messages.pop() ?? "";
      for (const message of messages) {
        for (const line of message.split("\n")) {
          if (!line.startsWith("data: ")) continue;
          const raw = line.slice(6).trim();
          if (!raw || raw === "ping") continue;
          try { handleDupEvent(JSON.parse(raw) as SseMsg); } catch { /* skip */ }
        }
      }
    }
  } catch (err) {
    if ((err as Error).name !== "AbortError") {
      dupProgress.textContent = "Scan failed";
    }
  } finally {
    dupScanBtn.disabled = false;
    dupAbortController = null;
  }
}

function handleDupEvent(msg: SseMsg): void {
  if (msg.type === "progress") {
    dupProgress.textContent = `${msg.current}/${msg.total}: ${msg.file.split("/").pop() ?? msg.file}`;
  } else if (msg.type === "result") {
    if (msg.groups.length === 0) {
      dupProgress.textContent = "No duplicates found.";
    } else {
      dupProgress.textContent = `Found ${msg.groups.length} group${msg.groups.length !== 1 ? "s" : ""}.`;
      renderDupResults(msg.groups);
    }
  } else if (msg.type === "error") {
    dupProgress.textContent = `Error: ${msg.message}`;
  }
}

function renderDupResults(groups: DupGroup[]): void {
  dupResults.innerHTML = "";
  for (const [gi, group] of groups.entries()) {
    const card = document.createElement("div");
    card.className = "dup-group";

    const header = document.createElement("div");
    header.className = "dup-group-header";
    header.textContent = `Group ${gi + 1} · ${group.files.length} files`;
    card.appendChild(header);

    for (const file of group.files) {
      const item = document.createElement("div");
      item.className = "dup-file-item";

      const nameLine = document.createElement("div");
      nameLine.className = "dup-file-name";
      nameLine.textContent = file.name;
      nameLine.title = file.path;

      const metaLine = document.createElement("div");
      metaLine.className = "dup-file-meta";

      const simBadge = document.createElement("span");
      simBadge.className = "dup-sim-badge";
      simBadge.textContent = `${Math.round(file.similarity * 100)}%`;

      const pathSpan = document.createElement("span");
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      const dirPart = file.path.split("/").slice(0, -1).join("/") || "/";
      pathSpan.textContent = dirPart;
      pathSpan.title = file.path;
      pathSpan.style.overflow = "hidden";
      pathSpan.style.textOverflow = "ellipsis";
      pathSpan.style.whiteSpace = "nowrap";

      metaLine.append(simBadge, pathSpan);
      item.append(nameLine, metaLine);
      card.appendChild(item);
    }

    dupResults.appendChild(card);
  }
}

// ── Files modal event wiring ──────────────────────────────────────────────────
filesBtn.addEventListener("click", openFilesModal);
filesModalClose.addEventListener("click", closeFilesModal);

filesModal.addEventListener("mousedown", (e) => {
  if (e.target === filesModal) closeFilesModal();
});

dupThresholdInput.addEventListener("input", () => {
  const pct = Math.round(parseFloat(dupThresholdInput.value) * 100);
  dupThresholdVal.textContent = `${pct}%`;
});

dupScanBtn.addEventListener("click", () => void runDupScan());

// ── Format conversion download (Part 8.2) ────────────────────────────────────
const saveasBtn      = document.getElementById("saveas-btn")!;
const saveasPopover  = document.getElementById("saveas-popover")!;
const saveasGp5      = document.getElementById("saveas-gp5") as HTMLAnchorElement;
const saveasGpx      = document.getElementById("saveas-gpx") as HTMLAnchorElement;

function updateSaveasLinks(): void {
  if (!currentScoreId) return;
  saveasGp5.href = `/api/score/${currentScoreId}/download?format=gp5`;
  saveasGpx.href = `/api/score/${currentScoreId}/download?format=gpx`;
}

function openSaveasPopover(): void {
  if (!currentScoreId) return;
  updateSaveasLinks();
  const rect = saveasBtn.getBoundingClientRect();
  saveasPopover.style.top = `${rect.bottom + 4}px`;
  saveasPopover.style.left = `${rect.left}px`;
  saveasPopover.classList.add("visible");
}

function closeSaveasPopover(): void {
  saveasPopover.classList.remove("visible");
}

saveasBtn.addEventListener("click", (e) => {
  e.stopPropagation();
  if (saveasPopover.classList.contains("visible")) {
    closeSaveasPopover();
  } else {
    openSaveasPopover();
  }
});

saveasGp5.addEventListener("click", () => closeSaveasPopover());
saveasGpx.addEventListener("click", () => closeSaveasPopover());

document.addEventListener("click", (e) => {
  if (!saveasPopover.contains(e.target as Node) && e.target !== saveasBtn) {
    closeSaveasPopover();
  }
});

document.addEventListener("keydown", (e) => {
  if (e.key === "Escape") closeSaveasPopover();
}, { capture: false });

// ── Track extraction (Part 8.1) ───────────────────────────────────────────────
const extractBtn      = document.getElementById("extract-btn")!;
const extractDialog   = document.getElementById("extract-dialog")!;
const extractFmtGp5   = document.getElementById("extract-fmt-gp5") as HTMLButtonElement;
const extractFmtGpx   = document.getElementById("extract-fmt-gpx") as HTMLButtonElement;
const extractInvert   = document.getElementById("extract-invert") as HTMLInputElement;
const extractSummary  = document.getElementById("extract-summary")!;
const extractDownload = document.getElementById("extract-download-btn") as HTMLButtonElement;
const extractClose    = document.getElementById("extract-close-btn") as HTMLButtonElement;

let extractFormat: "gp5" | "gpx" = "gp5";

function getSelectedTrackIndices(): number[] {
  return Array.from(
    trackList.querySelectorAll<HTMLInputElement>("input[type=checkbox]:checked")
  ).map((cb) => parseInt(cb.dataset.index ?? "0"));
}

function updateExtractSummary(): void {
  const selected = getSelectedTrackIndices();
  const total = trackList.querySelectorAll<HTMLInputElement>("input[type=checkbox]").length;
  const invert = extractInvert.checked;
  const kept = invert ? total - selected.length : selected.length;
  const keptStr = kept === 1 ? "1 track" : `${kept} tracks`;
  if (total === 0) {
    extractSummary.textContent = "No score loaded";
    extractDownload.disabled = true;
    return;
  }
  if (kept === 0) {
    extractSummary.textContent = "No tracks would remain";
    extractDownload.disabled = true;
    return;
  }
  extractSummary.textContent = `${keptStr} of ${total} will be exported`;
  extractDownload.disabled = false;
}

function openExtractDialog(): void {
  if (!currentScoreId) return;
  updateExtractSummary();
  extractDialog.classList.add("visible");
}

function closeExtractDialog(): void {
  extractDialog.classList.remove("visible");
}

async function runExtract(): Promise<void> {
  if (!currentScoreId) return;
  const selected = getSelectedTrackIndices();
  const invert = extractInvert.checked;
  const format = extractFormat;

  if (!invert && selected.length === 0) {
    extractSummary.textContent = "Select at least one track";
    return;
  }

  extractDownload.disabled = true;
  extractDownload.textContent = "Downloading…";

  try {
    const res = await fetch(`/api/score/${currentScoreId}/extract`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ tracks: selected, invert, format }),
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: res.statusText }));
      extractSummary.textContent = `Error: ${err.error ?? res.statusText}`;
      return;
    }

    const blob = await res.blob();
    const disposition = res.headers.get("Content-Disposition") ?? "";
    const fnMatch = disposition.match(/filename="([^"]+)"/);
    const fileName = fnMatch ? fnMatch[1] : `extracted.${format}`;

    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = fileName;
    document.body.appendChild(a);
    a.click();
    a.remove();
    URL.revokeObjectURL(url);

    closeExtractDialog();
  } catch (e) {
    extractSummary.textContent = `Error: ${e instanceof Error ? e.message : String(e)}`;
  } finally {
    extractDownload.disabled = false;
    extractDownload.textContent = "Download";
  }
}

extractBtn.addEventListener("click", openExtractDialog);
extractClose.addEventListener("click", closeExtractDialog);
extractDownload.addEventListener("click", () => void runExtract());

extractInvert.addEventListener("change", updateExtractSummary);

extractFmtGp5.addEventListener("click", () => {
  extractFormat = "gp5";
  extractFmtGp5.classList.add("active");
  extractFmtGpx.classList.remove("active");
});

extractFmtGpx.addEventListener("click", () => {
  extractFormat = "gpx";
  extractFmtGpx.classList.add("active");
  extractFmtGp5.classList.remove("active");
});

// ── URL ?id= auto-load ────────────────────────────────────────────────────────
const urlId = new URLSearchParams(location.search).get("id");
if (urlId) loadScore(urlId);
