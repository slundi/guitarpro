# Web Server Roadmap

A browser-based score viewer and analysis tool built on **axum** (Rust backend) and
**alphaTab** (JavaScript score renderer).  The server runs locally; the user opens
`http://localhost:<PORT>` in any browser.  No Electron, no WebView wrapper — the
`web_server` crate is a self-contained binary that serves static assets and a REST API.

---

## Architecture Overview

```
┌────────────────────────────────────────────────────────┐
│  Browser                                               │
│  ┌──────────────────┐   ┌──────────────────────────┐  │
│  │  alphaTab render │   │  Analysis UI panels      │  │
│  │  (SVG/Canvas)    │   │  repeats / form / finger │  │
│  └────────┬─────────┘   └────────────┬─────────────┘  │
│           │  fetch / SSE             │                 │
└───────────┼──────────────────────────┼─────────────────┘
            │  HTTP (localhost)        │
┌───────────┼──────────────────────────┼─────────────────┐
│  axum     │                          │                  │
│  ┌────────▼──────────────────────────▼──────────────┐  │
│  │  REST API  /api/*                                 │  │
│  └───────────────────────┬───────────────────────────┘  │
│                          │                              │
│  ┌───────────────────────▼───────────────────────────┐  │
│  │  guitarpro crate  (parse + convert + analysis)    │  │
│  └───────────────────────────────────────────────────┘  │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Static assets  (HTML / JS / CSS — embedded via   │  │
│  │  rust-embed or served from a dist/ folder)        │  │
│  └───────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
```

alphaTab loads the raw GP file bytes directly in the browser (no server-side
conversion needed for rendering).  The Rust backend is only called for analysis
results that alphaTab does not provide: repeat expansion, form clustering,
fingering suggestions, etc.

---

## Part 1 — Project Bootstrap ✅

### 1.1 axum server skeleton

- Add `axum`, `tokio` (full), `tower-http` (ServeDir / ServeFile, CORS,
  compression) to `web_server/Cargo.toml`
- `main.rs`: parse `--port` (default 3000) and `--open` flags with `argh`
- Bind `TcpListener`, start axum router
- If `--open` passed, call `open::that("http://localhost:{port}")` after bind
- Graceful shutdown on Ctrl-C (`tokio::signal`)

### 1.2 Static asset serving

- Create `web_server/frontend/` directory for HTML / JS / CSS sources
- Serve `frontend/dist/` via `tower_http::services::ServeDir` at `/`
- Fallback: serve `index.html` for all unmatched GET routes (SPA routing)
- Embed assets at compile time with `rust-embed` for single-binary deployment
  (feature-flagged so dev mode can use live files from disk)

### 1.3 Frontend toolchain

- `package.json` in `web_server/frontend/` with `vite` as bundler
- Install `@coderline/alphatab` via npm
- Minimal `index.html` + `main.ts` that instantiates `AlphaTabApi` on a
  `<div id="alphatab">` element
- `vite build` outputs to `frontend/dist/`; add a `just frontend-build` recipe
- Hot-reload in dev: `vite dev` proxies `/api/*` to axum

---

## Part 2 — File Loading ✅

### 2.1 Score session store

- `AppState`: `Arc<Mutex<HashMap<Uuid, LoadedFile>>>` shared across handlers
- `LoadedFile`: raw bytes + parsed `Song` + derived `LoadedScore` + file name
- Sessions expire after 1 hour of inactivity (background `tokio::time` sweep)

### 2.2 Upload endpoint

- `POST /api/score/upload` — multipart form, accepts `.gp3/.gp4/.gp5/.gp/.gpx`
- Returns `{ id: uuid, name: string, track_count: u8, measure_count: u16 }`
- Validate extension and file size (≤ 16 MB, reuse CLI constant)
- Return structured JSON errors with HTTP 400 on parse failure

### 2.3 Local file open

- `POST /api/score/open` — JSON body `{ path: string }` for server-side file
  paths (useful when the server runs on the same machine)
- Restricted to paths under a configurable `--root` directory (default: `$HOME`)

### 2.4 Raw bytes endpoint

- `GET /api/score/:id/raw` — returns raw GP bytes with correct `Content-Type`
- alphaTab fetches this URL directly via `ScoreLoader.loadScoreAsync` using
  `settings.file` = `/api/score/:id/raw`

### 2.5 Score metadata endpoint

- `GET /api/score/:id/info` — title, artist, album, tempo, time signature,
  track list `[{ index, name, string_count, tuning }]`

---

## Part 3 — Score Rendering with alphaTab ✅

### 3.1 Renderer bootstrap

- Initialise `AlphaTabApi` with `{ file: "/api/score/{id}/raw" }` on page load
- Settings: `player.enablePlayer = false` initially (playback is Part 4)
- `api.scoreLoaded` event → populate track selector, update page title

### 3.2 Track selector

- Sidebar list of tracks with checkboxes; reflects `api.tracks`
- "All tracks" / "Current track only" toggle
- Clicking a track calls `api.renderTracks([track])`

### 3.3 Rendering mode toggle

- Toolbar buttons: **Notation**, **Tab**, **Notation + Tab**
- Maps to `api.settings.display.layoutMode` and `staveProfile`
- Persisted in `localStorage`

### 3.4 Zoom and layout

- Zoom slider → `api.settings.display.scale`
- Layout toggle: **Page** (vertical scroll) vs **Horizontal** (single line)
- Print view: hide toolbar, trigger `window.print()`

### 3.5 Measure numbers and cursor

- Enable `api.settings.display.showMeasureNumbers`
- Highlight active measure on click: listen to `api.playedBeatChanged` /
  `api.beatMouseDown` → scroll to measure, update status bar

---

## Part 4 — Playback

### 4.1 SoundFont loading

- [ ] Bundle a compact `.sf2` / `.sf3` SoundFont (e.g. GeneralUser GS ~30 MB) in
  `frontend/public/`
- [ ] `settings.player.soundFont = "/soundfont/general-user.sf3"`
- [ ] `settings.player.enablePlayer = true`
- [ ] Show loading progress via `api.soundFontLoad` event

### 4.2 Transport controls

- [ ] Play / Pause toggle → `api.playPause()`
- [ ] Stop → `api.stop()`
- [ ] Current position display (bar : beat) from `api.playedBeatChanged`

### 4.3 Tempo and loop

- [ ] Tempo multiplier slider (50 % – 200 %) → `api.playbackSpeed`
- [ ] Loop selection: click two measures to set `api.tickPosition` range,
  enable `api.isLooping`
- [ ] Count-in toggle → `api.countInVolume`

### 4.4 Metronome

- [ ] Metronome toggle → `api.metronomeVolume`

---

## Part 5 — Analysis Overlays

All analysis is fetched from the axum backend (computed by the `guitarpro` crate)
and rendered as visual overlays on top of alphaTab output.

### 5.1 Repeat structure ✅

- `GET /api/score/:id/analysis/repeats` — returns the same JSON as
  `score_tool repeats --json`: written/sounding bar counts, repeat blocks
  `[{ open, close, total_plays, voltas }]`, play sequence
- Overlay: colour repeat-open bars with a left bracket, repeat-close bars
  with a right bracket; label with × N
- Expand button: show sounding bar sequence as a numbered list below the score

### 5.2 Form sections ✅

- `GET /api/score/:id/analysis/form?track=<name>&threshold=0.75` — returns
  section list `[{ start, end, label }]` (A / B / C / A' …)
- Overlay: colour-band each section across all tracks with a translucent fill
  and a label badge above the first measure
- Legend strip at the top of the score showing the full form sequence

### 5.3 Fingering annotations

- [ ] `GET /api/score/:id/analysis/fingering?track=<name>` — returns per-measure
  finger assignments `[{ measure, string, fret, finger, role, position_shift }]`
- [ ] Overlay: render finger numbers (1–4) below each fret digit in the tab staff
  using alphaTab's `CustomScoreRenderer` hook or a transparent SVG layer aligned
  to measure bounding boxes
- [ ] Colour-code: index=blue, middle=green, ring=orange, pinky=red; barre = filled

### 5.4 Simile marks

- [ ] Surface simile-mark measures from the repeats endpoint
- [ ] Overlay: show a "%" or "𝄎" glyph above the affected measure in the UI,
  with a tooltip showing the source measure range

---

## Part 6 — Navigation and Search

### 6.1 Jump to measure

- [ ] Input field: enter a measure number → `api.tickPosition =
  api.score.masterBars[n].start`
- [ ] Keyboard shortcut `g` opens the jump dialog (like vim `G`)

### 6.2 Section navigation

- [ ] Once form analysis is loaded, a dropdown/breadcrumb shows current section
  label (A, B, …)
- [ ] Prev / Next section buttons jump to the start of the previous/next section

### 6.3 Marker search

- [ ] `GET /api/score/:id/info` includes `markers: [{ measure, title }]`
- [ ] Search box filters markers; clicking a result jumps to that measure

---

## Part 7 — File Browser and Duplicate Detection

### 7.1 Local directory listing

- [ ] `GET /api/files?path=<dir>` — lists `.gp*` files under `--root`, returns
  `[{ name, path, size, modified }]`
- [ ] Simple tree-view sidebar in the UI; click a file to open it

### 7.2 Duplicate scan

- [ ] `POST /api/duplicates` — JSON body `{ dir: string, threshold: f64,
  recursive: bool }`; runs `command_duplicates` logic server-side
- [ ] Streams results via SSE (`text/event-stream`) so large directories show
  progress incrementally
- [ ] UI: grouped result cards showing duplicate sets with similarity scores;
  click any file to load it

---

## Part 8 — Track Extraction and Export

### 8.1 Track extraction

- [ ] `POST /api/score/:id/extract` — JSON body `{ tracks: [string], invert: bool,
  format: "gp5" | "gpx" }`
- [ ] Returns the extracted score as a binary download

### 8.2 Format conversion download

- [ ] `GET /api/score/:id/download?format=gp5` — re-encodes the loaded score and
  streams it as a file download

### 8.3 Analysis JSON export

- [ ] Download buttons in each analysis panel export the current JSON payload
  (repeats, form, fingering) as `.json`

### 8.4 SVG export

- [ ] alphaTab exposes `api.renderer` events with SVG strings per page/line
- [ ] Capture them and offer a "Save as SVG" download (per page or whole score)
- [ ] Alternative: `GET /api/score/:id/svg?measures=1-8&track=<name>` — server-side
  SVG generation via LilyPond or a headless renderer (future / optional)

---

## Part 9 — Polish and Infrastructure

### 9.1 Error handling and feedback

- [ ] Toast notification system (top-right corner) for API errors and successes
- [ ] Loading spinners during file parse and analysis fetch
- [ ] Structured error responses `{ error: string, detail: string }` from all
  API endpoints

### 9.2 Settings panel

- [ ] Configurable defaults: default rendering mode, zoom level, SoundFont path
- [ ] Persisted in `localStorage`
- [ ] Server-side: `--root`, `--port`, `--no-open` flags; optionally read from a
  `score_server.toml` config file

### 9.3 Keyboard shortcuts reference

| Key | Action |
|-----|--------|
| `Space` | Play / Pause |
| `Escape` | Stop |
| `g` | Jump to measure |
| `[` / `]` | Previous / Next section |
| `+` / `-` | Zoom in / out |
| `t` | Toggle tab / notation mode |
| `f` | Toggle fingering overlay |
| `r` | Toggle repeats overlay |

### 9.4 Build and distribution

- [ ] `just build-web` recipe: `vite build` then `cargo build -p web_server --release`
- [ ] `rust-embed` feature gate: `--features embed` bundles `frontend/dist/` into
  the binary; otherwise reads from disk (dev mode)
- [ ] Single binary: `score_server --open song.gp5` opens the file directly on launch
- [ ] Optional: `cargo-bundle` or `cargo-deb` packaging
