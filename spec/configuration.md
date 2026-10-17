# Configuration

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `CONF-1` | `[appearance] flavor`: latte, frappe, macchiato or mocha, or `gtk` to follow the GTK theme (`LOOK-5`). | planned | `spec_tests`, `theme::tests` |
| `CONF-2` | `[appearance] accent`: any Catppuccin accent name. It is ignored when `flavor` is `gtk`, which uses the theme's accent (`LOOK-5`). | planned | `spec_tests`, `theme::tests` |
| `CONF-3` | `[appearance] opacity`: the window background alpha, clamped to 0.0–1.0. | planned | `spec_tests`, `config::tests` |
| `CONF-4` | Settings are read from `$XDG_CONFIG_HOME/downer/config.ini` (GLib key-file / INI syntax). A missing file is not an error. A file that can't be read or parsed is warned about on stderr, and the defaults are used. Every key is optional, and a value of the wrong type is ignored. | planned | `config::tests` |
| `CONF-5` | Extra GTK CSS is read from `$XDG_CONFIG_HOME/downer/style.css` (see `LOOK-4`). A missing file is not an error. | planned | manual (`LOOK-4`) |
| `CONF-6` | Changes to `config.ini` and `style.css` apply when Downer is restarted. | planned | — |
| `CONF-7` | `data/config.example.ini` lists every setting with its default and a comment on its allowed values. It is installed to `share/doc/downer/`. | planned | `spec_tests::example_config_matches_spec` |
| `CONF-8` | When Downer starts and `config.ini` doesn't exist in `$XDG_CONFIG_HOME/downer/`, it creates the file (and the folder) from the example (`CONF-7`), with every setting commented out, so the file changes nothing until a line is uncommented. A file that exists is never written to, whether it is a regular file, a symlink (as with dotfiles or home-manager) or a dangling symlink. If the file can't be created, that is warned about once on stderr and Downer starts normally. | planned | `config::tests` (temp folder: a missing file is created and loads as the defaults; uncommenting a line takes effect; existing files and symlinks are left byte-for-byte unchanged; an unwritable folder only warns) |

## Defaults

<!-- spec:config:begin -->
| Section | Key | Default |
|---------|-----|---------|
| `appearance` | `flavor` | `mocha` |
| `appearance` | `accent` | `mauve` |
| `appearance` | `opacity` | `0.85` |
<!-- spec:config:end -->

## Implementation notes

- To add a key, edit the defaults table, `Config`, `spec_tests` and `data/config.example.ini`. The spec tests fail until they agree.
