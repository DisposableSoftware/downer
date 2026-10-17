{
  lib,
  rustPlatform,
  pkg-config,
  wrapGAppsHook4,
  gtk4,
}:
let
  manifest = (lib.importTOML ../Cargo.toml).package;
in
rustPlatform.buildRustPackage {
  pname = manifest.name;
  inherit (manifest) version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.unions [
      ../Cargo.toml
      ../Cargo.lock
      ../src
      ../data
      ../spec
    ];
  };
  cargoLock.lockFile = ../Cargo.lock;

  nativeBuildInputs = [
    pkg-config
    wrapGAppsHook4
  ];
  buildInputs = [ gtk4 ];

  postInstall = ''
    install -Dm644 data/io.github.DisposableSoftware.Downer.desktop -t $out/share/applications
    install -Dm644 data/config.example.ini -t $out/share/doc/downer
  '';

  meta = {
    description = "Minimal GTK4 Markdown viewer";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
    mainProgram = "downer";
  };
}
