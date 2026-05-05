# Chord diagrams

* **Header**: `byte`. This value is 0x01, indicating a Guitar Pro 4 format chord.
* **Sharp**: `byte`. Determines if the chord is displayed sharp or flat.
* Blank1
* Blank2
* Blank3: Bytes. Blank bytes needed for ascendant compatibility with versions 3 of the software.
* **Root**: `byte`: `-1 for the customized chords`, `0: C`, `1: C#`, ...
* **Major/minor**: `byte`, Determines the chord type as followed:
  * 0: M
  * 1: 7
  * 2: 7M
  * 3: 6
  * 4: m
  * 5: m7
  * 6: m7M
  * 7: m6
  * 8: sus2
  * 9: sus4
  * 10: 7sus2
  * 11: 7sus4
  * 12: dim
  * 13: aug
  * 14: 5
* **Nine, Eleven of Thirteen**: `byte`. Determines if the chord goes until the ninth, the eleventh, or the thirteenth.
* **Bass**: `integer`. Lowest note of the chord. It gives the chord inversions.
* **Diminished/Augmented**: `integer`. Tonality linked with 9/11/13: `0: perfect ("juste")`, `1: augmented`, `2: diminished`
* **add**: `byte`. Allows to determine if a 'add' (added note) is present in the chord.
* **Name**: `string`. 20 characters long string containing the chord name.
* Blank4,
* Blank5: Bytes. Blank bytes needed for ascendant compatibility with versions 3 of the software.
* **Fifth**: `byte`. Tonality of the fifth: `0: perfect ("juste")`, `1: augmented`, `2: diminished`
* **Ninth**: `byte`. Tonality of the ninth: `0: perfect ("juste")`, `1: augmented`, `2: diminished`. This tonality is valid only if the value "Nine, Eleven or Thirteen" is 11 or 13.
* **Eleventh**: `byte`. Tonality of the eleventh: `0: perfect ("juste")`, `1: augmented`, `2: diminished`. This tonality is valid only if the value "Nine, Eleven or Thirteen" is 13.
* **Base fret**: `integer`. Shows the fret from which the chord is displayed.
* **Frets**: List of 7 integers. Corresponds to the fret number played on each string, from the highest to the lowest: -1 means a string unplayed, 0 means a string played "blank" (ie no fret).
* **Number of barres**: `byte`. Indicates the number of barres there are in the chord. A chord can contain up to 5 barres.
* **Fret of the barre**: List of 5 Bytes. Indicates the fret number of each possible barre.
* **Barre start**: List of 5 Bytes. Indicates the first string of the barre, 1 being the highest.The list order matches the chord different barres frets list order.
* **Barre end**: List of 5 Bytes. Indicates the first string of the barre, 1 being the lowest. The list order matches the chord different barres frets list order.
* Omission1,
* Omission3,
* Omission5,
* Omission7,
* Omission9,
* Omission11,
* Omission13: Bytes. Gives the notes there are in the chord. If the value is `0x00`, the note is not in the chord, and if the value is `0x01`, the note is in the chord. 9, 11 or 13 can only be present for a "Nine, Eleven or Thirteen" big enough.
* Blank6: `byte`. Blank byte needed for ascendant compatibility with versions 3 of the software.
* **Fingering**: List of 7 Bytes. Describes the fingering used to play the chord. Below is given the fingering from the highest string to the lowest:
  * -2: unknown;
  * -1: X or 0 (no finger);
  * 0: thumb;
  * 1: index;
  * 2: middle finger;
  * 3: annular;
  * 4: little finger.
* **ShowDiagFingering**: `byte`. If it is `0x01`, the chord fingering is displayed,  if it is `0x00`, the chord fingering is masked.
