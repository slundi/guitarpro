use super::common::read_gp7;
use crate::model::song::Song;

// ==================== GP7 (Guitar Pro 7+) tests ====================

#[test]
fn test_gp7_keysig() {
    let song = read_gp7("test/keysig.gp");
    assert_eq!(song.tracks.len(), 1);
    assert_eq!(song.measure_headers.len(), 32);
}
#[test]
fn test_gp7_copyright() {
    let song = read_gp7("test/copyright.gp");
    assert!(!song.tracks.is_empty());
    assert!(
        !song.copyright.is_empty(),
        "copyright field should be populated"
    );
}
#[test]
fn test_gp7_tempo() {
    let song = read_gp7("test/tempo.gp");
    assert!(!song.measure_headers.is_empty());
    assert!(song.tempo > 0, "tempo should be parsed from automations");
}
#[test]
fn test_gp7_rest_centered() {
    let song = read_gp7("test/rest-centered.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_dotted_tuplets() {
    let song = read_gp7("test/dotted-tuplets.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_test_irr_tuplet() {
    let song = read_gp7("test/testIrrTuplet.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_repeats() {
    let song = read_gp7("test/repeats.gp");
    assert!(!song.measure_headers.is_empty());
    let has_repeat = song
        .measure_headers
        .iter()
        .any(|mh| mh.repeat_open || mh.repeat_close > 0);
    assert!(
        has_repeat,
        "repeats.gp should have at least one repeat marker"
    );
}
#[test]
fn test_gp7_repeated_bars() {
    let song = read_gp7("test/repeated-bars.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_volta() {
    let song = read_gp7("test/volta.gp");
    assert!(!song.measure_headers.is_empty());
    let has_volta = song
        .measure_headers
        .iter()
        .any(|mh| mh.repeat_alternative > 0);
    assert!(
        has_volta,
        "volta.gp should have at least one alternate ending"
    );
}
#[test]
fn test_gp7_multivoices() {
    let song = read_gp7("test/multivoices.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_double_bar() {
    let song = read_gp7("test/double-bar.gp");
    assert!(!song.measure_headers.is_empty());
    let has_double_bar = song.measure_headers.iter().any(|mh| mh.double_bar);
    assert!(
        has_double_bar,
        "double-bar.gp should have at least one double bar"
    );
}
#[test]
fn test_gp7_clefs() {
    let song = read_gp7("test/clefs.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_bend() {
    let song = read_gp7("test/bend.gp");
    assert!(!song.tracks.is_empty());
    let has_bend = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.bend.is_some()))
            })
        })
    });
    assert!(
        has_bend,
        "bend.gp should contain at least one note with a bend effect"
    );
}
#[test]
fn test_gp7_basic_bend() {
    let song = read_gp7("test/basic-bend.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_vibrato() {
    let song = read_gp7("test/vibrato.gp");
    assert!(!song.tracks.is_empty());
    let has_vibrato = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.vibrato))
            })
        })
    });
    assert!(
        has_vibrato,
        "vibrato.gp should contain at least one note with vibrato"
    );
}
#[test]
fn test_gp7_let_ring() {
    let song = read_gp7("test/let-ring.gp");
    assert!(!song.tracks.is_empty());
    let has_let_ring = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.let_ring))
            })
        })
    });
    assert!(
        has_let_ring,
        "let-ring.gp should contain at least one let-ring note"
    );
}
#[test]
fn test_gp7_palm_mute() {
    let song = read_gp7("test/palm-mute.gp");
    assert!(!song.tracks.is_empty());
    let has_palm_mute = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.palm_mute))
            })
        })
    });
    assert!(
        has_palm_mute,
        "palm-mute.gp should contain at least one palm-muted note"
    );
}
#[test]
fn test_gp7_accent() {
    let song = read_gp7("test/accent.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_sforzato() {
    let song = read_gp7("test/sforzato.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_heavy_accent() {
    let song = read_gp7("test/heavy-accent.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ghost_note() {
    let song = read_gp7("test/ghost-note.gp");
    assert!(!song.tracks.is_empty());
    let has_ghost = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.ghost_note))
            })
        })
    });
    assert!(
        has_ghost,
        "ghost-note.gp should contain at least one ghost note"
    );
}
#[test]
fn test_gp7_dead_note() {
    use crate::model::enums::NoteType;
    let song = read_gp7("test/dead-note.gp");
    assert!(!song.tracks.is_empty());
    let has_dead = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.kind == NoteType::Dead))
            })
        })
    });
    assert!(
        has_dead,
        "dead-note.gp should contain at least one dead note"
    );
}
#[test]
fn test_gp7_trill() {
    let song = read_gp7("test/trill.gp");
    assert!(!song.tracks.is_empty());
    let has_trill = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.trill.is_some()))
            })
        })
    });
    assert!(has_trill, "trill.gp should contain at least one trill note");
}
#[test]
fn test_gp7_tremolos() {
    let song = read_gp7("test/tremolos.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_grace() {
    let song = read_gp7("test/grace.gp");
    assert!(!song.tracks.is_empty());
    let has_grace = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.grace.is_some()))
            })
        })
    });
    assert!(has_grace, "grace.gp should contain at least one grace note");
}
#[test]
fn test_gp7_grace_before_beat() {
    let song = read_gp7("test/grace-before-beat.gp");
    assert!(!song.tracks.is_empty());
    let has_grace_before = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats.iter().any(|b| {
                    b.notes
                        .iter()
                        .any(|n| n.effect.grace.as_ref().is_some_and(|g| !g.is_on_beat))
                })
            })
        })
    });
    assert!(
        has_grace_before,
        "grace-before-beat.gp should contain a grace note before the beat"
    );
}
#[test]
fn test_gp7_grace_on_beat() {
    let song = read_gp7("test/grace-on-beat.gp");
    assert!(!song.tracks.is_empty());
    let has_grace_on = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats.iter().any(|b| {
                    b.notes
                        .iter()
                        .any(|n| n.effect.grace.as_ref().is_some_and(|g| g.is_on_beat))
                })
            })
        })
    });
    assert!(
        has_grace_on,
        "grace-on-beat.gp should contain a grace note on the beat"
    );
}
#[test]
fn test_gp7_artificial_harmonic() {
    let song = read_gp7("test/artificial-harmonic.gp");
    assert!(!song.tracks.is_empty());
    let has_harmonic = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| n.effect.harmonic.is_some()))
            })
        })
    });
    assert!(
        has_harmonic,
        "artificial-harmonic.gp should contain at least one harmonic note"
    );
}
#[test]
fn test_gp7_high_pitch() {
    let song = read_gp7("test/high-pitch.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_shift_slide() {
    let song = read_gp7("test/shift-slide.gp");
    assert!(!song.tracks.is_empty());
    let has_slide = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices.iter().any(|v| {
                v.beats
                    .iter()
                    .any(|b| b.notes.iter().any(|n| !n.effect.slides.is_empty()))
            })
        })
    });
    assert!(
        has_slide,
        "shift-slide.gp should contain at least one note with slide effect"
    );
}
#[test]
fn test_gp7_legato_slide() {
    let song = read_gp7("test/legato-slide.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slide_out_down() {
    let song = read_gp7("test/slide-out-down.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slide_out_up() {
    let song = read_gp7("test/slide-out-up.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slide_in_below() {
    let song = read_gp7("test/slide-in-below.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slide_in_above() {
    let song = read_gp7("test/slide-in-above.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_brush() {
    let song = read_gp7("test/brush.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_arpeggio() {
    let song = read_gp7("test/arpeggio.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_rasg() {
    let song = read_gp7("test/rasg.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_fade_in() {
    let song = read_gp7("test/fade-in.gp");
    assert!(!song.tracks.is_empty());
    let has_fade_in = song.tracks.iter().any(|t| {
        t.measures.iter().any(|m| {
            m.voices
                .iter()
                .any(|v| v.beats.iter().any(|b| b.effect.fade_in))
        })
    });
    assert!(
        has_fade_in,
        "fade-in.gp should contain at least one beat with fade-in"
    );
}
#[test]
fn test_gp7_volume_swell() {
    let song = read_gp7("test/volume-swell.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_pick_up_down() {
    let song = read_gp7("test/pick-up-down.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur() {
    let song = read_gp7("test/slur.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur_hammer_slur() {
    let song = read_gp7("test/slur_hammer_slur.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur_slur_hammer() {
    let song = read_gp7("test/slur_slur_hammer.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur_over_3_measures() {
    let song = read_gp7("test/slur_over_3_measures.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur_voices() {
    let song = read_gp7("test/slur_voices.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_slur_notes_effect_mask() {
    let song = read_gp7("test/slur-notes-effect-mask.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_dotted_gliss() {
    let song = read_gp7("test/dotted-gliss.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ottava1() {
    let song = read_gp7("test/ottava1.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ottava2() {
    let song = read_gp7("test/ottava2.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ottava3() {
    let song = read_gp7("test/ottava3.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ottava4() {
    let song = read_gp7("test/ottava4.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_ottava5() {
    let song = read_gp7("test/ottava5.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_mordents() {
    let song = read_gp7("test/mordents.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_turn() {
    let song = read_gp7("test/turn.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_barre() {
    let song = read_gp7("test/barre.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_fingering() {
    let song = read_gp7("test/fingering.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_fret_diagram() {
    let song = read_gp7("test/fret-diagram.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_fret_diagram_2instruments() {
    let song = read_gp7("test/fret-diagram_2instruments.gp");
    assert!(song.tracks.len() >= 2);
}
#[test]
fn test_gp7_text() {
    let song = read_gp7("test/text.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_timer() {
    let song = read_gp7("test/timer.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_directions() {
    let song = read_gp7("test/directions.gp");
    assert!(!song.measure_headers.is_empty());
}
#[test]
fn test_gp7_fermata() {
    let song = read_gp7("test/fermata.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_free_time() {
    let song = read_gp7("test/free-time.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_dynamic() {
    let song = read_gp7("test/dynamic.gp");
    assert!(!song.tracks.is_empty());
    let velocities: Vec<i16> = song
        .tracks
        .iter()
        .flat_map(|t| {
            t.measures.iter().flat_map(|m| {
                m.voices.iter().flat_map(|v| {
                    v.beats
                        .iter()
                        .flat_map(|b| b.notes.iter().map(|n| n.velocity))
                })
            })
        })
        .collect();
    assert!(!velocities.is_empty(), "dynamic.gp should contain notes");
    let has_varying = velocities.iter().any(|&v| v != velocities[0]);
    assert!(
        has_varying,
        "dynamic.gp should have varying velocities across notes"
    );
}
#[test]
fn test_gp7_crescendo_diminuendo() {
    let song = read_gp7("test/crescendo-diminuendo.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_wah() {
    let song = read_gp7("test/wah.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_all_percussion() {
    let song = read_gp7("test/all-percussion.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_beams_stems_ledger_lines() {
    let song = read_gp7("test/beams-stems-ledger-lines.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_chordnames_keyboard() {
    let song = read_gp7("test/chordnames_keyboard.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_tuplet_with_slur() {
    let song = read_gp7("test/tuplet-with-slur.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_tap_slap_pop() {
    let song = read_gp7("test/tap-slap-pop.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_tremolo_bar() {
    let song = read_gp7("test/tremolo-bar.gp");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gp7_test() {
    let song = read_gp7("test/test.gp");
    assert!(!song.tracks.is_empty());
}

#[test]
fn test_gp7_all_files_parse() {
    use std::fs;
    let test_dir = "../test";
    let mut pass = 0;
    let mut failures: Vec<String> = Vec::new();
    for entry in fs::read_dir(test_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "gp") {
            let fname = path.file_name().unwrap().to_str().unwrap().to_string();
            let data = fs::read(&path).unwrap();
            let mut song = Song::default();
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                song.read_gp(&data).unwrap();
            })) {
                Ok(_) => {
                    pass += 1;
                }
                Err(e) => {
                    let msg = if let Some(s) = e.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = e.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "unknown".to_string()
                    };
                    let short = &msg[..msg.len().min(100)];
                    failures.push(format!("{}: {}", fname, short));
                }
            }
        }
    }
    if !failures.is_empty() {
        for f in &failures {
            eprintln!("FAIL: {}", f);
        }
    }
    eprintln!(
        "{} pass, {} fail out of {}",
        pass,
        failures.len(),
        pass + failures.len()
    );
    assert!(
        failures.is_empty(),
        "{} files failed to parse",
        failures.len()
    );
}
