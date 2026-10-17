# Product

A minimal GTK 4 Markdown viewer for Linux, built for Hyprland. Display name **Downer**, binary `downer`, app ID `io.github.DisposableSoftware.Downer`.

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `PROD-1` | Lightweight and minimal: few dependencies, no libadwaita, fast startup. | done (ongoing constraint) | code review |
| `PROD-2` | Follow a freedesktop/XDG spec wherever one exists for the behaviour. | done (ongoing constraint) | code review |
| `PROD-3` | Launching `downer` with no arguments opens a window titled "Downer". What it shows is in `NAV-3`, and what files open is in `NAV-1`. | done | `scripts/manual/window.sh` |

## Non-goals

- No libadwaita. It fights custom theming.

## Stack constraints

- Rust and gtk4-rs (`gtk4` 0.11 with feature `gnome_49`), built with Nix.
- Use GIO/GLib for anything they provide (XDG dirs, MIME, file monitoring) rather than reimplementing it or adding a crate.
- Ask before adding a crate. Approved: `pulldown-cmark` (Markdown parsing, `VIEW-3`…`VIEW-5`), because GTK has no parser.
- File I/O is async through GIO. The main loop never blocks on I/O.
