#!/usr/bin/env bash
# Build glance RPM with vendored crates (offline rpmbuild).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)"
NAME=glance
TOP="$ROOT/packaging/rpmbuild"
STAGE_PARENT="$(mktemp -d)"
STAGE="$STAGE_PARENT/${NAME}-${VERSION}"
TARBALL="$TOP/SOURCES/${NAME}-${VERSION}.tar.gz"

echo "==> Preparing rpmbuild tree"
rm -rf "$TOP"
mkdir -p "$TOP"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS}

echo "==> Staging source ${NAME}-${VERSION}"
mkdir -p "$STAGE"
rsync -a \
  --exclude 'target' \
  --exclude 'vendor' \
  --exclude '.git' \
  --exclude '.mimocode' \
  --exclude 'packaging/rpmbuild' \
  "$ROOT/" "$STAGE/"

echo "==> Vendoring crates (offline build)"
(
  cd "$STAGE"
  cargo vendor vendor
  mkdir -p .cargo
  cat > .cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF
)

echo "==> Creating tarball $TARBALL"
tar -C "$STAGE_PARENT" -czf "$TARBALL" "$(basename "$STAGE")"
rm -rf "$STAGE_PARENT"

cp "$ROOT/packaging/glance.spec" "$TOP/SPECS/"

echo "==> rpmbuild"
# --nodeps: host may use rustup (rust/cargo not provided as RPMs)
rpmbuild \
  --define "_topdir $TOP" \
  --nodeps \
  -ba "$TOP/SPECS/glance.spec"

echo "==> Artifacts"
find "$TOP/RPMS" "$TOP/SRPMS" -type f -name '*.rpm' -print
