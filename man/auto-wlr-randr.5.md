% AUTO-WLR-RANDR(5) auto-wlr-randr | File Formats Manual
%
% August 2025

# NAME

auto-wlr-randr - configuration file for auto-wlr-randr

# DESCRIPTION

The configuration file for **auto-wlr-randr** is written in TOML format and defines display
profiles that can be automatically applied when certain outputs are connected.

# PROFILE MATCHING

When outputs are connected or disconnected, **auto-wlr-randr** evaluates profiles and activates
the first one that matches the current output configuration.

A profile matches when **all** of the following conditions are met:

1. The number of `settings` entries in the profile is equal to the number of currently
   connected outputs.
2. Each `output` pattern in the profile matches exactly one connected output.
3. No connected output is matched by more than one `settings` entry in the same profile.

This means a profile for two monitors is only activated when exactly two outputs are connected,
and a profile for three monitors is only activated when exactly three outputs are connected.
Define separate profiles for each output count you want to support (for example, laptop-only,
dual-monitor, and triple-monitor setups).

If no profile matches, the current display configuration is left unchanged and a warning is
logged. If **on_no_match_exec** is set in the configuration file, those commands are run
after no profile has matched for a short delay (currently 500 ms). The delay is cancelled
if a profile starts matching before it expires (for example while monitors are still being
connected to a dock).

Profiles are evaluated in the order they appear in the configuration file until one matches.
A matching profile configures every currently connected output listed in its **settings**
section. When multiple profiles could match the same output configuration, the first one
in the file is used.

If the same profile remains matched after outputs change (for example, swapping one monitor
for another while the output count stays the same), **auto-wlr-randr** re-applies that
profile with the updated output mapping.

# CONFIGURATION FILE FORMAT

The configuration file consists of optional top-level keys and profile definitions, each with
its own settings for different outputs.

```toml
on_no_match_exec = ["notify-send 'No matching display profile'"]

[profile.profile_id]
# ...
```

## Top-Level Keys

**on_no_match_exec**
: Array of shell commands to execute when no profile matches the currently connected outputs.
Commands are run asynchronously via **sh**(1), using the same semantics as profile **exec**.
Execution is delayed briefly so transient output changes (such as docking monitors one at a
time) do not trigger the hook prematurely. The hook is not run again for the same output
configuration until outputs change or the configuration is reloaded.

## Profile Definition

Each profile is defined under the `profile` section with a unique identifier:

```toml
[profile.profile_id]
exec = ["command1", "command2"]  # Optional commands to run when profile is activated

[[profile.profile_id.settings]]
output = "Output Name or Pattern"
on = true|false
mode = "WIDTHxHEIGHT@RATE"
preferred = true|false
pos = "X,Y"
left_of = "Other Output"
right_of = "Other Output"
above = "Other Output"
below = "Other Output"
scale = SCALE_FACTOR
transform = "normal|90|180|270|flipped|flipped-90|flipped-180|flipped-270"
adaptive_sync = true|false
```

## Configuration Keys

### Profile Section

**exec**
: Array of shell commands to execute when the profile is activated. Commands are run
asynchronously via **sh**(1). To run multiple commands in sequence within a single entry, separate
them with semicolons or use a script.

### Settings Section

Each profile contains one or more `settings` sections, each defining the configuration for a
specific output. All options mirror **wlr-randr**(1) arguments.

**output**
: Pattern to match the output. Each pattern in a profile must match a distinct connected
output. You can match against any of the following:

- **Output name**: The connector name (e.g., "DP-1", "eDP-1", "HDMI-A-1"). Connector names
  may change across reboots or when using USB-C docks.
- **Identifier**: `{make} {model} {serial}` or `{make} {model}` if no serial is available
  (e.g., `"Dell Inc. U2718Q"` or `"Dell Inc. U2718Q ABC123456"`).
- **Serial number**: The serial alone (e.g., `"ABC123456"`), useful when distinguishing
  between identical monitors.

Glob patterns are supported and work on all match targets (e.g., `"HDMI-*"`, `"Dell Inc.*"`,
or `"*ABC123456"`).

To find identifiers for your outputs, run:

```bash
wlr-randr --json | jq '.[] | {name, make, model, serial}'
```

**on**
: Whether the output should be enabled (**true**) or disabled (**false**). If omitted,
the output's enabled state is not changed.

**mode**
: Display mode in the format "WIDTHxHEIGHT@RATE" (e.g., "1920x1080@144Hz"). The refresh rate
part is optional. Should not be combined with **preferred** (a **wlr-randr** constraint;
not validated at parse time).

**preferred**
: When **true**, configures the output to use its preferred mode. Defaults to **false** if
omitted. Should not be combined with **mode**.

**pos**
: Absolute position of the output in the global coordinate space, in the format "X,Y"
(e.g., "1920,0"). Should not be combined with **left_of**, **right_of**, **above**, or
**below**.

**left_of**, **right_of**, **above**, **below**
: Position the output relative to another output. The value must be a connector name
(e.g., `"eDP-1"` or `"HDMI-A-1"`). Should not be combined with **pos** or with each other.

**scale**
: Scaling factor for the output (e.g., 1.0, 1.5, 2.0).

**transform**
: Display orientation/transformation. Valid values: normal, 90, 180, 270, flipped,
flipped-90, flipped-180, flipped-270

**adaptive_sync**
: Enables (**true**) or disables (**false**) adaptive synchronization (variable refresh
rate). If omitted, adaptive sync is not changed.

# EXAMPLES

## Basic Configuration

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

## Dual and Triple Monitor Profiles

When using exact output count matching, define a separate profile for each setup:

```toml
[profile.dual]
[[profile.dual.settings]]
output = "ABC123456"
on = true
mode = "2560x1440"
pos = "0,0"

[[profile.dual.settings]]
output = "DEF789012"
on = true
mode = "2560x1440"
pos = "2560,0"

[profile.triple]
[[profile.triple.settings]]
output = "ABC123456"
on = true
mode = "2560x1440"
pos = "0,0"

[[profile.triple.settings]]
output = "DEF789012"
on = true
mode = "2560x1440"
pos = "2560,0"

[[profile.triple.settings]]
output = "GHI345678"
on = true
mode = "1920x1080"
right_of = "DP-2"
```

## Relative Positioning

```toml
[[profile.office.settings]]
output = "eDP-1"
on = true

[[profile.office.settings]]
output = "Dell Inc. *"
on = true
mode = "2560x1440"
left_of = "eDP-1"
adaptive_sync = true
```

Check also the example [config.toml](https://github.com/nikromen/auto-wlr-randr/blob/main/files/config.toml)
for more examples.

# FILES

_~/.config/auto-wlr-randr/config.toml_
: Default location for the configuration file

# SEE ALSO

**auto-wlr-randr**(1), **auto-wlr-randrctl**(1), **wlr-randr**(1)

# BUGS

Please report bugs at: https://github.com/nikromen/auto-wlr-randr/issues

# COPYRIGHT

Copyright © 2025 Jiri Kyjovsky. License GPL-3.0-or-later: GNU GPL version 3 or later.
