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

const CONFIGURATION_MD: &str = include_str!("../spec/configuration.md");
const EXAMPLE_INI: &str = include_str!("../data/config.example.ini");

fn default_value(cfg: &crate::config::Config, section: &str, key: &str) -> Option<String> {
    Some(match (section, key) {
        ("appearance", "flavor") => cfg.flavor.clone(),
        ("appearance", "accent") => cfg.accent.clone(),
        ("appearance", "opacity") => cfg.opacity.to_string(),
        _ => return None,
    })
}
const CONFIG_KEY_COUNT: usize = 3;

fn config_rows() -> Vec<(String, String, String)> {
    table(CONFIGURATION_MD, "config")
        .into_iter()
        .map(|r| {
            let c = |i: usize| r[i].trim_matches('`').to_owned();
            (c(0), c(1), c(2))
        })
        .collect()
}

#[test]
fn config_defaults_match_spec() {
    let cfg = crate::config::Config::default();
    let rows = config_rows();
    assert_eq!(
        rows.len(),
        CONFIG_KEY_COUNT,
        "spec/configuration.md must list every config key"
    );
    for (section, key, default) in rows {
        let code = default_value(&cfg, &section, &key)
            .unwrap_or_else(|| panic!("spec lists unknown key [{section}] {key}"));
        assert_eq!(
            code, default,
            "[{section}] {key}: Config::default() vs spec"
        );
    }
}

#[test]
fn example_config_matches_spec() {
    let kf = gtk::glib::KeyFile::new();
    kf.load_from_data(EXAMPLE_INI, gtk::glib::KeyFileFlags::NONE)
        .unwrap();
    let mut count = 0;
    for group in kf.groups().iter() {
        count += kf.keys(group).map(|k| k.len()).unwrap_or(0);
    }
    assert_eq!(
        count, CONFIG_KEY_COUNT,
        "config.example.ini has keys the spec lacks"
    );
    for (section, key, default) in config_rows() {
        let value = kf
            .value(&section, &key)
            .unwrap_or_else(|_| panic!("config.example.ini lacks [{section}] {key}"));
        assert_eq!(
            value.as_str(),
            default,
            "[{section}] {key}: example vs spec"
        );
    }
}
