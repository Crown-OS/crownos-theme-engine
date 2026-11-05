use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("unknown template variable `{0}`")]
    UnknownVariable(String),
    #[error("unclosed `{{{{` at byte {0}")]
    Unclosed(usize),
}

#[derive(Debug, Error)]
pub enum TargetError {
    #[error("{}: {source}", path.display())]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{}: {message}", path.display())]
    Parse { path: PathBuf, message: String },
    #[error(transparent)]
    Template(#[from] TemplateError),
    #[error("`{program}` failed: {detail}")]
    Command { program: String, detail: String },
}

pub(crate) trait IoResultExt<T> {
    fn at(self, path: &Path) -> Result<T, TargetError>;
}

impl<T> IoResultExt<T> for io::Result<T> {
    fn at(self, path: &Path) -> Result<T, TargetError> {
        self.map_err(|source| TargetError::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}
