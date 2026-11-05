use std::borrow::Cow;
use std::env;
use std::path::Path;

/// `~/rest` becomes `$HOME/rest`; any other path is borrowed back unchanged.
pub fn expand_home(path: &Path) -> Cow<'_, Path> {
    match (path.strip_prefix("~"), dirs::home_dir()) {
        (Ok(rest), Some(home)) => Cow::Owned(home.join(rest)),
        _ => Cow::Borrowed(path),
    }
}

/// Whether a GTK theme directory named `name` exists anywhere GTK looks.
pub fn is_gtk_theme_installed(name: &str) -> bool {
    let data_dirs =
        env::var_os("XDG_DATA_DIRS").unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    dirs::home_dir()
        .map(|home| home.join(".themes"))
        .into_iter()
        .chain(dirs::data_dir().map(|data| data.join("themes")))
        .chain(env::split_paths(&data_dirs).map(|data| data.join("themes")))
        .any(|themes| themes.join(name).is_dir())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_leading_tilde_component_expands() {
        assert!(matches!(expand_home(Path::new("/etc/x")), Cow::Borrowed(_)));
        assert!(matches!(
            expand_home(Path::new("~user/x")),
            Cow::Borrowed(_)
        ));
        assert!(!expand_home(Path::new("~/x")).starts_with("~"));
    }
}
