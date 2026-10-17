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
          # INT-5: the module installs the package, and sets the Markdown default only on request.
          nixos-module =
            let
              eval =
                extra:
                (nixpkgs.lib.nixosSystem {
                  inherit (pkgs.stdenv.hostPlatform) system;
                  modules = [
                    self.nixosModules.default
                    {
                      boot.loader.grub.enable = false;
                      fileSystems."/" = {
                        device = "none";
                        fsType = "tmpfs";
                      };
                      system.stateVersion = "25.11";
                      programs.downer = {
                        enable = true;
                      }
                      // extra;
                    }
                  ];
                }).config;
              plain = eval { };
              default = eval { defaultMarkdownViewer = true; };
              # NixOS builds mimeapps.list, so its content is only readable in a derivation.
              mimeapps = c: c.environment.etc."xdg/mimeapps.list".source or "/dev/null";
            in
            assert builtins.elem pkg.outPath (map (p: p.outPath) plain.environment.systemPackages);
            pkgs.runCommand "downer-nixos-module" { } ''
              if grep -q Downer ${mimeapps plain}; then exit 1; fi
              grep -qx 'text/markdown=io.github.DisposableSoftware.Downer.desktop' ${mimeapps default}
              grep -qx 'text/x-markdown=io.github.DisposableSoftware.Downer.desktop' ${mimeapps default}
              touch $out
            '';
          fmt = pkgs.runCommand "downer-fmt" { nativeBuildInputs = [ pkgs.rustfmt ]; } ''
            cd ${./.}
            rustfmt --check --edition 2024 src/main.rs
            touch $out
          '';
        }
      );

      formatter = forAll (pkgs: pkgs.nixfmt);

      nixosModules.default = import ./nix/nixos-module.nix self;
      homeManagerModules.default = import ./nix/hm-module.nix self;

      overlays.default = final: _: { downer = final.callPackage ./nix/package.nix { }; };
    };
}
