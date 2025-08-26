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

**switch** _PROFILE_
: Switch to a specific profile. Changes the current output configuration to the specified
profile defined in the configuration file.

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
: Switch to the "home-office" profile defined in the config file

# SEE ALSO

**auto-wlr-randr**(1), **auto-wlr-randr**(5)

# BUGS

Please report bugs at: https://github.com/nikromen/auto-wlr-randr/issues

# COPYRIGHT

Copyright © 2025 Jiri Kyjovsky. License GPL-3.0-or-later: GNU GPL version 3 or later.
