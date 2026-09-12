# auto-wlr-randr

Automatic display configuration for Wayland compositors implementing the wlr-output-management protocol.

## About

auto-wlr-randr is a daemon that automatically monitors and configures connected displays in Wayland compositors. It detects when displays are connected or disconnected and applies the appropriate configuration profiles based on rules defined in the configuration file.

## Features

- Automatic detection of connected displays
- Profile-based configuration management
- Stands on shoulders of wlr-randr
- Command-line control utility
- Systemd integration

## Installation

### From Source

1. Clone this repository:

   ```bash
   git clone https://github.com/nikromen/auto-wlr-randr.git
   cd auto-wlr-randr
   ```

2. Build the project:

   ```bash
   cargo build --release
   ```

3. Install binaries and (optional) service:

   ```bash
   sudo cp target/release/auto-wlr-randr /usr/local/bin/
   sudo cp target/release/auto-wlr-randrctl /usr/local/bin/
   cp files/auto-wlr-randr.service ~/.config/systemd/user/auto-wlr-randr.service
   systemctl --user daemon-reload
   systemctl --user enable --now auto-wlr-randr.service
   ```

### From Crates.io

1. Install package:

   ```
   cargo install auto-wlr-randr
   ```

2. (optional) Install systemd service:

   Copy `files/auto-wlr-randr.service` file from the repository and run:

   ```bash
   systemctl --user daemon-reload
   systemctl --user enable --now auto-wlr-randr.service
   ```

### Fedora

If you're using Fedora, you can install an RPM package from Copr repository:

```bash
sudo dnf copr enable nikromen/auto-wlr-randr
sudo dnf install auto-wlr-randr
# or -git version which is built against main branch
sudo dnf install auto-wlr-randr-git
# enable the systemd service
systemctl --user enable --now auto-wlr-randr.service
```

## Manual

Read these:

```bash
man 1 auto-wlr-randr
man 5 auto-wlr-randr
man 1 auto-wlr-randrctl
```

Or read the short guilde below.

## Configuration

Create a configuration file at `~/.config/auto-wlr-randr/config.toml`.

For example configuration, take look at [example config file](./files/config.toml)

## Running as a Service

Enable and start the [user service](./files/auto-wlr-randr.service):

```bash
systemctl --user enable --now auto-wlr-randr.service
```

## Command-line Usage

### Daemon

```bash
auto-wlr-randr --config /path/to/config.toml
```

### Control Utility

```bash
# Show current status
auto-wlr-randrctl status

# Reload configuration
auto-wlr-randrctl reload

# Switch to a profile
auto-wlr-randrctl switch home-office
```

## Note

Inspired by [kanshi](https://gitlab.freedesktop.org/emersion/kanshi). auto-wlr-randr applies profiles through `wlr-randr` rather than the protocol directly, uses TOML configuration, and matches profiles by exact output count (see `man 5 auto-wlr-randr`). TOML and auto-wlr-randrctl are there to support tooling such as [wayrandr](https://github.com/nikromen/wayrandr).
