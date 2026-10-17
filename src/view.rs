//! Windows: one per file (`NAV-1`), the rendered document, reloading, links.

use crate::column::Column;
use crate::render::{self, Align, Block, Colors, Inline, Target};
use gtk::{gdk, gio, glib, prelude::*};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time::Duration;

const RELOAD_DELAY: Duration = Duration::from_millis(150);

struct Doc {
    window: gtk::ApplicationWindow,
    file: gio::File,
    scrolled: gtk::ScrolledWindow,
    column: Column,
    anchors: RefCell<Vec<(String, gtk::Widget)>>,
    monitor: RefCell<Option<gio::FileMonitor>>,
    reload: RefCell<Option<glib::SourceId>>,
    /// Whether the document has been rendered at least once.
    shown: Cell<bool>,
}

fn window(app: &gtk::Application, title: &str) -> gtk::ApplicationWindow {
    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title(title)
        .default_width(900)
        .default_height(700)
        .build();
    window.add_css_class("downer");
    window
}

fn centred(title: &str, detail: &str) -> gtk::Box {
    let page = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .valign(gtk::Align::Center)
        .halign(gtk::Align::Center)
        .margin_start(24)
        .margin_end(24)
        .build();
    let title = gtk::Label::builder()
        .label(title)
        .wrap(true)
        .justify(gtk::Justification::Center)
        .build();
    title.add_css_class("page-title");
    let detail = gtk::Label::builder()
        .label(detail)
        .wrap(true)
        .justify(gtk::Justification::Center)
        .build();
    detail.add_css_class("dim");
    page.append(&title);
    page.append(&detail);
    page
}

/// The window shown without arguments (`NAV-3`).
pub fn open_empty(app: &gtk::Application) {
    let window = window(app, "Downer");
    window.set_child(Some(&centred(
        "No file open",
        "Open a Markdown file with downer FILE",
    )));
    window.present();
}

pub fn open(app: &gtk::Application, file: &gio::File) {
    let name = file
        .basename()
        .map(|b| b.to_string_lossy().into_owned())
        .unwrap_or_else(|| file.uri().to_string());
    let window = window(app, &format!("{name} — Downer"));
    let column = Column::new();
    let viewport = gtk::Viewport::builder()
        .child(&column)
        .scroll_to_focus(false)
        .build();
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&viewport)
        .build();
    let doc = Rc::new(Doc {
        window: window.clone(),
        file: file.clone(),
        scrolled,
        column,
        anchors: RefCell::default(),
        monitor: RefCell::default(),
        reload: RefCell::default(),
        shown: Cell::new(false),
    });
    watch(&doc);
    // The window owns the only strong reference, through this handler.
    let strong = doc.clone();
    window.connect_close_request(move |_| {
        if let Some(monitor) = strong.monitor.take() {
            monitor.cancel();
        }
        if let Some(id) = strong.reload.take() {
            id.remove();
        }
        glib::Propagation::Proceed
    });
    load(&doc);
    window.present();
}

/// Renders the file again when it changes on disk (`NAV-5`).
fn watch(doc: &Rc<Doc>) {
    let monitor = match doc
        .file
        .monitor_file(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE)
    {
        Ok(m) => m,
        Err(e) => {
            glib::g_warning!("downer", "can't watch {}: {}", doc.file.parse_name(), e);
            return;
        }
    };
    let weak = Rc::downgrade(doc);
    monitor.connect_changed(move |_, _, _, event| {
        use gio::FileMonitorEvent as E;
        if matches!(
            event,
            E::Deleted | E::MovedOut | E::PreUnmount | E::Unmounted | E::AttributeChanged
        ) {
            return;
        }
        let Some(doc) = weak.upgrade() else { return };
        if let Some(id) = doc.reload.take() {
            id.remove();
        }
        let weak = weak.clone();
        let id = glib::timeout_add_local_once(RELOAD_DELAY, move || {
            if let Some(doc) = weak.upgrade() {
                doc.reload.take();
                load(&doc);
            }
        });
        doc.reload.replace(Some(id));
    });
    doc.monitor.replace(Some(monitor));
}

fn load(doc: &Rc<Doc>) {
    let weak = Rc::downgrade(doc);
    let file = doc.file.clone();
    glib::spawn_future_local(async move {
        let result = file.load_contents_future().await;
        let Some(doc) = weak.upgrade() else { return };
        match result {
            Ok((bytes, _)) => show(&doc, &render::decode(&bytes)),
            // A deleted file keeps its last rendering (`NAV-5`).
            Err(e) if doc.shown.get() && e.matches(gio::IOErrorEnum::NotFound) => {}
            Err(e) => {
                doc.shown.set(false);
                doc.window
                    .set_child(Some(&centred(&doc.file.parse_name(), e.message())));
            }
        }
    });
}

fn show(doc: &Rc<Doc>, src: &str) {
    let blocks = render::parse(src);
    let mut ctx = Ctx {
        doc: Rc::downgrade(doc),
        base: doc.file.uri().to_string(),
        colors: colors(&doc.window),
        anchors: Vec::new(),
    };
    let content = ctx.blocks(&blocks, 0);
    content.add_css_class("document");
    doc.column.set_child(&content);
    doc.anchors.replace(ctx.anchors);
    if !doc.shown.replace(true) {
        doc.window.set_child(Some(&doc.scrolled));
    }
}

fn scroll_to(doc: &Doc, name: &str) {
    let anchors = doc.anchors.borrow();
    let Some((_, widget)) = anchors.iter().find(|(a, _)| a == name) else {
        return;
    };
    if let Some(point) = widget.compute_point(&doc.column, &gtk::graphene::Point::zero()) {
        doc.scrolled
            .vadjustment()
            .set_value(f64::from(point.y()) - 12.0);
    }
}

fn activate_link(doc: &Weak<Doc>, base: &str, href: &str) {
    let Some(doc) = doc.upgrade() else { return };
    match render::resolve(href, base) {
        Some(Target::Fragment(name)) => scroll_to(&doc, &name),
        Some(Target::Uri(uri)) => {
            gtk::UriLauncher::new(&uri).launch(
                Some(&doc.window),
                gio::Cancellable::NONE,
                move |r| {
                    if let Err(e) = r
                        && !e.matches(gtk::DialogError::Dismissed)
                    {
                        glib::g_warning!("downer", "can't open {uri}: {e}");
                    }
                },
            );
        }
        None => {}
    }
}

/// Pango markup can't use CSS colour names, so look the palette up once per render.
#[allow(deprecated)]
fn colors(widget: &impl IsA<gtk::Widget>) -> Colors {
    let ctx = widget.style_context();
    let hex = |name: &str, fallback: &str| {
        ctx.lookup_color(name)
            .map(|c: gdk::RGBA| {
                let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                format!("#{:02x}{:02x}{:02x}", b(c.red()), b(c.green()), b(c.blue()))
            })
            .unwrap_or_else(|| fallback.to_owned())
    };
    Colors {
        code_bg: hex("surface0", "#313244"),
        strike: hex("overlay1", "#7f849c"),
    }
}

struct Ctx {
    doc: Weak<Doc>,
    base: String,
    colors: Colors,
    anchors: Vec<(String, gtk::Widget)>,
}

fn vbox(spacing: i32) -> gtk::Box {
    gtk::Box::new(gtk::Orientation::Vertical, spacing)
}

impl Ctx {
    fn label(&self, inlines: &[Inline]) -> gtk::Label {
        let label = gtk::Label::builder()
            .use_markup(true)
            .label(render::markup(inlines, &self.colors))
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .xalign(0.0)
            .selectable(true)
            .build();
        let doc = self.doc.clone();
        let base = self.base.clone();
        label.connect_activate_link(move |_, href| {
            activate_link(&doc, &base, href);
            glib::Propagation::Stop
        });
        label
    }

    fn plain(&self, text: &str) -> gtk::Label {
        gtk::Label::builder()
            .label(text)
            .wrap(true)
            .wrap_mode(gtk::pango::WrapMode::WordChar)
            .xalign(0.0)
            .selectable(true)
            .build()
    }

    fn blocks(&mut self, blocks: &[Block], depth: usize) -> gtk::Box {
        let content = vbox(12);
        for block in blocks {
            content.append(&self.block(block, depth));
        }
        content
    }

    fn block(&mut self, block: &Block, depth: usize) -> gtk::Widget {
        match block {
            Block::Heading {
                level,
                anchor,
                content,
            } => {
                let label = self.label(content);
                label.add_css_class("heading");
                label.add_css_class(&format!("h{level}"));
                self.anchors.push((anchor.clone(), label.clone().upcast()));
                label.upcast()
            }
            Block::Paragraph(inlines) => self.paragraph(inlines),
            Block::Code(code) => {
                let label = gtk::Label::builder()
                    .label(code)
                    .xalign(0.0)
                    .selectable(true)
                    .build();
                label.add_css_class("code");
                let scrolled = gtk::ScrolledWindow::builder()
                    .hscrollbar_policy(gtk::PolicyType::Automatic)
                    .vscrollbar_policy(gtk::PolicyType::Never)
                    .child(&label)
                    .build();
                scrolled.add_css_class("code-block");
                scrolled.upcast()
            }
            Block::Html(html) => {
                let label = self.plain(html);
                label.add_css_class("html");
                label.upcast()
            }
            Block::Quote(blocks) => {
                let quote = self.blocks(blocks, depth);
                quote.add_css_class("quote");
                quote.upcast()
            }
            Block::List { start, items } => self.list(*start, items, depth),
            Block::Rule => {
                let rule = gtk::Separator::new(gtk::Orientation::Horizontal);
                rule.add_css_class("rule");
                rule.upcast()
            }
            Block::Table { align, head, rows } => self.table(align, head, rows),
        }
    }

    /// Text runs as labels, with images between them as pictures (`VIEW-7`).
    fn paragraph(&mut self, inlines: &[Inline]) -> gtk::Widget {
        let mut parts: Vec<gtk::Widget> = Vec::new();
        let mut run: Vec<Inline> = Vec::new();
        let flush = |run: &mut Vec<Inline>, parts: &mut Vec<gtk::Widget>, ctx: &Ctx| {
            if !render::plain_text(run).trim().is_empty() {
                parts.push(ctx.label(run).upcast());
            }
            run.clear();
        };
        for inline in inlines {
            if let Inline::Image { dest, alt } = inline {
                flush(&mut run, &mut parts, self);
                parts.push(self.image(dest, alt));
            } else {
                run.push(inline.clone());
            }
        }
        flush(&mut run, &mut parts, self);
        if parts.len() == 1 {
            return parts.remove(0);
        }
        let paragraph = vbox(8);
        for part in &parts {
            paragraph.append(part);
        }
        paragraph.upcast()
    }

    fn image(&self, dest: &str, alt: &str) -> gtk::Widget {
        let alt_label = self.label(&[Inline::Image {
            dest: dest.to_owned(),
            alt: alt.to_owned(),
        }]);
        let Some(uri) = render::image_uri(dest, &self.base) else {
            return alt_label.upcast();
        };
        let picture = gtk::Picture::builder()
            .can_shrink(true)
            .content_fit(gtk::ContentFit::ScaleDown)
            .halign(gtk::Align::Start)
            .alternative_text(alt)
            .build();
        let holder = vbox(0);
        holder.append(&picture);
        glib::spawn_future_local(glib::clone!(
            #[weak]
            holder,
            #[weak]
            picture,
            async move {
                let file = gio::File::for_uri(&uri);
                let texture = match file.load_bytes_future().await {
                    Ok((bytes, _)) => gio::spawn_blocking(move || gdk::Texture::from_bytes(&bytes))
                        .await
                        .unwrap_or_else(|_| {
                            Err(glib::Error::new(
                                gio::IOErrorEnum::Failed,
                                "decoder panicked",
                            ))
                        }),
                    Err(e) => Err(e),
                };
                match texture {
                    Ok(texture) => picture.set_paintable(Some(&texture)),
                    Err(_) => {
                        holder.remove(&picture);
                        holder.append(&alt_label);
                    }
                }
            }
        ));
        holder.upcast()
    }

    fn list(&mut self, start: Option<u64>, items: &[render::Item], depth: usize) -> gtk::Widget {
        let list = vbox(6);
        list.add_css_class("list");
        let width = start.map_or(1, |s| format!("{}.", s + items.len() as u64).len());
        for (i, item) in items.iter().enumerate() {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            let marker: gtk::Widget = if let Some(checked) = item.task {
                let check = gtk::CheckButton::builder()
                    .active(checked)
                    .can_target(false)
                    .can_focus(false)
                    .valign(gtk::Align::Start)
                    .build();
                check.add_css_class("task");
                check.upcast()
            } else {
                let text = match start {
                    Some(s) => format!("{}.", s + i as u64),
                    None => ["•", "◦", "▪"][depth % 3].to_owned(),
                };
                let marker = gtk::Label::builder()
                    .label(text)
                    .xalign(1.0)
                    .width_chars(width as i32)
                    .valign(gtk::Align::Start)
                    .build();
                marker.add_css_class("marker");
                marker.upcast()
            };
            let content = vbox(8);
            content.set_hexpand(true);
            for block in &item.blocks {
                let depth = if matches!(block, Block::List { .. }) {
                    depth + 1
                } else {
                    depth
                };
                content.append(&self.block(block, depth));
            }
            row.append(&marker);
            row.append(&content);
            list.append(&row);
        }
        list.upcast()
    }

    fn table(
        &self,
        align: &[Align],
        head: &[Vec<Inline>],
        rows: &[Vec<Vec<Inline>>],
    ) -> gtk::Widget {
        let grid = gtk::Grid::new();
        grid.add_css_class("table");
        let cell = |inlines: &[Inline], col: usize, header: bool| {
            let label = self.label(inlines);
            // Wrapping cells in a sideways-scrolling grid report inconsistent heights
            // for a width, and GTK then gives the grid spare height.
            label.set_wrap(false);
            label.add_css_class("cell");
            if header {
                label.add_css_class("th");
            }
            let (x, justify) = match align.get(col).copied().unwrap_or(Align::None) {
                Align::Center => (0.5, gtk::Justification::Center),
                Align::Right => (1.0, gtk::Justification::Right),
                Align::Left | Align::None => (0.0, gtk::Justification::Left),
            };
            label.set_xalign(x);
            label.set_justify(justify);
            label
        };
        for (c, inlines) in head.iter().enumerate() {
            grid.attach(&cell(inlines, c, true), c as i32, 0, 1, 1);
        }
        for (r, row) in rows.iter().enumerate() {
            for (c, inlines) in row.iter().enumerate() {
                grid.attach(&cell(inlines, c, false), c as i32, r as i32 + 1, 1, 1);
            }
        }
        let scrolled = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Automatic)
            .vscrollbar_policy(gtk::PolicyType::Never)
            .halign(gtk::Align::Start)
            .propagate_natural_width(true)
            .valign(gtk::Align::Start)
            .child(&grid)
            .build();
        scrolled.add_css_class("table-scroll");
        scrolled.upcast()
    }
}
