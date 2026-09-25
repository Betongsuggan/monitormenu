{ lib, rustPlatform }:

# hyprctl / `niri msg` are taken from the session's PATH: the running
# compositor's own client always matches its IPC, and wrapping would pull
# whole compositors into the closure
rustPlatform.buildRustPackage {
  pname = "monitormenu";
  version = "0.1.0";

  src = lib.fileset.toSource {
    root = ./.;
    fileset = lib.fileset.unions [
      ./Cargo.toml
      ./Cargo.lock
      ./src
    ];
  };

  cargoLock.lockFile = ./Cargo.lock;

  meta = {
    description = "Launcher-driven monitor manager for Wayland compositors";
    homepage = "https://github.com/Betongsuggan/monitormenu";
    license = lib.licenses.gpl3Only;
    mainProgram = "monitormenu";
    platforms = lib.platforms.linux;
  };
}
