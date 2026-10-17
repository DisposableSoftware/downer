# Integration

freedesktop and Nix integration.

## Requirements

| ID | Spec | Requirement | Status | Verified by |
|----|------|-------------|--------|-------------|
| `INT-1` | Base Directory | Config paths come from `glib::user_config_dir()`. They are never hard-coded as `~/.config`. | done | code review (`config::dir`) |
| `INT-2` | Desktop Entry | `io.github.DisposableSoftware.Downer.desktop`: `Name=Downer`, `Exec=downer %U`, `StartupWMClass=io.github.DisposableSoftware.Downer`, `MimeType=text/markdown;text/x-markdown;`, `Categories=Utility;TextTools;Viewer;`. With it installed, Downer is offered for Markdown files, and `xdg-open file.md` opens Downer when it is the default handler for `text/markdown`. Downer never makes itself the default. It passes `desktop-file-validate`. | done | `scripts/manual/xdg-open.sh` (headless: `desktop-file-validate`; with the entry in a temp data dir and its `mimeinfo.cache`, `gio mime` offers Downer for `text/markdown`, and `gio open` on a `.md` file opens it) |
| `INT-3` | — | The package installs `bin/downer` (wrapped by `wrapGAppsHook4`), the desktop entry (`INT-2`) and the example config (`CONF-7`). `nix build` runs the unit tests and the spec tests. | done | `scripts/manual/xdg-open.sh`; `nix build` runs the tests |
| `INT-4` | Nix flake | The flake exports `packages.<system>.default`, `devShells.<system>.default`, `overlays.default` (attribute `downer`), `nixosModules.default` and `homeManagerModules.default`, for x86_64-linux and aarch64-linux. | planned | `nix flake show` |
| `INT-5` | NixOS module | `programs.downer`: `enable` installs the package system-wide, so the desktop entry is in the system profile and Downer is offered for Markdown files (`INT-2`); `package` (default: this flake's package); `defaultMarkdownViewer` (default `false`) makes Downer the default for `text/markdown` and `text/x-markdown` in `xdg.mime.defaultApplications`. | planned | `checks.<system>.nixos-module` (a NixOS system evaluated with the module: the package is in `environment.systemPackages`, and `/etc/xdg/mimeapps.list` names Downer only when `defaultMarkdownViewer` is set) |
| `INT-6` | home-manager module | `programs.downer`: `enable` installs the package in the user's profile; `package`; `settings`, an attribute set of sections, written to `$XDG_CONFIG_HOME/downer/config.ini` when not empty (`CONF-4`); `style`, lines of CSS, written to `style.css` when not empty (`CONF-5`); `defaultMarkdownViewer` (default `false`), as in `INT-5`, through `xdg.mimeApps.defaultApplications`. A generated `config.ini` is a symlink, so `CONF-8` leaves it alone. | planned | manual (evaluate a home-manager configuration with the module, using the `home-manager` input of a NixOS config: the generated `config.ini` parses with `settings = { appearance = { flavor = "latte"; opacity = 0.9; }; }` and loads as those values; `style.css` and `mimeapps.list` contents match; nothing is generated with the defaults) |
| `INT-7` | — | `nix flake check` runs the tests, `cargo clippy --all-targets -- -D warnings`, `rustfmt --check` and the NixOS module check (`INT-5`). | planned | `nix flake check` |

## Implementation notes

- `INT-6`: home-manager refuses to replace a file it didn't create. If Downer already made a commented `config.ini` on first run (`CONF-8`), setting `settings` later makes activation fail until that file is removed (or `home-manager.backupFileExtension` is set). The README says so.
- The flake has no home-manager input, to keep the lock file small, so `nix flake check` can't evaluate the home-manager module.
- GIO finds the apps for a MIME type through each data dir's `applications/mimeinfo.cache`, not by reading every desktop entry's `MimeType`. NixOS and home-manager profiles generate the cache, so the package doesn't ship one; `scripts/manual/xdg-open.sh` runs `update-desktop-database` on a temp copy, as a profile would.
