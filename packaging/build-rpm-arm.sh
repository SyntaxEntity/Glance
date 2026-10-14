#!/usr/bin/env bash
# Build glance aarch64 RPM inside a Fedora container (qemu-user emulation).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
IMAGE="${IMAGE:-fedora:44}"
OUT="$ROOT/packaging/arm-rpm"
mkdir -p "$OUT"

echo "==> Pulling $IMAGE (aarch64)"
podman pull --arch=arm64 "$IMAGE"

echo "==> Building aarch64 RPM in container (emulated, may take several minutes)"
podman run --rm --arch=arm64 \
  --security-opt label=disable \
  -v "$ROOT:/src:ro" \
  -v "$OUT:/out" \
  -w /tmp \
  "$IMAGE" \
  bash -c '
    set -euo pipefail
    echo "==> Installing build dependencies"
    dnf install -y --setopt=install_weak_deps=False \
      rpm-build rpmdevtools gcc make \
      rust cargo \
      gtk4-devel libadwaita-devel webkitgtk6.0-devel gtksourceview5-devel \
      glib2-devel cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel \
      pkgconf-pkg-config rsync tar gzip findutils which \
      2>&1 | tail -20

    echo "==> Staging source"
    cp -a /src /tmp/glance-src
    chmod -R u+w /tmp/glance-src
    cd /tmp/glance-src
    rm -rf target packaging/rpmbuild packaging/deb packaging/arm-rpm .git .mimocode .idea 2>/dev/null || true

    VERSION="$(sed -n "s/^version = \"\(.*\)\"/\1/p" Cargo.toml | head -1)"
    NAME=glance
    TOP=/tmp/rpmbuild
    STAGE_PARENT="$(mktemp -d)"
    STAGE="$STAGE_PARENT/${NAME}-${VERSION}"
    TARBALL="$TOP/SOURCES/${NAME}-${VERSION}.tar.gz"

    echo "==> Preparing rpmbuild tree"
    mkdir -p "$TOP"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

    echo "==> Staging ${NAME}-${VERSION}"
    mkdir -p "$STAGE"
    rsync -a --exclude target --exclude vendor --exclude .git \
      --exclude .mimocode --exclude "packaging/rpmbuild" \
      --exclude "packaging/deb" --exclude "packaging/arm-rpm" \
      /tmp/glance-src/ "$STAGE/"

    echo "==> Vendoring crates"
    (cd "$STAGE" && cargo vendor vendor)
    mkdir -p "$STAGE/.cargo"
    cat > "$STAGE/.cargo/config.toml" <<EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

    echo "==> Creating tarball"
    tar -C "$STAGE_PARENT" -czf "$TARBALL" "$(basename "$STAGE")"
    rm -rf "$STAGE_PARENT"

    cp /tmp/glance-src/packaging/glance.spec "$TOP/SPECS/"
    sed -i "s/^Release:.*/Release:        1%{?dist}/" "$TOP/SPECS/glance.spec"

    echo "==> rpmbuild (aarch64)"
    rpmbuild --define "_topdir $TOP" --target aarch64 --nodeps -ba "$TOP/SPECS/glance.spec"

    echo "==> Copying artifacts"
    find "$TOP/RPMS" "$TOP/SRPMS" -type f -name "*.rpm" -exec cp -v {} /out/ \;
    ls -lh /out/
  '
