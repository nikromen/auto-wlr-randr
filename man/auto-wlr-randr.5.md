% AUTO-WLR-RANDR(5) auto-wlr-randr | File Formats Manual
%
% August 2025

# NAME

auto-wlr-randr - configuration file for auto-wlr-randr

# DESCRIPTION

The configuration file for **auto-wlr-randr** is written in TOML format and defines display
profiles that can be automatically applied when certain outputs are connected.

# CONFIGURATION FILE FORMAT

The configuration file consists of profile definitions, each with its own settings for
different outputs.

## Profile Definition

Each profile is defined under the `profile` section with a unique identifier:

```toml
[profile.profile_id]
exec = ["command1", "command2"]  # Optional commands to run when profile is activated

[[profile.profile_id.settings]]
output = "Output Name or Pattern"
on = true|false
mode = "WIDTHxHEIGHT@RATE"
pos = "X,Y"
scale = SCALE_FACTOR
transform = "normal|90|180|270|flipped|flipped-90|flipped-180|flipped-270"
```

## Configuration Keys

### Profile Section

**exec**
: Array of commands to execute when the profile is activated. These are handled asynchronously.
You can also use a single command with semicolons to execute commands in sequence.

### Settings Section

Each profile contains one or more `settings` sections, each defining the configuration for a
specific output:

**output**
: Name of the output to configure. Can be either the output name (e.g., "DP-1") or the
manufacturer name (e.g., "Dell Inc. DELL XYZ"). Regular expressions are supported.

**on**
: Boolean indicating whether the output should be enabled (true) or disabled (false)

**mode**
: Display mode in the format "WIDTHxHEIGHT@RATE" (e.g., "1920x1080@144Hz"). The refresh rate
part is optional.

**pos**
: Position of the output relative to other outputs, in the format "X,Y" (e.g., "1920,0")

**scale**
: Scaling factor for the output (e.g., 1.0, 1.5, 2.0)

**transform**
: Display orientation/transformation. Valid values: normal, 90, 180, 270, flipped,
flipped-90, flipped-180, flipped-270

# EXAMPLES

```toml
[profile.home_office]
exec = ["hyprctl dispatch 'hyprexpo:expo' toggle"]

[[profile.home_office.settings]]
output = "Dell Inc. DELL XYZ ABC"
on = true
mode = "1920x1080@144Hz"

[[profile.home_office.settings]]
output = "DP-1"
on = true
mode = "1920x1080"
pos = "1920,0"
scale = 1.0

[profile.laptop_only]

[[profile.laptop_only.settings]]
output = "eDP-1"
on = true
mode = "1920x1080"
```

# FILES

_~/.config/auto-wlr-randr/config.toml_
: Default location for the configuration file

# SEE ALSO

**auto-wlr-randr**(1), **auto-wlr-randrctl**(1)

# BUGS

Please report bugs at: https://github.com/nikromen/auto-wlr-randr/issues

# COPYRIGHT

Copyright © 2025 Jiri Kyjovsky. License GPL-3.0-or-later: GNU GPL version 3 or later.
