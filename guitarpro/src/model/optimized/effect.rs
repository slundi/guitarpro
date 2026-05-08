use serde::{Deserialize, Serialize};

/// Point event attached to a Beat. Effect state is resolved by scanning
/// backwards to the last EffectEvent on the same track.
#[derive(Serialize, Deserialize, Debug)]
pub enum BeatEvent {
    Effect(EffectEvent),
    Tempo(TempoEvent),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EffectEvent {
    pub channel: EffectChannel,
    pub volume: Option<f32>, // 0.0–1.0, None = unchanged
    pub pan: Option<f32>,    // -1.0 (L) to 1.0 (R), None = unchanged
    pub chorus: Option<ChorusParams>,
    pub reverb: Option<ReverbParams>,
    pub delay: Option<DelayParams>,
    pub wah: Option<bool>,
    pub label: Option<String>, // displayed above staff: "Dist.", "Clean"
}

#[derive(Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Debug)]
pub enum EffectChannel {
    Clean,
    Crunch,
    Overdrive,
    Distortion,
    Acoustic,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct ChorusParams {
    pub mix: f32,
    pub rate: f32,
    pub depth: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct ReverbParams {
    pub mix: f32,
    pub decay: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct DelayParams {
    pub mix: f32,
    pub time_ms: u16,
    pub feedback: f32,
}

#[derive(Serialize, Deserialize, Copy, Clone, Debug)]
pub struct TempoEvent {
    pub bpm: f32,
}
