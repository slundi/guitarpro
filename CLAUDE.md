# Global Project Guide: scorelib Workspace

This file provides high-level guidance for the entire workspace.

## Workspace Structure
- `guitarpro/`: The core library (`guitarpro`).
- `cli/`: CLI tool (`score_tool`) for inspection and ASCII rendering.
- `web_server/`: Experimental server implementation.

## Global Development Workflow
1. **Formatting**: Always run `cargo fmt` before committing.
2. **Linting**: Check for idiomatic Rust with `cargo clippy`.
3. **Testing**: Run all workspace tests with `cargo test`.
4. **Commits**: Use Conventional Commits (e.g., `feat:`, `fix:`, `refactor:`, `docs:`).

## Commands
- Build all: `cargo build`
- Run CLI: `cargo run -p cli -- <args>`
- Run Server: `cargo run -p web_server`

## Coding Standards
- Avoid `expect()` and `unwrap()`. Migrating toward `thiserror` for error management.

Refer to `guitarpro/CLAUDE.md` for specific library architecture and parsing logic.