//! Generates the public C header from the Rust source with cbindgen.
//!
//! The header is written twice:
//! * `$OUT_DIR/guitarpro.h` — always fresh, matches exactly what was built.
//! * `include/guitarpro.h` — committed to the repository so C consumers can
//!   vendor the header without running a Rust build first. It is rewritten
//!   only when the content actually changes, so ordinary rebuilds never dirty
//!   the working tree.
//!
//! Header generation is best-effort: a read-only checkout or a cbindgen
//! failure emits a `cargo:warning` instead of breaking the build, because the
//! `cdylib` itself is perfectly usable without a regenerated header.

use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src");
    println!("cargo:rerun-if-changed=cbindgen.toml");

    let Ok(crate_dir) = std::env::var("CARGO_MANIFEST_DIR") else {
        println!("cargo:warning=CARGO_MANIFEST_DIR unset; skipping C header generation");
        return;
    };

    let config = match cbindgen::Config::from_file(Path::new(&crate_dir).join("cbindgen.toml")) {
        Ok(config) => config,
        Err(error) => {
            println!("cargo:warning=cannot read cbindgen.toml: {error}");
            return;
        }
    };

    let bindings = match cbindgen::Builder::new()
        .with_crate(&crate_dir)
        .with_config(config)
        .generate()
    {
        Ok(bindings) => bindings,
        Err(error) => {
            println!("cargo:warning=cbindgen could not generate the C header: {error}");
            return;
        }
    };

    let mut rendered = Vec::new();
    bindings.write(&mut rendered);

    if let Ok(out_dir) = std::env::var("OUT_DIR")
        && let Err(error) = std::fs::write(Path::new(&out_dir).join("guitarpro.h"), &rendered)
    {
        println!("cargo:warning=cannot write the header to OUT_DIR: {error}");
    }

    sync_committed_header(Path::new(&crate_dir).join("include/guitarpro.h"), &rendered);
}

/// Write `rendered` to `path` unless the file already holds exactly those
/// bytes. Skipping the identical case keeps `git status` clean across normal
/// rebuilds and avoids pointless mtime churn.
fn sync_committed_header(path: std::path::PathBuf, rendered: &[u8]) {
    if std::fs::read(&path).is_ok_and(|existing| existing == rendered) {
        return;
    }
    if let Some(parent) = path.parent()
        && let Err(error) = std::fs::create_dir_all(parent)
    {
        println!("cargo:warning=cannot create {}: {error}", parent.display());
        return;
    }
    if let Err(error) = std::fs::write(&path, rendered) {
        println!("cargo:warning=cannot update {}: {error}", path.display());
    }
}
