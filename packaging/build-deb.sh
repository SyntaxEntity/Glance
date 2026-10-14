#!/usr/bin/env bash
# Build glance DEB from the release binary (no extra build system).
# Produces packaging/deb/glance_<version>_amd64.deb
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)"
NAME=glance
ARCH=amd64
DEB_VER="${VERSION}-1"
OUT_DIR="$ROOT/packaging/deb"
STAGE="$OUT_DIR/${NAME}_${DEB_VER}_${ARCH}"
DEB_FILE="$OUT_DIR/${NAME}_${DEB_VER}_${ARCH}.deb"
BIN="$ROOT/target/release/glance"

if [[ ! -x "$BIN" ]]; then
  echo "==> Building release binary"
  (cd "$ROOT" && cargo build --release --locked)
fi

echo "==> Staging $STAGE"
rm -rf "$STAGE"
mkdir -p "$STAGE"/{DEBIAN,usr/bin,usr/share/doc/glance}

install -Dpm 0755 "$BIN" "$STAGE/usr/bin/glance"
install -Dpm 0644 "$ROOT/resources/glance.css" \
  "$STAGE/usr/share/glance/glance.css"

mkdir -p "$STAGE/usr/share/glance/preview/fonts"
install -Dpm 0644 "$ROOT"/resources/preview/*.{html,css,js} \
  "$STAGE/usr/share/glance/preview/"
install -Dpm 0644 "$ROOT"/resources/preview/fonts/* \
  "$STAGE/usr/share/glance/preview/fonts/"

install -Dpm 0644 "$ROOT/data/glance.desktop" \
  "$STAGE/usr/share/applications/glance.desktop"

install -Dpm 0644 "$ROOT/data/icons/hicolor/scalable/apps/glance.svg" \
  "$STAGE/usr/share/icons/hicolor/scalable/apps/glance.svg"
for size in 16 24 32 48 64 128 256 512; do
  install -Dpm 0644 "$ROOT/data/icons/hicolor/${size}x${size}/apps/glance.png" \
    "$STAGE/usr/share/icons/hicolor/${size}x${size}/apps/glance.png"
done

install -Dpm 0644 "$ROOT/LICENSE" "$STAGE/usr/share/doc/glance/copyright"
install -Dpm 0644 "$ROOT/README.md" "$STAGE/usr/share/doc/glance/README.md"
install -Dpm 0644 "$ROOT/THIRD_PARTY.md" "$STAGE/usr/share/doc/glance/THIRD_PARTY.md"
gzip -9n -c "$ROOT/README.md" > "$STAGE/usr/share/doc/glance/README.md.gz"
rm -f "$STAGE/usr/share/doc/glance/README.md"

# Runtime Depends: Debian/Ubuntu shared-library package names.
# GLib/GTK4/libadwaita/WebKitGTK6/GtkSourceView5 names follow current Debian sid / Ubuntu 25.04+.
cat > "$STAGE/DEBIAN/control" <<EOF
Package: ${NAME}
Version: ${DEB_VER}
Section: utils
Priority: optional
Architecture: ${ARCH}
Maintainer: Esperanta <esperanta@users.noreply.github.com>
Homepage: https://github.com/SyntaxEntity/Glance
Depends: libc6, libglib2.0-0, libgtk-4-1, libadwaita-1-0, libwebkitgtk-6.0-4, libgtksourceview-5-0
Recommends: hicolor-icon-theme
Description: Calm Markdown and LaTeX reader for GNOME
 Glance is a low-saturation, eye-care Markdown and LaTeX reader for the
 GNOME desktop. It renders Markdown with marked.js and math with KaTeX
 inside WebKitGTK, offers optional source editing via GtkSourceView,
 and browses a work directory as a file tree with a heading outline.
EOF

cat > "$STAGE/DEBIAN/copyright" <<'EOF'
Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/
Upstream-Name: glance
Source: https://github.com/SyntaxEntity/Glance

Files: *
Copyright: 2026 Esperanta
License: MIT
 Permission is hereby granted, free of charge, to any person obtaining a
 copy of this software and associated documentation files (the "Software"),
 to deal in the Software without restriction, including without limitation
 the rights to use, copy, modify, merge, publish, distribute, sublicense,
 and/or sell copies of the Software, and to permit persons to whom the
 Software is furnished to do so, subject to the following conditions:
 .
 The above copyright notice and this permission notice shall be included
 in all copies or substantial portions of the Software.
 .
 THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL
 THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
 FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 DEALINGS IN THE SOFTWARE.
EOF

# Required ownership/permissions for dpkg-deb
find "$STAGE" -type d -exec chmod 755 {} +
chmod 755 "$STAGE/usr/bin/glance"
chmod 644 "$STAGE/DEBIAN/control"
chmod 644 "$STAGE/DEBIAN/copyright"

SIZE_KB="$(du -sk "$STAGE/usr" | cut -f1)"
cat > "$STAGE/DEBIAN/md5sums" <<EOF
$(cd "$STAGE" && find usr -type f -exec md5sum {} +)
EOF
cat > "$STAGE/DEBIAN/Installed-Size" <<EOF
${SIZE_KB}
EOF
# Installed-Size belongs in control, not a separate file
sed -i "/^Architecture:/a Installed-Size: ${SIZE_KB}" "$STAGE/DEBIAN/control"
rm -f "$STAGE/DEBIAN/Installed-Size"

echo "==> Building $DEB_FILE"
rm -f "$DEB_FILE"
fakeroot dpkg-deb --build --root-owner-group "$STAGE" "$DEB_FILE"

echo "==> Verifying"
dpkg-deb --info "$DEB_FILE"
dpkg-deb --contents "$DEB_FILE" | head -40

echo "==> Artifact"
echo "$DEB_FILE"
