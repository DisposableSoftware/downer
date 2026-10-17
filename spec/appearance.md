# Appearance

Colours and type. The theming follows Junk Drawer (DisposableSoftware's file manager), so the two apps look alike under the same config.

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `LOOK-1` | Catppuccin palettes for latte, frappe, macchiato and mocha, with any of the 14 accents. These come from `CONF-1` and `CONF-2`. An unknown flavor falls back to mocha, and an unknown accent to mauve. | planned | `theme::tests`, manual (screenshot per flavor) |
| `LOOK-2` | The window background is the flavor's `base` colour at the opacity set by `CONF-3`. The scrolled view and everything in it, except where `LOOK-6` gives a background, are transparent, so the compositor's wallpaper or blur shows through. | planned | manual (screenshot) |
| `LOOK-3` | Every app style rule is scoped to `window.downer`, so the app never restyles other GTK widgets in the process. | planned | `theme::tests` (every rule in `data/style.css` starts with `window.downer`) |
| `LOOK-4` | User CSS from `CONF-5` is loaded last and can use the palette names (`@base`, `@mauve`, …, `@accent`, `@bg`). | planned | manual (a user `style.css` that sets heading colour to `@red`) |
| `LOOK-5` | With `flavor = gtk` (`CONF-1`), Downer uses the GTK theme's colours instead of a Catppuccin palette: `base` is the theme's window background, `text` its window text colour, and `accent` its accent colour, whatever `accent` says (`CONF-2`). The other shades are mixed from `base` and `text` in Catppuccin's proportions: `surface0` … `subtext1` lie between them, and `mantle` and `crust` are slightly darker than `base`. The 14 accent names keep their mocha values, so user CSS that uses them still works. The theme is read when Downer starts. | planned | `theme::tests` (every colour name `style.css` uses, and the 14 accent names, are defined for every flavor), manual (screenshots with the user's GTK theme and with `GTK_THEME=Default`) |
| `LOOK-6` | Markdown colours: body text is `text`. Links are the accent colour, underlined. Inline code is on `surface0`. Code blocks are on `mantle` with 12px padding and 6px rounded corners. Block quotes have `subtext0` text and a 3px `surface2` bar on the left. Table cells have a 1px `surface1` border, and the header row is bold on `mantle`. Thematic breaks are a 1px `surface1` line. Struck-through text is `overlay1`. Task list boxes are the accent colour when checked. | planned | manual (`scripts/manual/render.sh`, screenshot per flavor) |
| `LOOK-7` | Type: body text uses the GTK font. Code, inline and in blocks, uses the `monospace` family at 0.9 of the body size. Headings are bold, at 2, 1.5, 1.25, 1.1, 1 and 0.9 times the body size for levels 1 to 6. Level 6 is `subtext0`. Levels 1 and 2 have a 1px `surface1` line under them. | planned | manual (`scripts/manual/render.sh`, screenshot) |
| `LOOK-8` | Selected text is drawn in `text` on the accent at 30% opacity, whatever the GTK theme's own selection colours are. For every Catppuccin flavor and accent, `text` has a contrast ratio of at least 3:1 against that selection colour over `base`. | planned | `theme::tests` (all 4 flavors × 14 accents), manual (`GTK_THEME=Default`) |

## Implementation notes

- One `CssProvider` at `STYLE_PROVIDER_PRIORITY_USER`. It concatenates, in order: `data/themes/<flavor>.css` (`@define-color` palette, copied from Junk Drawer), the generated `@accent` and `@bg`, `data/style.css`, and the user's `style.css`.
- `LOOK-5`: named colours resolve across providers. libadwaita-style themes (adw-gtk3, Stylix) define `window_bg_color`, `window_fg_color` and `accent_bg_color`. GTK's built-in theme defines only `theme_bg_color`, `theme_fg_color` and `theme_selected_bg_color`, so the first set falls back to the second.
- The palette hex values come from https://catppuccin.com/palette.
