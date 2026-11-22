//! Google Chrome and Chromium in the GTK theme mode.
//!
//! These browsers read their frame, toolbar, omnibox, selection, window button
//! and context menu colors from GTK 3 widget nodes and ignore the GTK dark
//! preference. Thus named colors are not sufficient: the rules set each node
//! directly. The `headerbar.header-bar` node, a text view that is a direct
//! child of a window and a menu in a window that is not a popup occur only in
//! the browser. The `window.background` rule also applies to GTK 3 apps, with the
//! same palette values. The browser reads the file at start, so a restart
//! applies a new theme.

use crate::environment::Environment;
use crate::error::TargetError;
use crate::target::Target;
use crate::theme::Theme;
use crate::util::managed_block::{self, Comment};
use crate::util::{fs, process};

const RULES: &str = include_str!("../../templates/chrome-gtk3.css");
const BROWSERS: [&str; 2] = ["google-chrome-stable", "chromium"];

pub struct Chrome;

impl Target for Chrome {
    fn name(&self) -> &str {
        "chrome"
    }

    fn is_installed(&self) -> bool {
        BROWSERS.iter().any(|browser| process::is_on_path(browser))
    }

    fn apply(&self, theme: &Theme, environment: &Environment) -> Result<(), TargetError> {
        let rules = theme.render(RULES)?;
        fs::update(&environment.config_path("gtk-3.0/gtk.css"), |css| {
            Ok(managed_block::upsert_named(
                css,
                Comment::Css,
                self.name(),
                &rules,
            ))
        })?;
        Ok(())
    }
}
