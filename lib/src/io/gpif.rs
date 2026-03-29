use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Gpif {
    /// GP7 uses "GPVersion", GP6 uses "GPRevision"
    #[serde(rename = "GPVersion", default)]
    pub version: Option<String>,
    #[serde(rename = "GPRevision", default)]
    pub revision: Option<String>,
    #[serde(rename = "Score")]
    pub score: Score,
    #[serde(rename = "MasterTrack")]
    pub master_track: MasterTrack,
    #[serde(rename = "Tracks")]
    pub tracks: TracksWrapper,
    #[serde(rename = "MasterBars")]
    pub master_bars: MasterBarsWrapper,
    #[serde(rename = "Bars")]
    pub bars: BarsWrapper,
    #[serde(rename = "Voices")]
    pub voices: VoicesWrapper,
    #[serde(rename = "Beats")]
    pub beats: BeatsWrapper,
    #[serde(rename = "Notes")]
    pub notes: NotesWrapper,
    #[serde(rename = "Rhythms")]
    pub rhythms: RhythmsWrapper,
}

// ---------------------------------------------------------------------------
// Score metadata
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Score {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub sub_title: String,
    #[serde(default)]
    pub artist: String,
    #[serde(default)]
    pub album: String,
    #[serde(rename = "Words", default)]
    pub words: String,
    #[serde(rename = "Music", default)]
    pub music: String,
    #[serde(default)]
    pub copyright: String,
    #[serde(default)]
    pub tabber: String,
    #[serde(default)]
    pub instructions: String,
    #[serde(default)]
    pub notices: String,
}

// ---------------------------------------------------------------------------
// MasterTrack (tempo automations)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MasterTrack {
    #[serde(rename = "Tracks", default)]
    pub tracks_count: String,
    #[serde(rename = "Automations", default)]
    pub automations: Option<AutomationsWrapper>,
}

#[derive(Debug, Deserialize)]
pub struct AutomationsWrapper {
    #[serde(rename = "Automation", default)]
    pub automations: Vec<Automation>,
}

#[derive(Debug, Deserialize)]
pub struct Automation {
    #[serde(rename = "Type", default)]
    pub automation_type: String,
    #[serde(rename = "Value", default)]
    pub value: String,
    #[serde(rename = "Bar", default)]
    pub bar: i32,
    #[serde(rename = "Position", default)]
    pub position: i32,
}

// ---------------------------------------------------------------------------
// Tracks
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TracksWrapper {
    #[serde(rename = "Track", default)]
    pub tracks: Vec<Track>,
}

#[derive(Debug, Deserialize)]
pub struct Track {
    #[serde(rename = "@id", default)]
    pub id: i32,
    #[serde(rename = "Name", default)]
    pub name: String,
    #[serde(rename = "ShortName", default)]
    pub short_name: String,
    #[serde(rename = "Color", default)]
    pub color: Option<String>,
    /// GP6: track-level properties (Tuning, DiagramCollection, etc.)
    #[serde(rename = "Properties", default)]
    pub properties: Option<TrackPropertiesWrapper>,
    /// GP7: staves with per-staff properties
    #[serde(rename = "Staves", default)]
    pub staves: Option<StavesWrapper>,
    #[serde(rename = "GeneralMidi", default)]
    pub general_midi: Option<GeneralMidi>,
    #[serde(rename = "Transpose", default)]
    pub transpose: Option<Transpose>,
}

#[derive(Debug, Deserialize)]
pub struct TrackPropertiesWrapper {
    #[serde(rename = "Property", default)]
    pub properties: Vec<Property>,
}

#[derive(Debug, Deserialize)]
pub struct StavesWrapper {
    #[serde(rename = "Staff", default)]
    pub staves: Vec<Staff>,
}

#[derive(Debug, Deserialize)]
pub struct Staff {
    #[serde(rename = "Properties", default)]
    pub properties: Option<StaffPropertiesWrapper>,
}

#[derive(Debug, Deserialize)]
pub struct StaffPropertiesWrapper {
    #[serde(rename = "Property", default)]
    pub properties: Vec<Property>,
}

#[derive(Debug, Deserialize)]
pub struct GeneralMidi {
    #[serde(rename = "Program", default)]
    pub program: Option<i32>,
    #[serde(rename = "Port", default)]
    pub port: Option<i32>,
    #[serde(rename = "PrimaryChannel", default)]
    pub primary_channel: Option<i32>,
    #[serde(rename = "SecondaryChannel", default)]
    pub secondary_channel: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct Transpose {
    #[serde(rename = "Chromatic", default)]
    pub chromatic: Option<i32>,
    #[serde(rename = "Octave", default)]
    pub octave: Option<i32>,
}

// ---------------------------------------------------------------------------
// MasterBars (measure headers)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MasterBarsWrapper {
    #[serde(rename = "MasterBar", default)]
    pub master_bars: Vec<MasterBar>,
}

#[derive(Debug, Deserialize)]
pub struct MasterBar {
    #[serde(rename = "Key", default)]
    pub key: Option<Key>,
    #[serde(rename = "Time", default)]
    pub time: String,
    #[serde(rename = "Bars", default)]
    pub bars: String,
    #[serde(rename = "Repeat", default)]
    pub repeat: Option<Repeat>,
    #[serde(rename = "AlternateEndings", default)]
    pub alternate_endings: Option<String>,
    #[serde(rename = "DoubleBar", default)]
    pub double_bar: Option<String>,
    #[serde(rename = "Section", default)]
    pub section: Option<Section>,
    #[serde(rename = "Fermatas", default)]
    pub fermatas: Option<FermatasWrapper>,
    #[serde(rename = "FreeTime", default)]
    pub free_time: Option<String>,
    #[serde(rename = "Directions", default)]
    pub directions: Option<DirectionsWrapper>,
}

#[derive(Debug, Deserialize)]
pub struct Key {
    #[serde(rename = "AccidentalCount", default)]
    pub accidental_count: i32,
    #[serde(rename = "Mode", default)]
    pub mode: String,
}

impl Default for Key {
    fn default() -> Self {
        Key { accidental_count: 0, mode: "Major".to_string() }
    }
}

#[derive(Debug, Deserialize)]
pub struct Repeat {
    #[serde(rename = "@start", default)]
    pub start: String,
    #[serde(rename = "@end", default)]
    pub end: String,
    #[serde(rename = "@count", default)]
    pub count: i32,
}

#[derive(Debug, Deserialize)]
pub struct Section {
    #[serde(rename = "Letter", default)]
    pub letter: Option<String>,
    #[serde(rename = "Text", default)]
    pub text: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DirectionsWrapper {
    #[serde(rename = "Target", default)]
    pub target: Option<String>,
    #[serde(rename = "Jump", default)]
    pub jump: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FermatasWrapper {
    #[serde(rename = "Fermata", default)]
    pub fermatas: Vec<Fermata>,
}

#[derive(Debug, Deserialize)]
pub struct Fermata {
    #[serde(rename = "Type", default)]
    pub fermata_type: Option<String>,
    #[serde(rename = "Offset", default)]
    pub offset: Option<String>,
}

// ---------------------------------------------------------------------------
// Bars (per-track measures)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BarsWrapper {
    #[serde(rename = "Bar", default)]
    pub bars: Vec<Bar>,
}

#[derive(Debug, Deserialize)]
pub struct Bar {
    #[serde(rename = "@id", default)]
    pub id: i32,
    #[serde(rename = "Voices", default)]
    pub voices: String,
    #[serde(rename = "Clef", default)]
    pub clef: Option<String>,
    #[serde(rename = "SimileMark", default)]
    pub simile_mark: Option<String>,
}

// ---------------------------------------------------------------------------
// Voices
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct VoicesWrapper {
    #[serde(rename = "Voice", default)]
    pub voices: Vec<Voice>,
}

#[derive(Debug, Deserialize)]
pub struct Voice {
    #[serde(rename = "@id", default)]
    pub id: i32,
    #[serde(rename = "Beats", default)]
    pub beats: String,
}

// ---------------------------------------------------------------------------
// Beats
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BeatsWrapper {
    #[serde(rename = "Beat", default)]
    pub beats: Vec<Beat>,
}

#[derive(Debug, Deserialize)]
pub struct Beat {
    #[serde(rename = "@id", default)]
    pub id: i32,
    #[serde(rename = "Notes", default)]
    pub notes: Option<String>,
    #[serde(rename = "Rhythm", default)]
    pub rhythm: Option<RhythmRef>,
    #[serde(rename = "Dynamic", default)]
    pub dynamic: Option<String>,
    #[serde(rename = "GraceNotes", default)]
    pub grace_notes: Option<String>,
    /// Note: "Fadding" is a typo in the upstream GP6 XML format (should be "Fading").
    #[serde(rename = "Fadding", default)]
    pub fadding: Option<String>,
    #[serde(rename = "Tremolo", default)]
    pub tremolo: Option<String>,
    #[serde(rename = "Wah", default)]
    pub wah: Option<String>,
    #[serde(rename = "FreeText", default)]
    pub free_text: Option<String>,
    #[serde(rename = "Properties", default)]
    pub properties: Option<BeatPropertiesWrapper>,
}

#[derive(Debug, Deserialize)]
pub struct BeatPropertiesWrapper {
    #[serde(rename = "Property", default)]
    pub properties: Vec<BeatProperty>,
}

#[derive(Debug, Deserialize)]
pub struct BeatProperty {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "Direction", default)]
    pub direction: Option<String>,
    #[serde(rename = "Enable", default)]
    pub enable: Option<EnableTag>,
    #[serde(rename = "Float", default)]
    pub float: Option<f64>,
    #[serde(rename = "Flags", default)]
    pub flags: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct RhythmRef {
    #[serde(rename = "@ref", default)]
    pub r#ref: i32,
}

// ---------------------------------------------------------------------------
// Notes
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct NotesWrapper {
    #[serde(rename = "Note", default)]
    pub notes: Vec<Note>,
}

#[derive(Debug, Deserialize)]
pub struct Note {
    #[serde(rename = "@id", default)]
    pub id: i32,
    #[serde(rename = "Properties")]
    pub properties: NoteProperties,
    #[serde(rename = "Tie", default)]
    pub tie: Option<TieInfo>,
    #[serde(rename = "Vibrato", default)]
    pub vibrato: Option<String>,
    #[serde(rename = "LetRing", default)]
    pub let_ring: Option<EnableTag>,
    #[serde(rename = "AntiAccent", default)]
    pub anti_accent: Option<String>,
    #[serde(rename = "Accent", default)]
    pub accent: Option<i32>,
    #[serde(rename = "Trill", default)]
    pub trill: Option<i32>,
    #[serde(rename = "Ornament", default)]
    pub ornament: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TieInfo {
    #[serde(rename = "@origin", default)]
    pub origin: String,
    #[serde(rename = "@destination", default)]
    pub destination: String,
}

/// An empty self-closing tag used as a presence flag (e.g., `<Enable />`, `<LetRing />`).
#[derive(Debug, Deserialize)]
pub struct EnableTag;

#[derive(Debug, Deserialize)]
pub struct NoteProperties {
    #[serde(rename = "Property", default)]
    pub properties: Vec<Property>,
}

#[derive(Debug, Deserialize)]
pub struct Property {
    #[serde(rename = "@name", default)]
    pub name: String,
    // Value sub-elements — each property uses at most one of these
    #[serde(rename = "Fret", default)]
    pub fret: Option<i32>,
    #[serde(rename = "String", default)]
    pub string: Option<i32>,
    #[serde(rename = "Pitch", default)]
    pub pitch: Option<Pitch>,
    #[serde(rename = "Number", default)]
    pub number: Option<i32>,
    #[serde(rename = "Enable", default)]
    pub enable: Option<EnableTag>,
    #[serde(rename = "Float", default)]
    pub float: Option<f64>,
    #[serde(rename = "Flags", default)]
    pub flags: Option<i32>,
    #[serde(rename = "HFret", default)]
    pub hfret: Option<f64>,
    #[serde(rename = "HType", default)]
    pub htype: Option<String>,
    #[serde(rename = "Pitches", default)]
    pub pitches: Option<String>,
    #[serde(rename = "Direction", default)]
    pub direction: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Pitch {
    #[serde(rename = "Step", default)]
    pub step: String,
    #[serde(rename = "Octave", default)]
    pub octave: i32,
    #[serde(rename = "Accidental", default)]
    pub accidental: Option<String>,
}

// ---------------------------------------------------------------------------
// Rhythms
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct RhythmsWrapper {
    #[serde(rename = "Rhythm", default)]
    pub rhythms: Vec<Rhythm>,
}

#[derive(Debug, Deserialize)]
pub struct Rhythm {
    #[serde(rename = "@id", default)]
    pub id: i32,
    #[serde(rename = "NoteValue", default)]
    pub note_value: String,
    #[serde(rename = "AugmentationDot", default)]
    pub augmentation_dot: Option<AugmentationDot>,
    #[serde(rename = "PrimaryTuplet", default)]
    pub primary_tuplet: Option<PrimaryTuplet>,
}

#[derive(Debug, Deserialize)]
pub struct AugmentationDot {
    #[serde(rename = "@count", default)]
    pub count: i32,
}

#[derive(Debug, Deserialize)]
pub struct PrimaryTuplet {
    #[serde(rename = "@num", default)]
    pub num: i32,
    #[serde(rename = "@den", default)]
    pub den: i32,
}
