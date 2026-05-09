//! Score-level metadata: `<identification>` and `<part-list>`.

use crate::model::{legacy::song::Song, musicxml};

// ---------------------------------------------------------------------------
// Identification
// ---------------------------------------------------------------------------

pub(super) fn build_identification(song: &Song) -> musicxml::identification::Identification {
    use musicxml::identification::{Creator, Encoding, Identification, Rights, Supports};

    let mut creators = vec![];
    if !song.artist.is_empty() {
        creators.push(Creator {
            creator_type: Some("composer".to_string()),
            value: song.artist.clone(),
        });
    }
    if !song.author.is_empty() {
        creators.push(Creator {
            creator_type: Some("arranger".to_string()),
            value: song.author.clone(),
        });
    }
    if !song.words.is_empty() && song.words != song.author {
        creators.push(Creator {
            creator_type: Some("lyricist".to_string()),
            value: song.words.clone(),
        });
    }
    if !song.writer.is_empty() {
        creators.push(Creator {
            creator_type: Some("transcriber".to_string()),
            value: song.writer.clone(),
        });
    }

    let rights = if !song.copyright.is_empty() {
        vec![Rights {
            rights_type: None,
            value: song.copyright.clone(),
        }]
    } else {
        vec![]
    };

    let encoding = Some(Encoding {
        encoding_date: None,
        encoders: vec![],
        software: vec!["guitarpro (Rust)".to_string()],
        encoding_description: vec![],
        supports: vec![
            Supports {
                supports_type: "yes".to_string(),
                element: "accidental".to_string(),
                attribute: None,
                value: None,
            },
            Supports {
                supports_type: "yes".to_string(),
                element: "beam".to_string(),
                attribute: None,
                value: None,
            },
        ],
    });

    // Collect remaining free-text metadata into miscellaneous fields
    let mut misc_fields = vec![];
    if !song.subtitle.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "subtitle".to_string(),
            value: song.subtitle.clone(),
        });
    }
    if !song.album.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "album".to_string(),
            value: song.album.clone(),
        });
    }
    if !song.date.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "date".to_string(),
            value: song.date.clone(),
        });
    }
    if !song.instructions.is_empty() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: "instructions".to_string(),
            value: song.instructions.clone(),
        });
    }
    for (i, notice) in song.notice.iter().enumerate() {
        misc_fields.push(musicxml::identification::MiscellaneousField {
            name: format!("notice-{}", i + 1),
            value: notice.clone(),
        });
    }

    let miscellaneous = if misc_fields.is_empty() {
        None
    } else {
        Some(musicxml::identification::Miscellaneous {
            fields: misc_fields,
        })
    };

    Identification {
        creators,
        rights,
        encoding,
        source: None,
        relations: vec![],
        miscellaneous,
    }
}

// ---------------------------------------------------------------------------
// Part list
// ---------------------------------------------------------------------------

pub(super) fn build_part_list(song: &Song) -> musicxml::part_list::PartList {
    use musicxml::part_list::{MidiInstrument, PartListItem, PartName, ScorePart};

    let items = song
        .tracks
        .iter()
        .enumerate()
        .map(|(i, track)| {
            let part_id = format!("P{}", i + 1);
            let instrument_id = format!("P{}-I1", i + 1);

            let channel = song
                .channels
                .get(track.channel_index)
                .cloned()
                .unwrap_or_default();

            // Guitar Pro program is 0-based; MusicXML midi-program is 1-based
            let midi_program = if track.percussion_track {
                None
            } else {
                Some((channel.instrument as u8).saturating_add(1))
            };

            let midi_instrument = MidiInstrument {
                id: instrument_id.clone(),
                midi_channel: Some(channel.channel + 1), // MusicXML channels are 1-based
                midi_name: None,
                midi_bank: if channel.bank > 0 {
                    Some(channel.bank as u16)
                } else {
                    None
                },
                midi_program,
                midi_unpitched: if track.percussion_track {
                    Some(60)
                } else {
                    None
                },
                volume: Some(f64::from(channel.volume) / 127.0 * 100.0),
                pan: Some((f64::from(channel.balance) - 64.0) / 63.0 * 90.0),
                elevation: None,
            };

            PartListItem::ScorePart(ScorePart {
                id: part_id,
                identification: None,
                part_name: Some(PartName {
                    print_object: None,
                    justify: None,
                    value: Some(track.name.clone()),
                }),
                part_name_display: None,
                part_abbreviation: if !track.short_name.is_empty() {
                    Some(PartName {
                        print_object: None,
                        justify: None,
                        value: Some(track.short_name.clone()),
                    })
                } else {
                    None
                },
                part_abbreviation_display: None,
                groups: vec![],
                score_instruments: vec![musicxml::part_list::ScoreInstrument {
                    id: instrument_id,
                    instrument_name: track.name.clone(),
                    instrument_abbreviation: None,
                    instrument_sound: None,
                    solo: None,
                    ensemble: None,
                }],
                players: vec![],
                midi_devices: vec![],
                midi_instruments: vec![midi_instrument],
            })
        })
        .collect();

    musicxml::part_list::PartList { items }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::legacy::song::Song;

    fn empty_song() -> Song {
        Song::default()
    }

    #[test]
    fn identification_empty_song_has_no_creators() {
        let song = empty_song();
        let ident = build_identification(&song);
        assert!(ident.creators.is_empty());
    }

    #[test]
    fn identification_empty_song_has_no_rights() {
        let song = empty_song();
        let ident = build_identification(&song);
        assert!(ident.rights.is_empty());
    }

    #[test]
    fn identification_empty_song_has_no_miscellaneous() {
        let song = empty_song();
        let ident = build_identification(&song);
        assert!(ident.miscellaneous.is_none());
    }

    #[test]
    fn identification_artist_becomes_composer_creator() {
        let mut song = empty_song();
        song.artist = "Jimi Hendrix".to_string();
        let ident = build_identification(&song);
        assert_eq!(ident.creators.len(), 1);
        assert_eq!(ident.creators[0].creator_type.as_deref(), Some("composer"));
        assert_eq!(ident.creators[0].value, "Jimi Hendrix");
    }

    #[test]
    fn identification_copyright_becomes_rights() {
        let mut song = empty_song();
        song.copyright = "© 1970".to_string();
        let ident = build_identification(&song);
        assert_eq!(ident.rights.len(), 1);
        assert_eq!(ident.rights[0].value, "© 1970");
    }

    #[test]
    fn identification_encoding_software_present() {
        let song = empty_song();
        let ident = build_identification(&song);
        let enc = ident.encoding.unwrap();
        assert!(enc.software.iter().any(|s| s.contains("guitarpro")));
    }

    #[test]
    fn identification_subtitle_stored_in_miscellaneous() {
        let mut song = empty_song();
        song.subtitle = "Subtitle".to_string();
        let ident = build_identification(&song);
        let misc = ident.miscellaneous.unwrap();
        assert!(misc.fields.iter().any(|f| f.name == "subtitle"));
    }

    #[test]
    fn part_list_empty_tracks_produces_empty_items() {
        let song = empty_song();
        let list = build_part_list(&song);
        assert!(list.items.is_empty());
    }
}
