# Configuration

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `CONF-1` | `[appearance] flavor`: latte, frappe, macchiato or mocha, or `gtk` to follow the GTK theme (`LOOK-5`). | done | `spec_tests::config_defaults_match_spec`, `config::tests`, `theme::tests` |
| `CONF-2` | `[appearance] accent`: any Catppuccin accent name. It is ignored when `flavor` is `gtk`, which uses the theme's accent (`LOOK-5`). | done | `spec_tests`, `theme::tests::gtk_flavor_uses_the_theme_accent` |
| `CONF-3` | `[appearance] opacity`: the window background alpha, clamped to 0.0–1.0. | done | `spec_tests`, `config::tests::reads_settings_and_ignores_bad_values` |
| `CONF-4` | Settings are read from `$XDG_CONFIG_HOME/downer/config.ini` (GLib key-file / INI syntax). A missing file is not an error. A file that can't be read or parsed is warned about on stderr, and the defaults are used. Every key is optional, and a value of the wrong type is ignored. | done | `config::tests::reads_settings_and_ignores_bad_values`, `config::tests::broken_file_gives_defaults` |
| `CONF-5` | Extra GTK CSS is read from `$XDG_CONFIG_HOME/downer/style.css` (see `LOOK-4`). A missing file is not an error. | done | `scripts/manual/theme.sh` |
| `CONF-6` | Changes to `config.ini` and `style.css` apply when Downer is restarted. | done | code review (config and CSS are read once, at startup) |
| `CONF-7` | `data/config.example.ini` lists every setting with its default and a comment on its allowed values. It is installed to `share/doc/downer/`. | done | `spec_tests::example_config_matches_spec`, `scripts/manual/xdg-open.sh` (installed) |
| `CONF-8` | When Downer starts and `config.ini` doesn't exist in `$XDG_CONFIG_HOME/downer/`, it creates the file (and the folder) from the example (`CONF-7`), with every setting commented out, so the file changes nothing until a line is uncommented. A file that exists is never written to, whether it is a regular file, a symlink (as with dotfiles or home-manager) or a dangling symlink. If the file can't be created, that is warned about once on stderr and Downer starts normally. | done | `config::tests` (a missing file is created with every value commented out and loads as the defaults; uncommenting takes effect; existing files and dangling symlinks are left alone; an unwritable folder only warns) |

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
