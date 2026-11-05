#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    Dark,
    Light,
}

impl Mode {
    pub const fn from_dark_mode(dark_mode: bool) -> Self {
        if dark_mode { Self::Dark } else { Self::Light }
    }

    pub const fn name(self) -> &'static str {
        self.pick("dark", "light")
    }

    pub const fn is_dark(self) -> bool {
        matches!(self, Self::Dark)
    }

    pub const fn pick<T: Copy>(self, dark: T, light: T) -> T {
        match self {
            Self::Dark => dark,
            Self::Light => light,
        }
    }
}
