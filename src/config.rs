use gtk::glib;
use std::io::Write;
use std::path::{Path, PathBuf};

const CONFIG_EXAMPLE: &str = include_str!("../data/config.example.ini");

/// `$XDG_CONFIG_HOME/downer` (`INT-1`).
pub fn dir() -> PathBuf {
    glib::user_config_dir().join("downer")
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub flavor: String,
    pub accent: String,
    pub opacity: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            flavor: "mocha".into(),
            accent: "mauve".into(),
            opacity: 0.85,
        }
    }
}

impl Config {
    /// Reads `config.ini`, first creating it if missing (`CONF-8`).
    pub fn load() -> Self {
        Self::load_from(&dir())
    }

    fn load_from(dir: &Path) -> Self {
        create_missing(dir);
        read(&dir.join("config.ini"))
            .map(|kf| Self::from_keyfile(&kf))
            .unwrap_or_default()
    }

    fn from_keyfile(kf: &glib::KeyFile) -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = kf.string("appearance", "flavor") {
            cfg.flavor = v.to_string();
        }
        if let Ok(v) = kf.string("appearance", "accent") {
            cfg.accent = v.to_string();
        }
        if let Ok(v) = kf.double("appearance", "opacity") {
            cfg.opacity = v.clamp(0.0, 1.0);
        }
        cfg
    }
}

/// The key file at `path`, or `None` when it is missing or broken; broken is warned about.
fn read(path: &Path) -> Option<glib::KeyFile> {
    let kf = glib::KeyFile::new();
    match kf.load_from_file(path, glib::KeyFileFlags::NONE) {
        Ok(_) => Some(kf),
        Err(e) => {
            if !e.matches(glib::FileError::Noent) {
                glib::g_warning!("downer", "{}: {}", path.display(), e.message());
            }
            None
        }
    }
}

/// `example` with every setting commented out, so the file documents the defaults
/// without pinning them.
fn commented(example: &str) -> String {
    let mut out = String::from(
        "# Created by Downer with every default commented out. To change one,\n\
         # remove the leading '# ' and edit the value.\n",
    );
    for line in example.lines() {
        if line.starts_with(|c: char| c.is_ascii_lowercase()) && line.contains(" =") {
            out.push_str("# ");
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Writes the commented example if `config.ini` is missing from `dir` (`CONF-8`).
/// Anything already at the path, a dangling symlink included, is left alone.
fn create_missing(dir: &Path) {
    let path = dir.join("config.ini");
    if path.symlink_metadata().is_ok() {
        return;
    }
    let written = std::fs::create_dir_all(dir).and_then(|()| {
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?
            .write_all(commented(CONFIG_EXAMPLE).as_bytes())
    });
    if let Err(e) = written {
        glib::g_warning!("downer", "could not create {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("downer-test-config-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn reads_settings_and_ignores_bad_values() {
        let kf = glib::KeyFile::new();
        kf.load_from_data(
            "[appearance]\nflavor=latte\naccent=red\nopacity=3\n",
            glib::KeyFileFlags::NONE,
        )
        .unwrap();
        let cfg = Config::from_keyfile(&kf);
        assert_eq!((cfg.flavor.as_str(), cfg.accent.as_str()), ("latte", "red"));
        assert_eq!(cfg.opacity, 1.0);
        kf.load_from_data("[appearance]\nopacity=lots\n", glib::KeyFileFlags::NONE)
            .unwrap();
        assert_eq!(Config::from_keyfile(&kf), Config::default());
    }

    #[test]
    fn broken_file_gives_defaults() {
        let dir = temp("broken");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.ini"), "not [ a key file\x00").unwrap();
        assert_eq!(Config::load_from(&dir), Config::default());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn creates_missing_file_with_the_defaults_commented_out() {
        let dir = temp("create").join("downer");
        assert_eq!(Config::load_from(&dir), Config::default());
        let config = std::fs::read_to_string(dir.join("config.ini")).unwrap();
        assert!(config.contains("\n# flavor = mocha\n"));
        for line in config.lines() {
            assert!(
                line.is_empty() || line.starts_with('#') || line.starts_with('['),
                "not commented out: {line}"
            );
        }
        std::fs::write(
            dir.join("config.ini"),
            config.replace("# flavor = mocha", "flavor = latte"),
        )
        .unwrap();
        assert_eq!(Config::load_from(&dir).flavor, "latte");
        std::fs::remove_dir_all(dir.parent().unwrap()).unwrap();
    }

    #[test]
    fn never_writes_to_existing_files_or_symlinks() {
        let dir = temp("keep");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("config.ini"), "[appearance]\nflavor=frappe\n").unwrap();
        assert_eq!(Config::load_from(&dir).flavor, "frappe");
        assert_eq!(
            std::fs::read_to_string(dir.join("config.ini")).unwrap(),
            "[appearance]\nflavor=frappe\n"
        );
        // Like a dotfiles link whose target isn't there yet.
        std::fs::remove_file(dir.join("config.ini")).unwrap();
        std::os::unix::fs::symlink(dir.join("elsewhere.ini"), dir.join("config.ini")).unwrap();
        assert_eq!(Config::load_from(&dir), Config::default());
        assert!(!dir.join("elsewhere.ini").exists());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn starts_with_defaults_when_the_file_cant_be_created() {
        let blocker = temp("blocked");
        std::fs::write(&blocker, "a file, not a folder").unwrap();
        assert_eq!(
            Config::load_from(&blocker.join("downer")),
            Config::default()
        );
        std::fs::remove_file(&blocker).unwrap();
    }
}
