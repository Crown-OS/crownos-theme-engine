use std::env;
use std::str::FromStr;

use anyhow::{Context, Result, bail};
use crownos_theme_engine::{
    Engine, EngineConfig, Environment, Palette, SettingsChanges, Status, Theme,
};

const USAGE: &str = "usage: crownos-theme-engine [apply | watch | targets | variables]

  apply      write every app config for the current appearance (default)
  watch      apply, then re-apply whenever the appearance changes
  targets    list themed apps and whether they are installed
  variables  print every template variable with its current value";

enum Command {
    Apply,
    Watch,
    Targets,
    Variables,
}

impl FromStr for Command {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> Result<Self> {
        Ok(match name {
            "apply" => Self::Apply,
            "watch" => Self::Watch,
            "targets" => Self::Targets,
            "variables" => Self::Variables,
            _ => bail!("unknown command `{name}`\n\n{USAGE}"),
        })
    }
}

fn main() -> Result<()> {
    let command = env::args()
        .nth(1)
        .as_deref()
        .map_or(Ok(Command::Apply), str::parse)?;
    match command {
        Command::Apply => apply(),
        Command::Watch => watch(),
        Command::Targets => {
            for target in engine(EngineConfig::load())?.targets() {
                let state = if target.is_installed() {
                    "installed"
                } else {
                    "not installed"
                };
                println!("{:<12} {state}", target.name());
            }
            Ok(())
        }
        Command::Variables => {
            print_variables(&Theme::load(&EngineConfig::load()));
            Ok(())
        }
    }
}

fn engine(config: EngineConfig) -> Result<Engine> {
    let environment = Environment::detect().context("no XDG config directory for this user")?;
    Ok(Engine::new(config, environment))
}

fn apply() -> Result<()> {
    let config = EngineConfig::load();
    let theme = Theme::load(&config);
    let failures = report(&engine(config)?, &theme);
    if failures > 0 {
        bail!("{failures} target(s) failed");
    }
    Ok(())
}

fn watch() -> Result<()> {
    let changes = SettingsChanges::subscribe();
    let mut applied: Option<(EngineConfig, Theme)> = None;
    loop {
        let config = EngineConfig::load();
        let theme = Theme::load(&config);
        let unchanged = applied.as_ref().is_some_and(|(last_config, last_theme)| {
            *last_config == config && *last_theme == theme
        });
        if !unchanged {
            report(&engine(config.clone())?, &theme);
            applied = Some((config, theme));
        }
        changes.wait().context("the settings watcher stopped")?;
    }
}

/// Prints one line per target and returns how many failed.
fn report(engine: &Engine, theme: &Theme) -> usize {
    let outcomes = engine.apply(theme);
    for outcome in &outcomes {
        match &outcome.status {
            Status::Applied => println!("{:<12} applied", outcome.target),
            Status::NotInstalled => println!("{:<12} skipped (not installed)", outcome.target),
            Status::Failed(error) => eprintln!("{:<12} failed: {error}", outcome.target),
        }
    }
    outcomes
        .iter()
        .filter(|outcome| matches!(outcome.status, Status::Failed(_)))
        .count()
}

fn print_variables(theme: &Theme) {
    let mut value = String::new();
    for name in Theme::TEXT_VARIABLES.iter().chain(Palette::ROLES) {
        value.clear();
        theme.write_variable(name, &mut value);
        println!("{name:<16} {value}");
    }
}
