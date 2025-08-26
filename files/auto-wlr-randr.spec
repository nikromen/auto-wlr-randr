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
BuildRequires:  pandoc

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

# Generate man pages from markdown
mkdir -p man/man1 man/man5
pandoc -s -f markdown -t man man/auto-wlr-randr.1.md -o man/man1/auto-wlr-randr.1
pandoc -s -f markdown -t man man/auto-wlr-randrctl.1.md -o man/man1/auto-wlr-randrctl.1
pandoc -s -f markdown -t man man/auto-wlr-randr.5.md -o man/man5/auto-wlr-randr.5


%install
mkdir -p %{buildroot}%{_bindir}
install -p -m 0755 target/rpm/auto-wlr-randr %{buildroot}%{_bindir}/%{srcname}
install -p -m 0755 target/rpm/auto-wlr-randrctl %{buildroot}%{_bindir}/%{srcname}ctl

# Install systemd user unit file
mkdir -p %{buildroot}%{_userunitdir}
install -m 644 files/auto-wlr-randr.service %{buildroot}%{_userunitdir}/auto-wlr-randr.service

# Install example config
install -Dpm 0644 files/config.toml %{buildroot}%{_datadir}/auto-wlr-randr/config.toml.example

# Install man pages
mkdir -p %{buildroot}%{_mandir}/man1
mkdir -p %{buildroot}%{_mandir}/man5
install -m 644 man/man1/auto-wlr-randr.1 %{buildroot}%{_mandir}/man1/
install -m 644 man/man1/auto-wlr-randrctl.1 %{buildroot}%{_mandir}/man1/
install -m 644 man/man5/auto-wlr-randr.5 %{buildroot}%{_mandir}/man5/


%check
%{cargo_test}


%files
%license LICENSE
%doc README.md
%{_bindir}/%{srcname}
%{_bindir}/%{srcname}ctl
%{_datadir}/auto-wlr-randr/config.toml.example
%{_userunitdir}/%{srcname}.service
%{_mandir}/man1/auto-wlr-randr.1*
%{_mandir}/man1/auto-wlr-randrctl.1*
%{_mandir}/man5/auto-wlr-randr.5*


%changelog
%autochangelog
