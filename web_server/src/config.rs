//! `score_server.toml` config file.
//!
//! Fields mirror the CLI flags. Search order:
//! 1. `./score_server.toml`
//! 2. `$XDG_CONFIG_HOME/guitar-io/score_server.toml`
//! 3. `~/.config/guitar-io/score_server.toml`
//!
//! CLI flags override values read from the file.

use std::path::{Path, PathBuf};

use thiserror::Error;
use toml_span::Span;
use toml_span::value::{Table, ValueInner};

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("failed to read config file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("TOML syntax error in {path}: {message}")]
    Parse { path: PathBuf, message: String },
    #[error("invalid type for key '{key}': expected {expected} at bytes {}..{}", .span.start, .span.end)]
    InvalidType {
        key: String,
        expected: &'static str,
        span: Span,
    },
    #[error("invalid value for key '{key}': {message} at bytes {}..{}", .span.start, .span.end)]
    InvalidValue {
        key: String,
        message: String,
        span: Span,
    },
}

/// Values loaded from a `score_server.toml` file. All fields optional so a
/// partial file overrides only the keys it sets; CLI flags override in turn.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ServerConfig {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub root: Option<PathBuf>,
    pub open: Option<bool>,
}

impl ServerConfig {
    /// Parse a TOML string. Empty input yields the default (all `None`).
    pub fn parse(content: &str) -> Result<Self, ConfigError> {
        if content.trim().is_empty() {
            return Ok(Self::default());
        }
        let value = toml_span::parse(content).map_err(|e| ConfigError::Parse {
            path: PathBuf::new(),
            message: e.to_string(),
        })?;
        let root_table = match value.as_ref() {
            ValueInner::Table(t) => t,
            _ => {
                return Err(ConfigError::InvalidType {
                    key: "<root>".into(),
                    expected: "table",
                    span: value.span,
                });
            }
        };

        Ok(Self {
            port: get_typed_integer::<u16>(root_table, "port")?,
            host: get_str(root_table, "host")?.map(str::to_owned),
            root: get_str(root_table, "root")?.map(PathBuf::from),
            open: get_bool(root_table, "open")?,
        })
    }

    /// Read the first `score_server.toml` found in the standard locations.
    /// Returns `Ok(default())` if no file exists.
    pub fn load_default() -> Result<Self, ConfigError> {
        for path in default_search_paths() {
            if path.is_file() {
                return Self::load_from(&path);
            }
        }
        Ok(Self::default())
    }

    pub fn load_from(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(|source| ConfigError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let mut cfg = Self::parse(&content).map_err(|e| match e {
            ConfigError::Parse { message, .. } => ConfigError::Parse {
                path: path.to_path_buf(),
                message,
            },
            other => other,
        })?;
        // Resolve relative `root` paths against the config file's directory
        // so `root = "scores"` next to the config does what the operator meant.
        if let Some(ref mut r) = cfg.root
            && r.is_relative()
            && let Some(parent) = path.parent()
        {
            *r = parent.join(&r);
        }
        Ok(cfg)
    }
}

fn default_search_paths() -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from("score_server.toml")];

    let sub = Path::new("guitar-io").join("score_server.toml");

    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME")
        && !xdg.is_empty()
    {
        paths.push(PathBuf::from(xdg).join(&sub));
    }
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(home).join(".config").join(&sub));
    }
    paths
}

// ── Typed extractors (toml-span manual style) ────────────────────────────────

fn get_str<'a>(t: &'a Table<'a>, key: &str) -> Result<Option<&'a str>, ConfigError> {
    match t.get(key) {
        Some(v) => match v.as_ref() {
            ValueInner::String(s) => Ok(Some(s.as_ref())),
            _ => Err(ConfigError::InvalidType {
                key: key.to_string(),
                expected: "string",
                span: v.span,
            }),
        },
        None => Ok(None),
    }
}

fn get_bool(t: &Table<'_>, key: &str) -> Result<Option<bool>, ConfigError> {
    match t.get(key) {
        Some(v) => match v.as_ref() {
            ValueInner::Boolean(b) => Ok(Some(*b)),
            _ => Err(ConfigError::InvalidType {
                key: key.to_string(),
                expected: "boolean",
                span: v.span,
            }),
        },
        None => Ok(None),
    }
}

fn get_typed_integer<T>(t: &Table<'_>, key: &str) -> Result<Option<T>, ConfigError>
where
    T: TryFrom<i64>,
{
    let Some(v) = t.get(key) else {
        return Ok(None);
    };
    let ValueInner::Integer(n) = v.as_ref() else {
        return Err(ConfigError::InvalidType {
            key: key.to_string(),
            expected: "integer",
            span: v.span,
        });
    };
    T::try_from(*n)
        .map(Some)
        .map_err(|_| ConfigError::InvalidValue {
            key: key.to_string(),
            message: format!("value {n} is out of range for this field"),
            span: v.span,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_yields_default() {
        let cfg = ServerConfig::parse("").unwrap();
        assert_eq!(cfg, ServerConfig::default());
        let cfg = ServerConfig::parse("   \n\n").unwrap();
        assert_eq!(cfg, ServerConfig::default());
    }

    #[test]
    fn full_config_parses() {
        let toml = r#"
            port = 8080
            host = "0.0.0.0"
            root = "/srv/scores"
            open = false
        "#;
        let cfg = ServerConfig::parse(toml).unwrap();
        assert_eq!(cfg.port, Some(8080));
        assert_eq!(cfg.host, Some("0.0.0.0".to_string()));
        assert_eq!(cfg.root, Some(PathBuf::from("/srv/scores")));
        assert_eq!(cfg.open, Some(false));
    }

    #[test]
    fn partial_config_leaves_other_fields_none() {
        let cfg = ServerConfig::parse(r#"port = 4000"#).unwrap();
        assert_eq!(cfg.port, Some(4000));
        assert!(cfg.root.is_none());
        assert!(cfg.open.is_none());
    }

    #[test]
    fn port_overflow_reports_invalid_value_not_invalid_type() {
        let err = ServerConfig::parse(r#"port = 70000"#).unwrap_err();
        assert!(
            matches!(err, ConfigError::InvalidValue { ref key, .. } if key == "port"),
            "expected InvalidValue for out-of-range port, got {err:?}"
        );
    }

    #[test]
    fn negative_port_reports_invalid_value() {
        let err = ServerConfig::parse(r#"port = -1"#).unwrap_err();
        assert!(matches!(err, ConfigError::InvalidValue { ref key, .. } if key == "port"));
    }

    #[test]
    fn wrong_type_reports_invalid_type() {
        let err = ServerConfig::parse(r#"port = "3000""#).unwrap_err();
        assert!(
            matches!(err, ConfigError::InvalidType { ref key, expected: "integer", .. } if key == "port")
        );
    }

    #[test]
    fn syntax_error_reports_parse() {
        let err = ServerConfig::parse("port = ==").unwrap_err();
        assert!(matches!(err, ConfigError::Parse { .. }));
    }

    #[test]
    fn load_from_resolves_relative_root_against_config_dir() {
        let dir = std::env::temp_dir().join(format!("ws_cfg_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let cfg_path = dir.join("score_server.toml");
        std::fs::write(&cfg_path, r#"root = "scores""#).unwrap();

        let cfg = ServerConfig::load_from(&cfg_path).unwrap();
        assert_eq!(cfg.root, Some(dir.join("scores")));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_from_preserves_absolute_root() {
        let dir = std::env::temp_dir().join(format!("ws_cfg_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let cfg_path = dir.join("score_server.toml");
        std::fs::write(&cfg_path, r#"root = "/etc/scores""#).unwrap();

        let cfg = ServerConfig::load_from(&cfg_path).unwrap();
        assert_eq!(cfg.root, Some(PathBuf::from("/etc/scores")));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
