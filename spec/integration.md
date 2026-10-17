# Integration

freedesktop and Nix integration.

## Requirements

| ID | Spec | Requirement | Status | Verified by |
|----|------|-------------|--------|-------------|
| `INT-1` | Base Directory | Config paths come from `glib::user_config_dir()`. They are never hard-coded as `~/.config`. | planned | code review |
| `INT-2` | Desktop Entry | `io.github.DisposableSoftware.Downer.desktop`: `Name=Downer`, `Exec=downer %U`, `StartupWMClass=io.github.DisposableSoftware.Downer`, `MimeType=text/markdown;text/x-markdown;`, `Categories=Utility;TextTools;Viewer;`. With it installed, Downer is offered for Markdown files, and `xdg-open file.md` opens Downer when it is the default handler for `text/markdown`. Downer never makes itself the default. It passes `desktop-file-validate`. | planned | manual (`desktop-file-validate`; `XDG_DATA_DIRS` with `./result/share` and `gio open` on a `.md` file in a temp dir) |
| `INT-3` | — | The package installs `bin/downer` (wrapped by `wrapGAppsHook4`), the desktop entry (`INT-2`) and the example config (`CONF-7`). `nix build` runs the unit tests and the spec tests. | planned | manual (`ls -R result/share`) |
