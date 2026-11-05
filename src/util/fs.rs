//! Config file writes that never leave a half-written file and never touch
//! disk when nothing changed, so watchers in other apps stay quiet.

use std::borrow::Cow;
use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::error::{IoResultExt, TargetError};

pub fn read_or_empty(path: &Path) -> Result<String, TargetError> {
    match fs::read_to_string(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(String::new()),
        result => result.at(path),
    }
}

/// Atomically replaces the file behind `path`, following symlinks so dotfile
/// managers keep their links. Returns whether anything was written.
pub fn write_if_changed(path: &Path, contents: &str) -> Result<bool, TargetError> {
    let target = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    if fs::read(&target).is_ok_and(|current| current == contents.as_bytes()) {
        return Ok(false);
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).at(parent)?;
    }
    let temporary = temporary_sibling(&target);
    fs::write(&temporary, contents).at(&temporary)?;
    fs::rename(&temporary, &target).at(&target)?;
    Ok(true)
}

/// Rewrites a file through `edit`, which borrows the text back when it has
/// nothing to change.
pub fn update(
    path: &Path,
    edit: impl FnOnce(&str) -> Result<Cow<'_, str>, TargetError>,
) -> Result<bool, TargetError> {
    let text = read_or_empty(path)?;
    match edit(&text)? {
        Cow::Borrowed(_) => Ok(false),
        Cow::Owned(updated) => write_if_changed(path, &updated),
    }
}

fn temporary_sibling(path: &Path) -> PathBuf {
    let mut name = OsString::from(".");
    name.push(path.file_name().unwrap_or_default());
    name.push(".crownos-tmp");
    path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_identical_contents_and_creates_parents() {
        let dir = std::env::temp_dir().join(format!("crownos-fs-{}", std::process::id()));
        let file = dir.join("nested/app.conf");
        assert!(write_if_changed(&file, "a").expect("first write"));
        assert!(!write_if_changed(&file, "a").expect("same contents"));
        assert!(write_if_changed(&file, "b").expect("new contents"));
        assert_eq!(read_or_empty(&file).expect("readable"), "b");
        assert_eq!(
            read_or_empty(&dir.join("missing")).expect("missing is empty"),
            ""
        );
        fs::remove_dir_all(dir).expect("cleanup");
    }
}
