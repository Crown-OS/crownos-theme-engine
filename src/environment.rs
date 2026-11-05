use std::path::{Path, PathBuf};

/// Whether targets may run programs (reload signals, `gsettings`, `spicetify`)
/// after writing their files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalCommands {
    Run,
    Skip,
}

/// Where targets write, so the whole engine can be pointed at a scratch tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub config_home: PathBuf,
    pub runtime_dir: Option<PathBuf>,
    pub commands: ExternalCommands,
}

impl Environment {
    /// The session's XDG directories; `None` without a home directory.
    pub fn detect() -> Option<Self> {
        Some(Self {
            config_home: dirs::config_dir()?,
            runtime_dir: dirs::runtime_dir(),
            commands: ExternalCommands::Run,
        })
    }

    pub fn config_path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.config_home.join(relative)
    }

    pub fn runs_commands(&self) -> bool {
        self.commands == ExternalCommands::Run
    }
}
