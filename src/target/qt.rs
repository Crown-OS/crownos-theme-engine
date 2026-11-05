//! qt5ct / qt6ct: a generated color scheme plus the palette and icon settings
//! that point at it. Needs `QT_QPA_PLATFORMTHEME=qt5ct` / `qt6ct` in the session.

use std::fmt::Write;

use crate::color::{ColorFormat, Rgba};
use crate::environment::Environment;
use crate::error::TargetError;
use crate::palette::Palette;
use crate::target::Target;
use crate::theme::Theme;
use crate::util::{fs, ini, process};

const TOOLS: [&str; 2] = ["qt5ct", "qt6ct"];
const SCHEME_FILE: &str = "colors/crownos-theme.conf";

pub struct Qt;

impl Qt {
    fn tools(environment: &Environment) -> impl Iterator<Item = &'static str> + '_ {
        TOOLS
            .into_iter()
            .filter(|tool| process::is_on_path(tool) || environment.config_path(tool).is_dir())
    }
}

impl Target for Qt {
    fn name(&self) -> &str {
        "qt"
    }

    fn is_installed(&self) -> bool {
        TOOLS.into_iter().any(process::is_on_path)
    }

    fn apply(&self, theme: &Theme, environment: &Environment) -> Result<(), TargetError> {
        let scheme = color_scheme(&theme.palette);
        for tool in Self::tools(environment) {
            let dir = environment.config_path(tool);
            let scheme_path = dir.join(SCHEME_FILE);
            fs::write_if_changed(&scheme_path, &scheme)?;

            let scheme_path = scheme_path.to_string_lossy();
            let mut settings = vec![
                ("color_scheme_path", scheme_path.as_ref()),
                ("custom_palette", "true"),
            ];
            settings.extend(
                theme
                    .icon_theme
                    .as_deref()
                    .map(|icons| ("icon_theme", icons)),
            );
            fs::update(&dir.join(format!("{tool}.conf")), |text| {
                Ok(ini::set_keys(text, "Appearance", &settings))
            })?;
        }
        Ok(())
    }
}

/// Active, disabled and inactive colors in `QPalette::ColorRole` order.
fn color_scheme(palette: &Palette) -> String {
    let p = palette;
    let light = p.background.mix(Rgba::WHITE, 0.3);
    let dark = p.background.mix(Rgba::BLACK, 0.4);
    let active = [
        p.foreground,
        p.surface,
        light,
        p.background.mix(light, 0.5),
        dark,
        p.background.mix(dark, 0.5),
        p.foreground,
        Rgba::WHITE,
        p.foreground,
        p.surface,
        p.background,
        Rgba::BLACK,
        p.accent,
        p.on_accent,
        p.accent,
        p.accent_alt,
        p.background_alt,
        Rgba::BLACK,
        p.overlay,
        p.foreground,
        p.muted,
    ];
    let disabled = active.map(|color| color.mix(p.background, 0.5));
    let mut scheme = String::with_capacity(1024);
    scheme.push_str("[ColorScheme]");
    for (key, colors) in [
        ("active_colors", &active),
        ("disabled_colors", &disabled),
        ("inactive_colors", &active),
    ] {
        let _ = write!(scheme, "\n{key}=");
        for (index, color) in colors.iter().enumerate() {
            if index > 0 {
                scheme.push_str(", ");
            }
            let _ = color.write(ColorFormat::Argb, &mut scheme);
        }
    }
    scheme.push('\n');
    scheme
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mode::Mode;

    #[test]
    fn every_line_has_one_color_per_role() {
        let scheme = color_scheme(&Palette::resolve(Mode::Dark, |_| None));
        let lines: Vec<_> = scheme.lines().skip(1).collect();
        assert_eq!(lines.len(), 3);
        assert!(lines.iter().all(|line| line.split(", ").count() == 21));
        assert!(lines[0].starts_with("active_colors=#fffafafa, "));
    }
}
