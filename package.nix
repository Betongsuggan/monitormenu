{
  lib,
  rustPlatform,
  makeWrapper,
  wl-clipboard,
  libnotify,
}:

# hyprctl / `niri msg` are taken from the session's PATH: the running
# compositor's own client always matches its IPC, and wrapping would pull
# whole compositors into the closure. wl-copy and notify-send (for "Show as
# Nix") are small and come with the package
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

  nativeBuildInputs = [ makeWrapper ];

  postInstall = ''
    wrapProgram $out/bin/monitormenu \
      --suffix PATH : ${
        lib.makeBinPath [
          wl-clipboard
          libnotify
        ]
      }
  '';

  meta = {
    description = "Launcher-driven monitor manager for Wayland compositors";
    homepage = "https://github.com/Betongsuggan/monitormenu";
    license = lib.licenses.gpl3Only;
    mainProgram = "monitormenu";
    platforms = lib.platforms.linux;
  };
}
