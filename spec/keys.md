# Keybindings

Not modal. Keys with Ctrl are app-wide; plain keys are local to the window. The names and accelerators below are the defaults, and `CONF-10` lets the user change them. Style guide: `docs/interaction.md`.

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `KEY-1` | Window actions have keybindings, declared once in a table in the code and in the table below: close the window, quit, and show the shortcut overlay. Moving through the sidebar tree and scrolling keep the keys GTK gives them (`NAV-8`). Keys without Ctrl, Alt or Super don't fire while a text entry has focus. | planned | `keys::tests`, manual (each key on Hyprland) |
| `KEY-2` | `?` opens the shortcut overlay: a centred panel over the window, 12px radius, on `crust` at 90%, with a description in `text` and its keys in the accent for every binding, built from the live table so remaps show up. Any key press that isn't a modifier, or a click, closes it and does nothing else. It fits at the default window size without scrolling. | planned | manual (`scripts/manual/help.sh` screenshot, and a remap shows in it) |
| `KEY-3` | The README has a Keyboard section with a table of the default keys, and `data/keybindings.example.ini` lists every binding, commented out (`CONF-10`). | planned | code review (README, example file) |

## Default keybindings

Filled in when `KEY-1` is built. The first set is below. Descriptions start with a verb, except "Keyboard shortcuts".

| Name | Description | Default |
|------|-------------|---------|
| `close-window` | Close window | `<Ctrl>w;q` |
| `quit` | Quit | `<Ctrl>q` |
| `show-help` | Keyboard shortcuts | `question` |
