
/// Settings of the track.
#[derive(Clone)]
pub struct TrackSettings {
    tablature: bool,
    notation: bool,
    diagram_are_below: bool,
    show_rythm: bool,
    force_horizontal: bool,
    force_channels: bool,
    diagram_list: bool,
    diagram_in_score: bool,
    auto_let_ring: bool,
    auto_brush: bool,
    extend_rythmic: bool,
}
impl Default for TrackSettings { fn default() -> Self { TrackSettings {
    tablature: true,
    notation: true,
    diagram_are_below: false,
    show_rythm: false,
    force_horizontal: false,
    force_channels: false,
    diagram_list: true,
    diagram_in_score: false,
    auto_let_ring: false,
    auto_brush: false,
    extend_rythmic: false,
}}}
