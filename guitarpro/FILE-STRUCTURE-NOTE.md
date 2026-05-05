# Note

The first byte is the note header. It lists the information about the different parameters linked to the note:

| **Bit 7** | **Bit 6** | **Bit 5** | **Bit 4** | **Bit 3** | **Bit 2** | **Bit 1** | **Bit 0** |
|-----------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|
| Right hand or left hand fingering | Accentuated note | Note type (rest, empty note, normal note)  | Note dynamic  | Presence of effects linked to the note  | Ghost note  | Dotted note  | Time-independent duration  |

* **Note type**: `short int`. If the bit 5 is true, we start by writing the note type. The value is:
  * `0x0100` if the note is normal;
  * `0x0200` if the note is a ghost note;
  * `0x0300` if the note is a tie note.
* **Note duration**: If the bit 0 is true, we write the 2 following information linked to the note duration.
* **Time duration**: `byte`. The basic time duration is the quarter note, which is equivalent to a time duration of 1. Therefore, we obtain the relations:
  * -2:Whole Note
  * -1:Half note
  * 0:Quarter note
  * 1:Eighth note
  * 2:Sixteenth note
  * 3:Thirty-second note
  * ...
* **N-tuplet**: `byte`. This int matches the N in "N-tuplet" (ie 3, 5, 6, 7, 9, 10, 11, 12, 13).
* **Note dynamic**: `byte`. If the bit 4 is false, we then consider that the note is forte (value 6). If the bit 4 of the header byte is true, we write here the note strength with the following rule:
  * 1: ppp
  * 2: pp
  * 3: p
  * 4: mp
  * 5: mf
  * 7: f
  * 8: ff
  * 9: fff
* **Fret number**: `byte`. If the bit 5 is true, we write here the number of the fret on which the note is applied.
* **Fingering**: List of 2 bytes. If the bit 7 is true, we write here successively the fingerings left hand, then right hand, each of them being written on one byte. The fingering is coded this way: `-1: nothing`, `0: thumb`, `1: index`, `2: middle finger`, `3: annular`, `4: little finger`.
* **Effects on the note**: If the presence of an effect is indicated by the bit 3 of the header, it is saved at this place. The details of this operation is specified in the next paragraph. Those include for example vibratos, palm muting, slides, harmonices...

## Effects on the notes

a. Effects on the notes

The effects presence for the current note is set by the 2 header bytes. First byte:

| **Bit 7** | **Bit 6** | **Bit 5** | **Bit 4** | **Bit 3** | **Bit 2** | **Bit 1** | **Bit 0** |
|-----------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|
| Blank bit | Blank bit | Blank bit | Grace note presence | Let ring | Presence of a slide from the current note (version 3) | Presence of a hammer-on or a pull-off from the current note | Presence of a bend |

Second byte:

| **Bit 7** | **Bit 6** | **Bit 5** | **Bit 4** | **Bit 3** | **Bit 2** | **Bit 1** | **Bit 0** |
|-----------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|
| Blank bit | Left hand vibrato | Trill | Harmonic note | Presence of a slide from the current note | Tremolo picking | Palm Mute | Note played staccato |

* **Bend**: If the bit 0 of the first header byte is true, we write here the bend as described in section [Bends](#Bends).
* **Grace Note**: If the bit 4 of the first header byte is true, we write here the data relative to the Grace Note, as described in section [Grace Notes](#Grace-notes).
* **Tremolo Picking**: `byte`. If the bit 2 of the second header byte is true, the information linked to tremolo picking are saved here. It is encoded like this: `1: eighth note`, `2: sixteenth note`, `3: thirty-second note`.
* **Slide**: `byte`. If the bit 3 of the second header byte is true, the information linked to the slide is stored here and is coded like this:
  * -2: slide into from above
  * -1: slide into from below
  * 0: no slide
  * 1: shift slide
  * 2: legato slide
  * 3: slide out of downwards
  * 4: slide out of upwards
* **Harmonics**: `byte`. If the bit 4 of the second header byte is true, the information linked to the harmonics is stored here and is coded like this:
  * 0:  None
  * 1:  Natural
  * 15: Artificial+5
  * 17: Artificial+7
  * 22: Artificial+12
  * 3:  Tapped
  * 4:  Pitch
  * 5:  Semi
* **Trill**: If the bit 5 of the second header byte is true, the information linked to the Trill effect is stored here. It is written in two steps:
  * **Fret**: byte`. The fret the trill is made with.
  * **Period**: `byte`. The period between each note. The value is encoded as: `0: Quarter note`, `1: Eighth note`, `2: Sixteenth note`.

**The following effects are present if the bit of the header is true**:

* Let ring
* Presence of a hammer-on or a pull-off from the current note
* Left hand vibrato
* Palm Mute
* Note played staccato.

## Grace notes

The grace notes are stored in the file with 4 variables, written in the following order.

* **Fret**: `byte`. The fret number the grace note is made from.
* **Dynamic**: `byte`. The grace note dynamic is coded like this (default value is 6):
  * 1: ppp
  * 2: pp
  * 3: p
  * 4: mp
  * 5: mf
  * 6: f
  * 7: ff
  * 8: fff
* **Transition**: `byte`. This variable determines the transition type used to make the grace note: `0: None`, `1: Slide`, `2: Bend`, `3: Hammer`.
* **Duration**: `byte`. Determines the grace note duration, coded this way: `3: Sixteenth note`, `2: Twenty-fourth note`, `1: Thirty-second note`.

## Bends

* Type: `byte`. Gives the bend type. Different types are allowed and are context-dependent (tremolo bar or bend).
The model list is:
  * **Common**: 0: None
  * **Bend specific**:
    * 1: Bend
    * 2: Bend and Release
    * 3: Bend and Release and Bend
    * 4: Prebend
    * 5: Prebend and Release
  * **Tremolo bar specific**:
    * 6: Dip
    * 7: Dive
    * 8: Release (up)
    * 9: Inverted dip
    * 10: Return
    * 11: Release (down)
* **Value**: `integer`. Bend height. It is 100 per tone and goes by quarter tone.
  * Normal position:0
  * Quarter tone: 25
  * Half tone:50
  * Three-quarters tone:75
  * Whole tone:100
  * ... until
  * Three tones:300
* Number of points: `integer`. Number of points used to display the bend. Is followed by the list of points.
* **List of points**: Each point is written like this:
  * **Absolute time position**: `integer`. Gives the point position from the previous point. This value goes between 0 and 60 and represents sixties of the note duration.
  * **Vertical position**: `integer`. It is 100 per tone and goes by quarter tone.
    * Normal position:0
    * Quarter tone: 25
    * Half tone:50
    * Three-quarters tone:75
    * Whole tone:100
    * ... until
    * Three tones:300
* **Vibrato**: `byte`. Determines how to play the section, with different vibrato types: `0: none`, `1: fast`, `2: average`, `3: slow`.
