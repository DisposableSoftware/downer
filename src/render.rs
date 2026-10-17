//! Markdown to a plain block tree, with no GTK types, so it can be tested without a
//! display. `view.rs` turns the tree into widgets.

use gtk::glib;
use pulldown_cmark::{Alignment, Event, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading {
        level: u8,
        anchor: String,
        content: Vec<Inline>,
    },
    Paragraph(Vec<Inline>),
    Code(String),
    /// Raw HTML, shown as written (`VIEW-8`).
    Html(String),
    Quote(Vec<Block>),
    List {
        start: Option<u64>,
        items: Vec<Item>,
    },
    Rule,
    Table {
        align: Vec<Align>,
        head: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// `Some(checked)` for a task list item.
    pub task: Option<bool>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Align {
    None,
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Style {
    pub emphasis: bool,
    pub strong: bool,
    pub code: bool,
    pub strike: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Inline {
    Text {
        text: String,
        style: Style,
        link: Option<String>,
    },
    Break,
    Image {
        dest: String,
        alt: String,
    },
}

pub fn parse(src: &str) -> Vec<Block> {
    let opts = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let mut b = Builder::default();
    for event in Parser::new_ext(src, opts) {
        b.event(event);
    }
    b.flush();
    b.root
}

/// Decodes file contents, replacing invalid UTF-8 with U+FFFD (`NAV-4`).
pub fn decode(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

enum Frame {
    Quote(Vec<Block>),
    List(Option<u64>, Vec<Item>),
    Item(Option<bool>, Vec<Block>),
    Table {
        align: Vec<Align>,
        head: Vec<Vec<Inline>>,
        rows: Vec<Vec<Vec<Inline>>>,
        row: Vec<Vec<Inline>>,
    },
}

#[derive(Default)]
struct Builder {
    root: Vec<Block>,
    frames: Vec<Frame>,
    /// Inlines of the open paragraph, heading or cell, or of a tight list item's text.
    inlines: Option<Vec<Inline>>,
    heading: Option<u8>,
    emphasis: u32,
    strong: u32,
    strike: u32,
    links: Vec<String>,
    image: Option<(String, String)>,
    code: Option<String>,
    html: Option<String>,
    anchors: HashMap<String, usize>,
}

impl Builder {
    fn push_block(&mut self, block: Block) {
        for frame in self.frames.iter_mut().rev() {
            match frame {
                Frame::Quote(blocks) | Frame::Item(_, blocks) => {
                    blocks.push(block);
                    return;
                }
                Frame::List(..) | Frame::Table { .. } => {}
            }
        }
        self.root.push(block);
    }

    /// Ends the text of a tight list item before a nested block starts.
    fn flush(&mut self) {
        if let Some(inlines) = self.inlines.take()
            && !inlines.is_empty()
        {
            self.push_block(Block::Paragraph(inlines));
        }
    }

    fn style(&self) -> Style {
        Style {
            emphasis: self.emphasis > 0,
            strong: self.strong > 0,
            code: false,
            strike: self.strike > 0,
        }
    }

    fn text(&mut self, text: &str, style: Style) {
        if let Some(code) = &mut self.code {
            code.push_str(text);
            return;
        }
        if let Some(html) = &mut self.html {
            html.push_str(text);
            return;
        }
        if let Some((_, alt)) = &mut self.image {
            alt.push_str(text);
            return;
        }
        let link = self.links.last().cloned();
        let inlines = self.inlines.get_or_insert_with(Vec::new);
        if let Some(Inline::Text {
            text: last,
            style: s,
            link: l,
        }) = inlines.last_mut()
            && *s == style
            && *l == link
        {
            last.push_str(text);
            return;
        }
        inlines.push(Inline::Text {
            text: text.to_owned(),
            style,
            link,
        });
    }

    fn event(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(t) => self.text(&t, self.style()),
            Event::Code(t) => {
                let style = Style {
                    code: true,
                    ..self.style()
                };
                self.text(&t, style);
            }
            Event::Html(t) | Event::InlineHtml(t) => self.text(&t, self.style()),
            Event::SoftBreak => self.text(" ", self.style()),
            Event::HardBreak => {
                if let Some((_, alt)) = &mut self.image {
                    alt.push(' ');
                } else {
                    self.inlines
                        .get_or_insert_with(Vec::new)
                        .push(Inline::Break);
                }
            }
            Event::Rule => {
                self.flush();
                self.push_block(Block::Rule);
            }
            Event::TaskListMarker(checked) => {
                if let Some(Frame::Item(task, _)) = self.frames.last_mut() {
                    *task = Some(checked);
                }
            }
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag) {
        match tag {
            Tag::Paragraph => {
                self.flush();
                self.inlines = Some(Vec::new());
            }
            Tag::Heading { level, .. } => {
                self.flush();
                self.heading = Some(level as u8);
                self.inlines = Some(Vec::new());
            }
            Tag::BlockQuote(_) => {
                self.flush();
                self.frames.push(Frame::Quote(Vec::new()));
            }
            Tag::CodeBlock(_) => {
                self.flush();
                self.code = Some(String::new());
            }
            Tag::HtmlBlock => {
                self.flush();
                self.html = Some(String::new());
            }
            Tag::List(start) => {
                self.flush();
                self.frames.push(Frame::List(start, Vec::new()));
            }
            Tag::Item => {
                self.flush();
                self.frames.push(Frame::Item(None, Vec::new()));
            }
            Tag::Table(align) => {
                self.flush();
                self.frames.push(Frame::Table {
                    align: align
                        .into_iter()
                        .map(|a| match a {
                            Alignment::None => Align::None,
                            Alignment::Left => Align::Left,
                            Alignment::Center => Align::Center,
                            Alignment::Right => Align::Right,
                        })
                        .collect(),
                    head: Vec::new(),
                    rows: Vec::new(),
                    row: Vec::new(),
                });
            }
            Tag::TableCell => self.inlines = Some(Vec::new()),
            Tag::Emphasis => self.emphasis += 1,
            Tag::Strong => self.strong += 1,
            Tag::Strikethrough => self.strike += 1,
            Tag::Link { dest_url, .. } => self.links.push(dest_url.into_string()),
            Tag::Image { dest_url, .. } => {
                self.image = Some((dest_url.into_string(), String::new()))
            }
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                let inlines = self.inlines.take().unwrap_or_default();
                self.push_block(Block::Paragraph(inlines));
            }
            TagEnd::Heading(_) => {
                let content = self.inlines.take().unwrap_or_default();
                let anchor = self.anchor(&plain_text(&content));
                let level = self.heading.take().unwrap_or(1);
                self.push_block(Block::Heading {
                    level,
                    anchor,
                    content,
                });
            }
            TagEnd::BlockQuote(_) => {
                self.flush();
                if let Some(Frame::Quote(blocks)) = self.frames.pop() {
                    self.push_block(Block::Quote(blocks));
                }
            }
            TagEnd::CodeBlock => {
                let mut code = self.code.take().unwrap_or_default();
                if code.ends_with('\n') {
                    code.pop();
                }
                self.push_block(Block::Code(code));
            }
            TagEnd::HtmlBlock => {
                let html = self.html.take().unwrap_or_default();
                self.push_block(Block::Html(html.trim_end_matches('\n').to_owned()));
            }
            TagEnd::List(_) => {
                if let Some(Frame::List(start, items)) = self.frames.pop() {
                    self.push_block(Block::List { start, items });
                }
            }
            TagEnd::Item => {
                self.flush();
                if let Some(Frame::Item(task, blocks)) = self.frames.pop()
                    && let Some(Frame::List(_, items)) = self.frames.last_mut()
                {
                    items.push(Item { task, blocks });
                }
            }
            TagEnd::TableCell => {
                let cell = self.inlines.take().unwrap_or_default();
                if let Some(Frame::Table { row, .. }) = self.frames.last_mut() {
                    row.push(cell);
                }
            }
            TagEnd::TableHead => {
                if let Some(Frame::Table { head, row, .. }) = self.frames.last_mut() {
                    *head = std::mem::take(row);
                }
            }
            TagEnd::TableRow => {
                if let Some(Frame::Table { rows, row, .. }) = self.frames.last_mut() {
                    rows.push(std::mem::take(row));
                }
            }
            TagEnd::Table => {
                if let Some(Frame::Table {
                    align, head, rows, ..
                }) = self.frames.pop()
                {
                    self.push_block(Block::Table { align, head, rows });
                }
            }
            TagEnd::Emphasis => self.emphasis = self.emphasis.saturating_sub(1),
            TagEnd::Strong => self.strong = self.strong.saturating_sub(1),
            TagEnd::Strikethrough => self.strike = self.strike.saturating_sub(1),
            TagEnd::Link => {
                self.links.pop();
            }
            TagEnd::Image => {
                if let Some((dest, alt)) = self.image.take() {
                    self.inlines
                        .get_or_insert_with(Vec::new)
                        .push(Inline::Image { dest, alt });
                }
            }
            _ => {}
        }
    }

    /// GitHub's heading anchor, with `-1`, `-2` … for repeats (`VIEW-6`).
    fn anchor(&mut self, text: &str) -> String {
        let slug = slug(text);
        let n = self.anchors.entry(slug.clone()).or_insert(0);
        let anchor = if *n == 0 { slug } else { format!("{slug}-{n}") };
        *n += 1;
        anchor
    }
}

pub fn plain_text(inlines: &[Inline]) -> String {
    inlines
        .iter()
        .map(|i| match i {
            Inline::Text { text, .. } => text.as_str(),
            Inline::Image { alt, .. } => alt.as_str(),
            Inline::Break => " ",
        })
        .collect()
}

fn slug(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '-' | '_' | ' '))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

#[derive(Debug, PartialEq)]
pub enum Target {
    /// A heading anchor in this document.
    Fragment(String),
    Uri(String),
}

fn has_scheme(dest: &str) -> bool {
    dest.split_once(':').is_some_and(|(scheme, _)| {
        scheme.len() > 1
            && scheme.starts_with(|c: char| c.is_ascii_alphabetic())
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    })
}

/// Where a link goes: `base` is the open file's URI, against which relative links
/// resolve (`VIEW-6`).
pub fn resolve(dest: &str, base: &str) -> Option<Target> {
    if let Some(fragment) = dest.strip_prefix('#') {
        let name = glib::Uri::unescape_string(fragment, None::<&str>)
            .map(|s| s.to_string())
            .unwrap_or_else(|| fragment.to_owned());
        return Some(Target::Fragment(name));
    }
    if has_scheme(dest) {
        return Some(Target::Uri(dest.to_owned()));
    }
    let resolved = glib::Uri::resolve_relative(Some(base), dest, glib::UriFlags::NONE)
        .or_else(|_| {
            let escaped = glib::Uri::escape_string(dest, Some("/#?%"), true);
            glib::Uri::resolve_relative(Some(base), &escaped, glib::UriFlags::NONE)
        })
        .ok()?;
    Some(Target::Uri(resolved.to_string()))
}

/// The URI to load an image from, or `None` for remote images, which are never
/// fetched (`VIEW-7`).
pub fn image_uri(dest: &str, base: &str) -> Option<String> {
    let dest = if dest.starts_with('/') {
        glib::filename_to_uri(dest, None).ok()?.to_string()
    } else {
        dest.to_owned()
    };
    match resolve(&dest, base)? {
        Target::Uri(uri) => {
            let scheme = glib::Uri::peek_scheme(&uri)?.to_lowercase();
            (scheme != "http" && scheme != "https").then_some(uri)
        }
        Target::Fragment(_) => None,
    }
}

/// Colours Pango markup needs as literals, looked up from the theme.
pub struct Colors {
    pub code_bg: String,
    pub strike: String,
}

/// Pango markup for a run of inlines with no images in it.
pub fn markup(inlines: &[Inline], colors: &Colors) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Break => out.push('\n'),
            Inline::Image { alt, .. } => {
                out.push_str("<i>");
                out.push_str(&glib::markup_escape_text(alt));
                out.push_str("</i>");
            }
            Inline::Text { text, style, link } => {
                let mut open = String::new();
                let mut close = String::new();
                let mut wrap = |o: String, c: &str| {
                    open.push_str(&o);
                    close.insert_str(0, c);
                };
                if let Some(link) = link {
                    let href = glib::markup_escape_text(link);
                    wrap(format!("<a href=\"{href}\" title=\"{href}\">"), "</a>");
                }
                if style.strong {
                    wrap("<b>".into(), "</b>");
                }
                if style.emphasis {
                    wrap("<i>".into(), "</i>");
                }
                if style.strike {
                    wrap(
                        format!("<s><span foreground=\"{}\">", colors.strike),
                        "</span></s>",
                    );
                }
                if style.code {
                    wrap(
                        format!(
                            "<span font_family=\"monospace\" size=\"90%\" background=\"{}\">",
                            colors.code_bg
                        ),
                        "</span>",
                    );
                }
                out.push_str(&open);
                out.push_str(&glib::markup_escape_text(text));
                out.push_str(&close);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(t: &str) -> Inline {
        Inline::Text {
            text: t.into(),
            style: Style::default(),
            link: None,
        }
    }

    fn styled(t: &str, f: impl FnOnce(&mut Style)) -> Inline {
        let mut style = Style::default();
        f(&mut style);
        Inline::Text {
            text: t.into(),
            style,
            link: None,
        }
    }

    fn para(inlines: Vec<Inline>) -> Block {
        Block::Paragraph(inlines)
    }

    /// `VIEW-3`: every CommonMark block kind.
    #[test]
    fn blocks() {
        let blocks = parse(
            "# One\n\nTwo\n===\n\n###### Six\n\npara\n\n> quote\n> > nested\n\n\
             - a\n- b\n  - c\n\n7. seven\n8. eight\n\n```rust\nfn x() {\n    1\n}\n```\n\n    \
             indented\n\n---\n",
        );
        let heading = |level, anchor: &str, t: &str| Block::Heading {
            level,
            anchor: anchor.into(),
            content: vec![text(t)],
        };
        let item = |blocks| Item { task: None, blocks };
        assert_eq!(
            blocks,
            vec![
                heading(1, "one", "One"),
                heading(1, "two", "Two"),
                heading(6, "six", "Six"),
                para(vec![text("para")]),
                Block::Quote(vec![
                    para(vec![text("quote")]),
                    Block::Quote(vec![para(vec![text("nested")])]),
                ]),
                Block::List {
                    start: None,
                    items: vec![
                        item(vec![para(vec![text("a")])]),
                        item(vec![
                            para(vec![text("b")]),
                            Block::List {
                                start: None,
                                items: vec![item(vec![para(vec![text("c")])])],
                            },
                        ]),
                    ],
                },
                Block::List {
                    start: Some(7),
                    items: vec![
                        item(vec![para(vec![text("seven")])]),
                        item(vec![para(vec![text("eight")])]),
                    ],
                },
                Block::Code("fn x() {\n    1\n}".into()),
                Block::Code("indented".into()),
                Block::Rule,
            ]
        );
    }

    /// `VIEW-4`: inlines, escapes and entities.
    #[test]
    fn inlines() {
        let blocks =
            parse("*em* **strong** `code` [link](a.md) <https://x.org> a\nb  \nc \\* &amp; &copy;");
        let link = |t: &str, l: &str| Inline::Text {
            text: t.into(),
            style: Style::default(),
            link: Some(l.into()),
        };
        assert_eq!(
            blocks,
            vec![para(vec![
                styled("em", |s| s.emphasis = true),
                text(" "),
                styled("strong", |s| s.strong = true),
                text(" "),
                styled("code", |s| s.code = true),
                text(" "),
                link("link", "a.md"),
                text(" "),
                link("https://x.org", "https://x.org"),
                text(" a b"),
                Inline::Break,
                text("c * & ©"),
            ])]
        );
    }

    /// `VIEW-5`: tables with alignment, strikethrough, task lists.
    #[test]
    fn gfm() {
        let blocks = parse(
            "| a | b | c |\n|:--|:-:|--:|\n| 1 | 2 | 3 |\n\n~~gone~~\n\n- [x] done\n- [ ] todo\n",
        );
        assert_eq!(
            blocks[0],
            Block::Table {
                align: vec![Align::Left, Align::Center, Align::Right],
                head: vec![vec![text("a")], vec![text("b")], vec![text("c")]],
                rows: vec![vec![vec![text("1")], vec![text("2")], vec![text("3")]]],
            }
        );
        assert_eq!(blocks[1], para(vec![styled("gone", |s| s.strike = true)]));
        let Block::List { items, .. } = &blocks[2] else {
            panic!("{:?}", blocks[2]);
        };
        assert_eq!(items[0].task, Some(true));
        assert_eq!(items[1].task, Some(false));
        assert_eq!(items[1].blocks, vec![para(vec![text("todo")])]);
    }

    /// `VIEW-8`: raw HTML is kept as text.
    #[test]
    fn html_is_text() {
        let blocks = parse("<div>\n<b>hi</b>\n</div>\n\nan <span>inline</span> tag\n");
        assert_eq!(blocks[0], Block::Html("<div>\n<b>hi</b>\n</div>".into()));
        assert_eq!(blocks[1], para(vec![text("an <span>inline</span> tag")]));
        let colors = Colors {
            code_bg: "#000".into(),
            strike: "#111".into(),
        };
        let Block::Paragraph(p) = &blocks[1] else {
            unreachable!()
        };
        assert_eq!(markup(p, &colors), "an &lt;span&gt;inline&lt;/span&gt; tag");
    }

    #[test]
    fn images_are_inlines() {
        assert_eq!(
            parse("![a *cat*](cat.png)"),
            vec![para(vec![Inline::Image {
                dest: "cat.png".into(),
                alt: "a cat".into()
            }])]
        );
    }

    /// `VIEW-6`: GitHub anchors, repeats numbered.
    #[test]
    fn anchors() {
        let anchors: Vec<String> =
            parse("# Hello, World!\n## Hello World\n# Hello world\n## `code` & C++_x\n")
                .into_iter()
                .filter_map(|b| match b {
                    Block::Heading { anchor, .. } => Some(anchor),
                    _ => None,
                })
                .collect();
        assert_eq!(
            anchors,
            ["hello-world", "hello-world-1", "hello-world-2", "code--c_x"]
        );
    }

    /// `VIEW-6`, `VIEW-7`: resolving links and images.
    #[test]
    fn resolving() {
        let base = "file:///docs/guide/README.md";
        let uri = |s: &str| Some(Target::Uri(s.into()));
        assert_eq!(
            resolve("#intro", base),
            Some(Target::Fragment("intro".into()))
        );
        assert_eq!(
            resolve("#caf%C3%A9", base),
            Some(Target::Fragment("café".into()))
        );
        assert_eq!(resolve("https://x.org/a", base), uri("https://x.org/a"));
        assert_eq!(resolve("mailto:a@b.c", base), uri("mailto:a@b.c"));
        assert_eq!(
            resolve("other.md", base),
            uri("file:///docs/guide/other.md")
        );
        assert_eq!(resolve("../up.md#s", base), uri("file:///docs/up.md#s"));
        assert_eq!(
            resolve("my file.md", base),
            uri("file:///docs/guide/my%20file.md")
        );

        assert_eq!(
            image_uri("img/a.png", base).as_deref(),
            Some("file:///docs/guide/img/a.png")
        );
        assert_eq!(
            image_uri("/abs/b.png", base).as_deref(),
            Some("file:///abs/b.png")
        );
        assert_eq!(
            image_uri("file:///c.png", base).as_deref(),
            Some("file:///c.png")
        );
        assert_eq!(image_uri("https://x.org/d.png", base), None);
        assert_eq!(image_uri("HTTP://x.org/d.png", base), None);
    }

    /// `NAV-4`: invalid UTF-8 becomes U+FFFD, and the rest still renders.
    #[test]
    fn invalid_utf8() {
        let src = decode(b"# ok\n\nbad \xff byte");
        assert_eq!(parse(&src)[1], para(vec![text("bad \u{fffd} byte")]));
    }

    #[test]
    fn markup_escapes_and_styles() {
        let colors = Colors {
            code_bg: "#313244".into(),
            strike: "#7f849c".into(),
        };
        let Block::Paragraph(p) = &parse("**a<b** [x&y](q?a=1&b=2) `c`")[0] else {
            unreachable!()
        };
        assert_eq!(
            markup(p, &colors),
            "<b>a&lt;b</b> <a href=\"q?a=1&amp;b=2\" title=\"q?a=1&amp;b=2\">x&amp;y</a> \
             <span font_family=\"monospace\" size=\"90%\" background=\"#313244\">c</span>"
        );
    }
}
