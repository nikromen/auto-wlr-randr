Name:           auto-wlr-randr
Version:        1.0.0
Release:        %autorelease
Summary:        Automatic display configuration for Wayland compositors
License:        GPL-3.0-or-later
URL:            https://github.com/nikromen/auto-wlr-randr
Source0:        %{url}/archive/refs/tags/%{name}-%{version}.tar.gz

BuildRequires:  rust-packaging >= 21
BuildRequires:  gcc
BuildRequires:  cargo
BuildRequires:  systemd-rpm-macros
BuildRequires:  wayland-devel

Requires:       wlr-randr


%description
auto-wlr-randr is a daemon that automatically manages display configurations
for Wayland compositors implementing the wlr-output-management protocol.
It detects connected displays and applies appropriate configuration profiles,
making multi-monitor setups seamless in Wayland environments.


%prep
%autosetup


%build
%{cargo_build}


%install
%{cargo_install}
install -Dpm 0755 %{cargo_bin_path}/%{name} %{buildroot}%{_bindir}/%{name}
install -Dpm 0755 %{cargo_bin_path}/%{name}ctl %{buildroot}%{_bindir}/%{name}ctl

# Install systemd user unit file
mkdir -p %{buildroot}%{_userunitdir}
mkdir -p %{buildroot}%{_datadir}/auto-wlr-randr

# Install example config
install -Dpm 0644 config.toml %{buildroot}%{_datadir}/auto-wlr-randr/config.toml.example


%check
%{cargo_test}


%files
%license LICENSE
%doc README.md
%{_bindir}/%{name}
%{_bindir}/%{name}ctl
%{_userunitdir}/%{name}.service
%{_datadir}/auto-wlr-randr/config.toml.example


%changelog
%autochangelog
