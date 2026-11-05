use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crownos_config::{AccentColor, Appearance, Theme as ThemeOverrides};
use crownos_theme_engine::target::{self, Target};
use crownos_theme_engine::{EngineConfig, Environment, ExternalCommands, Theme};

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("crownos-theme-{name}-{}", std::process::id()));
        fs::remove_dir_all(&root).unwrap_or_default();
        Self(root)
    }

    fn environment(&self) -> Environment {
        Environment {
            config_home: self.0.clone(),
            runtime_dir: None,
            commands: ExternalCommands::Skip,
        }
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().expect("nested path")).expect("create dirs");
        fs::write(path, contents).expect("seed file");
    }

    fn read(&self, relative: &str) -> String {
        fs::read_to_string(self.0.join(relative))
            .unwrap_or_else(|error| panic!("{relative}: {error}"))
    }

    fn snapshot(&self) -> BTreeMap<PathBuf, String> {
        fn walk(dir: &Path, files: &mut BTreeMap<PathBuf, String>) {
            for entry in fs::read_dir(dir).expect("readable dir").flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, files);
                } else {
                    files.insert(
                        path.clone(),
                        fs::read_to_string(&path).expect("utf-8 config"),
                    );
                }
            }
        }
        let mut files = BTreeMap::new();
        walk(&self.0, &mut files);
        files
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap_or_default();
    }
}

fn theme(dark_mode: bool) -> Theme {
    let appearance = Appearance {
        dark_mode,
        accent: AccentColor::Blue,
        icon_theme: "Papirus".into(),
        ..Appearance::default()
    };
    Theme::from_settings(
        &appearance,
        &ThemeOverrides::default(),
        &EngineConfig::default(),
    )
}

fn apply_all(theme: &Theme, environment: &Environment) {
    for target in target::builtin() {
        target
            .apply(theme, environment)
            .unwrap_or_else(|error| panic!("{}: {error}", target.name()));
    }
}

#[test]
fn every_builtin_target_writes_and_is_idempotent() {
    let scratch = Scratch::new("idempotent");
    scratch.write("kitty/kitty.conf", "font_size 12\n");
    scratch.write(
        "gtk-3.0/settings.ini",
        "[Settings]\ngtk-font-name=Inter 11\n",
    );
    scratch.write(
        "yazi/theme.toml",
        "[flavor]\ndark = \"dracula\"\n\n[mgr]\ncwd = { fg = \"cyan\" }\n",
    );
    scratch.write("qt6ct/qt6ct.conf", "[Fonts]\nfixed=JetBrains Mono\n");
    let environment = scratch.environment();
    let theme = theme(true);

    apply_all(&theme, &environment);
    let first = scratch.snapshot();
    apply_all(&theme, &environment);
    assert_eq!(
        first,
        scratch.snapshot(),
        "a second apply must change nothing"
    );

    let kitty = scratch.read("kitty/kitty.conf");
    assert!(kitty.starts_with("font_size 12\n"));
    assert_eq!(kitty.matches("include crownos-theme.conf").count(), 1);
    assert!(
        scratch
            .read("kitty/crownos-theme.conf")
            .contains("cursor #2563eb\n")
    );

    let gtk = scratch.read("gtk-3.0/settings.ini");
    assert!(gtk.contains("gtk-font-name=Inter 11\n"));
    assert!(gtk.contains("gtk-application-prefer-dark-theme=1\n"));
    assert!(gtk.contains("gtk-icon-theme-name=Papirus\n"));
    assert!(
        scratch
            .read("gtk-4.0/gtk.css")
            .contains("@define-color accent_bg_color #2563eb;")
    );

    let qt = scratch.read("qt6ct/qt6ct.conf");
    assert!(qt.contains("fixed=JetBrains Mono"));
    assert!(qt.contains("custom_palette=true"));
    assert!(
        !scratch.0.join("qt5ct").exists(),
        "absent tools get no config"
    );

    let yazi = scratch.read("yazi/theme.toml");
    assert!(yazi.contains("dark = \"crownos\"") && yazi.contains("cwd = { fg = \"cyan\" }"));
    let flavor = scratch.read("yazi/flavors/crownos.yazi/flavor.toml");
    flavor
        .parse::<toml_edit::DocumentMut>()
        .expect("flavor is valid TOML");

    assert!(
        scratch
            .read("nvim/colors/crownos.lua")
            .contains("vim.o.background = \"dark\"")
    );
    assert!(
        scratch
            .read("fish/crownos-theme.fish")
            .contains("fish_color_command 5aa2ff")
    );
    assert!(
        scratch
            .read("spicetify/Themes/CrownOS/color.ini")
            .contains("button-active      = 2563eb")
    );
}

#[test]
fn switching_mode_rewrites_in_place() {
    let scratch = Scratch::new("mode");
    let environment = scratch.environment();
    apply_all(&theme(true), &environment);
    apply_all(&theme(false), &environment);

    let css = scratch.read("gtk-3.0/gtk.css");
    assert_eq!(css.matches(">>> crownos-theme-engine >>>").count(), 1);
    assert!(css.contains("@define-color window_bg_color #f3f3f3;"));
    assert!(
        scratch
            .read("gtk-4.0/settings.ini")
            .contains("gtk-application-prefer-dark-theme=0")
    );
    assert!(
        scratch
            .read("nvim/colors/crownos.lua")
            .contains("vim.o.background = \"light\"")
    );
}

#[test]
fn template_targets_render_user_templates() {
    let scratch = Scratch::new("template");
    scratch.write(
        "templates/agent.json",
        "{ \"accent\": \"{{accent}}\", \"mode\": \"{{mode}}\" }\n",
    );
    let spec = crownos_theme_engine::TemplateSpec {
        name: "agent".into(),
        template: scratch.0.join("templates/agent.json"),
        output: scratch.0.join("agent/themes/crownos.json"),
        reload: Vec::new(),
    };
    target::TemplateTarget::from(spec)
        .apply(&theme(false), &scratch.environment())
        .expect("renders");
    assert_eq!(
        scratch.read("agent/themes/crownos.json"),
        "{ \"accent\": \"#2563eb\", \"mode\": \"light\" }\n"
    );
}
