//! `org.gnome.desktop.interface`, which GTK, libadwaita and the portal read.

use crate::error::TargetError;
use crate::util::process;

const INTERFACE: &str = "org.gnome.desktop.interface";

pub fn is_available() -> bool {
    process::is_on_path("gsettings")
}

pub fn set(key: &str, value: &str) -> Result<(), TargetError> {
    process::run("gsettings", ["set", INTERFACE, key, value])
}

/// For keys only newer GNOME schemas have, such as `accent-color` (GNOME 47).
pub fn set_if_supported(key: &str, value: &str) {
    if process::run("gsettings", ["writable", INTERFACE, key]).is_ok() {
        set(key, value).unwrap_or_default();
    }
}
