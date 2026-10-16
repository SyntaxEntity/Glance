# 开发、打包与发布

> 面向维护者与 agent。项目现状与架构见 [PROJECT.md](../PROJECT.md)，对外介绍见 [README.md](../README.md)。

---

## 1. 工具链

| 组件 | 用途 | 本机实测版本 |
|------|------|--------------|
| Rust / cargo | 编译 | 1.97 |
| gtk4-devel / libadwaita-devel | UI | GTK 4.22 / libadwaita 1.9 |
| webkitgtk6.0-devel | 预览渲染 | WebKitGTK 6.0 |
| gtksourceview5-devel | 编辑器 | GtkSourceView 5.18 |
| rpmbuild / rpmdevtools | RPM 打包 | — |
| dpkg / dpkg-dev / fakeroot | DEB 打包 | — |
| podman + qemu-user-static | aarch64 模拟 | Fedora 44 镜像 |
| `gh` | GitHub release | 2.97，账号 `SyntaxEntity` |

```bash
sudo dnf install rpm-build rpmdevtools gcc gtk4-devel libadwaita-devel \
  webkitgtk6.0-devel gtksourceview5-devel glib2-devel cairo-devel \
  pango-devel gdk-pixbuf2-devel graphene-devel \
  dpkg dpkg-dev fakeroot podman qemu-user-static
```

---

## 2. 打包脚本

| 脚本 | 参数 | 产物 | 耗时 | 说明 |
|------|------|------|------|------|
| `packaging/build-rpm.sh` | 无 | x86_64 RPM | 数分钟 | `cargo vendor` 后离线 rpmbuild |
| `packaging/build-rpm-arm.sh` | `IMAGE` 环境变量（默认 `fedora:44`） | aarch64 RPM | **约 25 分钟** | podman + qemu 容器内构建 |
| `packaging/build-deb.sh` | `[ARCH] [BIN]` | DEB | 秒级 | 可复用外部二进制 |

**离线保证**：两个 RPM 脚本都会把 crate 源码 `cargo vendor` 进 staging 目录，并写入 `.cargo/config.toml` 指向 `vendored-sources`，因此 rpmbuild 阶段**不需要网络**。（注：`cargo vendor` 自身在首次运行时可能需要网络填充本地 registry 缓存。）

### 2.1 x86_64 RPM

```bash
./packaging/build-rpm.sh
# 产物在 packaging/rpmbuild/RPMS/x86_64/
```

> 若 `/var/tmp` 不可写（容器、只读挂载环境），rpmbuild 会因无法创建 `%_tmppath` 而失败。换一个可写的临时目录：
>
> ```bash
> rpmbuild --define "_topdir $PWD/packaging/rpmbuild" \
>          --define "_tmppath $PWD/packaging/rpmbuild/tmp" \
>          --nodeps -ba packaging/rpmbuild/SPECS/glance.spec
> ```

### 2.2 aarch64 RPM

```bash
./packaging/build-rpm-arm.sh
# 产物在 packaging/arm-rpm/

# 换基础镜像：
IMAGE=fedora:45 ./packaging/build-rpm-arm.sh
```

脚本内部在 `--arch=arm64` 的 Fedora 容器里装依赖、`cargo vendor`、然后 `rpmbuild --target aarch64`。qemu 用户态模拟，慢是正常的。

### 2.3 DEB

```bash
./packaging/build-deb.sh                 # amd64，用 target/release/glance
./packaging/build-deb.sh arm64 <二进制>  # arm64，复用已有 aarch64 二进制
```

**arm64 DEB 不必二次编译**，直接从 aarch64 RPM 里取出二进制即可：

```bash
mkdir -p /tmp/arm-deb-extract && cd /tmp/arm-deb-extract
rpm2cpio /path/to/glance-*-aarch64.rpm | cpio -id
cd - && ./packaging/build-deb.sh arm64 /tmp/arm-deb-extract/usr/bin/glance
```

---

## 3. 发布流程

```bash
# 1. 构建四包
./packaging/build-rpm.sh
./packaging/build-deb.sh
./packaging/build-rpm-arm.sh
./packaging/build-deb.sh arm64 /tmp/arm-deb-extract/usr/bin/glance

# 2. 提交 + tag + push
git tag -a v0.1.x -m "..."
git push origin master && git push origin v0.1.x

# 3. 创建 / 更新 release
gh release create v0.1.x --title "..." --notes "..." <packages...>
gh release upload v0.1.x <packages...> --clobber
```

### 版本 bump 检查清单

四处都要改，漏一处就会产出不一致的包：

- [ ] `Cargo.toml` → `version`
- [ ] `packaging/glance.spec` → `Version:`
- [ ] `packaging/glance.spec` → `%changelog` 新增条目
- [ ] 提交 → tag → push → release notes → 四包上传

**`%changelog` 注意**：格式为 `* 星期 月 日 年 名字 <邮箱> - 版本-发布号`。**星期几必须与该日期真实对应**，否则 rpmbuild 报警告（例如 `2026-10-14` 是周三，必须写 `Wed`）。DEB 版本号由 `build-deb.sh` 从 `Cargo.toml` 读取，无需手工同步。

---

## 4. 环境约束与坑

1. **本机 git 远程当前不可用** —— SSH 报 `Bad owner or permissions on /etc/ssh/ssh_config.d/20-systemd-ssh-proxy.conf`，实测 `/etc/ssh/ssh_config.d/` 属主为 `nobody:nobody` 而非 `root:root`（2026-10-17 复核）。**不要自行发明绕行方案**（如 `ssh -F /dev/null`），需 root 修复，**上报 Atlas**。
2. **推送邮箱隐私保护**：author **和** committer 必须是 `SyntaxEntity <265272538+SyntaxEntity@users.noreply.github.com>`，否则 GH007 拒绝推送。
3. **依赖版本偏新**：Debian / Ubuntu 稳定版可能不满足 GTK 4.22 / libadwaita 1.9 / WebKitGTK 6.0 / GtkSourceView 5.18。DEB 的 `Depends` 按 Debian sid / Ubuntu 25.04+ 的包名写。
4. **打包产物均已在 `.gitignore`**：`packaging/rpmbuild/`、`packaging/deb/`、`packaging/arm-rpm/`。发布用的二进制**不进仓库**，只挂 release。
5. **无 CI**：`.github/` 不存在，四包全靠本机脚本。若要补 CI，ARM 用 GitHub 官方 `ubuntu-24.04-arm` runner 可彻底免掉 qemu 的 25 分钟流程。
6. **maintainer 身份已统一**：全部使用 `SyntaxEntity <265272538+SyntaxEntity@users.noreply.github.com>`，与 git 提交身份一致。
   - `packaging/glance.spec` 的 `%changelog`、`packaging/build-deb.sh` 生成的 `DEBIAN/control` 的 `Maintainer` 字段
   - 早期的 `Esperanta <esperanta@…>` 身份已清除（git 历史由 Atlas 改写，当前树由本项目清理）

---

## 5. 无头环境自检

开发机若无显示（容器、CI），GTK 会因为连不上 `DISPLAY`/`WAYLAND_DISPLAY` 而报一堆 Mesa/dconf 警告。用 Broadway 后端即可跑通 UI 相关测试：

```bash
broadwayd -p 8085 :5 &
GDK_BACKEND=broadway BROADWAY_DISPLAY=:5 cargo test
```

仅验证编译与纯逻辑，则 `cargo check` / `cargo test` 足够（`outline` 测试不需要显示）。
