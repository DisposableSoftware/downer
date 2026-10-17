# View

How a Markdown file is shown in the window.

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `VIEW-1` | A window with a file open (`NAV-1`) shows the file's Markdown rendered as formatted text, never as raw source, in one column that scrolls vertically. | done | `render::tests`, `scripts/manual/render.sh` (headless screenshots of every flavor) |
| `VIEW-2` | The text column is at most 800 logical px wide and centred in the window, with at least 24px of padding on each side. Narrower windows wrap text to fit. Only code blocks (`VIEW-3`) and tables (`VIEW-5`) scroll sideways, each on its own; the page never does. | done | `scripts/manual/render.sh` (at 1920px the column is 800px and centred; at 500px text wraps, the window stays 500px wide, and only the code block is cut off and scrolls) |
| `VIEW-3` | CommonMark blocks are rendered: ATX and setext headings at six levels, paragraphs, block quotes (nested too), bullet and ordered lists (nested, ordered lists keep their start number), fenced and indented code blocks, and thematic breaks. Code blocks keep their whitespace and line breaks and do not wrap. | done | `render::tests::blocks`, `scripts/manual/render.sh` (screenshot) |
| `VIEW-4` | CommonMark inlines are rendered: emphasis, strong emphasis, inline code, links, autolinks, hard line breaks, and soft line breaks (shown as a space). Backslash escapes and entity references show the character they stand for. | done | `render::tests::inlines`, `render::tests::markup_escapes_and_styles`, `scripts/manual/render.sh` |
| `VIEW-5` | GitHub Flavored Markdown extensions are rendered: tables with a header row and the column alignment from the delimiter row, strikethrough, and task list items, which show a checked or unchecked box that cannot be toggled. | done | `render::tests::gfm`, `scripts/manual/render.sh` (alignment, strikethrough, checked and unchecked boxes) |
| `VIEW-6` | Clicking a link opens it in the default app for its URI scheme. A relative link resolves against the folder of the open file. A link that is only a fragment (`#name`) scrolls to the heading whose GitHub-style anchor (lowercased, punctuation removed, spaces as `-`, `-1`, `-2` … for repeats) is `name`, and does nothing when no heading matches. Hovering a link shows its target as a tooltip. | partial: opening a non-fragment link in the default app was not exercised (it would open the user's browser); it is `gtk::UriLauncher`, checked by code review | `render::tests::anchors`, `render::tests::resolving`, `scripts/manual/links.sh` on Hyprland (a `#middle` link scrolls to the heading, an unknown fragment does nothing, hovering shows `#middle`) |
| `VIEW-7` | Images that are local files (a relative path, an absolute path or a `file:` URI) are loaded without blocking and shown at their natural size, scaled down to fit the column width. Remote images (`http:`, `https:`) are never fetched. In their place, and in place of a local image that can't be loaded, the image's alt text is shown in italics. | done | `render::tests::resolving`, `scripts/manual/render.sh` (the local image loads; missing and remote images show their alt text in italics) |
| `VIEW-8` | Raw HTML, block or inline, is shown as plain text, exactly as written, and is never interpreted. | done | `render::tests::html_is_text`, `scripts/manual/render.sh` |
| `VIEW-9` | Text in any block can be selected with the mouse or keyboard and copied with Ctrl+C. A selection stays inside one block (a paragraph, heading, list item, table cell or code block). | done | `scripts/manual/select.sh` (headless: Tab selects a paragraph, then a code block; Ctrl+C copies them as plain text and with whitespace); mouse selection is GTK's own |

## Implementation notes

- Parsing: `pulldown-cmark` with `ENABLE_TABLES`, `ENABLE_STRIKETHROUGH` and `ENABLE_TASKLISTS`. Turning events into a plain block tree, with no GTK types, keeps `render::tests` free of a display.
- Widgets: a vertical `GtkBox` in a `GtkScrolledWindow`. Paragraphs, headings and list items are selectable `GtkLabel`s with Pango markup, which must escape all source text. Tables are `GtkGrid`s, and code blocks are labels in a horizontally scrolled window.
- `VIEW-2`: gtk4-rs 0.11 has no `GtkCustomLayout` binding, so `column.rs` is a one-child `gtk::Widget` subclass that measures and centres its child.
- Table cells don't wrap. Wrapping labels in a sideways-scrolling grid report a smaller height for a width than their overall minimum, GTK warns, and the grid gets spare height below its last row.
- Selectable labels select all their text when they get keyboard focus (GTK behaviour). Labels on the hint and error pages are plain, so the first one doesn't start selected.
