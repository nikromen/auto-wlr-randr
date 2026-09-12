% AUTO-WLR-RANDRCTL(1) auto-wlr-randrctl | General Commands Manual
%
% August 2025

# NAME

auto-wlr-randrctl - control utility for auto-wlr-randr daemon

# SYNOPSIS

**auto-wlr-randrctl** \[COMMAND\] \[ARGS\]

# DESCRIPTION

**auto-wlr-randrctl** is a control utility for the auto-wlr-randr daemon. It allows users to
interact with the daemon, check its status, reload configuration, and switch between profiles.

# COMMANDS

**reload**
: Reload the configuration file. Forces the daemon to reload its configuration file,
applying any changes made since the daemon was started or the config was last reloaded.

**status**
: Display current status information. Shows information about the currently active profile,
connected outputs, and daemon state.

**switch** _PROFILE_ \[**--force**\]
: Switch to a specific profile. The profile must match the currently connected outputs using
the same rules as automatic profile matching (see **auto-wlr-randr**(5)). If the profile does
not match, the command fails and no configuration is applied. With **--force**, the profile is
applied regardless of whether it matches current outputs; output patterns are still resolved
where possible.

**validate** **-c** _CONFIG_
: Validate a configuration file without starting the daemon or changing display settings.
Checks TOML structure, glob patterns, transform values, and other static issues. Warnings are
printed for suspicious but valid configurations. Exits with status 1 if validation errors are
found.

**dry-run** **-c** _CONFIG_
: Show what would be applied for the currently connected outputs. Prints JSON with the matched
profile, resolved output names, **wlr-randr** arguments, and profile **exec** commands. If no
profile matches, includes **on_no_match_exec** from the configuration. Does not require a
running daemon.

**-h, --help**
: Print help information

**-V, --version**
: Print version information

# EXAMPLES

**auto-wlr-randrctl status**
: Display current daemon status and active profile

**auto-wlr-randrctl reload**
: Reload the configuration file

**auto-wlr-randrctl switch home-office**
: Switch to the "home-office" profile if it matches current outputs

**auto-wlr-randrctl switch home-office --force**
: Switch to the "home-office" profile even if it does not match current outputs

**auto-wlr-randrctl validate -c ~/.config/auto-wlr-randr/config.toml**
: Validate a configuration file

**auto-wlr-randrctl dry-run -c ~/.config/auto-wlr-randr/config.toml**
: Preview profile matching and commands for current outputs

# SEE ALSO

**auto-wlr-randr**(1), **auto-wlr-randr**(5)

# BUGS

Please report bugs at: https://github.com/nikromen/auto-wlr-randr/issues

# COPYRIGHT

Copyright © 2025 Jiri Kyjovsky. License GPL-3.0-or-later: GNU GPL version 3 or later.
