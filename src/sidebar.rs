//! The folder sidebar (`NAV-6`…`NAV-10`, `LOOK-9`, `LOOK-10`): a tree of the folder's
//! Markdown files, as rows of a `ListBox` whose collapsed descendants are hidden, so
//! Up and Down skip them and the width follows the shown rows.

use crate::tree::{self, Node, Raw};
use gtk::{gio, glib, prelude::*};
use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::time::Duration;

const ATTRIBUTES: &str = "standard::name,standard::display-name,standard::type,\
                          standard::is-hidden,standard::is-symlink,standard::content-type";
const RESCAN_DELAY: Duration = Duration::from_millis(300);
const INDENT: i32 = 16;

struct Row {
    uri: String,
    is_dir: bool,
    /// Index of the folder row this row is in.
    parent: Option<usize>,
    widget: gtk::ListBoxRow,
    arrow: Option<gtk::Image>,
}

pub struct Sidebar {
    root: gio::File,
    pub widget: gtk::ScrolledWindow,
    list: gtk::ListBox,
    tree: RefCell<Vec<Node>>,
    rows: RefCell<Vec<Row>>,
    expanded: RefCell<HashSet<String>>,
    current: RefCell<Option<String>>,
    monitors: RefCell<Vec<gio::FileMonitor>>,
    rescan: RefCell<Option<glib::SourceId>>,
    /// Bumped on every scan, so a slow, stale scan is dropped.
    generation: Cell<u64>,
    on_open: Box<dyn Fn(gio::File)>,
    on_scanned: Box<dyn Fn(Result<(), glib::Error>, bool)>,
}

type Scan = Pin<Box<dyn Future<Output = Result<Vec<Raw>, glib::Error>>>>;

/// Lists `dir` and, below it, every folder `tree::descend` allows. `dirs` collects the
/// folders listed, to watch.
fn scan(dir: gio::File, dirs: Rc<RefCell<Vec<gio::File>>>) -> Scan {
    Box::pin(async move {
        let enumerator = dir
            .enumerate_children_future(
                ATTRIBUTES,
                gio::FileQueryInfoFlags::NONE,
                glib::Priority::LOW,
            )
            .await?;
        dirs.borrow_mut().push(dir.clone());
        let mut out = Vec::new();
        loop {
            let infos = enumerator
                .next_files_future(64, glib::Priority::LOW)
                .await?;
            if infos.is_empty() {
                break;
            }
            for info in infos {
                let child = dir.child(info.name());
                let mut raw = Raw {
                    name: info.display_name().to_string(),
                    uri: child.uri().to_string(),
                    is_dir: info.file_type() == gio::FileType::Directory,
                    is_symlink: info.is_symlink(),
                    hidden: info.is_hidden(),
                    content_type: info.content_type().map(|c| c.to_string()),
                    children: Vec::new(),
                };
                if tree::descend(&raw) {
                    // A subfolder that can't be read is left out, not an error.
                    raw.children = scan(child, dirs.clone()).await.unwrap_or_default();
                }
                out.push(raw);
            }
        }
        Ok(out)
    })
}

impl Sidebar {
    /// `on_scanned(result, first)` is called after each scan; `first` is true once.
    pub fn new(
        root: &gio::File,
        on_open: impl Fn(gio::File) + 'static,
        on_scanned: impl Fn(Result<(), glib::Error>, bool) + 'static,
    ) -> Rc<Self> {
        let list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .activate_on_single_click(true)
            .css_classes(["tree"])
            .build();
        let widget = gtk::ScrolledWindow::builder()
            // As wide as the widest shown row, up to 280px with the 6px padding
            // (LOOK-10). With Never, GTK ignores max-content-width; External has no
            // scrollbar either.
            .hscrollbar_policy(gtk::PolicyType::External)
            .propagate_natural_width(true)
            .max_content_width(274)
            .child(&list)
            .css_classes(["sidebar"])
            // Focus goes to the rows; a focused scrolled window ignores Up and Down.
            .focusable(false)
            .hexpand(false)
            .build();
        let sidebar = Rc::new(Self {
            root: root.clone(),
            widget,
            list,
            tree: RefCell::default(),
            rows: RefCell::default(),
            expanded: RefCell::default(),
            current: RefCell::default(),
            monitors: RefCell::default(),
            rescan: RefCell::default(),
            generation: Cell::new(0),
            on_open: Box::new(on_open),
            on_scanned: Box::new(on_scanned),
        });
        let weak = Rc::downgrade(&sidebar);
        sidebar.list.connect_row_activated(move |_, row| {
            if let Some(sidebar) = weak.upgrade() {
                sidebar.activate(row.index());
            }
        });
        sidebar.scan(true);
        sidebar
    }

    /// Stops watching the folder; called when the window closes.
    pub fn stop(&self) {
        for monitor in self.monitors.take() {
            monitor.cancel();
        }
        if let Some(id) = self.rescan.take() {
            id.remove();
        }
        self.generation.set(self.generation.get() + 1);
    }

    pub fn files(&self) -> HashSet<String> {
        tree::files(&self.tree.borrow())
    }

    pub fn first_file(&self) -> Option<gio::File> {
        tree::first_file(&self.tree.borrow()).map(|n| gio::File::for_uri(&n.uri))
    }

    fn scan(self: &Rc<Self>, first: bool) {
        let generation = self.generation.get() + 1;
        self.generation.set(generation);
        let weak = Rc::downgrade(self);
        let root = self.root.clone();
        glib::spawn_future_local(async move {
            let dirs = Rc::new(RefCell::new(Vec::new()));
            let result = scan(root, dirs.clone()).await;
            let Some(sidebar) = weak.upgrade() else {
                return;
            };
            if sidebar.generation.get() != generation {
                return;
            }
            match result {
                Ok(listing) => {
                    sidebar.tree.replace(tree::build(&listing));
                    sidebar.rebuild();
                    sidebar.watch(&dirs.borrow());
                    (sidebar.on_scanned)(Ok(()), first);
                }
                Err(e) => (sidebar.on_scanned)(Err(e), first),
            }
        });
    }

    /// Rescans shortly after anything changes in a listed folder (`NAV-10`).
    fn watch(self: &Rc<Self>, dirs: &[gio::File]) {
        let mut monitors = Vec::new();
        for dir in dirs {
            let Ok(monitor) =
                dir.monitor_directory(gio::FileMonitorFlags::WATCH_MOVES, gio::Cancellable::NONE)
            else {
                continue;
            };
            let weak = Rc::downgrade(self);
            monitor.connect_changed(move |_, _, _, event| {
                use gio::FileMonitorEvent as E;
                if matches!(
                    event,
                    E::Changed | E::ChangesDoneHint | E::AttributeChanged | E::PreUnmount
                ) {
                    return;
                }
                let Some(sidebar) = weak.upgrade() else {
                    return;
                };
                if let Some(id) = sidebar.rescan.take() {
                    id.remove();
                }
                let weak = weak.clone();
                let id = glib::timeout_add_local_once(RESCAN_DELAY, move || {
                    if let Some(sidebar) = weak.upgrade() {
                        sidebar.rescan.take();
                        sidebar.scan(false);
                    }
                });
                sidebar.rescan.replace(Some(id));
            });
            monitors.push(monitor);
        }
        for old in self.monitors.replace(monitors) {
            old.cancel();
        }
    }

    fn rebuild(&self) {
        while let Some(row) = self.list.row_at_index(0) {
            self.list.remove(&row);
        }
        let mut rows = Vec::new();
        let tree = self.tree.borrow();
        self.add_rows(&tree, 0, None, &mut rows);
        // Folders that no longer exist are forgotten.
        let dirs: HashSet<String> = rows
            .iter()
            .filter(|r| r.is_dir)
            .map(|r| r.uri.clone())
            .collect();
        self.expanded.borrow_mut().retain(|u| dirs.contains(u));
        drop(tree);
        self.rows.replace(rows);
        self.update();
    }

    fn add_rows(&self, nodes: &[Node], depth: i32, parent: Option<usize>, rows: &mut Vec<Row>) {
        for node in nodes {
            let line = gtk::Box::builder()
                .spacing(6)
                .margin_start(depth * INDENT)
                .build();
            let arrow = node.children.as_ref().map(|_| {
                let arrow = gtk::Image::from_icon_name("pan-end-symbolic");
                arrow.add_css_class("arrow");
                arrow
            });
            match &arrow {
                Some(arrow) => line.append(arrow),
                None => line.append(&gtk::Box::builder().width_request(16).build()),
            }
            line.append(&gtk::Image::from_icon_name(if node.children.is_some() {
                "folder-symbolic"
            } else {
                "text-x-generic-symbolic"
            }));
            line.append(
                &gtk::Label::builder()
                    .label(&node.name)
                    .xalign(0.0)
                    .hexpand(true)
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .build(),
            );
            let widget = gtk::ListBoxRow::builder().child(&line).build();
            widget.set_tooltip_text(Some(&node.name));
            self.list.append(&widget);
            let index = rows.len();
            rows.push(Row {
                uri: node.uri.clone(),
                is_dir: node.children.is_some(),
                parent,
                widget,
                arrow,
            });
            if let Some(children) = &node.children {
                self.add_rows(children, depth + 1, Some(index), rows);
            }
        }
    }

    /// Shows the rows whose folders are all expanded, and marks the current one.
    fn update(&self) {
        let rows = self.rows.borrow();
        let expanded = self.expanded.borrow();
        let current = self.current.borrow();
        let mut shown = vec![false; rows.len()];
        for (i, row) in rows.iter().enumerate() {
            shown[i] = match row.parent {
                None => true,
                Some(p) => shown[p] && expanded.contains(&rows[p].uri),
            };
            row.widget.set_visible(shown[i]);
            if let Some(arrow) = &row.arrow {
                arrow.set_icon_name(Some(if expanded.contains(&row.uri) {
                    "pan-down-symbolic"
                } else {
                    "pan-end-symbolic"
                }));
            }
            if current.as_deref() == Some(row.uri.as_str()) {
                row.widget.add_css_class("current");
            } else {
                row.widget.remove_css_class("current");
            }
        }
    }

    fn activate(&self, index: i32) {
        let (uri, is_dir) = {
            let rows = self.rows.borrow();
            let Some(row) = usize::try_from(index).ok().and_then(|i| rows.get(i)) else {
                return;
            };
            (row.uri.clone(), row.is_dir)
        };
        if is_dir {
            let mut expanded = self.expanded.borrow_mut();
            if !expanded.remove(&uri) {
                expanded.insert(uri);
            }
            drop(expanded);
            self.update();
        } else {
            self.select(&uri);
            (self.on_open)(gio::File::for_uri(&uri));
        }
    }

    /// Marks `uri` as the current file and expands the folders above it (`NAV-8`).
    pub fn select(&self, uri: &str) {
        if let Some(path) = tree::ancestors(&self.tree.borrow(), uri) {
            self.expanded.borrow_mut().extend(path);
        }
        self.current.replace(Some(uri.to_owned()));
        self.update();
    }
}
