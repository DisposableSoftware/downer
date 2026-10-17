# Navigation

Opening files and windows.

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `NAV-1` | `downer [FILE\|URI…]` opens each argument in a window of its own. A path is relative to the current folder. A URI is anything GIO can read (`file:` and, with GVfs, others). When Downer is already running, the files open as new windows of the running instance, and the launching process exits. | planned | `scripts/manual/open.sh` (two files give two windows; a second launch while running adds a window in the same process) |
| `NAV-2` | A window showing a file is titled `<file name> — Downer`. | planned | `scripts/manual/open.sh` |
| `NAV-3` | Without arguments, Downer opens one window titled "Downer" that shows a centred hint: "No file open", and below it, dimmed, "Open a Markdown file with downer FILE". | planned | `scripts/manual/open.sh` (screenshot) |
| `NAV-4` | A file that can't be read (missing, no permission, a folder) shows a centred message in its window with the file name and GIO's error text. Downer keeps running. Bytes that are not valid UTF-8 are shown as U+FFFD, and the rest of the file is rendered. | planned | `render::tests` (invalid UTF-8), `scripts/manual/open.sh` (missing file, folder) |
| `NAV-5` | When the open file changes on disk, including when an editor saves it by replacing it, the window renders it again within one second, and keeps the vertical scroll position (clamped to the new length). When the file is deleted, the window keeps showing the last version, and renders the file again if it is created again at the same path. | planned | `scripts/manual/reload.sh` (an append and an atomic rename are both shown; the scroll position is kept; delete then recreate) |

## Implementation notes

- `NAV-1`: `gio::ApplicationFlags::HANDLES_OPEN`; `connect_open` makes one window per `gio::File`, and `connect_activate` (no arguments) makes the `NAV-3` window.
- `NAV-5`: a `gio::FileMonitor` on the file, with `WATCH_MOVES`, so a rename onto the path counts as a change. Reload after a short delay, so a burst of events from one save gives one reload.
