{
  description = "Launcher-driven monitor manager for Hyprland";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        runtimeDeps = with pkgs; [ hyprland ];

        buildInputs = with pkgs; [ rustToolchain ] ++ runtimeDeps;

        nativeBuildInputs = with pkgs; [ pkg-config makeWrapper ];

      in {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "monitormenu";
          version = "0.1.0";

          src = ./.;

          cargoLock = { lockFile = ./Cargo.lock; };

          inherit nativeBuildInputs buildInputs;

          postInstall = ''
            wrapProgram $out/bin/monitormenu \
              --prefix PATH : ${pkgs.lib.makeBinPath runtimeDeps}
          '';

          meta = with pkgs.lib; {
            description = "Launcher-driven monitor manager for Hyprland";
            license = licenses.gpl3;
            maintainers = [ ];
            platforms = platforms.linux;
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = buildInputs
            ++ (with pkgs; [ cargo-watch rust-analyzer ]);

          inherit nativeBuildInputs;

          shellHook = ''
            echo "monitormenu development environment"
            echo "cargo build  - Build the project"
            echo "cargo run    - Run the project"
            echo "cargo test   - Run tests"
          '';
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/monitormenu";
        };
      });
}
