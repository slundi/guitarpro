pub mod guitarpro;
pub mod legacy;
pub mod loss;
pub mod mscz;
pub mod musicxml;
pub mod optimized;

/// The string and fret to play `midi` on among `strings` (number, open
/// MIDI pitch): the fewest frets up (0–24), on none of the strings
/// `taken` (a chord's other notes': a GP beat holds one note per string);
/// `None` when no free string reaches it.
pub(crate) fn free_string_fret(midi: i32, strings: &[(i8, i8)], taken: &[i8]) -> Option<(i8, i16)> {
    let mut best: Option<(i8, i16)> = None;
    for &(string_num, open_midi) in strings {
        if taken.contains(&string_num) {
            continue;
        }
        let fret = midi - i32::from(open_midi);
        if let Ok(fret) = i16::try_from(fret)
            && (0..=24).contains(&fret)
            && best.is_none_or(|(_, best_fret)| fret < best_fret)
        {
            best = Some((string_num, fret));
        }
    }
    best
}

/// Where a voice's tied notes go on: per pitch (MIDI), the string the tie
/// started on, as GP ties a note to the one before it on its string. A
/// converter placing notes by pitch claims those strings for the notes
/// ending ties ([`HeldStrings::claim`]) before placing the others, then
/// records the beat's tie starts ([`HeldStrings::start`]).
#[derive(Default)]
pub(crate) struct HeldStrings(std::collections::HashMap<i32, i8>);

impl HeldStrings {
    /// The string and fret for a note of `midi` ending a tie: its tie
    /// start's string, when it's free (not `taken`) and reaches it; claimed
    /// in `taken`. The start is used up either way, so a later tie end
    /// without a start of its own doesn't take its string.
    pub(crate) fn claim(
        &mut self,
        midi: i32,
        strings: &[(i8, i8)],
        taken: &mut Vec<i8>,
    ) -> Option<(i8, i16)> {
        let string = self.0.remove(&midi)?;
        let on: Vec<(i8, i8)> = strings
            .iter()
            .copied()
            .filter(|&(number, _)| number == string)
            .collect();
        let placed = free_string_fret(midi, &on, taken)?;
        taken.push(placed.0);
        Some(placed)
    }

    /// A note of `midi` on `string` starting a tie: the note tied from it
    /// goes on there. Only tie starts are kept: a unison's other note (on
    /// another string, not tied on) mustn't take its place.
    pub(crate) fn start(&mut self, midi: i32, string: i8) {
        self.0.insert(midi, string);
    }
}
