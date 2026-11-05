//! Keeps third-party app configs in step with the CrownOS appearance.
//!
//! The appearance chosen in crownos-settings (`appearance.ron`, `theme.ron`)
//! resolves into one [`Theme`]: a mode, a [`Palette`] of color roles, fonts and
//! icon themes. Every themed app is a [`Target`] that writes its own config
//! from that theme. Targets own generated files outright and touch user-owned
//! files only through marked blocks or single keys, so user edits survive.
//!
//! ```no_run
//! use crownos_theme_engine::{Engine, EngineConfig, Environment, Theme};
//!
//! let config = EngineConfig::load();
//! let theme = Theme::load(&config);
//! let environment = Environment::detect().expect("a home directory");
//! for outcome in Engine::new(config, environment).apply(&theme) {
//!     println!("{}: {:?}", outcome.target, outcome.status);
//! }
//! ```

pub mod color;
pub mod config;
pub mod engine;
pub mod environment;
pub mod error;
pub mod mode;
pub mod palette;
pub mod target;
pub mod theme;
mod util;
pub mod watch;

pub use color::{ColorFormat, Rgba};
pub use config::{EngineConfig, ModePair, TemplateSpec};
pub use engine::{Engine, Outcome, Status};
pub use environment::{Environment, ExternalCommands};
pub use error::{TargetError, TemplateError};
pub use mode::Mode;
pub use palette::Palette;
pub use target::Target;
pub use theme::Theme;
pub use watch::SettingsChanges;
