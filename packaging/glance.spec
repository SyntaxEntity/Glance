# RPM package for Glance — Markdown + LaTeX reader (GNOME)
#
# Vendored, offline cargo build (no network at rpmbuild time).
# Build via:  ./packaging/build-rpm.sh
#
# Requires on the build host:
#   sudo dnf install rpm-build rpmdevtools gcc \
#     gtk4-devel libadwaita-devel webkitgtk6.0-devel gtksourceview5-devel \
#     glib2-devel cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel

Name:           glance
Version:        0.1.3
Release:        1%{?dist}
Summary:        Calm Markdown and LaTeX reader for GNOME
License:        MIT
URL:            https://github.com/SyntaxEntity/Glance
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  rust
BuildRequires:  cargo
BuildRequires:  gcc
BuildRequires:  make
BuildRequires:  gtk4-devel
BuildRequires:  libadwaita-devel
BuildRequires:  webkitgtk6.0-devel
BuildRequires:  gtksourceview5-devel
BuildRequires:  glib2-devel
BuildRequires:  cairo-devel
BuildRequires:  pango-devel
BuildRequires:  gdk-pixbuf2-devel
BuildRequires:  graphene-devel
BuildRequires:  pkgconf-pkg-config

Recommends:     hicolor-icon-theme

%description
Glance is a low-saturation, eye-care Markdown and LaTeX reader for the
GNOME desktop. It renders Markdown with marked.js and math with KaTeX
inside WebKitGTK, offers optional source editing via GtkSourceView,
and browses a work directory as a file tree with a heading outline.

%prep
%autosetup -n %{name}-%{version}

%build
export CARGO_HOME="%{_builddir}/.cargo"
# Tarball ships crates in vendor/ and .cargo/config.toml (offline).
cargo build --release --offline --locked --verbose

%install
install -Dpm 0755 target/release/glance %{buildroot}%{_bindir}/glance

install -Dpm 0644 resources/glance.css \
    %{buildroot}%{_datadir}/glance/glance.css

install -d %{buildroot}%{_datadir}/glance/preview/fonts
install -Dpm 0644 resources/preview/*.{html,css,js} \
    %{buildroot}%{_datadir}/glance/preview/
install -Dpm 0644 resources/preview/fonts/* \
    %{buildroot}%{_datadir}/glance/preview/fonts/

install -Dpm 0644 data/glance.desktop \
    %{buildroot}%{_datadir}/applications/glance.desktop

install -Dpm 0644 data/icons/hicolor/scalable/apps/glance.svg \
    %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/glance.svg
for size in 16 24 32 48 64 128 256 512; do
    install -Dpm 0644 data/icons/hicolor/${size}x${size}/apps/glance.png \
        %{buildroot}%{_datadir}/icons/hicolor/${size}x${size}/apps/glance.png
done

install -Dpm 0644 README.md %{buildroot}%{_docdir}/%{name}/README.md
install -Dpm 0644 THIRD_PARTY.md %{buildroot}%{_docdir}/%{name}/THIRD_PARTY.md

%files
%license LICENSE
%doc README.md THIRD_PARTY.md
%{_bindir}/glance
%{_datadir}/glance/
%{_datadir}/applications/glance.desktop
%{_datadir}/icons/hicolor/scalable/apps/glance.svg
%{_datadir}/icons/hicolor/*/apps/glance.png

%changelog
* Thu Oct 22 2026 SyntaxEntity <265272538+SyntaxEntity@users.noreply.github.com> - 0.1.3-1
- Fix "open folder" and "open file" buttons sharing the same icon
- Fix sidebar .md icon that fell back to a missing-image placeholder
- Report an error to the user when saving with no open file
- Keep settings when the config file has missing fields
- Add DeepSeek disclosure card; align README with the workspace layout
- Add CI building all four packages on tag, plus an MSRV pin job
- Move packaging build artifacts out of the source tree
- Clear cargo fmt drift and clippy warnings

* Wed Oct 14 2026 SyntaxEntity <265272538+SyntaxEntity@users.noreply.github.com> - 0.1.2-1
- Fill document shape in icon (was transparent)
- Retint icon to low-saturation eye-care palette

* Wed Oct 14 2026 SyntaxEntity <265272538+SyntaxEntity@users.noreply.github.com> - 0.1.1-1
- Add hicolor application icon (SVG + multi-size PNG)
- Use custom icon in desktop entry and window

* Wed Sep 23 2026 SyntaxEntity <265272538+SyntaxEntity@users.noreply.github.com> - 0.1.0-1
- Initial RPM release
