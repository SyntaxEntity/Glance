#!/usr/bin/env bash
# Build glance RPM with vendored crates (offline rpmbuild).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1)"
NAME=glance
# 构建产物统一放 target/packaging/（已在 .gitignore 的 /target 内），
# 源码树 packaging/ 只保留脚本与 spec。
TOP="$ROOT/target/packaging/rpmbuild"
STAGE_PARENT="$(mktemp -d)"
STAGE="$STAGE_PARENT/${NAME}-${VERSION}"
TARBALL="$TOP/SOURCES/${NAME}-${VERSION}.tar.gz"

echo "==> Preparing rpmbuild tree"
rm -rf "$TOP"
mkdir -p "$TOP"/{BUILD,BUILDROOT,RPMS,SOURCES,SPECS,SRPMS,tmp}

echo "==> Staging source ${NAME}-${VERSION}"
mkdir -p "$STAGE"
rsync -a \
  --exclude 'target' \
  --exclude 'vendor' \
  --exclude '.git' \
  --exclude '.mimocode' \
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
# _tmppath 指到构建目录内：避免依赖宿主的 /var/tmp（容器/只读环境下常常不可写）
rpmbuild \
  --define "_topdir $TOP" \
  --define "_tmppath $TOP/tmp" \
  --nodeps \
  -ba "$TOP/SPECS/glance.spec"

echo "==> Artifacts"
find "$TOP/RPMS" "$TOP/SRPMS" -type f -name '*.rpm' -print
