//! The color roles app configs are written from.
//!
//! Each role reads one CrownOS design token (`surface.bg`, `text.primary`, ...)
//! so a `theme.ron` edit reaches every app. A role whose token is unset falls
//! back to the stock crownui palette for the active mode. Terminal roles use
//! `terminal.*` tokens, which a theme may set to retune the ANSI colors.

use crownos_config::AccentColor;

use crate::color::Rgba;
use crate::mode::Mode;

macro_rules! palette {
    (
        tokens { $($role:ident: $token:literal, $dark:literal, $light:literal;)* }
        derived($mode:ident) { $($(#[$doc:meta])* $derived:ident = $value:expr;)* }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct Palette {
            $(pub $role: Rgba,)*
            $($(#[$doc])* pub $derived: Rgba,)*
        }

        impl Palette {
            /// Every role name, as used in templates.
            pub const ROLES: &[&str] = &[$(stringify!($role),)* $(stringify!($derived),)*];

            pub fn role(&self, name: &str) -> Option<Rgba> {
                match name {
                    $(stringify!($role) => Some(self.$role),)*
                    $(stringify!($derived) => Some(self.$derived),)*
                    _ => None,
                }
            }

            /// Resolves every role from `token`, falling back to the stock
            /// palette, and flattens translucent tokens onto the mode's backdrop.
            pub fn resolve(mode: Mode, token: impl Fn(&str) -> Option<Rgba>) -> Self {
                let backdrop = mode.pick(Rgba::BLACK, Rgba::WHITE);
                let pick = |name: &str, dark: u32, light: u32| {
                    token(name)
                        .unwrap_or(Rgba::rgb(mode.pick(dark, light)))
                        .over(backdrop)
                };
                let $mode = mode;
                $(let $role = pick($token, $dark, $light);)*
                Self {
                    $($derived: $value,)*
                    $($role,)*
                }
            }
        }
    };
}

palette! {
    tokens {
        background: "surface.bg", 0x0f0f10, 0xf3f3f3;
        background_alt: "surface.sunken", 0x121212, 0xf2f2f6;
        surface: "surface.raised", 0x171718, 0xffffff;
        overlay: "popover.bg", 0x1d1d1d, 0xffffff;
        border: "surface.border", 0x27272a, 0xdfdfe3;
        hover: "popover.hover_bg", 0x2a2a2a, 0xf1f0f7;
        foreground: "text.primary", 0xfafafa, 0x17151f;
        foreground_alt: "text.body", 0xd4d4d8, 0x403e4a;
        muted: "text.muted", 0x9f9f9f, 0x8b8996;
        disabled: "text.disabled", 0x5a5a60, 0xc2c0cb;
        accent: "accent.end", 0x6d48e8, 0x6d48e8;
        accent_alt: "accent.start", 0x8f6dfb, 0x8f6dfb;
        on_accent: "text.on_accent", 0xffffff, 0xffffff;
        success: "status.success", 0x4ade80, 0x159c4e;
        warning: "status.warning", 0xfbbf24, 0xb45409;
        danger: "status.danger", 0xf87171, 0xc22727;
        black: "terminal.black", 0x27272a, 0x17151f;
        red: "terminal.red", 0xf87171, 0xc22727;
        green: "terminal.green", 0x4ade80, 0x159c4e;
        yellow: "terminal.yellow", 0xfbbf24, 0xb45409;
        blue: "terminal.blue", 0x60a5fa, 0x2563eb;
        magenta: "terminal.magenta", 0xc084fc, 0x9333ea;
        cyan: "terminal.cyan", 0x22d3ee, 0x0e7490;
        white: "terminal.white", 0xd4d4d8, 0x78768a;
        bright_black: "terminal.bright_black", 0x52525b, 0x646473;
        bright_red: "terminal.bright_red", 0xfca5a5, 0xdc2626;
        bright_green: "terminal.bright_green", 0x86efac, 0x16a34a;
        bright_yellow: "terminal.bright_yellow", 0xfde68a, 0xca8a04;
        bright_blue: "terminal.bright_blue", 0x93c5fd, 0x3b82f6;
        bright_magenta: "terminal.bright_magenta", 0xd8b4fe, 0xa855f7;
        bright_cyan: "terminal.bright_cyan", 0x67e8f9, 0x0891b2;
        bright_white: "terminal.bright_white", 0xfafafa, 0x8b8996;
    }
    derived(mode) {
        /// The accent softened into the background, for selected text.
        selection = accent.mix(background, mode.pick(0.6, 0.75));
        /// The accent stop that reads best as text on the background.
        accent_text = mode.pick(accent_alt, accent);
    }
}

/// The `(start, end)` gradient stops of a named accent, as crownui paints them.
pub const fn accent_stops(accent: AccentColor) -> (Rgba, Rgba) {
    match accent {
        AccentColor::Purple => (Rgba::rgb(0x8f6dfb), Rgba::rgb(0x6d48e8)),
        AccentColor::Blue => (Rgba::rgb(0x5aa2ff), Rgba::rgb(0x2563eb)),
        AccentColor::Green => (Rgba::rgb(0x4ade80), Rgba::rgb(0x16a34a)),
        AccentColor::Orange => (Rgba::rgb(0xfb923c), Rgba::rgb(0xea580c)),
        AccentColor::Pink => (Rgba::rgb(0xf472b6), Rgba::rgb(0xdb2777)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unset_tokens_fall_back_to_the_stock_palette() {
        let dark = Palette::resolve(Mode::Dark, |_| None);
        let light = Palette::resolve(Mode::Light, |_| None);
        assert_eq!(dark.background, Rgba::rgb(0x0f0f10));
        assert_eq!(light.background, Rgba::rgb(0xf3f3f3));
    }

    #[test]
    fn tokens_override_and_translucency_is_flattened() {
        let palette = Palette::resolve(Mode::Dark, |token| match token {
            "surface.bg" => Rgba::parse("#101010"),
            "surface.border" => Rgba::parse("#ffffff80"),
            _ => None,
        });
        assert_eq!(palette.background, Rgba::rgb(0x101010));
        assert_eq!(palette.border, Rgba::rgb(0x808080));
    }

    #[test]
    fn every_role_name_resolves() {
        let palette = Palette::resolve(Mode::Dark, |_| None);
        assert!(
            Palette::ROLES
                .iter()
                .all(|name| palette.role(name).is_some())
        );
    }
}
