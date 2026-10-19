# Downer: specification

This folder is the source of truth for what Downer does. The code implements the spec. If the code and the spec disagree, the code is wrong, or the spec must be changed first.

## Files

| File | Prefix | Covers |
|------|--------|--------|
| [product.md](product.md) | `PROD` | Goals, non-goals, platform and stack constraints |
| [view.md](view.md) | `VIEW` | Rendering Markdown in the window |
| [navigation.md](navigation.md) | `NAV` | Opening files and windows, reloading |
| [appearance.md](appearance.md) | `LOOK` | Catppuccin and GTK theme colours, type |
| [keys.md](keys.md) | `KEY` | Keybindings and the shortcut overlay |
| [configuration.md](configuration.md) | `CONF` | `config.ini`, `style.css` |
| [integration.md](integration.md) | `INT` | freedesktop and Nix integration |
| [roadmap.md](roadmap.md) | — | The order of planned work, which `/implement` follows |

Add a file per area as features arrive, and list it here.

## Requirement format

Each requirement is one line in a table:

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `EXAMPLE-1` | One sentence that can be checked. | done | `module::tests` |

- **IDs are permanent.** Do not renumber them or reuse one. When a requirement is dropped, keep its row, set the status to `dropped`, and give the reason.
- **Status** is one of:
  - `planned`: agreed but not built.
  - `partial`: built, but with a known gap. The gap is written in the row (`partial: …`).
  - `done`: built and verified.
  - `dropped`: no longer wanted (`dropped: superseded by …`).
- Write behaviour, not implementation. Keep the "how" short, and put it under **Implementation notes** when it is not obvious or a past attempt failed.
- **Verified by** names the test, or says `manual` with what to check.

`spec_tests::requirement_rows_are_well_formed` checks every row in `spec/*.md`: IDs are unique and the status is one of the four.

`scripts/release-check.sh` reads the status column: a requirement that became `done` or `partial` since the last release means a minor bump. `/release` pushes `main` and acts on that recommendation.

## Machine-checked sections

Tables between `<!-- spec:NAME:begin -->` and `<!-- spec:NAME:end -->` markers can be parsed by `src/spec_tests.rs` (`table(doc, "NAME")`) and compared with the code, so the spec can't silently drift. Use them for lists the code also has, such as default keybindings or config keys, and keep their column layout unchanged.
