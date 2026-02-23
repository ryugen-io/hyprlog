//! TOML configuration loading, `source = "..."` include resolution, and per-app override merging.
//!
//! Separated from struct definitions so that the loading logic (file I/O, cycle detection,
//! merge strategy) stays independent of the serde schema.

mod structs;

pub use structs::{
    AppConfig, AppFileConfig, AppTerminalConfig, CleanupConfig, FileConfig, GeneralConfig,
    HighlightConfig, HyprlandConfig, IconsConfig, JsonConfig, MessageConfigFile, PatternsConfig,
    PresetConfig, RetentionConfig, ScopeConfigFile, ShellConfig, TagConfigFile, TerminalConfig,
};

use crate::fmt::{Alignment, Color, IconType, Transform};
use crate::internal;
use crate::level::Level;
use hypr_conf::{ConfigMetaSpec, resolve_config_path_strict};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

const TYPE_VALUE: &str = "logging";
const CONFIG_EXTENSIONS: &[&str] = &["conf"];

fn config_meta_spec() -> ConfigMetaSpec<'static> {
    ConfigMetaSpec::for_type(TYPE_VALUE, CONFIG_EXTENSIONS)
}

/// A completely empty config file must still produce a working logger — `#[serde(default)]`
/// on every field ensures zero-config works out of the box.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    /// Severity filtering and app identity apply to all outputs — they belong above any specific backend.
    pub general: GeneralConfig,
    /// Terminal output needs its own color, icon, and structure settings independent of file output.
    pub terminal: TerminalConfig,
    /// REPL theme and prompt settings don't affect non-interactive usage — separate section avoids clutter.
    pub shell: ShellConfig,
    /// File output has different concerns than terminal — base directory, rotation, and naming patterns.
    pub file: FileConfig,
    /// JSONL output serves a different purpose (machine-readable queries) than text logs — separate config.
    pub json: JsonConfig,
    /// Hyprland integration is optional — these settings only matter when the feature flag is active.
    pub hyprland: HyprlandConfig,
    /// Retention defaults belong in config so `hyprlog cleanup` works without flags every time.
    pub cleanup: CleanupConfig,
    /// Tag appearance varies by user preference — some want `[INFO]`, others `INFO:` or `ℹ`.
    pub tag: TagConfigFile,
    /// Scope column width and alignment affect readability — different users prefer different layouts.
    pub scope: ScopeConfigFile,
    /// Message transforms (uppercase, lowercase) apply independently of tag/scope transforms.
    pub message: MessageConfigFile,
    /// Auto-highlighting makes URLs, paths, and error keywords visible without manual markup.
    pub highlight: HighlightConfig,
    /// Named colors avoid repeating hex codes across tag, highlight, and scope config sections.
    pub colors: HashMap<String, String>,
    /// Icon glyphs vary by font availability — users need to override defaults for their environment.
    pub icons: IconsConfig,
    /// Frequently-used log commands are tedious to retype — presets bundle them into single names.
    pub presets: HashMap<String, PresetConfig>,
    /// Different apps sharing one config need to diverge on level, output path, or terminal settings
    /// without maintaining separate config files for each.
    pub apps: HashMap<String, AppConfig>,
}

/// Scans raw TOML for `source = "..."` directives before deserialization,
/// since serde cannot handle them -- they are a Hyprland-style extension.
/// Returns the extracted paths and the remaining TOML content stripped of those lines.
#[doc(hidden)]
#[must_use]
pub fn extract_sources(content: &str) -> (Vec<String>, String) {
    let mut sources = Vec::new();
    let mut remaining = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("source") && trimmed.contains('=') {
            if let Some(path) = trimmed
                .split('=')
                .nth(1)
                .map(|s| s.trim().trim_matches('"').trim_matches('\''))
                && !path.is_empty()
            {
                sources.push(path.to_string());
            }
        } else {
            remaining.push_str(line);
            remaining.push('\n');
        }
    }

    (sources, remaining)
}

impl Config {
    /// Primary entry point — CLI and library consumers both need the user's full config
    /// with all `source = "..."` includes resolved and per-app overrides merged.
    ///
    /// # Errors
    /// Fails if the config directory can't be determined or TOML parsing hits a syntax error.
    pub fn load() -> Result<Self, crate::Error> {
        internal::debug("CONFIG", "Loading config from default location");
        let config_path = Self::get_config_path()?;
        let config = Self::load_with_sources(&config_path, &mut HashSet::new())?;
        internal::info(
            "CONFIG",
            &format!("Config loaded from {}", config_path.display()),
        );
        Ok(config)
    }

    /// Loads configuration from an explicit path instead of the default location.
    ///
    /// Useful for FFI callers and tests that need to point at a non-standard config file.
    ///
    /// # Errors
    /// Returns error if the file cannot be read, parsed, or contains cyclic includes.
    pub fn load_from(path: &Path) -> Result<Self, crate::Error> {
        Self::load_with_sources(path, &mut HashSet::new())
    }

    /// Recursive loader that expands `source = "..."` includes while tracking
    /// visited paths in `seen` to break include cycles.
    fn load_with_sources(path: &Path, seen: &mut HashSet<PathBuf>) -> Result<Self, crate::Error> {
        if !path.exists() {
            internal::debug("CONFIG", "Config file not found, using defaults");
            return Ok(Self::default());
        }

        let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

        if !seen.insert(canonical.clone()) {
            internal::warn(
                "CONFIG",
                &format!("Cyclic include detected: {}", canonical.display()),
            );
            return Err(crate::Error::CyclicInclude(canonical));
        }

        let content = fs::read_to_string(path)?;
        let (sources, toml_content) = extract_sources(&content);
        let mut config: Self = toml::from_str(&toml_content)?;

        for source_path in sources {
            internal::debug("CONFIG", &format!("Processing source: {source_path}"));
            let expanded = shellexpand::tilde(&source_path);
            let source_file = Path::new(expanded.as_ref());
            if source_file.exists() {
                let source_config = Self::load_with_sources(source_file, seen)?;
                config.merge(source_config);
            } else {
                internal::warn("CONFIG", &format!("Source file not found: {source_path}"));
            }
        }

        Ok(config)
    }

    /// Folds a sourced config's map-type fields into `self` without overwriting
    /// existing keys, so the primary file's values take precedence over includes.
    pub fn merge(&mut self, other: Self) {
        for (k, v) in other.colors {
            self.colors.entry(k).or_insert(v);
        }
        for (k, v) in other.presets {
            self.presets.entry(k).or_insert(v);
        }
        for (k, v) in other.icons.nerdfont {
            self.icons.nerdfont.entry(k).or_insert(v);
        }
        for (k, v) in other.icons.ascii {
            self.icons.ascii.entry(k).or_insert(v);
        }
        for (k, v) in other.tag.labels {
            self.tag.labels.entry(k).or_insert(v);
        }
        for (k, v) in other.highlight.keywords {
            self.highlight.keywords.entry(k).or_insert(v);
        }
        for (k, v) in other.apps {
            self.apps.entry(k).or_insert(v);
        }
    }

    /// Per-app overrides let multiple binaries share one config file while still
    /// diverging on level, terminal, or file settings — without this, each app
    /// would need its own config file.
    #[must_use]
    pub fn for_app(&self, app_name: &str) -> Self {
        let mut config = self.clone();

        if let Some(app_config) = self.apps.get(app_name) {
            // Level gates everything else, so it must be overridden first
            if let Some(ref level) = app_config.level {
                config.general.level.clone_from(level);
            }

            // Terminal settings are per-field optional so partial overrides work
            if let Some(ref terminal) = app_config.terminal {
                if let Some(enabled) = terminal.enabled {
                    config.terminal.enabled = enabled;
                }
                if let Some(colors) = terminal.colors {
                    config.terminal.colors = colors;
                }
                if let Some(ref icons) = terminal.icons {
                    config.terminal.icons.clone_from(icons);
                }
                if let Some(ref structure) = terminal.structure {
                    config.terminal.structure.clone_from(structure);
                }
            }

            // File overrides let an app redirect its logs to a separate directory
            if let Some(ref file) = app_config.file {
                if let Some(enabled) = file.enabled {
                    config.file.enabled = enabled;
                }
                if let Some(ref base_dir) = file.base_dir {
                    config.file.base_dir.clone_from(base_dir);
                }
            }
        }

        config
    }

    /// XDG-compliant path under `~/.config/hypr/` — matches Hyprland's config directory convention.
    ///
    /// # Errors
    /// Fails when the platform has no concept of a config directory (unlikely on Linux).
    pub fn get_config_path() -> Result<PathBuf, crate::Error> {
        let default_path = directories::BaseDirs::new()
            .map(|dirs| dirs.config_dir().join("hypr").join("hyprlog.conf"))
            .ok_or(crate::Error::ConfigDirNotFound)?;

        // Main default config is valid without metadata.
        if default_path.exists() {
            return Ok(default_path);
        }

        if let Some(root) = default_path.parent()
            && let Some(found) = discover_metadata_config(root)
        {
            return Ok(found);
        }

        Ok(default_path)
    }

    /// Config stores level as a string for TOML ergonomics — this converts to the typed enum the logger needs.
    #[must_use]
    pub fn parse_level(&self) -> Level {
        self.general.level.parse().unwrap_or(Level::Info)
    }

    /// String-based config needs conversion to the typed enum the icon renderer expects.
    #[must_use]
    pub fn parse_icon_type(&self) -> IconType {
        match self.terminal.icons.to_lowercase().as_str() {
            "ascii" => IconType::Ascii,
            "none" => IconType::None,
            _ => IconType::NerdFont,
        }
    }

    /// Accepts multiple aliases ("uppercase"/"upper") for user convenience — maps them all to one enum variant.
    #[must_use]
    pub fn parse_transform(&self) -> Transform {
        match self.tag.transform.to_lowercase().as_str() {
            "uppercase" | "upper" => Transform::Uppercase,
            "lowercase" | "lower" => Transform::Lowercase,
            "capitalize" | "cap" => Transform::Capitalize,
            _ => Transform::None,
        }
    }

    /// Defaults to center alignment since tag labels have consistent short widths.
    #[must_use]
    pub fn parse_alignment(&self) -> Alignment {
        match self.tag.alignment.to_lowercase().as_str() {
            "left" => Alignment::Left,
            "right" => Alignment::Right,
            _ => Alignment::Center,
        }
    }

    /// Scope transform is independent of tag transform — users may want uppercase tags but lowercase scopes.
    #[must_use]
    pub fn parse_scope_transform(&self) -> Transform {
        match self.scope.transform.to_lowercase().as_str() {
            "uppercase" | "upper" => Transform::Uppercase,
            "lowercase" | "lower" => Transform::Lowercase,
            "capitalize" | "cap" => Transform::Capitalize,
            _ => Transform::None,
        }
    }

    /// Defaults to left alignment since scope names have variable length and left-align reads more naturally.
    #[must_use]
    pub fn parse_scope_alignment(&self) -> Alignment {
        match self.scope.alignment.to_lowercase().as_str() {
            "right" => Alignment::Right,
            "center" => Alignment::Center,
            _ => Alignment::Left,
        }
    }

    /// Message body transforms are rare but some users want all-lowercase or all-uppercase output.
    #[must_use]
    pub fn parse_message_transform(&self) -> Transform {
        match self.message.transform.to_lowercase().as_str() {
            "uppercase" | "upper" => Transform::Uppercase,
            "lowercase" | "lower" => Transform::Lowercase,
            "capitalize" | "cap" => Transform::Capitalize,
            _ => Transform::None,
        }
    }

    /// Named color lookup lets config sections reference `accent` instead of repeating `#ff79c6` everywhere.
    #[must_use]
    pub fn get_color(&self, name: &str) -> Option<Color> {
        self.colors.get(name).map(|hex| Color::from_hex(hex))
    }
}

fn discover_metadata_config(config_root: &Path) -> Option<PathBuf> {
    let fallback = config_root.join("hyprlog.conf");
    resolve_config_path_strict(config_root, &fallback, &config_meta_spec())
}
