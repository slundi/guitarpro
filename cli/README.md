# CLI

Ideas:

* [ ] `-l` Load bellow parameters from a file (YAML?, JSON?, other?)
* [ ] `-f` Find in files
  * [ ] song information (artist, title, album, ...) with wildcard and regexes
  * [ ] instrument
  * [ ] `-ft <value>` [tuning](https://en.wikipedia.org/wiki/List_of_guitar_tunings) (value example: `E-A-D-g-b-e`, `EADgbe`, ` E2–A2–D3–G3–B3–E4`, )
  * [ ] range (string count, piano keys count, drum elements, ...)
  * [ ] `-f? <value|range>` tempo: is constant, range if variable, min, max, ... (example: `80`, `60-90`, `60>`, `<120` or `0-120`, `60,80,90` or `80,70-85` for a list)
  * [ ] `-fb <beat>`beats
  * [ ] `-fn <notes>` notes (example: `DADDC` anywhere in the track, `|DADDC|` in a mesure, `|A|E|CC|` measures with those notes)
  * [ ] `-fr` Repetitions:
    * [ ] `-frm <1|2|4>` [same mesures](https://musescore.org/en/handbook/4/measure-and-multi-measure-repeats)
    * [ ] `-frs` [repeat signs](https://musescore.org/en/handbook/4/repeat-signs)
    * [ ] `-frv` [voltas](https://musescore.org/en/handbook/4/voltas)
  * [ ] `-fv 0.8` detect verse with a similarity percentage
* [ ] `-x` Extract:
  * [ ] `-xi <format>` above information in various format (CSV, JSON)
  * [ ] `-xt <number|instrument> <format>` tracks
  * [ ] `-xl <format>` lyrics
* [ ] `-c format` Conversion between formats with alerts when information are lost (like GP5 -> GP3)
* [ ] `-r` Replace repetitions 
  * [ ] `m` [same mesures](https://musescore.org/en/handbook/4/measure-and-multi-measure-repeats)
  * [ ] `s` repeat signs and `v` voltas when mesures are the same for all tracks
* [ ] `-t -0.5` Change tuning if possible
* [ ] `-ts x2` Divide/multiply time signatures (I had once an guitar tab that needed to be rewritten by changing the time signature and the beams)
* [ ] `-p` Apply page format parametters (margin, spacing, ...)
