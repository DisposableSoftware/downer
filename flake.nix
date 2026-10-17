{
  description = "Downer — a minimal GTK4 Markdown viewer";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAll = f: nixpkgs.lib.genAttrs systems (system: f nixpkgs.legacyPackages.${system});
    in
    {
      packages = forAll (pkgs: {
        default = pkgs.callPackage ./nix/package.nix { };
      });

      devShells = forAll (pkgs: {
        default = pkgs.mkShell {
          inputsFrom = [ self.packages.${pkgs.stdenv.hostPlatform.system}.default ];
          packages = with pkgs; [
            clippy
            rustfmt
            rust-analyzer
          ];
        };
      });

      # `nix flake check` runs the tests (by building the package), clippy and rustfmt.
      checks = forAll (
        pkgs:
        let
          pkg = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
        in
        {
          tests = pkg;
          clippy = pkg.overrideAttrs (old: {
            pname = "downer-clippy";
            nativeBuildInputs = old.nativeBuildInputs ++ [ pkgs.clippy ];
            buildPhase = "cargo clippy --all-targets --offline -- -D warnings";
            doCheck = false;
            installPhase = "touch $out";
            postInstall = "";
            dontFixup = true;
          });
          fmt = pkgs.runCommand "downer-fmt" { nativeBuildInputs = [ pkgs.rustfmt ]; } ''
            cd ${./.}
            rustfmt --check --edition 2024 src/main.rs
            touch $out
          '';
        }
      );

      formatter = forAll (pkgs: pkgs.nixfmt);

      overlays.default = final: _: { downer = final.callPackage ./nix/package.nix { }; };
    };
}
