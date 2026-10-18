//! The folder tree of a folder window (`NAV-7`), as plain data so it can be tested
//! without a display. `sidebar.rs` scans the folder into `Raw` and shows `Node`s.

use gtk::gio;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::iter::Peekable;
use std::str::Chars;

/// One entry of a folder listing, as GIO reports it.
#[derive(Debug, Clone)]
pub struct Raw {
    pub name: String,
    pub uri: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub hidden: bool,
    pub content_type: Option<String>,
    /// The listing of a folder that was scanned (`None` for files and skipped folders).
    pub children: Vec<Raw>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: String,
    pub uri: String,
    /// `Some` for a folder.
    pub children: Option<Vec<Node>>,
}

pub fn is_markdown(content_type: &str) -> bool {
    gio::content_type_is_a(content_type, "text/markdown")
}

/// Whether a scanned entry is a folder to descend into: symlinked folders are not
/// followed, and hidden ones are left out.
pub fn descend(raw: &Raw) -> bool {
    raw.is_dir && !raw.is_symlink && !raw.hidden
}

/// Filters, prunes and sorts a folder listing (`NAV-7`).
pub fn build(listing: &[Raw]) -> Vec<Node> {
    let mut dirs = Vec::new();
    let mut files = Vec::new();
    for raw in listing {
        if raw.hidden {
            continue;
        }
        if raw.is_dir {
            if !descend(raw) {
                continue;
            }
            let children = build(&raw.children);
            if !children.is_empty() {
                dirs.push(Node {
                    name: raw.name.clone(),
                    uri: raw.uri.clone(),
                    children: Some(children),
                });
            }
        } else if raw.content_type.as_deref().is_some_and(is_markdown) {
            files.push(Node {
                name: raw.name.clone(),
                uri: raw.uri.clone(),
                children: None,
            });
        }
    }
    dirs.sort_by(|a, b| natural_cmp(&a.name, &b.name));
    files.sort_by(|a, b| natural_cmp(&a.name, &b.name));
    dirs.extend(files);
    dirs
}

/// The file a folder window opens with (`NAV-9`).
pub fn first_file(tree: &[Node]) -> Option<&Node> {
    ["readme.md", "readme.markdown", "index.md"]
        .iter()
        .find_map(|want| {
            tree.iter()
                .find(|n| n.children.is_none() && n.name.to_lowercase() == *want)
        })
}

/// Every file URI in the tree.
pub fn files(tree: &[Node]) -> HashSet<String> {
    let mut out = HashSet::new();
    for node in tree {
        match &node.children {
            Some(children) => out.extend(files(children)),
            None => {
                out.insert(node.uri.clone());
            }
        }
    }
    out
}

/// The URIs of the folders that lead to `uri`, outermost first.
pub fn ancestors(tree: &[Node], uri: &str) -> Option<Vec<String>> {
    for node in tree {
        match &node.children {
            None if node.uri == uri => return Some(Vec::new()),
            Some(children) => {
                if let Some(mut path) = ancestors(children, uri) {
                    path.insert(0, node.uri.clone());
                    return Some(path);
                }
            }
            None => {}
        }
    }
    None
}

/// For a link target already resolved to a URI: the tree file it opens, and the
/// fragment to scroll to, when the link stays in the window (`NAV-11`).
pub fn link_in_tree(target: &str, files: &HashSet<String>) -> Option<(String, Option<String>)> {
    let (uri, fragment) = match target.split_once('#') {
        Some((u, f)) => (u, Some(f)),
        None => (target, None),
    };
    let fragment = fragment.filter(|f| !f.is_empty()).map(|f| {
        gtk::glib::Uri::unescape_string(f, None::<&str>)
            .map(|s| s.to_string())
            .unwrap_or_else(|| f.to_owned())
    });
    files.contains(uri).then(|| (uri.to_owned(), fragment))
}

/// Compares names ignoring case, with runs of digits compared as numbers.
pub fn natural_cmp(a: &str, b: &str) -> Ordering {
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        let ord = match (x.peek(), y.peek()) {
            (None, None) => return a.cmp(b),
            (None, _) => return Ordering::Less,
            (_, None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                take_number(&mut x).cmp(&take_number(&mut y))
            }
            _ => {
                let (c, d) = (x.next().unwrap(), y.next().unwrap());
                c.to_lowercase().cmp(d.to_lowercase())
            }
        };
        if ord != Ordering::Equal {
            return ord;
        }
    }
}

fn take_number(it: &mut Peekable<Chars>) -> u128 {
    let mut n: u128 = 0;
    while let Some(d) = it.peek().and_then(|c| c.to_digit(10)) {
        n = n.saturating_mul(10).saturating_add(d.into());
        it.next();
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(name: &str, ct: &str) -> Raw {
        Raw {
            name: name.into(),
            uri: format!("file:///d/{name}"),
            is_dir: false,
            is_symlink: false,
            hidden: name.starts_with('.'),
            content_type: Some(ct.into()),
            children: Vec::new(),
        }
    }

    fn md(name: &str) -> Raw {
        file(name, "text/markdown")
    }

    fn dir(name: &str, children: Vec<Raw>) -> Raw {
        Raw {
            is_dir: true,
            children,
            ..file(name, "inode/directory")
        }
    }

    fn names(tree: &[Node]) -> Vec<String> {
        tree.iter()
            .map(|n| match &n.children {
                Some(c) => format!("{}/[{}]", n.name, names(c).join(",")),
                None => n.name.clone(),
            })
            .collect()
    }

    /// `NAV-7`: Markdown only, hidden and empty folders left out, symlinked folders
    /// not followed, folders first, natural order.
    #[test]
    fn builds_a_pruned_sorted_tree() {
        let link = Raw {
            is_symlink: true,
            ..dir("linked", vec![md("in-link.md")])
        };
        let linked_file = Raw {
            is_symlink: true,
            ..md("linked-file.md")
        };
        let listing = vec![
            md("b10.md"),
            md("B2.md"),
            file("notes.txt", "text/plain"),
            md(".hidden.md"),
            dir(".git", vec![md("x.md")]),
            dir(
                "empty",
                vec![file("a.png", "image/png"), dir("deeper", vec![])],
            ),
            dir("Zeta", vec![dir("inner", vec![md("deep.md")])]),
            dir("alpha", vec![md("10.md"), md("9.md")]),
            link,
            linked_file,
            md("a.md"),
        ];
        assert_eq!(
            names(&build(&listing)),
            [
                "alpha/[9.md,10.md]",
                "Zeta/[inner/[deep.md]]",
                "a.md",
                "B2.md",
                "b10.md",
                "linked-file.md",
            ]
        );
    }

    /// `NAV-9`: README first, then index.md, ignoring case, top level only.
    #[test]
    fn picks_the_first_file() {
        let pick = |listing: Vec<Raw>| first_file(&build(&listing)).map(|n| n.name.clone());
        assert_eq!(
            pick(vec![md("index.md"), md("ReadMe.MD")]).as_deref(),
            Some("ReadMe.MD")
        );
        assert_eq!(
            pick(vec![md("index.md"), md("a.md")]).as_deref(),
            Some("index.md")
        );
        assert_eq!(
            pick(vec![md("README.markdown")]).as_deref(),
            Some("README.markdown")
        );
        assert_eq!(
            pick(vec![dir("sub", vec![md("README.md")]), md("a.md")]),
            None
        );
        assert_eq!(pick(vec![]), None);
    }

    #[test]
    fn finds_ancestors() {
        let tree = build(&[dir("a", vec![dir("b", vec![md("c.md")])]), md("top.md")]);
        assert_eq!(
            ancestors(&tree, "file:///d/c.md"),
            Some(vec!["file:///d/a".to_owned(), "file:///d/b".to_owned()])
        );
        assert_eq!(ancestors(&tree, "file:///d/top.md"), Some(vec![]));
        assert_eq!(ancestors(&tree, "file:///d/none.md"), None);
    }

    /// `NAV-11`: only links to files in the tree stay in the window.
    #[test]
    fn links_in_the_tree() {
        let tree = build(&[md("a.md"), dir("sub", vec![md("b.md")])]);
        let files = files(&tree);
        assert_eq!(
            link_in_tree("file:///d/b.md#part%201", &files),
            Some(("file:///d/b.md".into(), Some("part 1".into())))
        );
        assert_eq!(
            link_in_tree("file:///d/a.md", &files),
            Some(("file:///d/a.md".into(), None))
        );
        assert_eq!(link_in_tree("file:///d/notes.txt", &files), None);
        assert_eq!(link_in_tree("file:///elsewhere/a.md", &files), None);
        assert_eq!(link_in_tree("https://x.org/a.md", &files), None);
    }
}
