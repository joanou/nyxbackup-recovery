// Copyright (c) 2026 AltDrive, LLC
// SPDX-License-Identifier: Apache-2.0
// Nyx Backup Recovery - https://nyxbackup.com

//! Persistent user settings: download bandwidth, log level, theme.  Plain
//! JSON under [`crate::paths::settings_file`].  Missing file = defaults.

use crate::paths;
use serde::{Deserialize, Serialize};

/// User-tunable settings for the Recovery Tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Download bandwidth cap in kbps.  0 = unlimited.  Applied via
    /// `RateLimitedBackend` to the storage backend at restore time.
    #[serde(default)]
    pub download_bandwidth_kbps: u32,

    /// `tracing` log level: "error", "warn", "info", "debug", "trace".
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Theme name.  Matches one of the values the main app exposes
    /// (dracula, nord, catppuccin-mocha, solarized-dark, solarized-light,
    /// enchant, cyber).
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Locale override.  `"auto"` (default) follows the OS locale via
    /// `navigator.language`; otherwise one of the 24 supported codes
    /// ("en", "es", "fr", "de", "ja", ...).  Lets users on systems whose
    /// OS locale is unsupported (Hebrew, Arabic, Thai, ...) pick a
    /// language they understand instead of falling silently to English.
    /// Mirrors the main app's language picker behaviour.
    #[serde(default = "default_locale")]
    pub locale: String,
}

fn default_log_level() -> String {
    "info".to_string()
}
fn default_theme() -> String {
    "cyber".to_string()
}
fn default_locale() -> String {
    "auto".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            download_bandwidth_kbps: 0,
            log_level: default_log_level(),
            theme: default_theme(),
            locale: default_locale(),
        }
    }
}

impl Settings {
    /// Read settings from disk; returns defaults if the file is missing
    /// or unreadable.  Never an error - the Recovery Tool tolerates a
    /// missing or corrupt settings file by reverting to defaults.
    pub fn load() -> Self {
        let path = paths::settings_file();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Persist to disk.  Creates the data dir if needed.  Returns an error
    /// the GUI can surface as a toast, but does NOT block the user - all
    /// settings are also held in memory for the session.
    pub fn save(&self) -> std::io::Result<()> {
        let path = paths::settings_file();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(&path, s)?;
        Ok(())
    }
}
