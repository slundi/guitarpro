# When adding new tools to this template, always check the official registry to
# find the correct attribute name:
# * Search Tool: NixOS Package Search
# * Usage: If you find ripgrep, simply add pkgs.ripgrep to the buildInputs in your flake.nix.
{
  description = "Rust Template with automatic pre-commit hooks";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        inherit (pkgs) lib;

        vscodeExtensions = with pkgs.vscode-extensions; [
          rust-lang.rust-analyzer
          tamasfe.even-better-toml
          jnoortheen.nix-ide
          mkhl.direnv
          vadimcn.vscode-lldb
          redhat.vscode-yaml
          # ryanluker.vscode-coverage-gutters
          # Note: If swellaby is not in your nixpkgs channel,
          # you may need to use a community overlay or skip it here.
          # swellaby.vscode-rust-test-adapter
          # vscode-extensions.nefrob.vscode-just-syntax
        ];

        # A read-only symlink farm `<publisher>.<name> -> /nix/store/...`,
        # the exact layout VSCodium expects in an `--extensions-dir`.
        flakeExtensionsFarm =
          pkgs.buildEnv {
            name = "guitar-io-vscode-extensions";
            paths = vscodeExtensions;
            pathsToLink = [ "/share/vscode/extensions" ];
          }
          + "/share/vscode/extensions";

        # `codium` shim: keeps the system VSCodium binary, but launches it against
        # a merged extension directory = the user's own extensions + whatever this
        # flake provides on top. The user's copy of an extension always wins, so
        # nothing they installed is shadowed or downgraded.
        codiumShim = pkgs.writeShellScriptBin "codium" ''
          set -uo pipefail

          # 1. Find the real VSCodium, skipping this shim.
          self="$(command readlink -f "$0" 2>/dev/null || echo "$0")"
          real_codium=""
          for candidate in $(type -ap codium); do
            resolved="$(command readlink -f "$candidate" 2>/dev/null || echo "$candidate")"
            if [ "$resolved" != "$self" ]; then
              real_codium="$candidate"
              break
            fi
          done

          if [ -z "$real_codium" ]; then
            echo "codium: no system VSCodium found on PATH (this shim only adds extensions to an existing install)." >&2
            exit 127
          fi

          merged_dir="''${XDG_CACHE_HOME:-$HOME/.cache}/guitar-io/codium-extensions"

          (
            export PATH=${
              lib.makeBinPath (
                with pkgs;
                [
                  coreutils
                  jq
                  gnugrep
                  findutils
                ]
              )
            }:$PATH
            mkdir -p "$merged_dir"

            # Drop the symlinks we manage; keep real directories, which is where
            # extensions the user installs from inside the editor end up.
            find "$merged_dir" -maxdepth 1 -type l -delete
            rm -f "$merged_dir/extensions.json"

            # Where does the system install keep its extensions? A NixOS
            # `vscode-with-extensions` wrapper bakes in an immutable store path.
            user_dirs=()
            if [ -n "''${VSCODE_EXTENSIONS:-}" ]; then
              user_dirs+=("$VSCODE_EXTENSIONS")
            fi
            baked_dir="$(grep -oP -- '--extensions-dir\s+\K\S+' "$(readlink -f "$real_codium")" 2>/dev/null | head -n1 || true)"
            if [ -n "$baked_dir" ]; then
              user_dirs+=("$baked_dir")
            fi
            user_dirs+=("$HOME/.vscode-oss/extensions")

            declare -A seen_ids=()
            records=()

            collect() {
              local base="$1" link_it="$2" entry manifest target line extension_id version
              [ -d "$base" ] || return 0
              for entry in "$base"/*/; do
                manifest="$entry/package.json"
                [ -f "$manifest" ] || continue
                target="$(readlink -f "$entry")"
                line="$(jq -r '[((.publisher // "") + "." + (.name // "")), (.version // "0.0.0")] | @tsv' "$manifest" 2>/dev/null)" || continue
                extension_id="''${line%%$'\t'*}"
                version="''${line##*$'\t'}"
                [ "$extension_id" = "." ] && continue
                local lowered="''${extension_id,,}"
                [ -n "''${seen_ids[$lowered]:-}" ] && continue
                seen_ids[$lowered]=1
                if [ "$link_it" = yes ]; then
                  ln -sfn "$target" "$merged_dir/$extension_id"
                fi
                records+=("$extension_id"$'\t'"$version"$'\t'"$target")
              done
            }

            # Priority: already-materialised installs, then the system set, then us.
            collect "$merged_dir" no
            for base in "''${user_dirs[@]}"; do
              collect "$base" yes
            done

            added_before=''${#records[@]}
            collect "${flakeExtensionsFarm}" yes
            added=$((''${#records[@]} - added_before))

            # VSCodium reads this manifest for extension metadata; mirror the
            # format `vscode-with-extensions` generates.
            printf '%s\n' "''${records[@]}" |
              jq -R -s 'split("\n")
                | map(select(length > 0) | split("\t"))
                | map({
                    identifier: {id: .[0], uuid: ""},
                    version: .[1],
                    location: {"$mid": 1, fsPath: .[2], path: .[2], scheme: "file"},
                    relativeLocation: .[0],
                    metadata: {
                      id: "",
                      installedTimestamp: 0,
                      isApplicationScoped: false,
                      isPreReleaseVersion: false,
                      preRelease: false,
                      publisherDisplayName: (.[0] | split(".")[0]),
                      publisherId: "",
                      targetPlatform: "undefined",
                      updated: false
                    }
                  })' > "$merged_dir/extensions.json"

            if [ "$added" -gt 0 ]; then
              echo "codium: added $added extension(s) from the dev shell." >&2
            fi
          )

          # A baked-in `--extensions-dir` loses to ours: last occurrence wins.
          exec "$real_codium" --extensions-dir "$merged_dir" "$@"
        '';

        # Define the rust toolchain from your toml
        rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

        # Read project metadata from Cargo.toml for reuse below
        cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
      in
      {
        # `nix build` and `nix run` — builds the release binary
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = cargoToml.package.name;
          version = cargoToml.package.version;
          src = ./.;
          # Cargo.lock must be committed (true for executables — see .gitignore)
          cargoLock.lockFile = ./Cargo.lock;
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustToolchain
            cargo-dist
            cargo-nextest
            cargo-cross
            cargo-machete
            cargo-audit
            cargo-deny
            cargo-outdated # Reports dependencies with newer releases available
            cargo-llvm-cov # Coverage instrumentation via LLVM
            grcov # Coverage report formatter (lcov, html, cobertura)
            jaq # JSON processor (used by coverage-check)
            prek # pre-commit
            gitleaks # The compiled secret scanner or trufflehog or ripgrep
            just # Command runner
            bacon # Background checker (cargo check/clippy/test on save)
            git-cliff # Changelog generator from conventional commits
            mold # Fast linker (referenced in .cargo/config.toml)
            nil # Nix language server (referenced in .vscode/settings.json)
            nodejs # Required by pnpm and Vite
            pnpm # Frontend package manager (web_server/frontend/)
            # `codium` shim — a no-op unless a system VSCodium is on PATH.
            codiumShim
          ];

          # This runs when the shell starts
          shellHook = ''
            echo "Rust Dev Shell Loaded"

            # Nothing is copied into the user's profile: `codium` is a shim that
            # execs the system VSCodium against a merged extension directory.
            if [ -n "$(type -ap codium | sed -n 2p)" ]; then
              echo "Tip: Run 'codium .' — your extensions plus the ones this flake pins."
            else
              echo "Note: no system VSCodium on PATH — 'codium' will not work."
            fi

            # Automatically install hooks if .git exists
            if [ -d .git ] && command -v prek >/dev/null; then
              prek install
            fi
          '';
        };
      }
    );
}
