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

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (system: let
      overlays = [(import rust-overlay)];
      pkgs = import nixpkgs {inherit system overlays;};
      extensions = with pkgs.vscode-extensions; [
        rust-lang.rust-analyzer
        tamasfe.even-better-toml
        jnoortheen.nix-ide
        mkhl.direnv
        vadimcn.vscode-lldb
        redhat.vscode-yaml
        # Note: If swellaby is not in your nixpkgs channel,
        # you may need to use a community overlay or skip it here.
        swellaby.vscode-rust-test-adapter
      ];
      # Create a custom VSCodium with these extensions
      custom-codium = pkgs.vscode-with-extensions.override {
        vscode = pkgs.vscodium;
        vscodeExtensions = extensions;
      };

      # Define the rust toolchain from your toml
      rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
    in {
      devShells.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          rustToolchain
          cargo-dist
          cargo-nextest
          cargo-machete
          cargo-audit
          cargo-deny
          prek # pre-commit
          gitleaks # The compiled secret scanner or trufflehog or ripgrep
          just # Command runner
          custom-codium
        ];

        # This runs when the shell starts
        shellHook = ''
          echo "🦀 Rust Dev Shell Loaded"
          echo "💡 Tip: Run 'codium .' to start coding with all extensions pre-installed."
          # Automatically install hooks if .git exists
          if [ -d .git ] && command -v prek >/dev/null; then
            prek install
          fi
        '';
      };
    });
}
