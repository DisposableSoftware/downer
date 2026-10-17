# View

How a Markdown file is shown in the window.

## Requirements

| ID | Requirement | Status | Verified by |
|----|-------------|--------|-------------|
| `VIEW-1` | A window with a file open (`NAV-1`) shows the file's Markdown rendered as formatted text, never as raw source, in one column that scrolls vertically. | planned | `render::tests`, `scripts/manual/render.sh` (screenshot) |
| `VIEW-2` | The text column is at most 800 logical px wide and centred in the window, with at least 24px of padding on each side. Narrower windows wrap text to fit. Only code blocks (`VIEW-3`) and tables (`VIEW-5`) scroll sideways, each on its own; the page never does. | planned | `scripts/manual/render.sh` (screenshots at 1600px and 500px window widths) |
| `VIEW-3` | CommonMark blocks are rendered: ATX and setext headings at six levels, paragraphs, block quotes (nested too), bullet and ordered lists (nested, ordered lists keep their start number), fenced and indented code blocks, and thematic breaks. Code blocks keep their whitespace and line breaks and do not wrap. | planned | `render::tests` (each block kind) |
| `VIEW-4` | CommonMark inlines are rendered: emphasis, strong emphasis, inline code, links, autolinks, hard line breaks, and soft line breaks (shown as a space). Backslash escapes and entity references show the character they stand for. | planned | `render::tests` |
| `VIEW-5` | GitHub Flavored Markdown extensions are rendered: tables with a header row and the column alignment from the delimiter row, strikethrough, and task list items, which show a checked or unchecked box that cannot be toggled. | planned | `render::tests`, `scripts/manual/render.sh` (screenshot) |
| `VIEW-6` | Clicking a link opens it in the default app for its URI scheme. A relative link resolves against the folder of the open file. A link that is only a fragment (`#name`) scrolls to the heading whose GitHub-style anchor (lowercased, punctuation removed, spaces as `-`, `-1`, `-2` … for repeats) is `name`, and does nothing when no heading matches. Hovering a link shows its target as a tooltip. | planned | `render::tests` (anchors and resolving), manual (a `#` link scrolls; an `https` link opens the browser) |
| `VIEW-7` | Images that are local files (a relative path, an absolute path or a `file:` URI) are loaded without blocking and shown at their natural size, scaled down to fit the column width. Remote images (`http:`, `https:`) are never fetched. In their place, and in place of a local image that can't be loaded, the image's alt text is shown in italics. | planned | `render::tests` (which images load), `scripts/manual/render.sh` (screenshot with a local, a missing and a remote image) |
| `VIEW-8` | Raw HTML, block or inline, is shown as plain text, exactly as written, and is never interpreted. | planned | `render::tests` |
| `VIEW-9` | Text in any block can be selected with the mouse or keyboard and copied with Ctrl+C. A selection stays inside one block (a paragraph, heading, list item, table cell or code block). | planned | manual (select and paste a paragraph and a code block) |

## Implementation notes

- Parsing: `pulldown-cmark` with `ENABLE_TABLES`, `ENABLE_STRIKETHROUGH` and `ENABLE_TASKLISTS`. Turning events into a plain block tree, with no GTK types, keeps `render::tests` free of a display.
- Widgets: a vertical `GtkBox` in a `GtkScrolledWindow`. Paragraphs, headings and list items are selectable `GtkLabel`s with Pango markup, which must escape all source text. Tables are `GtkGrid`s, and code blocks are labels in a horizontally scrolled window.
