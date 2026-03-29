use crate::Song;
use std::fs;
use std::path::Path;

#[test]
fn test_audit_all_files() {
    let test_dir = Path::new("../test");
    // Handle running from lib or root
    let test_dir = if test_dir.exists() {
        test_dir
    } else {
        Path::new("./test")
    };

    if !test_dir.exists() {
        eprintln!("Test directory not found!");
        return;
    }

    let mut results = Vec::new();
    let mut files: Vec<_> = fs::read_dir(test_dir)
        .expect("Cannot read dir")
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file())
        .collect();
    files.sort();

    for path in files {
        let filename = path.file_name().unwrap().to_str().unwrap().to_string();
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let data = match fs::read(&path) {
            Ok(d) => d,
            Err(e) => {
                results.push(format!("{}: READ_ERROR ({})", filename, e));
                continue;
            }
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut song = Song::default();
            match extension.as_str() {
                "gp3" => {
                    let _ = song.read_gp3(&data);
                }
                "gp4" => {
                    let _ = song.read_gp4(&data);
                }
                "gp5" => {
                    let _ = song.read_gp5(&data);
                }
                "gp" => {
                    let _ = song.read_gp(&data);
                }
                "gpx" => {
                    let _ = song.read_gpx(&data);
                }
                _ => return "SKIP".to_string(),
            }
            "OK".to_string()
        }));

        match result {
            Ok(status) => results.push(format!("{}: {}", filename, status)),
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    format!("PANIC: {}", s)
                } else if let Some(s) = e.downcast_ref::<String>() {
                    format!("PANIC: {}", s)
                } else {
                    "PANIC: Unknown".to_string()
                };
                results.push(format!("{}: {}", filename, msg));
            }
        }
    }

    // Write report
    let report = results.join("\n");
    fs::write("audit_report.txt", report).expect("Unable to write report");
}

#[test]
fn test_let_it_be_gp3() {
    let path = Path::new("../test/the-beatles-let_it_be.gp3");
    let path = if path.exists() {
        path
    } else {
        Path::new("./test/the-beatles-let_it_be.gp3")
    };
    let data = fs::read(path).expect("File not found");
    let mut song = Song::default();
    let _ = song.read_gp3(&data);
}

#[test]
fn test_demo_v5_gp5() {
    let path = Path::new("../test/Demo v5.gp5");
    let path = if path.exists() {
        path
    } else {
        Path::new("./test/Demo v5.gp5")
    };
    let data = fs::read(path).expect("File not found");
    let mut song = Song::default();
    let _ = song.read_gp5(&data);
}
