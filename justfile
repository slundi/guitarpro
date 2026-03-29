set show-recipe-help := true

# --- Variables ---
binary_name := "rust_template" # Change this to your project name

# --- Default ---
[help]
default:
    @just --list

# --- Development ---

# Build the project in debug mode
build:
    cargo build

# Run the project
run *args:
    cargo run -- {{args}}

# Watch for changes and run (requires cargo-watch)
watch:
    cargo watch -x run

# --- Quality Control ---

# Run all tests
test:
    cargo nextest run

# Run a full health check (Vulnerabilities, Unused Deps, Licenses)
health-check:
    cargo audit
    cargo machete
    cargo deny check

# Run clippy with strict warnings
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# Format all code
fmt:
    cargo fmt --all

# Run all pre-commit hooks manually on all files
check:
    prek run --all-files

# Force update hooks
update-hooks:
    prek autoupdate

# Run gitleaks to scan for secrets
scan-secrets:
    gitleaks detect --verbose --redact

# --- Cleanup ---

# Clean build artifacts
clean:
    cargo clean

# --- CI Simulation ---

# Run the full pipeline as it would run in CI
ci: fmt lint test
    @echo "✅ All checks passed!"
