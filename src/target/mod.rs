//! One module per themed app.
//!
//! Adding an app means adding a module with a type implementing [`Target`] and
//! listing it in [`builtin`]. Apps that only need a rendered file can skip the
//! code entirely and be declared as a [`TemplateTarget`] in `theme-engine.ron`.

mod fish;
mod gtk;
mod icons;
mod kitty;
mod neovim;
mod qt;
mod spicetify;
mod template;
mod yazi;

pub use fish::Fish;
pub use gtk::Gtk;
pub use icons::Icons;
pub use kitty::Kitty;
pub use neovim::Neovim;
pub use qt::Qt;
pub use spicetify::Spicetify;
pub use template::TemplateTarget;
pub use yazi::Yazi;

use crate::environment::Environment;
use crate::error::TargetError;
use crate::theme::Theme;

pub trait Target {
    /// Stable identifier, used to disable the target in `theme-engine.ron`.
    fn name(&self) -> &str;

    /// Whether the app is present; targets for missing apps are skipped so
    /// they leave no stray config behind.
    fn is_installed(&self) -> bool {
        true
    }

    /// Writes the app's config for `theme` and, when the environment allows,
    /// asks running instances to reload. Must be idempotent: applying the same
    /// theme twice changes nothing on disk.
    fn apply(&self, theme: &Theme, environment: &Environment) -> Result<(), TargetError>;
}

pub fn builtin() -> Vec<Box<dyn Target>> {
    vec![
        Box::new(Gtk),
        Box::new(Icons),
        Box::new(Qt),
        Box::new(Kitty),
        Box::new(Fish),
        Box::new(Neovim),
        Box::new(Yazi),
        Box::new(Spicetify),
    ]
}
