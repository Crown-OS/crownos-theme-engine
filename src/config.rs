//! The engine's own settings, stored as `~/.config/crownos/theme-engine.ron`.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EngineConfig {
    /// Target names to leave alone, e.g. `["spicetify"]`.
    pub disabled: Vec<String>,
    /// GTK theme per mode; applied only when that theme is installed.
    pub gtk_theme: Option<ModePair>,
    /// Extra apps themed from user-supplied templates.
    pub templates: Vec<TemplateSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModePair {
    pub dark: String,
    pub light: String,
}

/// A config file rendered from a template, see [`crate::Theme::render`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemplateSpec {
    pub name: String,
    /// The template file; `~/` expands to the home directory.
    pub template: PathBuf,
    /// Where the rendered file is written; `~/` expands to the home directory.
    pub output: PathBuf,
    /// Program and arguments run after the output changes, e.g. a reload command.
    #[serde(default)]
    pub reload: Vec<String>,
}

impl EngineConfig {
    pub const SECTION: &str = "theme-engine";

    /// Reads the settings, writing the defaults out when the file is missing.
    pub fn load() -> Self {
        crownos_config::load(Self::SECTION)
    }
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            disabled: Vec::new(),
            gtk_theme: Some(ModePair {
                dark: "adw-gtk3-dark".to_owned(),
                light: "adw-gtk3".to_owned(),
            }),
            templates: Vec::new(),
        }
    }
}
