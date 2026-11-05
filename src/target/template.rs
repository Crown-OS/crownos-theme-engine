//! Apps declared in `theme-engine.ron` instead of code, e.g. a terminal coding
//! agent's theme JSON:
//!
//! ```ron
//! templates: [(
//!     name: "opencode",
//!     template: "~/.config/crownos/templates/opencode.json",
//!     output: "~/.config/opencode/themes/crownos.json",
//! )],
//! ```

use std::fs as std_fs;

use crate::config::TemplateSpec;
use crate::environment::Environment;
use crate::error::{IoResultExt, TargetError};
use crate::target::Target;
use crate::theme::Theme;
use crate::util::{fs, paths, process};

pub struct TemplateTarget(TemplateSpec);

impl From<TemplateSpec> for TemplateTarget {
    fn from(spec: TemplateSpec) -> Self {
        Self(spec)
    }
}

impl Target for TemplateTarget {
    fn name(&self) -> &str {
        &self.0.name
    }

    fn apply(&self, theme: &Theme, environment: &Environment) -> Result<(), TargetError> {
        let template_path = paths::expand_home(&self.0.template);
        let template = std_fs::read_to_string(&template_path).at(&template_path)?;
        let changed = fs::write_if_changed(
            &paths::expand_home(&self.0.output),
            &theme.render(&template)?,
        )?;
        if changed
            && environment.runs_commands()
            && let Some((program, args)) = self.0.reload.split_first()
        {
            process::run(program, args)?;
        }
        Ok(())
    }
}
