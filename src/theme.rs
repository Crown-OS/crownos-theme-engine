//! The resolved look every target is written from.

use crownos_config::{AccentColor, Appearance, Theme as ThemeOverrides};

use crate::color::{ColorFormat, Rgba};
use crate::config::EngineConfig;
use crate::error::TemplateError;
use crate::mode::Mode;
use crate::palette::{Palette, accent_stops};
use crate::util::template;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub mode: Mode,
    pub accent: AccentColor,
    pub palette: Palette,
    pub ui_font: String,
    pub mono_font: String,
    pub icon_theme: Option<String>,
    pub cursor_theme: Option<String>,
    pub gtk_theme: Option<String>,
}

impl Theme {
    /// Template variables that are text rather than colors.
    pub const TEXT_VARIABLES: &[&str] = &[
        "mode",
        "accent_name",
        "ui_font",
        "mono_font",
        "icon_theme",
        "cursor_theme",
        "gtk_theme",
    ];

    /// The current CrownOS appearance, read from `appearance.ron` and `theme.ron`.
    pub fn load(config: &EngineConfig) -> Self {
        Self::from_settings(
            &crownos_config::load(Appearance::SECTION),
            &crownos_config::load(ThemeOverrides::SECTION),
            config,
        )
    }

    /// Layers the stock palette, the named accent, the custom accent gradient and
    /// the per-mode token edits, in the same order crownui does.
    pub fn from_settings(
        appearance: &Appearance,
        overrides: &ThemeOverrides,
        config: &EngineConfig,
    ) -> Self {
        let mode = Mode::from_dark_mode(appearance.dark_mode);
        let (accent_start, accent_end) = overrides
            .accent
            .as_ref()
            .and_then(|gradient| Some((Rgba::parse(&gradient.start)?, Rgba::parse(&gradient.end)?)))
            .unwrap_or_else(|| accent_stops(appearance.accent));
        let slots = overrides.slots(appearance.dark_mode);
        let palette = Palette::resolve(mode, |token| {
            slots
                .get(token)
                .and_then(|hex| Rgba::parse(hex))
                .or(match token {
                    "accent.start" => Some(accent_start),
                    "accent.end" => Some(accent_end),
                    _ => None,
                })
        });
        let named = |name: &str| (!name.is_empty()).then(|| name.to_owned());
        Self {
            mode,
            accent: appearance.accent,
            palette,
            ui_font: overrides.font.family.clone(),
            mono_font: overrides.font.mono.clone(),
            icon_theme: named(&appearance.icon_theme),
            cursor_theme: named(&appearance.cursor_theme),
            gtk_theme: config
                .gtk_theme
                .as_ref()
                .map(|pair| mode.pick(&pair.dark, &pair.light).clone()),
        }
    }

    /// The GNOME name of the accent, as `org.gnome.desktop.interface accent-color` spells it.
    pub const fn accent_name(&self) -> &'static str {
        match self.accent {
            AccentColor::Purple => "purple",
            AccentColor::Blue => "blue",
            AccentColor::Green => "green",
            AccentColor::Orange => "orange",
            AccentColor::Pink => "pink",
        }
    }

    /// Fills `{{variable}}` placeholders. Colors are `{{role}}` (`#rrggbb`),
    /// `{{role.bare}}`, `{{role.rgb}}` or `{{role.argb}}`; see
    /// [`Palette::ROLES`] and [`Self::TEXT_VARIABLES`].
    ///
    /// # Errors
    ///
    /// Fails on an unknown variable or an unclosed placeholder.
    pub fn render(&self, template: &str) -> Result<String, TemplateError> {
        template::render(template, |name, out| self.write_variable(name, out))
    }

    /// Appends the value of `name` to `out`; `false` when no such variable exists.
    pub fn write_variable(&self, name: &str, out: &mut String) -> bool {
        let text = match name {
            "mode" => self.mode.name(),
            "accent_name" => self.accent_name(),
            "ui_font" => &self.ui_font,
            "mono_font" => &self.mono_font,
            "icon_theme" => self.icon_theme.as_deref().unwrap_or_default(),
            "cursor_theme" => self.cursor_theme.as_deref().unwrap_or_default(),
            "gtk_theme" => self.gtk_theme.as_deref().unwrap_or_default(),
            _ => return self.write_color(name, out),
        };
        out.push_str(text);
        true
    }

    fn write_color(&self, name: &str, out: &mut String) -> bool {
        let (role, suffix) = name
            .split_once('.')
            .map_or((name, None), |(role, suffix)| (role, Some(suffix)));
        match (self.palette.role(role), ColorFormat::from_suffix(suffix)) {
            (Some(color), Some(format)) => color.write(format, out).is_ok(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use crownos_config::schema::theme::AccentGradient;

    use super::*;

    fn theme(appearance: &Appearance, overrides: &ThemeOverrides) -> Theme {
        Theme::from_settings(appearance, overrides, &EngineConfig::default())
    }

    #[test]
    fn named_accent_then_gradient_then_token_wins() {
        let appearance = Appearance {
            accent: AccentColor::Green,
            ..Appearance::default()
        };
        let mut overrides = ThemeOverrides::default();
        assert_eq!(
            theme(&appearance, &overrides).palette.accent,
            Rgba::rgb(0x16a34a)
        );

        overrides.accent = Some(AccentGradient {
            start: "#ff0000".into(),
            end: "#00ff00".into(),
        });
        assert_eq!(
            theme(&appearance, &overrides).palette.accent,
            Rgba::rgb(0x00ff00)
        );

        overrides.dark.insert("accent.end".into(), "#0000ff".into());
        assert_eq!(
            theme(&appearance, &overrides).palette.accent,
            Rgba::rgb(0x0000ff)
        );
    }

    #[test]
    fn mode_picks_the_matching_gtk_theme_and_token_set() {
        let appearance = Appearance {
            dark_mode: false,
            ..Appearance::default()
        };
        let mut overrides = ThemeOverrides::default();
        overrides.dark.insert("surface.bg".into(), "#000000".into());
        let light = theme(&appearance, &overrides);
        assert_eq!(light.gtk_theme.as_deref(), Some("adw-gtk3"));
        assert_eq!(light.palette.background, Rgba::rgb(0xf3f3f3));
    }

    #[test]
    fn renders_text_and_every_color_format() {
        let theme = theme(&Appearance::default(), &ThemeOverrides::default());
        let rendered = theme
            .render("{{mode}} {{accent}} {{accent.bare}} {{ accent.rgb }}")
            .expect("known variables");
        assert_eq!(rendered, "dark #6d48e8 6d48e8 109, 72, 232");
        assert!(theme.render("{{accent.hsl}}").is_err());
        let mut out = String::new();
        assert!(
            Theme::TEXT_VARIABLES
                .iter()
                .all(|name| theme.write_variable(name, &mut out))
        );
    }
}
