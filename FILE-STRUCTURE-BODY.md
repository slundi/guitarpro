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
