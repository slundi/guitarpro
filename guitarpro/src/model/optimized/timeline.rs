//! Global measure timeline: tempo, time/key signatures, markers, navigation (repeats, jumps).

use serde::{Deserialize, Serialize};

use crate::model::optimized::{
    global::MeasureIndex,
    metadata::{KeySignature, TimeSignature},
};

/// One entry per measure, shared across all tracks.
#[derive(Serialize, Deserialize, Debug)]
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
}

// ---------------------------------------------------------------------------
// Barline style and volta endings
// ---------------------------------------------------------------------------

/// A barline with optional visual style and volta-ending bracket.
///
/// Barlines are global — the same style appears on all staves for that measure.
/// Repeat barline *logic* (jump targets) is handled by [`NavigationEvent`];
/// this type covers purely visual overrides and volta bracket display.
#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
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
#[derive(Serialize, Deserialize, Debug)]
pub struct NavigationEvent {
    pub measure_index: MeasureIndex,
    pub kind: JumpKind,
    pub repeat_count: Option<u8>, // total passes for RepeatClose (e.g. 2 = play twice)
    pub volta: Option<u8>,        // 1, 2, 3, … for alternate endings
    pub volta_last: bool,         // true if this is the final volta bracket
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
}

// ---------------------------------------------------------------------------
// Arrangement: resolved playback order
// ---------------------------------------------------------------------------

/// Flattened playback order after resolving repeats, voltas, and D.C./D.S. jumps.
///
/// Build with [`Arrangement::resolve`].
pub struct Arrangement {
    /// Sequence of measure indices to play, in playback order.
    /// Repeats, voltas, and jumps are already expanded.
    pub order: Vec<MeasureIndex>,
}

struct RepeatFrame {
    open_pos: usize,
    count: u8, // total passes (from the RepeatClose event)
    pass: u8,  // current pass, 1-indexed
}

impl Arrangement {
    /// Resolve the global timeline into a flat playback sequence.
    ///
    /// Handles:
    /// - `RepeatOpen` / `RepeatClose` with pass counting
    /// - Volta (alternate ending) brackets via `NavigationEvent::volta`
    /// - `DaCapo`, `DalSegno` (simple return jumps)
    /// - `DaCapoAlFine` / `Fine` (return, stop at Fine marker)
    /// - `DaCapoAlCoda` / `DalSegnoAlCoda` / `Coda` (return, branch at Coda)
    ///
    /// Convention: repeats are **not** observed on a D.C. / D.S. return pass.
    /// A safety guard caps iterations at `timeline.len() * 32` to handle
    /// malformed input (e.g. a missing `RepeatClose`).
    pub fn resolve(timeline: &[MeasureDef]) -> Self {
        let n = timeline.len();
        let mut order: Vec<MeasureIndex> = Vec::with_capacity(n.saturating_mul(2));
        let mut pos = 0usize;
        let mut repeat_stack: Vec<RepeatFrame> = Vec::new();
        let mut segno_pos: Option<usize> = None;
        let mut coda_pos: Option<usize> = None;
        let mut stop_at_fine = false;
        let mut stop_at_coda = false;
        // After a D.C./D.S. jump, repeats are skipped (standard practice).
        let mut in_return_pass = false;
        // Remembers the final pass count of a just-exited repeat so that
        // the immediately following last-volta bracket is matched correctly.
        let mut volta_context: Option<u8> = None;

        let guard_limit = n.saturating_mul(32).max(512);
        let mut guard = 0usize;

        while pos < n {
            guard += 1;
            if guard > guard_limit {
                break;
            }

            let measure = &timeline[pos];

            // Effective repeat pass for volta matching:
            // prefer the active frame, fall back to post-loop context, then 1.
            let effective_pass: u8 = repeat_stack
                .last()
                .map(|f| f.pass)
                .or(volta_context)
                .unwrap_or(1);

            // Collect the volta tag from any event on this measure (if any).
            let volta_tag: Option<u8> = measure.navigation.iter().find_map(|ev| ev.volta);

            // Clear the post-loop context once we leave the bracketed area.
            if volta_tag.is_none() && repeat_stack.is_empty() {
                volta_context = None;
            }

            // Output this measure only when its volta number matches the current pass
            // (or when it carries no volta tag).
            if volta_tag.is_none_or(|v| v == effective_pass) {
                order.push(measure.index);
            }

            let mut next_pos: Option<usize> = None;
            let should_process = volta_tag.is_none_or(|v| v == effective_pass);

            for ev in &measure.navigation {
                // RepeatClose inside a skipped volta bracket must not affect the pass
                // counter — the barline belongs to a different ending than this pass.
                if !should_process && ev.kind == JumpKind::RepeatClose {
                    continue;
                }

                match ev.kind {
                    JumpKind::Segno => {
                        segno_pos = Some(pos);
                    }

                    JumpKind::Coda => {
                        // Record the first Coda marker as the destination.
                        if coda_pos.is_none() {
                            coda_pos = Some(pos);
                        }
                        if stop_at_coda {
                            stop_at_coda = false;
                            if let Some(dest) = coda_pos
                                && dest > pos
                            {
                                // Jump forward to the Coda section.
                                next_pos = Some(dest);
                            }
                            // If dest <= pos we are already in/past the Coda section;
                            // continue forward naturally.
                        }
                    }

                    JumpKind::Fine => {
                        if stop_at_fine {
                            return Self { order };
                        }
                    }

                    JumpKind::RepeatOpen => {
                        if !in_return_pass && !repeat_stack.iter().any(|f| f.open_pos == pos) {
                            repeat_stack.push(RepeatFrame {
                                open_pos: pos,
                                count: 0,
                                pass: 1,
                            });
                        }
                    }

                    JumpKind::RepeatClose => {
                        if !in_return_pass {
                            let count = ev.repeat_count.unwrap_or(2);
                            if let Some(frame) = repeat_stack.last_mut() {
                                if frame.count == 0 {
                                    frame.count = count;
                                }
                                if frame.pass < frame.count {
                                    frame.pass += 1;
                                    next_pos = Some(frame.open_pos);
                                } else {
                                    // Save the final pass so the next volta bracket
                                    // (last ending) is matched correctly.
                                    volta_context = Some(frame.count);
                                    repeat_stack.pop();
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

            pos = next_pos.unwrap_or(pos + 1);
        }

        Self { order }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Helpers ---

    fn measure(idx: u16, events: Vec<NavigationEvent>) -> MeasureDef {
        MeasureDef {
            index: MeasureIndex(idx),
            tempo: None,
            time_signature: None,
            key_signature: None,
            marker: None,
            navigation: events,
            tick_resolution: 960,
            duration_ticks: 3840,
            barline_left: None,
            barline_right: None,
            gp_beams: None,
        }
    }

    fn nav(kind: JumpKind) -> NavigationEvent {
        NavigationEvent {
            measure_index: MeasureIndex(0),
            kind,
            repeat_count: None,
            volta: None,
            volta_last: false,
        }
    }

    fn repeat_close(count: u8) -> NavigationEvent {
        NavigationEvent {
            measure_index: MeasureIndex(0),
            kind: JumpKind::RepeatClose,
            repeat_count: Some(count),
            volta: None,
            volta_last: false,
        }
    }

    /// RepeatClose that also marks a volta bracket (e.g. first ending).
    fn volta_close(count: u8, volta: u8, last: bool) -> NavigationEvent {
        NavigationEvent {
            measure_index: MeasureIndex(0),
            kind: JumpKind::RepeatClose,
            repeat_count: Some(count),
            volta: Some(volta),
            volta_last: last,
        }
    }

    /// End of the last volta bracket — no jump back, just carries the bracket tag.
    fn volta_end(volta: u8) -> NavigationEvent {
        NavigationEvent {
            measure_index: MeasureIndex(0),
            kind: JumpKind::RepeatClose,
            repeat_count: None,
            volta: Some(volta),
            volta_last: true,
        }
    }

    fn idx(n: u16) -> MeasureIndex {
        MeasureIndex(n)
    }

    // --- Tests ---

    #[test]
    fn empty_timeline() {
        assert!(Arrangement::resolve(&[]).order.is_empty());
    }

    #[test]
    fn linear_no_navigation() {
        let timeline: Vec<_> = (0u16..5).map(|i| measure(i, vec![])).collect();
        let order = Arrangement::resolve(&timeline).order;
        assert_eq!(order, (0u16..5).map(idx).collect::<Vec<_>>());
    }

    /// |: 0 | 1 | 2 :| 3   →   0 1 2  0 1 2  3
    #[test]
    fn simple_repeat_twice() {
        let timeline = vec![
            measure(0, vec![nav(JumpKind::RepeatOpen)]),
            measure(1, vec![]),
            measure(2, vec![repeat_close(2)]),
            measure(3, vec![]),
        ];
        assert_eq!(
            Arrangement::resolve(&timeline).order,
            vec![idx(0), idx(1), idx(2), idx(0), idx(1), idx(2), idx(3)]
        );
    }

    /// |: 0 | 1 :|×3   →   0 1  0 1  0 1
    #[test]
    fn repeat_three_times() {
        let timeline = vec![
            measure(0, vec![nav(JumpKind::RepeatOpen)]),
            measure(1, vec![repeat_close(3)]),
            measure(2, vec![]),
        ];
        assert_eq!(
            Arrangement::resolve(&timeline).order,
            vec![idx(0), idx(1), idx(0), idx(1), idx(0), idx(1), idx(2)]
        );
    }

    /// |: 0 | 1 [1. 2 :| [2. 3] 4
    ///
    /// Pass 1: 0 1 2 → jump back
    /// Pass 2: 0 1 (skip 2) 3 → continue
    /// Then:   4
    #[test]
    fn volta_two_endings() {
        let timeline = vec![
            measure(0, vec![nav(JumpKind::RepeatOpen)]),
            measure(1, vec![]),
            measure(2, vec![volta_close(2, 1, false)]), // 1st ending + RepeatClose
            measure(3, vec![volta_end(2)]),             // 2nd (last) ending
            measure(4, vec![]),
        ];
        assert_eq!(
            Arrangement::resolve(&timeline).order,
            //        pass 1                 pass 2
            vec![idx(0), idx(1), idx(2), idx(0), idx(1), idx(3), idx(4)]
        );
    }

    /// |: 0 [1. 1 :| [2. 2] [3. 3] 4   (3 endings, repeat_count = 3)
    #[test]
    fn volta_three_endings() {
        let timeline = vec![
            measure(0, vec![nav(JumpKind::RepeatOpen)]),
            measure(1, vec![volta_close(3, 1, false)]),
            measure(2, vec![volta_close(3, 2, false)]),
            measure(3, vec![volta_end(3)]),
            measure(4, vec![]),
        ];
        assert_eq!(
            Arrangement::resolve(&timeline).order,
            vec![
                idx(0),
                idx(1), // pass 1
                idx(0),
                idx(2), // pass 2
                idx(0),
                idx(3), // pass 3
                idx(4),
            ]
        );
    }

    /// 0 | 1 | 2 D.C. | 3
    ///
    /// Forward: 0 1 2 → D.C. → return pass: 0 1 2 (D.C. ignored) 3 (end of piece)
    #[test]
    fn da_capo() {
        let timeline = vec![
            measure(0, vec![]),
            measure(1, vec![]),
            measure(2, vec![nav(JumpKind::DaCapo)]),
            measure(3, vec![]),
        ];
        assert_eq!(
            Arrangement::resolve(&timeline).order,
            vec![idx(0), idx(1), idx(2), idx(0), idx(1), idx(2), idx(3)]
        );
    }

    /// 0 | 1 $ | 2 | 3 D.S. | 4
    ///
    /// Forward: 0 1 2 3 → D.S. → return from $: 1 2 3 (D.S. ignored) 4 (end of piece)
    #[test]
    fn dal_segno() {
        let timeline = vec![
            measure(0, vec![]),
            measure(1, vec![nav(JumpKind::Segno)]),
            measure(2, vec![]),
            measure(3, vec![nav(JumpKind::DalSegno)]),
            measure(4, vec![]),
        ];
        assert_eq!(
            Arrangement::resolve(&timeline).order,
            vec![
                idx(0),
                idx(1),
                idx(2),
                idx(3),
                idx(1),
                idx(2),
                idx(3),
                idx(4)
            ]
        );
    }

    /// 0 | 1 Fine | 2 | 3 D.C. al Fine
    ///
    /// Forward: 0 1 2 3 → D.C. al Fine → 0 1 (Fine → stop)
    #[test]
    fn da_capo_al_fine() {
        let timeline = vec![
            measure(0, vec![]),
            measure(1, vec![nav(JumpKind::Fine)]),
            measure(2, vec![]),
            measure(3, vec![nav(JumpKind::DaCapoAlFine)]),
        ];
        assert_eq!(
            Arrangement::resolve(&timeline).order,
            vec![idx(0), idx(1), idx(2), idx(3), idx(0), idx(1)]
        );
    }

    /// 0 | 1 $ | 2 Fine | 3 | 4 D.S. al Fine
    ///
    /// Forward: 0 1 2 3 4 → D.S. al Fine → 1 2 (Fine → stop)
    #[test]
    fn dal_segno_al_fine() {
        let timeline = vec![
            measure(0, vec![]),
            measure(1, vec![nav(JumpKind::Segno)]),
            measure(2, vec![nav(JumpKind::Fine)]),
            measure(3, vec![]),
            measure(4, vec![nav(JumpKind::DalSegnoAlFine)]),
        ];
        assert_eq!(
            Arrangement::resolve(&timeline).order,
            vec![idx(0), idx(1), idx(2), idx(3), idx(4), idx(1), idx(2)]
        );
    }

    /// 0 | 1 ⊕(dest) | 2 | 3 D.C. al Coda
    ///
    /// coda_pos is set to 1 (first Coda seen).
    /// On the return pass, when we reach measure 1 (Coda), dest == pos → no forward
    /// jump needed; execution continues from 1 to end.
    /// Result: 0 1 2 3  0 1 2 3
    #[test]
    fn da_capo_al_coda_single_marker() {
        let timeline = vec![
            measure(0, vec![]),
            measure(1, vec![nav(JumpKind::Coda)]),
            measure(2, vec![]),
            measure(3, vec![nav(JumpKind::DaCapoAlCoda)]),
        ];
        assert_eq!(
            Arrangement::resolve(&timeline).order,
            vec![
                idx(0),
                idx(1),
                idx(2),
                idx(3),
                idx(0),
                idx(1),
                idx(2),
                idx(3)
            ]
        );
    }

    /// Malformed input: RepeatOpen with no matching RepeatClose.
    /// The safety guard must terminate without hanging.
    #[test]
    fn safety_guard_missing_repeat_close() {
        let timeline: Vec<_> = std::iter::once(measure(0, vec![nav(JumpKind::RepeatOpen)]))
            .chain((1u16..8).map(|i| measure(i, vec![])))
            .collect();
        let order = Arrangement::resolve(&timeline).order;
        // All measures must appear at least once.
        assert!(order.contains(&idx(0)));
        assert!(order.contains(&idx(7)));
    }
}
