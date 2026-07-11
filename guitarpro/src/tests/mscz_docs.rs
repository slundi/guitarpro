//! Documentation-integrity checks for the MSCZ format (roadmap Part 6).
//!
//! Every file listed as a Part 6 deliverable is validated here:
//!
//! * `guitarpro/CLAUDE.md` has an MSCZ section referencing the correct
//!   module paths.
//! * `.claude/skills/gp-mscz-format/SKILL.md` exists and carries valid
//!   YAML frontmatter (`name` + `description` + trailing `---` fence).
//! * Root, guitarpro, and CLI READMEs mention `.mscz` in supported formats.
//! * `docs/Roadmap-web.md` links to `Roadmap-mscz.md`.
//!
//! These tests catch documentation drift (e.g. someone renames a module
//! but forgets to update the skill, or drops `.mscz` from a README while
//! shipping code changes).

use std::fs;
use std::path::PathBuf;

/// Root of the workspace, resolved from the guitarpro crate manifest dir.
fn workspace_root() -> PathBuf {
    // guitarpro/  ← CARGO_MANIFEST_DIR
    // guitarpro/../  ← workspace root
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("guitarpro manifest dir has a parent")
        .to_path_buf()
}

fn read(path: &str) -> String {
    let full = workspace_root().join(path);
    fs::read_to_string(&full).unwrap_or_else(|e| {
        panic!("cannot read {}: {e}", full.display());
    })
}

// ---------------------------------------------------------------------------
// guitarpro/CLAUDE.md
// ---------------------------------------------------------------------------

#[test]
fn claude_md_has_mscz_section() {
    let text = read("guitarpro/CLAUDE.md");
    assert!(
        text.contains("## MSCZ"),
        "guitarpro/CLAUDE.md should have an MSCZ section header"
    );
}

#[test]
fn claude_md_references_actual_module_paths() {
    let text = read("guitarpro/CLAUDE.md");
    // Any renaming of these modules must be reflected in the doc.
    for expected in [
        "src/io/mscz/container.rs",
        "src/io/mscz/parse.rs",
        "src/model/mscz/mod.rs",
        "src/model/mscz/mscx.rs",
        "src/convert/mscz/to_optimized.rs",
        "src/convert/mscz/from_optimized.rs",
        "src/convert/mscz/validate.rs",
    ] {
        assert!(
            text.contains(expected),
            "guitarpro/CLAUDE.md missing module reference '{expected}'"
        );
    }
}

#[test]
fn referenced_modules_actually_exist_on_disk() {
    for path in [
        "guitarpro/src/io/mscz/container.rs",
        "guitarpro/src/io/mscz/parse.rs",
        "guitarpro/src/io/mscz/mod.rs",
        "guitarpro/src/model/mscz/mod.rs",
        "guitarpro/src/model/mscz/mscx.rs",
        "guitarpro/src/convert/mscz/to_optimized.rs",
        "guitarpro/src/convert/mscz/from_optimized.rs",
        "guitarpro/src/convert/mscz/validate.rs",
    ] {
        assert!(
            workspace_root().join(path).is_file(),
            "documented module '{path}' missing on disk — CLAUDE.md drifted"
        );
    }
}

// ---------------------------------------------------------------------------
// .claude/skills/gp-mscz-format/SKILL.md
// ---------------------------------------------------------------------------

#[test]
fn mscz_skill_exists_with_frontmatter() {
    let text = read(".claude/skills/gp-mscz-format/SKILL.md");
    assert!(
        text.starts_with("---\n"),
        "SKILL.md must begin with YAML frontmatter"
    );
    // The frontmatter block is closed by a second `---` fence.
    let body_after_first = &text[4..];
    let second_fence = body_after_first
        .find("\n---\n")
        .expect("SKILL.md YAML frontmatter must be closed with '---'");
    let frontmatter = &body_after_first[..second_fence];

    assert!(
        frontmatter.contains("name: gp-mscz-format"),
        "SKILL.md frontmatter should declare `name: gp-mscz-format`"
    );
    assert!(
        frontmatter.contains("description:"),
        "SKILL.md frontmatter needs a description field"
    );
}

#[test]
fn mscz_skill_lists_the_key_entry_points() {
    let text = read(".claude/skills/gp-mscz-format/SKILL.md");
    for symbol in [
        "read_mscz",
        "read_mscz_bytes",
        "write_mscz",
        "mscx_to_loaded_score",
        "loaded_score_to_mscx",
        "LossReport",
        "MsczArchive",
        "MsczFile",
    ] {
        assert!(
            text.contains(symbol),
            "SKILL.md should mention the API symbol '{symbol}'"
        );
    }
}

// ---------------------------------------------------------------------------
// READMEs
// ---------------------------------------------------------------------------

#[test]
fn root_readme_lists_mscz_in_supported_formats() {
    let text = read("README.md");
    assert!(
        text.contains(".mscz"),
        "root README should mention .mscz in the supported formats section"
    );
    assert!(
        text.contains("MuseScore"),
        "root README should mention MuseScore"
    );
    assert!(
        text.contains("docs/Roadmap-mscz.md") || text.contains("Roadmap-mscz"),
        "root README should link to the MSCZ roadmap"
    );
}

#[test]
fn guitarpro_readme_documents_mscz() {
    let text = read("guitarpro/README.md");
    assert!(text.contains(".mscz"));
    // A code example using the MSCZ entry point catches accidental removals
    // when reformatting the README.
    assert!(
        text.contains("read_mscz"),
        "guitarpro/README.md should show read_mscz usage"
    );
}

#[test]
fn cli_readme_documents_mscz_subcommands() {
    let text = read("cli/README.md");
    for expected in ["mscz list", "mscz extract", "mscz thumbnail"] {
        assert!(
            text.contains(expected),
            "cli/README.md missing sub-command reference '{expected}'"
        );
    }
    assert!(
        text.contains(".mscz"),
        "cli/README.md should mention .mscz in the supported formats section"
    );
}

// ---------------------------------------------------------------------------
// Roadmaps
// ---------------------------------------------------------------------------

#[test]
fn web_roadmap_cross_links_mscz_roadmap() {
    let text = read("docs/Roadmap-web.md");
    assert!(
        text.contains("Roadmap-mscz.md"),
        "docs/Roadmap-web.md should cross-link to Roadmap-mscz.md"
    );
}

#[test]
fn mscz_roadmap_still_exists() {
    // Sanity check — if this ever disappears the other tests need to fail
    // loud rather than silently succeed against a stale copy elsewhere.
    let path = workspace_root().join("docs/Roadmap-mscz.md");
    assert!(
        path.is_file(),
        "docs/Roadmap-mscz.md must exist (the Part 6 deliverables reference it)"
    );
}
