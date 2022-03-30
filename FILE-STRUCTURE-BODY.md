# Body

Contains:

* Measures
* Tracks
* Measure-track pairs
  * Beat 1 : note 1..i
  * Beat 2 : note 1..i
  * Beat i : note 1..i

## Measures

The measures are written one after another, their number having been specified previously. The first byte is the measure's header. It lists the data given in the current measure.

| **Bit 7** | **Bit 6** | **Bit 5** | **Bit 4** | **Bit 3** | **Bit 2** | **Bit 1** | **Bit 0** |
|-----------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|
| Presence of a double bar  | Tonality of the measure  | Presence of a marker  | Number of alternate ending | End of repeat | Beginning of repeat | Denominator of the (key) signature | Numerator of the (key) signature. |

Each of these elements is present only if the corresponding bit is a 1. The different elements are written (if they are present) from lowest to highest bit.  
Exceptions are made for the double bar and the beginning of repeat whose sole presence is enough, complementary data is not necessary.

* **Numerator of the (key) signature**: Byte. Numerator of the (key) signature of the piece
* **Denominator of the (key) signature**: Byte. Denominator of the (key) signature of the piece
* **End of repeat**: Byte. Number of repeats until the previous Beginning of repeat. Nombre de renvoi jusqu'au début de renvoi précédent.
* **Number of alternate ending**: Byte. The number of alternate ending.
* **Marker**: The markers are written in two steps:
  1) First is written an `integer` equal to the marker's name length + 1
  2) a string containing the marker's name. Finally the marker's color is written.
* **Tonality of the measure**: Byte. This value encodes a key (signature) change on the current piece. It is encoded as: `0: C`, `1: G (#)`, `2: D (##)`, `-1: F (b)`, ...

## Tracks

The tracks are written one after another, their number having been specified previously. The first byte is the track's header. It precises the track's attributes:

| **bit 7 to 3** | **bit 2**   | **bit 1**                | **bit 0**   |
|----------------|-------------|--------------------------|-------------|
| Blank bits     | Banjo track | 12 stringed guitar track | Drums track |

* **Name**: `string`. A 40 characters long string containing the track's name.
* **Number of strings**: `integer`. An integer equal to the number of strings of the track.
* **Tuning of the strings**: Table of integers. The tuning of the strings is stored as a 7-integers table, the "Number of strings" first integers being really used. The strings are stored from the highest to the lowest.
* **Port**: `integer`. The number of the MIDI port used.
* **Channel**: `integer`. The number of the MIDI channel used. The channel 10 is the drums channel.
* **ChannelE**: `integer`. The number of the MIDI channel used for effects.
* **Number of frets**: `integer`. The number of frets of the instrument.
* **Height of the capo**: `integer`. The number of the fret on which a capo is present. If no capo is used, the value is `0x00000000`.
* **Track's color**: `color`. The track's displayed color in Guitar Pro.

## Measures-tracks pairs

The list of beats per track and per measure is then written to the file in the following order:

* Measure 1/Track 1
* Measure 1/Track 2
* ...
* Measure 1/Track M
* Measure 2/Track 1
* ...
* Measure 2/Track M
* ...
* Measure N/Track 1
* Measure N/Track 2
* ...
* Measure N/Track M

A measure-track pair is written in two steps. We first write the number of beats in the current pair:

* **Number of beats**: `integer`. Integer indicating the number of beats the measure-track pair contains.

After what we directly write the beat details (which are described in the next section).

## A beat

The first byte is the beat header. It lists the data present in the current beat:

| **Bit 7** | **Bit 6** | **Bit 5** | **Bit 4** | **Bit 3** | **Bit 2** | **Bit 1** | **Bit 0** |
|-----------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|
| Blank bit | Status: `True` if the beat is empty of if it is a rest | The beat is a n-tuplet | Presence of a mix table change event | Presence of effects | Presence of a text | Presence of a chord diagram | Dotted notes |

* **Status**: `byte`. If the bit 6 is `true`, we start by writing the beat status. The value is `0x00` if the beat is empty and 0x02 if it is a rest.
* **Beat duration**: `byte`. The basic beat duration is a eighth note, which is equivalent to a time duration of 1. We thus obtain the following relations:
  * -2: Whole note
  * -1: Half note
  * 0: Quarter note
  * 1: Eighth note
  * 2: Sixteenth note
  * 3: Thirty-second note
  * ...
* *N-tuplet*: `integer`. If the bit 5 of the header byte is true, this integer corresponds to the'n' in 'n-tuplet': 3, 5, 6, 7, 9, 10, 11, 12 and 13.
* *Chord diagram*: If the presence of a chord diagram is indicated by the bit 1 of the header, it is then written here. The detail of this operation is [here](FILE-STRUCTURE-CHORD-DIAGRAMS.md).
* **Text**: If the presence of a text is indicated by the bit 2 of the header, it is written here. It behaves like most of the previous strings. We first find an integer coding the text length + 1, followed by the string containing the text (at the format described in [GENERAL](FILE-STRUCTURE.md)).
* **Effect on beats**: If the presence of an effect is indicated by the bit 3 of the header, it is written at this place. The detail of this operation is specified [here](FILE-STRUCTURE-EFFECTS.md). Effects on beats include tremolo bars, bends...
* **Mix table change event**: If the presence of an event linked to the mix table is indicated by the bit 4 of the header, it is written here. The detail of this operation is specified in  Mix table change event.
* **Note**: The note itself is written here. The detail of this operation is specified in [Note](FILE-STRUCTURE-??????.md).

## Mix table change event

* **Instrument**: `byte`. Gives the number of the new instrument. The value is -1 if no instrument change occurs.
* **Volume**: `byte`. Gives the new volume value. The value is -1 if no volume change occurs.
* **Pan**: `byte`. Gives the new pan value. The value is -1 if no pan change occurs.
* **Chorus**: `byte`. Gives the new chorus value. The value is -1 if no chorus change occurs.
* **Reverb**: `byte`. Gives the new reverb value. The value is -1 if no reverb change occurs.
* **Phaser**: `byte`. Gives the new phaser value. The value is -1 if no phaser change occurs.
* **Tremolo**: `byte`. Gives the new tremolo value. The value is -1 if no tremolo change occurs.
* **Tempo**: `integer`. Gives the new tempo value. The value is -1 if no tempo change occurs.
* **Volume change duration**: `byte`. Gives the volume change duration in beats.
* **Pan change duration**: `byte`. Gives the pan change duration in beats.
* **Chorus change duration**: `byte`. Gives the chorus change duration in beats.
* **Reverb change duration**: `byte`. Gives the reverb change duration in beats.
* **Phaser change duration**: `byte`. Gives the phaser change duration in beats.
* **Tremolo change duration**: `byte`. Gives the tremolo change duration in beats.
* **Tempo change duration**: `byte`. Gives the tempo change duration in beats.

The next byte precises if the changes apply only to the current track (if the matching bit is 0), or to every track (if it is 1).

| **Bit 7** | **Bit 6** | **Bit 5** | **Bit 4** | **Bit 3** | **Bit 2** | **Bit 1** | **Bit 0** |
|-----------|-----------|-----------|-----------|-----------|-----------|-----------|-----------|
| Blank bit | Blank bit | Tremolo   | Phaser    | Reverb    | Chorus    | Pan       | Volume    |
