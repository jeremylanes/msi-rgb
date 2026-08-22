Name:           msi-rgb
Version:        0.1.0
Release:        1%{?dist}
Summary:        Native Linux RGB keyboard controller for MSI Katana 15 HX laptops

License:        MIT OR Apache-2.0
URL:            https://github.com/luxa/msi-rgb
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  gtk4-devel
BuildRequires:  desktop-file-utils

Requires:       gtk4
Requires:       openrgb >= 0.9

%description
A modern, native Rust and GTK4 application for controlling 4-zone RGB keyboards
on MSI gaming laptops (specifically MSI Katana 15 HX B14WGK / MS-1565).
Features full per-zone color configuration, animated lighting effects, custom
profile persistence, startup profile automation, and CLI control.

%prep
%autosetup

%build
cargo build --release

%install
install -D -m 0755 target/release/%{name} %{buildroot}%{_bindir}/%{name}
install -D -m 0644 packaging/com.msi.mysticlight.rgb.desktop %{buildroot}%{_datadir}/applications/com.msi.mysticlight.rgb.desktop
install -D -m 0644 packaging/com.msi.mysticlight.rgb.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/com.msi.mysticlight.rgb.svg

%check
desktop-file-validate %{buildroot}%{_datadir}/applications/com.msi.mysticlight.rgb.desktop || true

%files
%{_bindir}/%{name}
%{_datadir}/applications/com.msi.mysticlight.rgb.desktop
%{_datadir}/icons/hicolor/scalable/apps/com.msi.mysticlight.rgb.svg

%changelog
* Sat Aug 22 2026 MSI Linux Community <dev@msi-rgb.linux> - 0.1.0-1
- Initial release for Fedora Linux 44 (Wayland/Hyprland)
