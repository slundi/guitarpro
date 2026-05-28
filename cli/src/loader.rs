use std::fs;
use std::io::Read;
use std::path::Path;

use guitarpro::Song;

const MAX_FILE_SIZE: usize = 16 * 1024 * 1024; // 16 MB

/// Load a Guitar Pro file from disk and return the parsed `Song` and the
/// detected format string (e.g. `"GP5"`, `"GPX"`).
pub fn load_song(path_str: &str) -> anyhow::Result<(Song, String)> {
    let path = Path::new(path_str);
    if !path.exists() {
        anyhow::bail!("File '{}' not found", path_str);
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_uppercase())
        .unwrap_or_else(|| "UNKNOWN".to_string());

    let size = fs::metadata(path_str)
        .map(|m| m.len() as usize)
        .unwrap_or(0);

    if size > MAX_FILE_SIZE {
        anyhow::bail!("File is too large (> 16 MB)");
    }

    let mut file = fs::File::open(path_str)?;
    let mut data = Vec::with_capacity(size);
    file.read_to_end(&mut data)?;

    let mut song = Song::default();
    match ext.as_str() {
        "GP3" => song.read_gp3(&data)?,
        "GP4" => song.read_gp4(&data)?,
        "GP5" => song.read_gp5(&data)?,
        "GP" => song.read_gp(&data)?,
        "GPX" => song.read_gpx(&data)?,
        _ => anyhow::bail!(
            "Unsupported format '{}'. Supported extensions: .gp3, .gp4, .gp5, .gp, .gpx",
            ext
        ),
    }

    Ok((song, ext))
}
