//! Checks the spec's tables against themselves and, as they appear, against the code,
//! so the spec can't silently drift. See spec/README.md.

use std::collections::HashSet;
use std::path::Path;

/// Body rows of the table between `<!-- spec:NAME:begin -->` and `...:end -->`.
#[allow(dead_code)]
pub fn table(doc: &str, name: &str) -> Vec<Vec<String>> {
    let begin = format!("<!-- spec:{name}:begin -->");
    let end = format!("<!-- spec:{name}:end -->");
    let start = doc
        .find(&begin)
        .unwrap_or_else(|| panic!("missing {begin}"))
        + begin.len();
    let stop = doc[start..]
        .find(&end)
        .unwrap_or_else(|| panic!("missing {end}"))
        + start;
    doc[start..stop]
        .lines()
        .map(str::trim)
        .filter(|l| l.starts_with('|'))
        .skip(2)
        .map(|l| {
            l.trim_matches('|')
                .split('|')
                .map(|c| c.trim().to_owned())
                .collect()
        })
        .collect()
}

#[test]
fn requirement_rows_are_well_formed() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("spec");
    let mut ids = HashSet::new();
    for entry in std::fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for line in text.lines() {
            // A requirement row starts with an ID like `PROD-1`.
            let Some(id) = line
                .strip_prefix("| `")
                .and_then(|l| l.split('`').next())
                .filter(|id| {
                    id.split_once('-').is_some_and(|(p, n)| {
                        !p.is_empty()
                            && p.chars().all(|c| c.is_ascii_uppercase())
                            && n.parse::<u32>().is_ok()
                    })
                })
            else {
                continue;
            };
            assert!(ids.insert(id.to_owned()), "{id} is used twice");
            let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
            let ok = cells.iter().skip(2).any(|c| {
                ["planned", "partial", "done", "dropped"]
                    .iter()
                    .any(|s| c.starts_with(s))
            });
            assert!(ok, "{id} in {} has no valid status", path.display());
        }
    }
    assert!(!ids.is_empty(), "no requirement rows found in spec/");
}
