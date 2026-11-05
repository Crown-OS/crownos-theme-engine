//! Freedesktop icon and cursor themes for GTK apps.

use crate::environment::Environment;
use crate::error::TargetError;
use crate::target::Target;
use crate::target::gtk::GTK_VERSIONS;
use crate::theme::Theme;
use crate::util::{fs, gsettings, ini};

pub struct Icons;

impl Target for Icons {
    fn name(&self) -> &str {
        "icons"
    }

    fn apply(&self, theme: &Theme, environment: &Environment) -> Result<(), TargetError> {
        let themes = [
            (
                "gtk-icon-theme-name",
                "icon-theme",
                theme.icon_theme.as_deref(),
            ),
            (
                "gtk-cursor-theme-name",
                "cursor-theme",
                theme.cursor_theme.as_deref(),
            ),
        ];
        let settings: Vec<_> = themes
            .iter()
            .filter_map(|(ini_key, _, name)| Some((*ini_key, (*name)?)))
            .collect();
        if settings.is_empty() {
            return Ok(());
        }

        for version in GTK_VERSIONS {
            fs::update(
                &environment.config_path(version).join("settings.ini"),
                |text| Ok(ini::set_keys(text, "Settings", &settings)),
            )?;
        }

        if environment.runs_commands() && gsettings::is_available() {
            for (_, gsettings_key, name) in themes {
                if let Some(name) = name {
                    gsettings::set(gsettings_key, name)?;
                }
            }
        }
        Ok(())
    }
}
