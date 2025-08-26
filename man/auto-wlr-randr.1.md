% AUTO-WLR-RANDR(1) auto-wlr-randr | General Commands Manual
%
% August 2025

# NAME

auto-wlr-randr - automatic display configuration for Wayland compositors

# SYNOPSIS

**auto-wlr-randr** \[OPTIONS\]

# DESCRIPTION

**auto-wlr-randr** is a daemon that automatically monitors and configures connected displays in
Wayland compositors that implement the wlr-output-management protocol. It detects when displays
are connected or disconnected and applies the appropriate configuration profiles based on rules
defined in the configuration file.

# OPTIONS

**-c, --config** _PATH_
: Path to the configuration file (default: ~/.config/auto-wlr-randr/config.toml)

**-l, --log-level** _LEVEL_
: Set log verbosity level (default: info). Possible values: trace, debug, info, warn, error

**-h, --help**
: Print help information

**-V, --version**
: Print version information

# USAGE

**auto-wlr-randr** is typically run as a systemd user service, but can also be run manually.
When launched, it monitors display changes and applies the appropriate profile from the
configuration file.

# FILES

_~/.config/auto-wlr-randr/config.toml_
: Default location for the configuration file

_~/.config/systemd/user/auto-wlr-randr.service_
: User-level systemd service file

# SEE ALSO

**auto-wlr-randrctl**(1), **auto-wlr-randr**(5)

# BUGS

Please report bugs at: https://github.com/nikromen/auto-wlr-randr/issues

# COPYRIGHT

Copyright © 2025 Jiri Kyjovsky. License GPL-3.0-or-later: GNU GPL version 3 or later.
