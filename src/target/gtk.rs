//! GTK 3/4 and libadwaita: named colors, dark preference and theme name.

use crate::environment::Environment;
use crate::error::TargetError;
use crate::target::Target;
use crate::theme::Theme;
use crate::util::managed_block::{self, Comment};
use crate::util::{fs, gsettings, ini, paths};

const COLORS: &str = include_str!("../../templates/gtk.css");

pub(crate) const GTK_VERSIONS: [&str; 2] = ["gtk-3.0", "gtk-4.0"];

pub struct Gtk;

impl Target for Gtk {
    fn name(&self) -> &str {
        "gtk"
    }

    fn apply(&self, theme: &Theme, environment: &Environment) -> Result<(), TargetError> {
        let colors = theme.render(COLORS)?;
        let gtk_theme = theme
            .gtk_theme
            .as_deref()
            .filter(|name| paths::is_gtk_theme_installed(name));
        let mut settings = vec![(
            "gtk-application-prefer-dark-theme",
            theme.mode.pick("1", "0"),
        )];
        settings.extend(gtk_theme.map(|name| ("gtk-theme-name", name)));

        for version in GTK_VERSIONS {
            let dir = environment.config_path(version);
            fs::update(&dir.join("gtk.css"), |css| {
                Ok(managed_block::upsert(css, Comment::Css, &colors))
            })?;
            fs::update(&dir.join("settings.ini"), |text| {
                Ok(ini::set_keys(text, "Settings", &settings))
            })?;
        }

        if environment.runs_commands() && gsettings::is_available() {
            gsettings::set("color-scheme", theme.mode.pick("prefer-dark", "default"))?;
            if let Some(name) = gtk_theme {
                gsettings::set("gtk-theme", name)?;
            }
            gsettings::set_if_supported("accent-color", theme.accent_name());
        }
        Ok(())
    }
}
