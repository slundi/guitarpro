# guitarpro-ffi

C ABI for the [`guitarpro`](../guitarpro) score library. Parse and convert
Guitar Pro, MuseScore and MusicXML files from C, C++, Swift, Kotlin/JNA,
Python `ctypes`, or anything else that can call a C function.

## Design

The Rust model — `Song`, `LoadedScore`, the arena-indexed ID newtypes, the
`Song*Ops` traits — **never crosses the boundary**. Mirroring it in C would
mean maintaining a second, lossier copy of the model, and the round-trip
escape hatches the library works hard to preserve (`gp_strings`,
`gp_ornament`, `gp_fermatas`, the misc store) are exactly what such a mirror
drops first.

So the boundary carries **data**, not objects: scores go in as bytes and come
out as bytes, with an opaque handle in between. Everything is already
`serde`-serializable on the Rust side, so this costs nothing to maintain.

## API

| Function | Purpose |
|---|---|
| `gpio_version()` | Library version string |
| `gpio_parse(data, len, format)` | Bytes → `GpioScore *` |
| `gpio_score_format(score)` | Format a handle was parsed as |
| `gpio_score_to_json(score)` | Handle → optimized-model JSON |
| `gpio_convert(data, len, in, out)` | Bytes → bytes, one format to another |
| `gpio_last_error()` | Message for the last failed call |
| `gpio_buffer_free(buffer)` | Release a `GpioBuffer` |
| `gpio_score_free(score)` | Release a `GpioScore *` |

Formats: `GPIO_FORMAT_AUTO` (input only), `GP3`, `GP4`, `GP5`, `GPX`, `GP`,
`MUSIC_XML`, `SCORE`, `MSCZ`. Every format works as both input and output,
except `AUTO`.

### Conventions

- **Failure** is a `NULL` pointer, or a `GpioBuffer` whose `data` is `NULL`.
  The reason is available from `gpio_last_error()` until the next `gpio_*`
  call on the same thread.
- **Ownership** of every returned buffer and handle passes to the caller.
  Release it with the matching `*_free`. Never call `free()` on `buffer.data`:
  the allocation comes from Rust, and `capacity` is what makes deallocation
  possible.
- **Format codes** are `uint32_t`, not the enum type, so an out-of-range value
  from C is rejected rather than becoming undefined behaviour. Use the
  `GPIO_FORMAT_*` constants.
- **Panics** are caught at every entry point and reported as errors.
  Unwinding never crosses the boundary.
- **Thread safety**: the error channel is thread-local and handles are
  independent, so separate threads may hold separate handles freely. A single
  handle is not synchronized — do not share one across threads without
  external locking.
- **Input size** is capped per format (16 MB for Guitar Pro, 32 MB for MSCZ,
  64 MB for the text formats) so a malformed length field cannot drive an
  unbounded allocation.

## Building

```sh
cargo build -p guitarpro-ffi --release
```

Produces `target/release/libguitarpro_ffi.{so,a}` (`.dylib` on macOS,
`.dll`/`.lib` on Windows) and regenerates `include/guitarpro.h` via cbindgen.
The header is committed, so C consumers can vendor it without running a Rust
build first; `build.rs` rewrites it only when the content actually changes, so
ordinary rebuilds leave the working tree clean.

## Example

```c
#include "guitarpro.h"

GpioScore *score = gpio_parse(data, len, GPIO_FORMAT_AUTO);
if (!score) {
    fprintf(stderr, "%s\n", gpio_last_error());
    return 1;
}

printf("format: %u\n", gpio_score_format(score));

GpioBuffer json = gpio_score_to_json(score);
if (json.data) {
    fwrite(json.data, 1, json.len, stdout);
}
gpio_buffer_free(json);
gpio_score_free(score);
```

A complete program is in [`c/demo.c`](c/demo.c):

```sh
just ffi-demo test/Chords.gp5          # print as JSON
just ffi-demo test/Chords.gp5 mscz     # convert to MuseScore
```

## Testing

```sh
cargo test -p guitarpro-ffi
```

Unit tests cover the buffer, error channel and format codec; `tests/c_abi.rs`
drives the entry points through raw pointers the way a C caller would. The
allocation paths have also been exercised under AddressSanitizer via the C
demo, which reports no leaks or invalid frees.

## Known gaps

Two pre-existing limitations in the underlying library bound what survives a
round trip. Both reproduce through `score_tool convert` with no FFI involved:

- **MusicXML export → import.** `song_to_score_partwise` writes absent
  optional attributes as `attr=""`, which the importer rejects with
  `invalid type: string "", expected u16`. Export works; re-importing the
  result does not.
- **GP4 → GP5.** The GP5 writer emits a page-setup block the GP5 reader then
  rejects (`Type conversion failed for page width`).
