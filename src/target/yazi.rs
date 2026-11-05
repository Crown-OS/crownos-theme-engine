//! A `crownos` flavor, selected for both terminal backgrounds in `theme.toml`.

use std::borrow::Cow;
use std::path::Path;

use toml_edit::{DocumentMut, table, value};

use crate::environment::Environment;
use crate::error::TargetError;
use crate::target::Target;
use crate::theme::Theme;
use crate::util::{fs, process};

const TEMPLATE: &str = include_str!("../../templates/yazi.toml");
const FLAVOR: &str = "crownos";

pub struct Yazi;

impl Target for Yazi {
    fn name(&self) -> &str {
        "yazi"
    }

    fn is_installed(&self) -> bool {
        process::is_on_path("yazi")
    }

    fn apply(&self, theme: &Theme, environment: &Environment) -> Result<(), TargetError> {
        let dir = environment.config_path("yazi");
        fs::write_if_changed(
            &dir.join(format!("flavors/{FLAVOR}.yazi/flavor.toml")),
            &theme.render(TEMPLATE)?,
        )?;
        let theme_file = dir.join("theme.toml");
        fs::update(&theme_file, |text| select_flavor(text, &theme_file))?;
        Ok(())
    }
}

fn select_flavor<'a>(text: &'a str, path: &Path) -> Result<Cow<'a, str>, TargetError> {
    let invalid = |message: String| TargetError::Parse {
        path: path.to_path_buf(),
        message,
    };
    let mut document: DocumentMut = text.parse().map_err(|error| invalid(format!("{error}")))?;
    let flavor = document
        .entry("flavor")
        .or_insert(table())
        .as_table_like_mut()
        .ok_or_else(|| invalid("`flavor` is not a table".to_owned()))?;
    flavor.insert("dark", value(FLAVOR));
    flavor.insert("light", value(FLAVOR));
    let updated = document.to_string();
    Ok(if updated == text {
        Cow::Borrowed(text)
    } else {
        Cow::Owned(updated)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selects_the_flavor_and_keeps_the_rest() {
        let text = "[flavor]\ndark = \"dracula\" # mine\n\n[mgr]\ncwd = { fg = \"cyan\" }\n";
        let updated = select_flavor(text, Path::new("theme.toml")).expect("valid toml");
        assert!(updated.contains("dark = \"crownos\""));
        assert!(updated.contains("light = \"crownos\""));
        assert!(updated.contains("cwd = { fg = \"cyan\" }"));
        assert!(matches!(
            select_flavor(&updated, Path::new("theme.toml")),
            Ok(Cow::Borrowed(_))
        ));
        assert!(select_flavor("flavor = 1", Path::new("theme.toml")).is_err());
    }
}
