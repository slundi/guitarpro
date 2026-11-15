//! Global measure timeline: tempo, time/key signatures, markers, navigation (repeats, jumps).

use serde::{Deserialize, Serialize};

use crate::model::{
    endings::ending_mask_numbers,
    optimized::{
        global::MeasureIndex,
        metadata::{KeySignature, TimeSignature},
    },
};

/// One entry per measure, shared across all tracks.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MeasureDef {
    pub index: MeasureIndex,
    pub tempo: Option<f32>, // BPM; None = unchanged from previous
    pub time_signature: Option<TimeSignature>, // None = unchanged
    pub key_signature: Option<KeySignature>, // None = unchanged
    pub marker: Option<Marker>,
    pub navigation: Vec<NavigationEvent>,
    pub tick_resolution: u16, // ticks per quarter note (e.g. 960)
    pub duration_ticks: u32,  // derived: time_sig * resolution
    /// Left-edge barline style override. `None` = renderer default (usually nothing).
    pub barline_left: Option<Barline>,
    /// Right-edge barline style override. `None` = renderer default (single thin line).
    pub barline_right: Option<Barline>,
    /// GP-format beam grouping for the time signature (4 bytes, always written when
    /// time_signature is Some). `None` = use default beaming based on numerator.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_beams: Option<[u8; 4]>,
    /// GP7/GPX fermatas at this measure (type, offset) pairs, e.g. ("Medium", "1/1").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gp_fermatas: Vec<(String, String)>,
    /// GP7/GPX free-time bar flag.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub gp_free_time: bool,
}

impl MeasureDef {
    /// Whether the measure holds a field only the GP import fills (beams,
    /// at least with the first time signature, fermatas, free time, marker
    /// colors). See [`Score::is_gp_sourced`](crate::model::optimized::global::Score::is_gp_sourced).
    pub(crate) fn has_gp_fields(&self) -> bool {
        self.gp_beams.is_some()
            || !self.gp_fermatas.is_empty()
            || self.gp_free_time
            || self
                .marker
                .as_ref()
                .is_some_and(|marker| marker.gp_color.is_some())
    }

    /// Whether its navigation has an event of `kind`.
    pub fn has_navigation(&self, kind: JumpKind) -> bool {
        self.navigation.iter().any(|event| event.kind == kind)
    }
}

// ---------------------------------------------------------------------------
// Barline style and volta endings
// ---------------------------------------------------------------------------

/// A barline with optional visual style and volta-ending bracket.
///
/// Barlines are global — the same style appears on all staves for that measure.
/// Repeat barline *logic* (jump targets) is handled by [`NavigationEvent`];
/// this type covers purely visual overrides and volta bracket display.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Barline {
    pub style: BarlineStyle,
    /// Volta bracket attached to this barline (first/second/nth ending).
    pub ending: Option<Ending>,
}

/// Visual style of a barline.
#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum BarlineStyle {
    Regular, // single thin line (default)
    Dotted,
    Dashed,
    Heavy,      // single thick line
    LightLight, // double thin (section boundary)
    LightHeavy, // thin + thick (final barline)
    HeavyLight, // thick + thin (start of final section)
    HeavyHeavy, // double thick
    Tick,       // short tick mark (jazz notation)
    Short,      // short single line (between staves only)
    None,       // invisible barline (no line drawn)
}

/// A first/second/nth ending volta bracket.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct Ending {
    /// Ending numbers covered by this bracket (e.g. `[1]`, `[1, 2]`, `[3]`).
    pub numbers: Vec<u8>,
    /// Display text override (e.g. `"1."`, `"2.-3."`). `None` = derive from `numbers`.
    pub text: Option<String>,
    pub kind: EndingKind,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum EndingKind {
    /// Bracket starts here (open hook on the left).
    Start,
    /// Bracket ends here with a closing hook on the right.
    Stop,
    /// Bracket ends here without a closing hook (continues past the barline).
    Discontinue,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Marker {
    pub label: String,
    pub kind: MarkerKind,
    /// GP-specific marker color (0xRRGGBB). `None` = use renderer default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gp_color: Option<u32>,
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum MarkerKind {
    Intro,
    Verse,
    PreChorus,
    Chorus,
    Bridge,
    Outro,
    Solo,
    Break,
    Custom,
}

/// Structural navigation event (repeat barlines, jumps). Applied globally to
/// all tracks. Use [`Arrangement::resolve`] to flatten into a playback order.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NavigationEvent {
    pub measure_index: MeasureIndex,
    pub kind: JumpKind,
    pub repeat_count: Option<u8>, // total passes for RepeatClose (e.g. 2 = play twice)
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum JumpKind {
    RepeatOpen,     // |:
    RepeatClose,    // :|
    Segno,          // $
    Coda,           // ⊕
    DaCapo,         // D.C.
    DalSegno,       // D.S.
    DaCapoAlCoda,   // D.C. al Coda
    DalSegnoAlCoda, // D.S. al Coda
    DaCapoAlFine,   // D.C. al Fine
    DalSegnoAlFine, // D.S. al Fine
    Fine,
    ToCoda, // To Coda
}

/// One volta bracket: measures `first..=last`, played on the passes in
/// `numbers`.
#[derive(Debug, Clone, PartialEq)]
pub struct EndingSpan<'a> {
    pub first: usize,
    pub last: usize,
    pub numbers: &'a [u8],
    /// Display text (`"1."`); `None` = derive from `numbers`.
    pub text: Option<&'a str>,
    /// A closing hook at the end (`Stop`), or none (`Discontinue`).
    pub closed: bool,
}

/// The volta brackets the barlines' [`Ending`]s describe: each runs from
/// the `Start` on a measure's left barline through the `Stop`/`Discontinue`
/// on a right barline (one without a start covers just its own measure).
/// An ending left without a stop ends at the next repeat barline: through a
/// backward repeat (closed), or before a forward one or another start
/// (open). One with a stop (for its own passes) runs through the repeats
/// before it, which are nested in it. Endings without numbers are left out.
pub fn ending_spans(timeline: &[MeasureDef]) -> Vec<EndingSpan<'_>> {
    fn ending(barline: &Option<Barline>) -> Option<&Ending> {
        barline.as_ref().and_then(|b| b.ending.as_ref())
    }
    fn close<'a>(
        spans: &mut Vec<EndingSpan<'a>>,
        open: &mut Option<(usize, &'a Ending)>,
        last: usize,
        closed: bool,
    ) {
        if let Some((first, start)) = open.take()
            && !start.numbers.is_empty()
        {
            spans.push(EndingSpan {
                first,
                last,
                numbers: &start.numbers,
                text: start.text.as_deref(),
                closed,
            });
        }
    }
    fn start_at(measure: &MeasureDef) -> Option<&Ending> {
        ending(&measure.barline_left).filter(|e| e.kind == EndingKind::Start)
    }
    fn end_at(measure: &MeasureDef) -> Option<&Ending> {
        ending(&measure.barline_right).filter(|e| e.kind != EndingKind::Start)
    }
    // Per position: the first end at or after it, before another ending
    // starts.
    let mut next_end: Vec<Option<&Ending>> = vec![None; timeline.len() + 1];
    for (position, measure) in timeline.iter().enumerate().rev() {
        next_end[position] = if start_at(measure).is_some() {
            None
        } else {
            end_at(measure).or(next_end[position + 1])
        };
    }
    // Whether the open ending has its stop at or after `position`: an end
    // for other passes is one of its own, not the open ending's.
    let stops_later = |open: Option<(usize, &Ending)>, position: usize| {
        next_end[position].is_some_and(|end| {
            open.is_none_or(|(_, start)| end.numbers.is_empty() || end.numbers == start.numbers)
        })
    };
    let mut spans = Vec::new();
    let mut open: Option<(usize, &Ending)> = None;
    for (position, measure) in timeline.iter().enumerate() {
        let start = start_at(measure);
        // Its own start closes the open ending anyway.
        let stops = stops_later(open, position);
        if (measure.has_navigation(JumpKind::RepeatOpen) && !stops) || start.is_some() {
            close(&mut spans, &mut open, position.saturating_sub(1), false);
        }
        if let Some(start) = start {
            open = Some((position, start));
        }
        match end_at(measure) {
            // One without a start covers just its own measure.
            Some(end) => {
                // An end for other passes is an ending of its own: the open
                // one stops before it.
                if open.is_some_and(|(first, start)| {
                    first < position && !end.numbers.is_empty() && end.numbers != start.numbers
                }) {
                    close(&mut spans, &mut open, position - 1, false);
                }
                open.get_or_insert((position, end));
                close(
                    &mut spans,
                    &mut open,
                    position,
                    end.kind == EndingKind::Stop,
                );
            }
            None if measure.has_navigation(JumpKind::RepeatClose)
                && !stops_later(open, position + 1) =>
            {
                close(&mut spans, &mut open, position, true)
            }
            None => {}
        }
    }
    close(
        &mut spans,
        &mut open,
        timeline.len().saturating_sub(1),
        false,
    );
    spans
}

/// The passes each measure is played on, from its [`ending_spans`]; `None`
/// for a measure outside endings.
pub fn ending_numbers(timeline: &[MeasureDef]) -> Vec<Option<&[u8]>> {
    spans_numbers(timeline.len(), &ending_spans(timeline))
}

/// Per measure of `len`: the passes of the span it's in, if any.
pub(crate) fn spans_numbers<'a>(len: usize, spans: &[EndingSpan<'a>]) -> Vec<Option<&'a [u8]>> {
    let mut numbers = vec![None; len];
    for span in spans {
        numbers[span.first..=span.last].fill(Some(span.numbers));
    }
    numbers
}

/// Marks measures `first..=last` as one volta bracket over `numbers`: a
/// `Start` on the first's left barline (with `text`), a `Stop` (`closed`)
/// or `Discontinue` on the last's right one. Barline styles are kept.
pub fn set_ending(
    timeline: &mut [MeasureDef],
    first: usize,
    last: usize,
    numbers: &[u8],
    text: Option<String>,
    closed: bool,
) {
    if first > last || last >= timeline.len() {
        return;
    }
    let mark = |barline: &mut Option<Barline>, kind, text| {
        barline
            .get_or_insert(Barline {
                style: BarlineStyle::Regular,
                ending: None,
            })
            .ending = Some(Ending {
            numbers: numbers.to_vec(),
            text,
            kind,
        });
    };
    mark(&mut timeline[first].barline_left, EndingKind::Start, text);
    let kind = if closed {
        EndingKind::Stop
    } else {
        EndingKind::Discontinue
    };
    mark(&mut timeline[last].barline_right, kind, None);
}

/// Marks per-measure endings (the passes each measure is played on, empty
/// outside endings, as GP keeps them) as volta brackets: consecutive
/// measures with the same passes form one, closed when it ends at a repeat
/// (`closes`), open otherwise.
pub fn set_endings(
    timeline: &mut [MeasureDef],
    passes: &[Vec<u8>],
    closes: impl Fn(usize) -> bool,
) {
    let numbers = |index: usize| passes.get(index).map_or(&[][..], Vec::as_slice);
    let mut first = 0;
    while first < timeline.len() {
        if numbers(first).is_empty() {
            first += 1;
            continue;
        }
        let mut last = first;
        while last + 1 < timeline.len() && !closes(last) && numbers(last + 1) == numbers(first) {
            last += 1;
        }
        set_ending(timeline, first, last, numbers(first), None, closes(last));
        first = last + 1;
    }
}

// ---------------------------------------------------------------------------
// Serialized form: scores saved before endings were barline `Ending`s
// ---------------------------------------------------------------------------

/// [`MeasureDef`] as serialized, by either form of its endings: its
/// navigation read with the older form's fields, the rest as it is. Its
/// fields mirror [`MeasureDef`]'s (not flattened in, which would buffer
/// each measure); [`SerializedMeasureDef::into_measure`] names them all.
#[derive(Deserialize)]
pub(crate) struct SerializedMeasureDef {
    index: MeasureIndex,
    tempo: Option<f32>,
    time_signature: Option<TimeSignature>,
    key_signature: Option<KeySignature>,
    marker: Option<Marker>,
    #[serde(default)]
    navigation: Vec<SerializedNavigationEvent>,
    tick_resolution: u16,
    duration_ticks: u32,
    barline_left: Option<Barline>,
    barline_right: Option<Barline>,
    #[serde(default)]
    gp_beams: Option<[u8; 4]>,
    #[serde(default)]
    gp_fermatas: Vec<(String, String)>,
    #[serde(default)]
    gp_free_time: bool,
}

impl SerializedMeasureDef {
    /// The measure, its navigation still to fill in from the serialized
    /// one given with it.
    fn into_measure(self) -> (MeasureDef, Vec<SerializedNavigationEvent>) {
        let SerializedMeasureDef {
            index,
            tempo,
            time_signature,
            key_signature,
            marker,
            navigation: serialized,
            tick_resolution,
            duration_ticks,
            barline_left,
            barline_right,
            gp_beams,
            gp_fermatas,
            gp_free_time,
        } = self;
        let measure = MeasureDef {
            index,
            tempo,
            time_signature,
            key_signature,
            marker,
            navigation: Vec::new(),
            tick_resolution,
            duration_ticks,
            barline_left,
            barline_right,
            gp_beams,
            gp_fermatas,
            gp_free_time,
        };
        (measure, serialized)
    }
}

/// [`NavigationEvent`] as serialized. Scores saved before endings were
/// barline [`Ending`]s give a measure's on a `RepeatClose` (`volta`), one
/// without a `repeat_count` standing for just the ending: from a GP file,
/// as GP's pass bitmask; from a MusicXML one, as the ending's first number
/// on its last measure, marked `volta_last`.
#[derive(Deserialize)]
pub(crate) struct SerializedNavigationEvent {
    measure_index: MeasureIndex,
    kind: JumpKind,
    repeat_count: Option<u8>,
    #[serde(default)]
    volta: Option<u8>,
    #[serde(default)]
    volta_last: bool,
}

impl From<SerializedNavigationEvent> for NavigationEvent {
    fn from(event: SerializedNavigationEvent) -> Self {
        NavigationEvent {
            measure_index: event.measure_index,
            kind: event.kind,
            repeat_count: event.repeat_count,
        }
    }
}

/// A [`Score`](crate::model::optimized::global::Score)'s timeline as
/// deserialized: in the current form, or with the `volta`s of a score
/// saved before endings were barline [`Ending`]s, still to turn into those
/// ([`SerializedTimeline::into_timeline`]).
pub(crate) enum SerializedTimeline {
    Current(Vec<MeasureDef>),
    Older(Vec<SerializedMeasureDef>),
}

/// Deserializes a [`Score`](crate::model::optimized::global::Score)'s
/// timeline (see [`SerializedTimeline`]).
pub(crate) fn deserialize_timeline<'de, D>(deserializer: D) -> Result<SerializedTimeline, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // The older form has fields the current one lacks (`volta`s), which
    // only a self-describing format can default by name: a binary one (as
    // `.msor`'s Postcard) reads just the current form.
    if !deserializer.is_human_readable() {
        return Vec::<MeasureDef>::deserialize(deserializer).map(SerializedTimeline::Current);
    }
    let serialized = Vec::<SerializedMeasureDef>::deserialize(deserializer)?;
    // A score in the current form (no `volta`s) has nothing to migrate.
    if serialized
        .iter()
        .flat_map(|measure| &measure.navigation)
        .all(|event| event.volta.is_none())
    {
        return Ok(SerializedTimeline::Current(
            serialized
                .into_iter()
                .map(|serialized| {
                    let (mut measure, navigation) = serialized.into_measure();
                    measure.navigation = navigation.into_iter().map(Into::into).collect();
                    measure
                })
                .collect(),
        ));
    }
    Ok(SerializedTimeline::Older(serialized))
}

/// [`deserialize_timeline`] of a timeline without its score: an older
/// one's voltas are read as GP's only by its measures' GP fields.
#[cfg(test)]
pub(crate) fn deserialize_timeline_alone<'de, D>(
    deserializer: D,
) -> Result<Vec<MeasureDef>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(deserialize_timeline(deserializer)?
        .into_timeline(|timeline| timeline.iter().any(MeasureDef::has_gp_fields)))
}

impl SerializedTimeline {
    /// The timeline, an older one's `volta`s turned into barline endings
    /// on the measures without one. `gp_sourced` tells, from the measures,
    /// whether the score is from a GP file
    /// ([`Score::is_gp_sourced`](crate::model::optimized::global::Score::is_gp_sourced)):
    /// its voltas are GP pass bitmasks, another's ending numbers.
    pub(crate) fn into_timeline(
        self,
        gp_sourced: impl FnOnce(&[MeasureDef]) -> bool,
    ) -> Vec<MeasureDef> {
        match self {
            SerializedTimeline::Current(timeline) => timeline,
            SerializedTimeline::Older(serialized) => migrate_voltas(serialized, gp_sourced),
        }
    }
}

/// [`SerializedTimeline::into_timeline`] of an older timeline.
fn migrate_voltas(
    serialized: Vec<SerializedMeasureDef>,
    gp_sourced: impl FnOnce(&[MeasureDef]) -> bool,
) -> Vec<MeasureDef> {
    let (mut timeline, navigations): (Vec<MeasureDef>, Vec<Vec<SerializedNavigationEvent>>) =
        serialized
            .into_iter()
            .map(SerializedMeasureDef::into_measure)
            .unzip();
    // Only the MusicXML import marked a last ending (`volta_last`), and
    // only the GP import kept GP's own fields: a score with the one or
    // without any of the others is from MusicXML, whose voltas are all
    // ending numbers (one tagged on a repeat's close ahead of its stop
    // too). The others are GP pass bitmasks.
    let from_musicxml =
        navigations.iter().flatten().any(|event| event.volta_last) || !gp_sourced(&timeline);
    let mut passes = Vec::with_capacity(timeline.len());
    // The numbers of the last ending start a MusicXML score kept on a
    // barline (only styled ones were): its `volta_last` is where it stops.
    let mut kept_start: Option<Vec<u8>> = None;
    for (measure, navigation) in timeline.iter_mut().zip(navigations) {
        let volta = navigation.iter().find_map(|event| event.volta);
        if let Some(ending) = measure
            .barline_left
            .as_ref()
            .and_then(|barline| barline.ending.as_ref())
            .filter(|ending| ending.kind == EndingKind::Start)
        {
            kept_start = Some(ending.numbers.clone());
        }
        let last = navigation.iter().find(|event| event.volta_last);
        if let Some(number) = last.and_then(|event| event.volta)
            && from_musicxml
            && measure
                .barline_right
                .as_ref()
                .is_none_or(|barline| barline.ending.is_none())
        {
            // Its stop, for the start's numbers when it's that ending's
            // (whose first number the volta is).
            let numbers = kept_start
                .take()
                .filter(|numbers| numbers.first() == Some(&number))
                .unwrap_or_else(|| vec![number]);
            measure
                .barline_right
                .get_or_insert(Barline {
                    style: BarlineStyle::Regular,
                    ending: None,
                })
                .ending = Some(Ending {
                numbers,
                text: None,
                kind: EndingKind::Stop,
            });
        } else if measure
            .barline_right
            .as_ref()
            .is_some_and(|barline| barline.ending.is_some())
        {
            kept_start = None;
        }
        passes.push(match volta {
            None => Vec::new(),
            Some(number) if from_musicxml => vec![number],
            Some(mask) => ending_mask_numbers(mask),
        });
        measure.navigation = navigation
            .into_iter()
            // A close standing for just an ending isn't a repeat.
            .filter(|event| {
                event.kind != JumpKind::RepeatClose
                    || event.repeat_count.is_some()
                    || event.volta.is_none()
            })
            .map(Into::into)
            .collect();
    }
    let closes: Vec<bool> = timeline
        .iter()
        .map(|measure| measure.has_navigation(JumpKind::RepeatClose))
        .collect();
    // The barline endings the score kept stay as they are.
    for (pass, numbers) in passes.iter_mut().zip(ending_numbers(&timeline)) {
        if numbers.is_some() {
            pass.clear();
        }
    }
    set_endings(&mut timeline, &passes, |index| closes[index]);
    timeline
}

/// Per measure: the coda sign after it, where an al Coda pass branches to
/// from a To Coda there. One pass, from the end.
fn next_codas(timeline: &[MeasureDef]) -> Vec<Option<usize>> {
    let mut next_codas = vec![None; timeline.len()];
    let mut next = None;
    for (pos, measure) in timeline.iter().enumerate().rev() {
        next_codas[pos] = next;
        if measure.has_navigation(JumpKind::Coda) {
            next = Some(pos);
        }
    }
    next_codas
}

/// Per measure: where its coda sign branches to on an al Coda pass, the
/// next coda sign, unless a To Coda lies before that (which branches
/// instead). `None` for the coda's own sign, with none after it, and for a
/// measure without one. One pass, from the end.
pub(crate) fn coda_branches(timeline: &[MeasureDef]) -> Vec<Option<usize>> {
    let mut branches = vec![None; timeline.len()];
    // The next coda sign after the measure, and whether a To Coda lies
    // between them.
    let mut next: Option<usize> = None;
    let mut to_coda_between = false;
    for (pos, measure) in timeline.iter().enumerate().rev() {
        if measure.has_navigation(JumpKind::Coda) {
            branches[pos] = next.filter(|_| !to_coda_between);
            next = Some(pos);
            to_coda_between = false;
        } else if measure.has_navigation(JumpKind::ToCoda) {
            to_coda_between = true;
        }
    }
    branches
}

// ---------------------------------------------------------------------------
// Arrangement: resolved playback order
// ---------------------------------------------------------------------------

/// The most steps [`Arrangement::resolve`] takes when long repeats scale
/// its guard up: bounds the played order a crafted score can produce.
const MAX_RESOLVE_STEPS: usize = 1_000_000;

/// Flattened playback order after resolving repeats, voltas, and D.C./D.S. jumps.
///
/// Build with [`Arrangement::resolve`].
pub struct Arrangement {
    /// Sequence of measure indices to play, in playback order.
    /// Repeats, voltas, and jumps are already expanded.
    pub order: Vec<MeasureIndex>,
    /// Per entry of `order`: the pass of the innermost repeat it plays in
    /// (1-based; 1 outside any, the last ending's on a D.C./D.S. return).
    pub passes: Vec<u8>,
    /// Whether it took a D.C./D.S. jump (one it can't follow, as to a
    /// missing segno, isn't taken).
    pub jumped: bool,
    /// Whether it reached a D.S. it can't follow: one in a bar it plays
    /// before any jump (one on a D.C./D.S. return pass isn't taken
    /// anyway), with no segno played before it or in its bar. A jump in a
    /// bar it never plays (as in an ending skipped on every pass) isn't
    /// reached.
    pub unfollowable: bool,
    /// Whether it stopped at the safety guard (malformed input, as a
    /// repeat that never closes): `order` is then cut short.
    pub truncated: bool,
    /// How many bars it plays: `order`'s length, unless it kept only the
    /// first ones ([`Arrangement::resolve_first`]).
    pub played: usize,
    /// Per measure: whether it plays at all (also past the bars `order`
    /// keeps).
    pub reached: Vec<bool>,
}

/// The final pass of a repeat just finished, for the endings after it.
#[derive(Clone, Copy)]
struct VoltaContext {
    pass: u8,
    /// Whether it closed in an ending: the endings after are then its own,
    /// not an enclosing repeat's.
    in_ending: bool,
}

struct RepeatFrame {
    open_pos: usize,
    count: u8, // total passes (from the RepeatClose event)
    pass: u8,  // current pass, 1-indexed
    /// Opened inside an ending: its passes don't choose the endings, the
    /// enclosing repeat's (`outer_pass`) do.
    in_ending: bool,
    outer_pass: u8,
    /// The volta context when it opened, restored when a repeat nested in
    /// an ending ends.
    saved_context: Option<VoltaContext>,
}

/// How a timeline's endings play, worked out once for
/// [`Arrangement::resolve_with`] and the repeats analysis: each measure's
/// ending passes, which repeats are nested in an ending, and each ending
/// group's last ending.
pub(crate) struct EndingPlan<'a> {
    /// Per measure: the passes of the ending it's in.
    pub(crate) endings: Vec<Option<&'a [u8]>>,
    /// Per measure: whether its RepeatClose ends a repeat that opened in
    /// the same ending, nested in it, rather than one the ending is of.
    nested_close: Vec<bool>,
    /// Per measure: whether its RepeatOpen starts a repeat nested in its
    /// ending (closing in it too), not one that runs on past it.
    nested_open: Vec<bool>,
    /// Per measure in endings: the last ending of its group, which a
    /// D.C./D.S. return pass plays; 0 when that one repeats back.
    last_ending: Vec<u8>,
}

impl<'a> EndingPlan<'a> {
    pub(crate) fn new(timeline: &[MeasureDef], spans: &[EndingSpan<'a>]) -> Self {
        let n = timeline.len();
        let endings = spans_numbers(n, spans);
        // Per measure: whether its RepeatClose ends a repeat that opened in
        // the same ending, nested in it, rather than one the ending is of.
        let mut nested_close = vec![false; n];
        // Per measure: whether its RepeatOpen starts a repeat nested in its
        // ending (closing in it too), not one that runs on past it.
        let mut nested_open = vec![false; n];
        let has = |pos: usize, kind| timeline[pos].has_navigation(kind);
        for span in spans {
            let mut opens: Vec<usize> = Vec::new();
            for (pos, nested) in nested_close
                .iter_mut()
                .enumerate()
                .take(span.last + 1)
                .skip(span.first)
            {
                if has(pos, JumpKind::RepeatOpen) {
                    opens.push(pos);
                }
                if has(pos, JumpKind::RepeatClose)
                    && let Some(open) = opens.pop()
                {
                    *nested = true;
                    nested_open[open] = true;
                }
            }
        }
        // Per measure in endings: the last ending of its group, which a
        // D.C./D.S. return pass plays; 0 (none) when that one repeats back,
        // as a lone `1.`: the final pass skips it. A group is a run of
        // adjacent endings, a new one starting where the numbers start
        // over (the next repeat's endings, right after the last one's).
        let mut last_ending = vec![0u8; n];
        let top = |span: &EndingSpan<'_>| span.numbers.iter().copied().max().unwrap_or(0);
        let bottom = |span: &EndingSpan<'_>| span.numbers.iter().copied().min().unwrap_or(0);
        let mut group_start = 0;
        while let Some(first) = spans.get(group_start) {
            let mut last = top(first);
            let mut group_end = group_start + 1;
            while let Some(span) = spans
                .get(group_end)
                .filter(|span| span.first == spans[group_end - 1].last + 1 && bottom(span) > last)
            {
                last = last.max(top(span));
                group_end += 1;
            }
            let group = &spans[group_start..group_end];
            let repeats_back = group.iter().any(|span| {
                span.numbers.contains(&last)
                    && has(span.last, JumpKind::RepeatClose)
                    && !nested_close[span.last]
            });
            if repeats_back {
                last = 0;
            }
            let end = group.last().map_or(first.last, |span| span.last);
            last_ending[first.first..=end].fill(last);
            group_start = group_end;
        }
        EndingPlan {
            endings,
            nested_close,
            nested_open,
            last_ending,
        }
    }

    /// How many passes the repeat closed at `pos` plays: the first, and
    /// one more each time the close sends it back, which a close in an
    /// ending does only on that ending's passes (below
    /// [`EndingPlan::close_passes`]).
    pub(crate) fn played_passes(&self, pos: usize, repeat_count: Option<u8>) -> u8 {
        let count = self.close_passes(pos, repeat_count);
        match self.endings[pos] {
            Some(numbers) if !self.nested_close[pos] => {
                let back = (1..count).filter(|pass| numbers.contains(pass)).count();
                u8::try_from(back + 1).unwrap_or(u8::MAX)
            }
            _ => count,
        }
    }

    /// The pass count the repeat closed at `pos` loops to, `repeat_count`
    /// its close's count: a repeat with endings goes on at least to its
    /// last ending's number (how many it plays: [`EndingPlan::played_passes`]).
    pub(crate) fn close_passes(&self, pos: usize, repeat_count: Option<u8>) -> u8 {
        let count = repeat_count.unwrap_or(2);
        if self.endings[pos].is_some() && !self.nested_close[pos] {
            count.max(self.last_ending[pos])
        } else {
            count
        }
    }
}

impl Arrangement {
    /// Resolve the global timeline into a flat playback sequence.
    ///
    /// Handles:
    /// - `RepeatOpen` / `RepeatClose` with pass counting; a `RepeatClose`
    ///   without an open repeat repeats from the start, or from after the
    ///   last repeat and its endings
    /// - Volta (alternate ending) brackets: the barlines' [`Ending`]s (see
    ///   [`ending_numbers`]), each played on its passes (a repeat with
    ///   endings plays at least as many as its last ending's number); a
    ///   repeat opened inside an ending is nested in it
    /// - `DaCapo`, `DalSegno` (simple return jumps)
    /// - `DaCapoAlFine` / `Fine` (return, stop at Fine marker)
    /// - `DaCapoAlCoda` / `DalSegnoAlCoda` / `Coda` (return, branch at Coda),
    ///   or at an explicit `ToCoda` to the next `Coda`
    ///
    /// Convention: repeats are **not** observed on a D.C. / D.S. return pass,
    /// which goes straight to each repeat's last ending.
    /// A safety guard caps iterations at `timeline.len() * 32` times half
    /// the largest repeat count (at least 512, and at most the larger of
    /// `timeline.len() * 32` and 1,000,000) to handle malformed input (e.g.
    /// a missing `RepeatClose`); [`Arrangement::truncated`] tells when it
    /// stopped there.
    pub fn resolve(timeline: &[MeasureDef]) -> Self {
        Self::resolve_with(timeline, &ending_spans(timeline), usize::MAX)
    }

    /// [`Arrangement::resolve`], keeping only the first `keep` bars in
    /// `order` (and `passes`): [`Arrangement::played`] still counts them
    /// all, without holding a long order in memory.
    pub fn resolve_first(timeline: &[MeasureDef], keep: usize) -> Self {
        Self::resolve_with(timeline, &ending_spans(timeline), keep)
    }

    /// [`Arrangement::resolve_first`], given the timeline's
    /// [`ending_spans`].
    pub(crate) fn resolve_with(
        timeline: &[MeasureDef],
        spans: &[EndingSpan<'_>],
        keep: usize,
    ) -> Self {
        Self::resolve_with_plan(timeline, &EndingPlan::new(timeline, spans), keep)
    }

    /// [`Arrangement::resolve_first`], given how the timeline's endings
    /// play (worked out once for the repeats analysis too).
    pub(crate) fn resolve_with_plan(
        timeline: &[MeasureDef],
        plan: &EndingPlan<'_>,
        keep: usize,
    ) -> Self {
        let n = timeline.len();
        let EndingPlan {
            endings,
            nested_close,
            nested_open,
            last_ending,
        } = plan;
        let has = |pos: usize, kind| timeline[pos].has_navigation(kind);
        let capacity = n.saturating_mul(2).min(keep);
        let mut order: Vec<MeasureIndex> = Vec::with_capacity(capacity);
        let mut passes: Vec<u8> = Vec::with_capacity(capacity);
        let mut pos = 0usize;
        let mut repeat_stack: Vec<RepeatFrame> = Vec::new();
        let mut segno_pos: Option<usize> = None;
        let next_codas = next_codas(timeline);
        let coda_branches = coda_branches(timeline);
        let mut stop_at_fine = false;
        let mut stop_at_coda = false;
        // After a D.C./D.S. jump, repeats are skipped (standard practice).
        let mut in_return_pass = false;
        let mut unfollowable = false;
        // Remembers the final pass count of a just-exited repeat so that
        // the immediately following last-volta bracket is matched correctly,
        // and whether it closed in an ending (so those after are its own).
        let mut volta_context: Option<VoltaContext> = None;
        // Where a repeat closed without an open one starts: the beginning,
        // or after the last repeat (and its endings) ended.
        let mut implicit_open = 0usize;

        // Scaled by the largest repeat count, so a long repeat (MSCZ keeps
        // counts up to 255) plays out in full.
        let max_count = timeline
            .iter()
            .flat_map(|measure| measure.navigation.iter())
            .filter_map(|event| event.repeat_count)
            .max()
            .unwrap_or(2)
            .max(2);
        // Capped, so a crafted score (many measures, nested repeats, high
        // counts) can't make the order huge: at most the larger of the
        // unscaled guard and `MAX_RESOLVE_STEPS`.
        let unscaled = n.saturating_mul(32).max(512);
        let guard_limit = unscaled
            .saturating_mul(usize::from(max_count / 2).max(1))
            .min(unscaled.max(MAX_RESOLVE_STEPS));
        let mut guard = 0usize;
        let mut truncated = false;
        let mut played = 0usize;
        let mut reached = vec![false; n];

        while pos < n {
            guard += 1;
            if guard > guard_limit {
                truncated = true;
                break;
            }

            let measure = &timeline[pos];

            // Effective repeat pass for volta matching: the endings right
            // after a repeat that closed in an ending belong to it, else to
            // the active frame (for one nested in an ending: the enclosing
            // repeat's), or the repeat just finished; 1 outside any.
            let active = repeat_stack
                .last()
                .map(|f| if f.in_ending { f.outer_pass } else { f.pass });
            let effective_pass: u8 = match volta_context {
                Some(context) if context.in_ending => Some(context.pass),
                context => active.or(context.map(|context| context.pass)),
            }
            .unwrap_or(1);
            let in_finished_endings = volta_context.is_some_and(|context| context.in_ending);

            let ending = endings[pos];

            // Clear the post-loop context once we leave the bracketed area.
            if ending.is_none() {
                volta_context = None;
            }

            // Output this measure only when its ending matches the current
            // pass (on a return pass, which skips repeats: the last), or
            // when it's in no ending.
            let pass = if in_return_pass {
                last_ending[pos]
            } else {
                effective_pass
            };
            let should_process = ending.is_none_or(|numbers| numbers.contains(&pass));
            if should_process && played < keep {
                order.push(measure.index);
                // An ending's pass is the one it's matched by; elsewhere, the
                // enclosing repeat's (none on a return pass, which skips them).
                let pass = match ending {
                    Some(_) => pass,
                    None if in_return_pass => 1,
                    None => active.unwrap_or(1),
                };
                passes.push(pass.max(1));
            }
            if should_process {
                played += 1;
                reached[pos] = true;
            }

            let mut next_pos: Option<usize> = None;

            // A repeat's final pass skips its earlier endings, close and all:
            // the repeat ends there, and the endings after match its pass.
            // Once it has ended in an ending, the closes in its later
            // skipped endings are its own, not the enclosing repeat's, as
            // are those of repeats nested in a skipped ending.
            if !should_process
                && !in_return_pass
                && !in_finished_endings
                && !nested_close[pos]
                && has(pos, JumpKind::RepeatClose)
                && let Some(frame) = repeat_stack.last()
                && frame.count != 0
                && frame.pass >= frame.count
            {
                volta_context = Some(VoltaContext {
                    pass: frame.count,
                    in_ending: true,
                });
                repeat_stack.pop();
                implicit_open = pos + 1;
            }

            // A measure in a volta bracket skipped on this pass isn't played:
            // its marks and jumps don't count. Its RepeatClose belongs to a
            // different ending than this pass, and a Fine, To Coda, D.C. or
            // D.S. there mustn't stop or branch from a measure never heard.
            let navigation = if should_process {
                measure.navigation.as_slice()
            } else {
                &[]
            };
            // A segno anywhere in the measure is one a D.S. there goes back
            // to, whatever the order its events came in.
            if navigation.iter().any(|ev| ev.kind == JumpKind::Segno) {
                segno_pos = Some(pos);
            }
            for ev in navigation {
                // A D.S. on a return pass isn't taken anyway.
                if !in_return_pass
                    && segno_pos.is_none()
                    && matches!(
                        ev.kind,
                        JumpKind::DalSegno | JumpKind::DalSegnoAlCoda | JumpKind::DalSegnoAlFine
                    )
                {
                    unfollowable = true;
                }
                match ev.kind {
                    JumpKind::Segno => {
                        segno_pos = Some(pos);
                    }

                    // On the al Coda pass, a coda sign printed where it
                    // branches off jumps to the one at the coda; the coda's
                    // own (with none after) plays on. With a To Coda on
                    // the way to that one, the pass branches there instead.
                    JumpKind::Coda => {
                        if stop_at_coda && let Some(dest) = coda_branches[pos] {
                            stop_at_coda = false;
                            next_pos = Some(dest);
                        }
                    }

                    // An explicit "To Coda": on the al Coda pass, jump to the
                    // coda sign after it.
                    JumpKind::ToCoda => {
                        if stop_at_coda && let Some(dest) = next_codas[pos] {
                            stop_at_coda = false;
                            next_pos = Some(dest);
                        }
                    }

                    JumpKind::Fine => {
                        if stop_at_fine {
                            return Self {
                                order,
                                passes,
                                jumped: in_return_pass,
                                unfollowable,
                                truncated,
                                played,
                                reached,
                            };
                        }
                    }

                    JumpKind::RepeatOpen => {
                        if !in_return_pass && !repeat_stack.iter().any(|f| f.open_pos == pos) {
                            let in_ending = nested_open[pos];
                            repeat_stack.push(RepeatFrame {
                                open_pos: pos,
                                count: 0,
                                pass: 1,
                                in_ending,
                                outer_pass: effective_pass,
                                saved_context: volta_context,
                            });
                            // One starting in an ending and running on past
                            // it (as at a last ending) chooses the endings
                            // after by its own passes.
                            if !in_ending {
                                volta_context = None;
                            }
                        }
                    }

                    JumpKind::RepeatClose => {
                        if !in_return_pass {
                            let nested = nested_close[pos];
                            let count = plan.close_passes(pos, ev.repeat_count);
                            // Closed without an open repeat: implicitly from
                            // the start or the last repeat's end.
                            if repeat_stack.is_empty() {
                                repeat_stack.push(RepeatFrame {
                                    open_pos: implicit_open,
                                    count: 0,
                                    pass: 1,
                                    in_ending: false,
                                    outer_pass: effective_pass,
                                    saved_context: volta_context,
                                });
                            }
                            if let Some(frame) = repeat_stack.last_mut() {
                                if frame.count == 0 {
                                    frame.count = count;
                                }
                                if frame.pass < frame.count {
                                    frame.pass += 1;
                                    next_pos = Some(frame.open_pos);
                                } else if nested {
                                    // Ended inside the ending it opened in:
                                    // back to the enclosing repeat's endings.
                                    volta_context = frame.saved_context;
                                    repeat_stack.pop();
                                } else {
                                    // Save the final pass so the next volta bracket
                                    // (last ending) is matched correctly.
                                    volta_context = Some(VoltaContext {
                                        pass: frame.count,
                                        in_ending: ending.is_some(),
                                    });
                                    repeat_stack.pop();
                                    implicit_open = pos + 1;
                                }
                            }
                        }
                    }

                    // D.C./D.S. jumps execute only on the first (forward) pass.
                    // On the return pass they are silently ignored to avoid infinite loops.
                    JumpKind::DaCapo => {
                        if !in_return_pass {
                            in_return_pass = true;
                            stop_at_fine = false;
                            stop_at_coda = false;
                            repeat_stack.clear();
                            volta_context = None;
                            next_pos = Some(0);
                        }
                    }

                    JumpKind::DalSegno => {
                        if !in_return_pass && let Some(sp) = segno_pos {
                            in_return_pass = true;
                            stop_at_fine = false;
                            stop_at_coda = false;
                            repeat_stack.clear();
                            volta_context = None;
                            next_pos = Some(sp);
                        }
                    }

                    JumpKind::DaCapoAlCoda => {
                        if !in_return_pass {
                            in_return_pass = true;
                            stop_at_fine = false;
                            stop_at_coda = true;
                            repeat_stack.clear();
                            volta_context = None;
                            next_pos = Some(0);
                        }
                    }

                    JumpKind::DalSegnoAlCoda => {
                        if !in_return_pass && let Some(sp) = segno_pos {
                            in_return_pass = true;
                            stop_at_fine = false;
                            stop_at_coda = true;
                            repeat_stack.clear();
                            volta_context = None;
                            next_pos = Some(sp);
                        }
                    }

                    JumpKind::DaCapoAlFine => {
                        if !in_return_pass {
                            in_return_pass = true;
                            stop_at_fine = true;
                            stop_at_coda = false;
                            repeat_stack.clear();
                            volta_context = None;
                            next_pos = Some(0);
                        }
                    }

                    JumpKind::DalSegnoAlFine => {
                        if !in_return_pass && let Some(sp) = segno_pos {
                            in_return_pass = true;
                            stop_at_fine = true;
                            stop_at_coda = false;
                            repeat_stack.clear();
                            volta_context = None;
                            next_pos = Some(sp);
                        }
                    }
                }

                if next_pos.is_some() {
                    break; // only one jump per measure
                }
            }

            // The endings after a finished repeat belong to it: an implicit
            // repeat starts past them.
            if next_pos.is_none()
                && repeat_stack.is_empty()
                && ending.is_some()
                && volta_context.is_some()
            {
                implicit_open = pos + 1;
            }
            pos = next_pos.unwrap_or(pos + 1);
        }

        Self {
            order,
            passes,
            jumped: in_return_pass,
            unfollowable,
            truncated,
            played,
            reached,
        }
    }
}
