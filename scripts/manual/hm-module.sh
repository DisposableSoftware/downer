#! /usr/bin/env nix-shell
#! nix-shell -i bash -p jq grim imagemagick sway
# Spec: INT-6. Usage: scripts/manual/hm-module.sh HOME_MANAGER_FLAKE_REF
# (for example a pinned github:nix-community/home-manager/<rev>). Evaluates a
# home-manager configuration with the module, checks the generated files, and, under
# headless.sh, checks that Downer loads the generated config.ini.
cd "$(dirname "$0")/../.." || exit 1
. scripts/manual/lib.sh
HM=${1:?home-manager flake ref}
OUT=${OUT:-$(mktemp -d)}; mkdir -p "$OUT"

# hm SETTINGS-NIX: builds the files a home-manager config with the module would link,
# into $OUT/<name>, and prints the names of the generated ones.
hm() {
  local expr="
    let
      downer = builtins.getFlake (toString ./.);
      hm = builtins.getFlake \"$HM\";
      pkgs = downer.inputs.nixpkgs.legacyPackages.x86_64-linux;
      c = (hm.lib.homeManagerConfiguration {
        inherit pkgs;
        modules = [ downer.homeManagerModules.default {
          home = { username = \"test\"; homeDirectory = \"/home/test\"; stateVersion = \"25.11\"; };
          # home-manager only writes mimeapps.list when it manages it.
          xdg.mimeApps.enable = true;
          programs.downer = { enable = true; } // ($1);
        } ];
      }).config;
      files = pkgs.lib.filterAttrs (n: _: pkgs.lib.hasPrefix \"downer/\" n) c.xdg.configFile;
    in pkgs.linkFarm \"downer-hm\" (
      pkgs.lib.mapAttrsToList (n: f: { name = baseNameOf n; path = f.source; }) files
      ++ pkgs.lib.optional (c.xdg.mimeApps.defaultApplications != { })
           { name = \"mimeapps.list\"; path = c.xdg.configFile.\"mimeapps.list\".source; }
      ++ [ { name = \"has-package\"; path = pkgs.writeText \"p\" (builtins.toJSON (builtins.elem
             downer.packages.x86_64-linux.default.outPath (map (p: p.outPath) c.home.packages))); } ])"
  rm -rf "$OUT/hm"; nix build --impure --no-link --print-out-paths --expr "$expr" | xargs -I{} cp -rL {} "$OUT/hm"
  chmod -R u+w "$OUT/hm"
}

hm '{ }'
check "defaults: package installed" true "$(cat "$OUT/hm/has-package")"
check "defaults: nothing generated" "has-package" "$(ls "$OUT/hm" | tr '\n' ' ' | xargs)"

hm '{ settings.appearance = { flavor = "latte"; opacity = 0.9; }; style = "window.downer .h1 { color: @red; }"; defaultMarkdownViewer = true; }'
check "config.ini generated" 1 "$(grep -c '^flavor=latte$' "$OUT/hm/config.ini")"
check "style.css generated" "window.downer .h1 { color: @red; }" "$(cat "$OUT/hm/style.css")"
check "mimeapps: text/markdown" 1 "$(grep -c '^text/markdown=io.github.DisposableSoftware.Downer.desktop$' "$OUT/hm/mimeapps.list")"
check "mimeapps: text/x-markdown" 1 "$(grep -c '^text/x-markdown=io.github.DisposableSoftware.Downer.desktop$' "$OUT/hm/mimeapps.list")"
cp "$OUT/hm/config.ini" "$OUT/config.ini"; echo "config.ini:"; sed 's/^/  /' "$OUT/config.ini"

if [ -n "${DOWNER_HEADLESS:-}" ]; then
  # Downer reads the generated file through a symlink, as home-manager links it.
  CFG=$(mktemp -d); mkdir "$CFG/downer"; ln -s "$OUT/config.ini" "$CFG/downer/config.ini"
  printf '# Latte\n' >"$CFG/t.md"
  use_workspace
  CFG=$CFG launch "$CFG/t.md"; shot "$OUT/latte.png"
  # latte base #eff1f5 at opacity 0.9 over headless sway's black background.
  check "loads flavor and opacity" "srgb(215,217,220)" "$(magick "$OUT/latte.png" -format '%[pixel:p{5,5}]' info:)"
  check "symlinked config.ini left alone" 1 "$([ -L "$CFG/downer/config.ini" ] && echo 1)"
fi
exit $FAILS
