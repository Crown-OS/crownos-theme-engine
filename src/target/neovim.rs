//! A `crownos` colorscheme plus a `plugin/` hook that selects it. Users opt out
//! with `vim.g.crownos_theme = false`. Running instances showing the
//! colorscheme are refreshed over their RPC sockets; sockets that no longer
//! answer, such as those left behind by a crash, are skipped.

use std::ffi::OsStr;
use std::fs as std_fs;
use std::path::PathBuf;

use crate::environment::Environment;
use crate::error::TargetError;
use crate::target::Target;
use crate::theme::Theme;
use crate::util::{fs, process};

const COLORSCHEME: &str = include_str!("../../templates/nvim.lua");
const HOOK: &str = include_str!("../../templates/nvim-hook.lua");
const REFRESH: &str =
    r#"execute(get(g:, 'colors_name', '') ==# 'crownos' ? 'colorscheme crownos' : '')"#;

pub struct Neovim;

impl Neovim {
    /// Default `--listen` sockets: `$XDG_RUNTIME_DIR/nvim.<pid>.<n>`.
    fn sockets(environment: &Environment) -> Vec<PathBuf> {
        environment
            .runtime_dir
            .as_deref()
            .and_then(|dir| std_fs::read_dir(dir).ok())
            .into_iter()
            .flatten()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("nvim."))
            .map(|entry| entry.path())
            .collect()
    }
}

impl Target for Neovim {
    fn name(&self) -> &str {
        "neovim"
    }

    fn is_installed(&self) -> bool {
        process::is_on_path("nvim")
    }

    fn apply(&self, theme: &Theme, environment: &Environment) -> Result<(), TargetError> {
        let dir = environment.config_path("nvim");
        let changed =
            fs::write_if_changed(&dir.join("colors/crownos.lua"), &theme.render(COLORSCHEME)?)?;
        fs::write_if_changed(&dir.join("plugin/crownos-theme.lua"), HOOK)?;
        if changed && environment.runs_commands() {
            for socket in Self::sockets(environment) {
                let args = [
                    OsStr::new("--server"),
                    socket.as_os_str(),
                    OsStr::new("--remote-expr"),
                    OsStr::new(REFRESH),
                ];
                process::run("nvim", args).unwrap_or_default();
            }
        }
        Ok(())
    }
}
