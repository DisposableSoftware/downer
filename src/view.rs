//! Windows: one per file or folder (`NAV-1`, `NAV-6`), the rendered document,
//! reloading, links.

use crate::column::Column;
use crate::render::{self, Align, Block, Colors, Inline, Target};
use crate::sidebar::Sidebar;
use crate::table::Table;
use gtk::{gdk, gio, glib, prelude::*};
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::time::Duration;

const RELOAD_DELAY: Duration = Duration::from_millis(150);

struct Doc {
    window: gtk::ApplicationWindow,
    /// The file shown in the main view; `None` in a folder window before one is picked.
    file: RefCell<Option<gio::File>>,
    /// The folder of a folder window.
    folder: Option<gio::File>,
    sidebar: RefCell<Option<Rc<Sidebar>>>,
    /// The main view: the document, or a centred message.
    main: gtk::Box,
    /// The main view with the file name bar (`VIEW-11`) above it.
    view: gtk::Box,
    name_bar: gtk::Label,
    scrolled: gtk::ScrolledWindow,
    column: Column,
    anchors: RefCell<Vec<(String, gtk::Widget)>>,
    monitor: RefCell<Option<gio::FileMonitor>>,
    reload: RefCell<Option<glib::SourceId>>,
    /// Whether the current file has been rendered at least once.
    shown: Cell<bool>,
    /// A heading to scroll to once the next rendering is laid out (`NAV-11`).
    fragment: RefCell<Option<String>>,
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
        .hexpand(true)
        .vexpand(true)
        .margin_start(24)
        .margin_end(24)
        .build();
    let title = gtk::Label::builder()
        .label(title)
        .wrap(true)
        .justify(gtk::Justification::Center)
        .build();
    title.add_css_class("page-title");
    page.append(&title);
    if !detail.is_empty() {
        let detail = gtk::Label::builder()
            .label(detail)
            .wrap(true)
            .justify(gtk::Justification::Center)
            .build();
        detail.add_css_class("dim");
        page.append(&detail);
    }
    page
}

fn name(file: &gio::File) -> String {
    file.basename()
        .map(|b| b.to_string_lossy().into_owned())
        .unwrap_or_else(|| file.uri().to_string())
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

/// Opens a file window, or a folder window when `file` is a folder (`NAV-6`).
pub fn open(app: &gtk::Application, file: &gio::File) {
    let app = app.clone();
    let file = file.clone();
    // Holds the application while GIO answers, so it doesn't quit with no window.
    let hold = app.hold();
    glib::spawn_future_local(async move {
        let is_dir = file
            .query_info_future(
                "standard::type",
                gio::FileQueryInfoFlags::NONE,
                glib::Priority::DEFAULT,
            )
            .await
            .is_ok_and(|info| info.file_type() == gio::FileType::Directory);
        if is_dir {
            open_folder(&app, &file);
        } else {
            open_file_window(&app, &file);
        }
        drop(hold);
    });
}

fn new_doc(app: &gtk::Application, title: &str, folder: Option<gio::File>) -> Rc<Doc> {
    let window = window(app, title);
    let column = Column::new();
    let viewport = gtk::Viewport::builder()
        .child(&column)
        .scroll_to_focus(false)
        .build();
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .hexpand(true)
        .child(&viewport)
        .build();
    let main = gtk::Box::builder().hexpand(true).vexpand(true).build();
    let name_bar = gtk::Label::builder()
        .css_classes(["file-name"])
        .xalign(0.0)
        .ellipsize(gtk::pango::EllipsizeMode::Start)
        .focusable(false)
        .can_target(false)
        .visible(false)
        .build();
    let view = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .hexpand(true)
        .build();
    view.append(&name_bar);
    view.append(&main);
    let doc = Rc::new(Doc {
        window: window.clone(),
        file: RefCell::default(),
        folder,
        sidebar: RefCell::default(),
        main,
        view,
        name_bar,
        scrolled,
        column,
        anchors: RefCell::default(),
        monitor: RefCell::default(),
        reload: RefCell::default(),
        shown: Cell::new(false),
        fragment: RefCell::default(),
    });
    // The window owns the only strong reference, through this handler.
    let strong = doc.clone();
    window.connect_close_request(move |_| {
        stop_watching(&strong);
        if let Some(sidebar) = strong.sidebar.take() {
            sidebar.stop();
        }
        glib::Propagation::Proceed
    });
    doc
}

fn open_file_window(app: &gtk::Application, file: &gio::File) {
    let doc = new_doc(app, &format!("{} — Downer", name(file)), None);
    doc.window.set_child(Some(&doc.view));
    show_file(&doc, file);
    doc.window.present();
}

fn open_folder(app: &gtk::Application, folder: &gio::File) {
    let doc = new_doc(
        app,
        &format!("{} — Downer", name(folder)),
        Some(folder.clone()),
    );
    let open = Rc::downgrade(&doc);
    let scanned = Rc::downgrade(&doc);
    let sidebar = Sidebar::new(
        folder,
        move |file| {
            if let Some(doc) = open.upgrade() {
                show_file(&doc, &file);
            }
        },
        move |result, first| {
            let Some(doc) = scanned.upgrade() else { return };
            match result {
                Err(e) => set_main(&doc, &centred(&folder_name(&doc), e.message())),
                Ok(()) if first => first_file(&doc),
                Ok(()) => {}
            }
        },
    );
    let body = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    body.append(&sidebar.widget);
    body.append(&doc.view);
    doc.window.set_child(Some(&body));
    doc.sidebar.replace(Some(sidebar));
    doc.window.present();
}

fn folder_name(doc: &Doc) -> String {
    doc.folder
        .as_ref()
        .map(|f| f.parse_name().to_string())
        .unwrap_or_default()
}

/// What a folder window opens with (`NAV-9`).
fn first_file(doc: &Rc<Doc>) {
    let sidebar = doc.sidebar.borrow().clone();
    let Some(sidebar) = sidebar else { return };
    match sidebar.first_file() {
        Some(file) => {
            sidebar.select(&file.uri());
            show_file(doc, &file);
        }
        None if sidebar.files().is_empty() => set_main(doc, &centred("No Markdown files", "")),
        None => set_main(doc, &centred("Select a file", "")),
    }
    sidebar.focus();
}

fn set_main(doc: &Doc, child: &impl IsA<gtk::Widget>) {
    while let Some(old) = doc.main.first_child() {
        doc.main.remove(&old);
    }
    doc.main.append(child);
    doc.name_bar
        .set_visible(child.upcast_ref::<gtk::Widget>() == doc.scrolled.upcast_ref::<gtk::Widget>());
}

fn update_title(doc: &Doc) {
    let file = doc.file.borrow().as_ref().map(name);
    doc.name_bar.set_label(file.as_deref().unwrap_or_default());
    let title = match (&doc.folder, file) {
        (Some(folder), Some(file)) => format!("{file} — {} — Downer", name(folder)),
        (Some(folder), None) => format!("{} — Downer", name(folder)),
        (None, Some(file)) => format!("{file} — Downer"),
        (None, None) => "Downer".to_owned(),
    };
    doc.window.set_title(Some(&title));
}

/// Shows `file` in the main view, from the top, and follows its changes (`NAV-5`).
fn show_file(doc: &Rc<Doc>, file: &gio::File) {
    stop_watching(doc);
    doc.file.replace(Some(file.clone()));
    doc.shown.set(false);
    doc.scrolled.vadjustment().set_value(0.0);
    update_title(doc);
    watch(doc, file);
    load(doc);
}

fn stop_watching(doc: &Doc) {
    if let Some(monitor) = doc.monitor.take() {
        monitor.cancel();
    }
    if let Some(id) = doc.reload.take() {
        id.remove();
    }
}

/// Renders the file again when it changes on disk (`NAV-5`).
fn watch(doc: &Rc<Doc>, file: &gio::File) {
    let monitor =
        match file.monitor_file(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE) {
            Ok(m) => m,
            Err(e) => {
                glib::g_warning!("downer", "can't watch {}: {}", file.parse_name(), e);
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
    let Some(file) = doc.file.borrow().clone() else {
        return;
    };
    let weak = Rc::downgrade(doc);
    glib::spawn_future_local(async move {
        let result = file.load_contents_future().await;
        let Some(doc) = weak.upgrade() else { return };
        // Another file was picked while this one loaded.
        if doc.file.borrow().as_ref() != Some(&file) {
            return;
        }
        match result {
            Ok((bytes, _)) => show(&doc, &file, &render::decode(&bytes)),
            // A deleted file keeps its last rendering (`NAV-5`).
            Err(e) if doc.shown.get() && e.matches(gio::IOErrorEnum::NotFound) => {}
            Err(e) => {
                doc.shown.set(false);
                set_main(&doc, &centred(&file.parse_name(), e.message()));
            }
        }
    });
}

fn show(doc: &Rc<Doc>, file: &gio::File, src: &str) {
    let blocks = render::parse(src);
    let mut ctx = Ctx {
        doc: Rc::downgrade(doc),
        base: file.uri().to_string(),
        colors: colors(&doc.window),
        anchors: Vec::new(),
    };
    let content = ctx.blocks(&blocks, 0);
    content.add_css_class("document");
    doc.column.set_child(&content);
    doc.anchors.replace(ctx.anchors);
    if !doc.shown.replace(true) {
        set_main(doc, &doc.scrolled);
    }
    if let Some(name) = doc.fragment.take() {
        let weak = Rc::downgrade(doc);
        doc.column.after_next_allocate(move || {
            // Headings have their positions once this layout pass is done.
            glib::idle_add_local_once(move || {
                if let Some(doc) = weak.upgrade() {
                    scroll_to(&doc, &name);
                }
            });
        });
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
            // A file of the folder's tree opens in the window (`NAV-11`).
            let sidebar = doc.sidebar.borrow().clone();
            if let Some(sidebar) = sidebar
                && let Some((file, fragment)) = crate::tree::link_in_tree(&uri, &sidebar.files())
            {
                sidebar.select(&file);
                show_file(&doc, &gio::File::for_uri(&file));
                doc.fragment.replace(fragment);
                return;
            }
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
        let table = Table::new();
        table.set_halign(gtk::Align::Start);
        let cell = |inlines: &[Inline], row: usize, col: usize| {
            let label = self.label(inlines);
            label.add_css_class("cell");
            if row == 0 {
                label.add_css_class("th");
                label.add_css_class("top");
            }
            if col == 0 {
                label.add_css_class("left");
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
            table.attach(&cell(inlines, 0, c), 0, c);
        }
        for (r, row) in rows.iter().enumerate() {
            for (c, inlines) in row.iter().enumerate() {
                table.attach(&cell(inlines, r + 1, c), r + 1, c);
            }
        }
        table.upcast()
    }
}
