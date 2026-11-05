use crate::environment::Environment;
use crate::error::TargetError;
use crate::target::Target;
use crate::theme::Theme;
use crate::util::managed_block::{self, Comment};
use crate::util::{fs, process};

const TEMPLATE: &str = include_str!("../../templates/kitty.conf");
const THEME_FILE: &str = "crownos-theme.conf";

pub struct Kitty;

impl Target for Kitty {
    fn name(&self) -> &str {
        "kitty"
    }

    fn is_installed(&self) -> bool {
        process::is_on_path("kitty")
    }

    fn apply(&self, theme: &Theme, environment: &Environment) -> Result<(), TargetError> {
        let dir = environment.config_path("kitty");
        let changed = fs::write_if_changed(&dir.join(THEME_FILE), &theme.render(TEMPLATE)?)?;
        fs::update(&dir.join("kitty.conf"), |config| {
            Ok(managed_block::upsert(
                config,
                Comment::Hash,
                &format!("include {THEME_FILE}"),
            ))
        })?;
        if changed && environment.runs_commands() {
            process::signal_all("kitty", "USR1")?;
        }
        Ok(())
    }
}
