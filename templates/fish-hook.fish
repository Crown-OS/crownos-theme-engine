# Installed by crownos-theme-engine; disable the "fish" target to remove theming.
status is-interactive; or return

function __crownos_theme --on-variable __crownos_theme_serial
    set -l theme $__fish_config_dir/crownos-theme.fish
    test -r $theme; and source $theme
end

# Applied at the first prompt so it wins over themes set earlier in conf.d.
function __crownos_theme_init --on-event fish_prompt
    functions --erase __crownos_theme_init
    __crownos_theme
end
