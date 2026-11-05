//! fish ≥ 4.3 keeps theme colors in global variables, so the theme is a script
//! sourced by a `conf.d` hook. Bumping a universal variable makes every running
//! shell re-source it.

use crate::environment::Environment;
use crate::error::TargetError;
use crate::target::Target;
use crate::theme::Theme;
use crate::util::{fs, process};

const TEMPLATE: &str = include_str!("../../templates/fish.fish");
const HOOK: &str = include_str!("../../templates/fish-hook.fish");

pub struct Fish;

impl Target for Fish {
    fn name(&self) -> &str {
        "fish"
    }

    fn is_installed(&self) -> bool {
        process::is_on_path("fish")
    }

    fn apply(&self, theme: &Theme, environment: &Environment) -> Result<(), TargetError> {
        let dir = environment.config_path("fish");
        let script = theme.render(TEMPLATE)?;
        let changed = fs::write_if_changed(&dir.join("crownos-theme.fish"), &script)?;
        fs::write_if_changed(&dir.join("conf.d/crownos-theme.fish"), HOOK)?;
        if changed && environment.runs_commands() {
            let serial = crownos_config::hash_bytes(script.as_bytes());
            process::run(
                "fish",
                [
                    "--no-config",
                    "-c",
                    &format!("set -U __crownos_theme_serial {serial}"),
                ],
            )?;
        }
        Ok(())
    }
}
