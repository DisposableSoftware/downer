# Integration

freedesktop and Nix integration.

## Requirements

| ID | Spec | Requirement | Status | Verified by |
|----|------|-------------|--------|-------------|
| `INT-1` | Base Directory | Config paths come from `glib::user_config_dir()`. They are never hard-coded as `~/.config`. | done | code review (`config::dir`) |
| `INT-2` | Desktop Entry | `io.github.DisposableSoftware.Downer.desktop`: `Name=Downer`, `Exec=downer %U`, `StartupWMClass=io.github.DisposableSoftware.Downer`, `MimeType=text/markdown;text/x-markdown;`, `Categories=Utility;TextTools;Viewer;`. With it installed, Downer is offered for Markdown files, and `xdg-open file.md` opens Downer when it is the default handler for `text/markdown`. Downer never makes itself the default. It passes `desktop-file-validate`. | done | `scripts/manual/xdg-open.sh` (headless: `desktop-file-validate`; with the entry in a temp data dir and its `mimeinfo.cache`, `gio mime` offers Downer for `text/markdown`, and `gio open` on a `.md` file opens it) |
| `INT-3` | — | The package installs `bin/downer` (wrapped by `wrapGAppsHook4`), the desktop entry (`INT-2`) and the example config (`CONF-7`). `nix build` runs the unit tests and the spec tests. | done | `scripts/manual/xdg-open.sh`; `nix build` runs the tests |

## Implementation notes

- GIO finds the apps for a MIME type through each data dir's `applications/mimeinfo.cache`, not by reading every desktop entry's `MimeType`. NixOS and home-manager profiles generate the cache, so the package doesn't ship one; `scripts/manual/xdg-open.sh` runs `update-desktop-database` on a temp copy, as a profile would.
