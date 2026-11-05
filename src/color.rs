//! Opaque-friendly 8-bit colors and the text forms app configs expect.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// How a color is spelled inside a config file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    /// `#rrggbb`
    Hash,
    /// `rrggbb`
    Bare,
    /// `r, g, b`
    Rgb,
    /// `#aarrggbb`
    Argb,
}

impl ColorFormat {
    /// The format named by a template suffix: `{{accent}}`, `{{accent.bare}}`, ...
    pub fn from_suffix(suffix: Option<&str>) -> Option<Self> {
        match suffix {
            None => Some(Self::Hash),
            Some("bare") => Some(Self::Bare),
            Some("rgb") => Some(Self::Rgb),
            Some("argb") => Some(Self::Argb),
            Some(_) => None,
        }
    }
}

impl Rgba {
    pub const BLACK: Self = Self::rgb(0x000000);
    pub const WHITE: Self = Self::rgb(0xffffff);

    pub const fn rgb(hex: u32) -> Self {
        Self {
            r: (hex >> 16) as u8,
            g: (hex >> 8) as u8,
            b: hex as u8,
            a: u8::MAX,
        }
    }

    /// `#rrggbb` or `#rrggbbaa`, with or without the `#`.
    pub fn parse(text: &str) -> Option<Self> {
        let digits = text.strip_prefix('#').unwrap_or(text);
        if !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let value = u32::from_str_radix(digits, 16).ok()?;
        match digits.len() {
            6 => Some(Self::rgb(value)),
            8 => Some(Self {
                a: value as u8,
                ..Self::rgb(value >> 8)
            }),
            _ => None,
        }
    }

    /// Linear blend toward `other`; `weight` 0 keeps `self`, 1 gives `other`.
    pub fn mix(self, other: Self, weight: f32) -> Self {
        let blend = |from: u8, to: u8| {
            (f32::from(from) + (f32::from(to) - f32::from(from)) * weight).round() as u8
        };
        Self {
            r: blend(self.r, other.r),
            g: blend(self.g, other.g),
            b: blend(self.b, other.b),
            a: u8::MAX,
        }
    }

    /// Composites a translucent color onto an opaque backdrop.
    pub fn over(self, backdrop: Self) -> Self {
        match self.a {
            u8::MAX => self,
            alpha => backdrop.mix(self, f32::from(alpha) / 255.0),
        }
    }

    pub fn write(self, format: ColorFormat, out: &mut impl fmt::Write) -> fmt::Result {
        let Self { r, g, b, a } = self;
        match format {
            ColorFormat::Hash => write!(out, "#{r:02x}{g:02x}{b:02x}"),
            ColorFormat::Bare => write!(out, "{r:02x}{g:02x}{b:02x}"),
            ColorFormat::Rgb => write!(out, "{r}, {g}, {b}"),
            ColorFormat::Argb => write!(out, "#{a:02x}{r:02x}{g:02x}{b:02x}"),
        }
    }
}

impl fmt::Display for Rgba {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.write(ColorFormat::Hash, formatter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn formatted(color: Rgba, format: ColorFormat) -> String {
        let mut out = String::new();
        color
            .write(format, &mut out)
            .expect("writing to a String cannot fail");
        out
    }

    #[test]
    fn parses_both_widths_and_rejects_garbage() {
        assert_eq!(Rgba::parse("#6d48e8"), Some(Rgba::rgb(0x6d48e8)));
        assert_eq!(Rgba::parse("ffffff24").map(|color| color.a), Some(0x24));
        assert_eq!(Rgba::parse("+12345"), None);
        assert_eq!(Rgba::parse("#12345"), None);
    }

    #[test]
    fn every_format_spells_the_same_color() {
        let color = Rgba::rgb(0x0a10ff);
        assert_eq!(formatted(color, ColorFormat::Hash), "#0a10ff");
        assert_eq!(formatted(color, ColorFormat::Bare), "0a10ff");
        assert_eq!(formatted(color, ColorFormat::Rgb), "10, 16, 255");
        assert_eq!(formatted(color, ColorFormat::Argb), "#ff0a10ff");
    }

    #[test]
    fn translucent_colors_flatten_onto_the_backdrop() {
        let half_white = Rgba {
            a: 0x80,
            ..Rgba::WHITE
        };
        assert_eq!(half_white.over(Rgba::BLACK), Rgba::rgb(0x808080));
        assert_eq!(Rgba::WHITE.over(Rgba::BLACK), Rgba::WHITE);
    }
}
