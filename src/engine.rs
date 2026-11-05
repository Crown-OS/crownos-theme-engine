use crate::config::EngineConfig;
use crate::environment::Environment;
use crate::error::TargetError;
use crate::target::{self, Target, TemplateTarget};
use crate::theme::Theme;

#[derive(Debug)]
pub enum Status {
    Applied,
    NotInstalled,
    Failed(TargetError),
}

#[derive(Debug)]
pub struct Outcome<'a> {
    pub target: &'a str,
    pub status: Status,
}

/// The enabled targets and the directories they write to.
pub struct Engine {
    targets: Vec<Box<dyn Target>>,
    environment: Environment,
}

impl Engine {
    /// The built-in targets plus the configured template targets, minus the
    /// disabled ones.
    pub fn new(config: EngineConfig, environment: Environment) -> Self {
        let EngineConfig {
            disabled,
            templates,
            ..
        } = config;
        let custom = templates
            .into_iter()
            .map(|spec| Box::new(TemplateTarget::from(spec)) as Box<dyn Target>);
        let targets = target::builtin()
            .into_iter()
            .chain(custom)
            .filter(|target| !disabled.iter().any(|name| name == target.name()))
            .collect();
        Self {
            targets,
            environment,
        }
    }

    /// Registers an app supported outside this crate.
    #[must_use]
    pub fn with_target(mut self, target: impl Target + 'static) -> Self {
        self.targets.push(Box::new(target));
        self
    }

    pub fn targets(&self) -> impl Iterator<Item = &dyn Target> {
        self.targets.iter().map(AsRef::as_ref)
    }

    /// Applies `theme` to every target in order; one failing target does not
    /// stop the rest.
    pub fn apply(&self, theme: &Theme) -> Vec<Outcome<'_>> {
        self.targets()
            .map(|target| Outcome {
                target: target.name(),
                status: if !target.is_installed() {
                    Status::NotInstalled
                } else {
                    match target.apply(theme, &self.environment) {
                        Ok(()) => Status::Applied,
                        Err(error) => Status::Failed(error),
                    }
                },
            })
            .collect()
    }
}
