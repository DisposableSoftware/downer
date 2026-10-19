# Roadmap

The order of approved work: `planned` requirements, and `partial` ones with a known bug from `/bug`. Change it here when priorities change. `/implement` works through this file: the verification backlog first, then the requirements in the order listed.

## Verification backlog

Built requirements that are not fully verified. Clear these before starting new planned work.


## Planned

In order. Each item comes from the [style guide](https://github.com/DisposableSoftware/style-guide); the page is named.

1. `LOOK-14`: selection at 35%, contrast test at 35% (`docs/palette.md`).
2. `LOOK-15`: sidebar focus outline at `alpha(@accent, 0.8)` (`docs/components.md`).
3. `LOOK-16`: rename the Markdown `heading` class (`docs/known-gaps.md`).
4. `LOOK-17`: fixed-width overlay scrollbars in the main view (`docs/components.md`).
5. `CONF-9`: `[window] width` and `height` (`docs/layout.md`). Update the defaults table, `Config`, `config.example.ini` and `spec_tests` together.
6. `CONF-10`, `KEY-1`, `KEY-2`, `KEY-3`: `keybindings.ini`, the key table, the `?` overlay and the README Keyboard section (`docs/interaction.md`). Build `CONF-10` first, and confirm the default key set with the user before coding.
7. `INT-8`: AppStream metainfo (`docs/layout.md`).

No rows for these, because they change no behaviour: make `theme::ACCENTS` private, and take the font-rendering hook and the shared sidebar and scrollbar CSS from Junk Drawer (`docs/known-gaps.md`, duplicated theme code). Do them alongside `LOOK-15` and `LOOK-17`.
