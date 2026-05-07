use super::common::read_gpx;
use crate::model::song::Song;

// ==================== GPX (Guitar Pro 6) tests ====================

#[test]
fn test_gpx_keysig() {
    let song = read_gpx("test/keysig.gpx");
    assert_eq!(song.tracks.len(), 1);
    assert_eq!(song.measure_headers.len(), 32);
}
#[test]
fn test_gpx_copyright() {
    let song = read_gpx("test/copyright.gpx");
    assert!(!song.tracks.is_empty());
    assert!(
        !song.copyright.is_empty(),
        "copyright field should be populated"
    );
}
#[test]
fn test_gpx_tempo() {
    let song = read_gpx("test/tempo.gpx");
    assert!(!song.measure_headers.is_empty());
    assert!(song.tempo > 0, "tempo should be parsed from automations");
}
#[test]
fn test_gpx_rest_centered() {
    let song = read_gpx("test/rest-centered.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_dotted_tuplets() {
    let song = read_gpx("test/dotted-tuplets.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_tuplets() {
    let song = read_gpx("test/tuplets.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_tuplets2() {
    let song = read_gpx("test/tuplets2.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_test_irr_tuplet() {
    let song = read_gpx("test/testIrrTuplet.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_repeats() {
    let song = read_gpx("test/repeats.gpx");
    assert!(!song.measure_headers.is_empty());
    let has_repeat = song
        .measure_headers
        .iter()
        .any(|mh| mh.repeat_open || mh.repeat_close > 0);
    assert!(
        has_repeat,
        "repeats.gpx should have at least one repeat marker"
    );
}
#[test]
fn test_gpx_repeated_bars() {
    let song = read_gpx("test/repeated-bars.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_volta() {
    let song = read_gpx("test/volta.gpx");
    assert!(!song.measure_headers.is_empty());
    let has_volta = song
        .measure_headers
        .iter()
        .any(|mh| mh.repeat_alternative > 0);
    assert!(
        has_volta,
        "volta.gpx should have at least one alternate ending"
    );
}
#[test]
fn test_gpx_multivoices() {
    let song = read_gpx("test/multivoices.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_double_bar() {
    let song = read_gpx("test/double-bar.gpx");
    assert!(!song.measure_headers.is_empty());
    let has_double_bar = song.measure_headers.iter().any(|mh| mh.double_bar);
    assert!(
        has_double_bar,
        "double-bar.gpx should have at least one double bar"
    );
}
#[test]
fn test_gpx_clefs() {
    let song = read_gpx("test/clefs.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_bend() {
    let song = read_gpx("test/bend.gpx");
    assert!(!song.tracks.is_empty());
    // Verify that at least one note has a bend effect
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
        "bend.gpx should contain at least one note with a bend effect"
    );
}
#[test]
fn test_gpx_basic_bend() {
    let song = read_gpx("test/basic-bend.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_vibrato() {
    let song = read_gpx("test/vibrato.gpx");
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
        "vibrato.gpx should contain at least one note with vibrato"
    );
}
#[test]
fn test_gpx_let_ring() {
    let song = read_gpx("test/let-ring.gpx");
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
        "let-ring.gpx should contain at least one let-ring note"
    );
}
#[test]
fn test_gpx_palm_mute() {
    let song = read_gpx("test/palm-mute.gpx");
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
        "palm-mute.gpx should contain at least one palm-muted note"
    );
}
#[test]
fn test_gpx_accent() {
    let song = read_gpx("test/accent.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_sforzato() {
    let song = read_gpx("test/sforzato.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_heavy_accent() {
    let song = read_gpx("test/heavy-accent.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ghost_note() {
    let song = read_gpx("test/ghost-note.gpx");
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
        "ghost-note.gpx should contain at least one ghost note"
    );
}
#[test]
fn test_gpx_dead_note() {
    use crate::model::enums::NoteType;
    let song = read_gpx("test/dead-note.gpx");
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
        "dead-note.gpx should contain at least one dead note"
    );
}
#[test]
fn test_gpx_trill() {
    let song = read_gpx("test/trill.gpx");
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
    assert!(
        has_trill,
        "trill.gpx should contain at least one trill note"
    );
}
#[test]
fn test_gpx_tremolos() {
    let song = read_gpx("test/tremolos.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_grace() {
    let song = read_gpx("test/grace.gpx");
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
    assert!(
        has_grace,
        "grace.gpx should contain at least one grace note"
    );
}
#[test]
fn test_gpx_grace_before_beat() {
    let song = read_gpx("test/grace-before-beat.gpx");
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
        "grace-before-beat.gpx should contain a grace note before the beat"
    );
}
#[test]
fn test_gpx_grace_on_beat() {
    let song = read_gpx("test/grace-on-beat.gpx");
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
        "grace-on-beat.gpx should contain a grace note on the beat"
    );
}
#[test]
fn test_gpx_artificial_harmonic() {
    let song = read_gpx("test/artificial-harmonic.gpx");
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
        "artificial-harmonic.gpx should contain at least one harmonic note"
    );
}
#[test]
fn test_gpx_high_pitch() {
    let song = read_gpx("test/high-pitch.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_shift_slide() {
    let song = read_gpx("test/shift-slide.gpx");
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
        "shift-slide.gpx should contain at least one note with slide effect"
    );
}
#[test]
fn test_gpx_legato_slide() {
    let song = read_gpx("test/legato-slide.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slide_out_down() {
    let song = read_gpx("test/slide-out-down.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slide_out_up() {
    let song = read_gpx("test/slide-out-up.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slide_in_below() {
    let song = read_gpx("test/slide-in-below.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slide_in_above() {
    let song = read_gpx("test/slide-in-above.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_brush() {
    let song = read_gpx("test/brush.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_arpeggio() {
    let song = read_gpx("test/arpeggio.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_rasg() {
    let song = read_gpx("test/rasg.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_fade_in() {
    let song = read_gpx("test/fade-in.gpx");
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
        "fade-in.gpx should contain at least one beat with fade-in"
    );
}
#[test]
fn test_gpx_volume_swell() {
    let song = read_gpx("test/volume-swell.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_pick_up_down() {
    let song = read_gpx("test/pick-up-down.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur() {
    let song = read_gpx("test/slur.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur_hammer_slur() {
    let song = read_gpx("test/slur_hammer_slur.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur_slur_hammer() {
    let song = read_gpx("test/slur_slur_hammer.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur_over_3_measures() {
    let song = read_gpx("test/slur_over_3_measures.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur_voices() {
    let song = read_gpx("test/slur_voices.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_slur_notes_effect_mask() {
    let song = read_gpx("test/slur-notes-effect-mask.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_dotted_gliss() {
    let song = read_gpx("test/dotted-gliss.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ottava1() {
    let song = read_gpx("test/ottava1.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ottava2() {
    let song = read_gpx("test/ottava2.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ottava3() {
    let song = read_gpx("test/ottava3.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ottava4() {
    let song = read_gpx("test/ottava4.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_ottava5() {
    let song = read_gpx("test/ottava5.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_mordents() {
    let song = read_gpx("test/mordents.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_turn() {
    let song = read_gpx("test/turn.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_barre() {
    let song = read_gpx("test/barre.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_fingering() {
    let song = read_gpx("test/fingering.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_fret_diagram() {
    let song = read_gpx("test/fret-diagram.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_fret_diagram_2instruments() {
    let song = read_gpx("test/fret-diagram_2instruments.gpx");
    assert!(song.tracks.len() >= 2);
}
#[test]
fn test_gpx_text() {
    let song = read_gpx("test/text.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_timer() {
    let song = read_gpx("test/timer.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_directions() {
    let song = read_gpx("test/directions.gpx");
    assert!(!song.measure_headers.is_empty());
}
#[test]
fn test_gpx_fermata() {
    let song = read_gpx("test/fermata.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_free_time() {
    let song = read_gpx("test/free-time.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_dynamic() {
    let song = read_gpx("test/dynamic.gpx");
    assert!(!song.tracks.is_empty());
    // Verify that notes have varying velocities (not all the same default)
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
    assert!(!velocities.is_empty(), "dynamic.gpx should contain notes");
    let has_varying = velocities.iter().any(|&v| v != velocities[0]);
    assert!(
        has_varying,
        "dynamic.gpx should have varying velocities across notes"
    );
}
#[test]
fn test_gpx_crescendo_diminuendo() {
    let song = read_gpx("test/crescendo-diminuendo.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_wah() {
    let song = read_gpx("test/wah.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_all_percussion() {
    let song = read_gpx("test/all-percussion.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_beams_stems_ledger_lines() {
    let song = read_gpx("test/beams-stems-ledger-lines.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_chordnames_keyboard() {
    let song = read_gpx("test/chordnames_keyboard.gpx");
    assert!(!song.tracks.is_empty());
}
#[test]
fn test_gpx_tuplet_with_slur() {
    let song = read_gpx("test/tuplet-with-slur.gpx");
    assert!(!song.tracks.is_empty());
}

#[test]
fn test_gpx_all_files_parse() {
    use std::fs;
    let test_dir = "../test";
    let mut pass = 0;
    let mut failures: Vec<String> = Vec::new();
    for entry in fs::read_dir(test_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "gpx") {
            let fname = path.file_name().unwrap().to_str().unwrap().to_string();
            let data = fs::read(&path).unwrap();
            let mut song = Song::default();
            match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                song.read_gpx(&data).unwrap();
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
