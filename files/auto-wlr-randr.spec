%global srcname auto-wlr-randr

%if 0%{?git_build}
%global pkg_name auto-wlr-randr-git
%else
%global pkg_name %srcname
%endif


Name:           %pkg_name
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

%if 0%{?git_build}
Provides:       auto-wlr-randr = %{version}-%{release}
Obsoletes:      auto-wlr-randr < %{version}-%{release}
%endif


%description
auto-wlr-randr is a daemon that automatically manages display configurations
for Wayland compositors implementing the wlr-output-management protocol.
It detects connected displays and applies appropriate configuration profiles,
making multi-monitor setups seamless in Wayland environments.
%if 0%{?git_build}

This is a development build from the main branch.
%endif


%prep
%autosetup


%build
cargo build --profile rpm --all-features


%install
mkdir -p %{buildroot}%{_bindir}
install -p -m 0755 target/rpm/auto-wlr-randr %{buildroot}%{_bindir}/%{srcname}
install -p -m 0755 target/rpm/auto-wlr-randrctl %{buildroot}%{_bindir}/%{srcname}ctl

# Install systemd user unit file
mkdir -p %{buildroot}%{_userunitdir}
install -m 644 files/auto-wlr-randr.service %{buildroot}%{_userunitdir}/auto-wlr-randr.service

# Install example config
install -Dpm 0644 files/config.toml %{buildroot}%{_datadir}/auto-wlr-randr/config.toml.example


%check
%{cargo_test}


%files
%license LICENSE
%doc README.md
%{_bindir}/%{srcname}
%{_bindir}/%{srcname}ctl
%{_datadir}/auto-wlr-randr/config.toml.example
%{_userunitdir}/%{srcname}.service


%changelog
%autochangelog
