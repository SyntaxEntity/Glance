# Glance

![Rust](https://img.shields.io/badge/Rust-1.92%2B-blue?style=for-the-badge&logo=rust)
![GTK](https://img.shields.io/badge/GTK-4-blue?style=for-the-badge&logo=gnome)
![license](https://img.shields.io/badge/license-MIT-blue?style=for-the-badge)
![DeepSeek Assisted Development](https://img.shields.io/badge/DeepSeek%20Assisted-Development-brightgreen?style=for-the-badge&logo=deepseek)

低饱和护眼的 **Markdown + LaTeX** 阅读器，面向 GNOME / Linux 桌面的长文阅读场景。公式字体本地打包，完全离线；默认只读预览，需要时才进入编辑。

> 版本 `0.1.2` ｜ 状态：开发中 ｜ 已知限制见 [已知限制](#已知限制)

## 功能

- **Markdown 渲染**：标题、列表、表格、代码块、引用、图片
- **LaTeX 公式**：行内 `$...$` 与块级 `$$...$$`（KaTeX，离线字体，不依赖 CDN）
- **源码编辑**：GtkSourceView 语法高亮，默认关闭；`Ctrl+S` 保存
- **工作目录文件树**：侧栏只显示目录结构与 `.md` 文件
- **文档大纲**：按标题生成 TOC，点击跳转到锚点
- **护眼主题**：暖纸色低饱和配色，GTK 与 WebView 两端一致
- **导出 PDF**：通过 WebKit PrintOperation 写出 PDF（见[已知限制](#已知限制)）

## 安装

从 [Releases](https://github.com/SyntaxEntity/Glance/releases) 下载对应架构的包：

| 发行版 | 架构 | 文件 |
|---|---|---|
| Fedora / RHEL 系 | x86_64 | `glance-0.1.2-1.fc44.x86_64.rpm` |
| Fedora / RHEL 系 | aarch64 | `glance-0.1.2-1.fc44.aarch64.rpm` |
| Debian / Ubuntu 系 | amd64 | `glance_0.1.2-1_amd64.deb` |
| Debian / Ubuntu 系 | arm64 | `glance_0.1.2-1_arm64.deb` |

```bash
# Fedora / RHEL 系
sudo dnf install ./glance-0.1.2-1.fc44.x86_64.rpm

# Debian / Ubuntu 系
sudo apt install ./glance_0.1.2-1_amd64.deb
```

安装后可从桌面菜单启动，或直接执行 `glance`。

**运行依赖**：GTK 4 / libadwaita / WebKitGTK 6.0 / GtkSourceView 5。GNOME 桌面通常已自带，其他发行版请先确认版本（见[已知限制](#已知限制)）。

## 使用

首次使用：

1. 点左上角 **文件夹** 图标选择工作目录（例如本仓库的 `docs/`）
2. 在侧栏「文件」中选择 `.md` 文件
3. 需要修改时点右上角 **铅笔** 按钮切换到编辑模式（默认预览）

| 快捷键 / 操作 | 作用 |
|--------|------|
| `Ctrl+S` | 保存当前文件 |
| 点击大纲项 | 跳转到对应标题 |
| 右上角铅笔 | 切换编辑 / 预览（默认预览） |
| 右上角打印 | 导出 PDF |

工作目录、上次打开的文件等状态保存在 `$XDG_CONFIG_HOME/glance/config.json`（未设置 `XDG_CONFIG_HOME` 时为 `~/.config/glance/config.json`），无需手工编辑。

### LaTeX 示例

```markdown
行内：$E = mc^2$

块级：

$$
\int_{-\infty}^{\infty} e^{-x^2}\,\mathrm{d}x = \sqrt{\pi}
$$
```

## 构建（开发）

**构建依赖（Fedora / RHEL 系）**：

```bash
sudo dnf install gtk4-devel libadwaita-devel webkitgtk6.0-devel \
  gtksourceview5-devel gobject-introspection-devel glib2-devel \
  cairo-devel pango-devel gdk-pixbuf2-devel graphene-devel
```

**构建与运行**：

```bash
cargo build --release
./target/release/glance [file.md]

# 或经封装脚本（自动挑 release/debug 二进制）
./run.sh docs/welcome.md
```

**测试**：

```bash
cargo test
```

编辑器相关的 2 项测试需要 GTK 显示。无头环境用 Broadway 后端：

```bash
broadwayd -p 8085 :5 &
GDK_BACKEND=broadway BROADWAY_DISPLAY=:5 cargo test
```

打包与发布流程见 [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md)。

## 项目结构

```
src/
  main.rs        入口
  window.rs      主窗口 / 交互 / PDF 导出
  sidebar.rs     文件树 + 大纲
  editor.rs      GtkSourceView 编辑器
  viewer.rs      WebKit 预览
  outline.rs     标题大纲解析
  config.rs      工作目录等配置
resources/
  glance.css     GTK 护眼主题
  preview/       WebView 模板（marked.js + KaTeX）
docs/            示例笔记 + 开发文档
data/glance.desktop
data/icons/      应用图标（hicolor：SVG + 多尺寸 PNG）
packaging/       RPM / DEB 打包脚本
```

## 技术栈

| 组件 | 用途 |
|------|------|
| Rust + gtk4-rs（GTK 4.22） | 原生 GNOME UI |
| libadwaita（1.9） | HIG 风格控件 |
| WebKitGTK 6 | Markdown / LaTeX 渲染 |
| GtkSourceView 5 | 源码编辑与高亮 |
| marked.js | Markdown → HTML |
| KaTeX | LaTeX 公式 |

## 已知限制

- **平台**：开发与验证环境是 Fedora 44（Wayland 会话）。其他发行版未做实际运行验证。
- **依赖版本偏新**：需要 GTK 4.22 / libadwaita 1.9 / WebKitGTK 6.0 / GtkSourceView 5.18。Debian / Ubuntu 稳定版的库版本可能偏旧，装包前请先确认。
- **未验证**：Windows / macOS 未构建、未运行（依赖 GTK 与 WebKitGTK，无支持计划）。
- **未验证**：当前环境无法启动真实 GUI 会话，界面交互、快捷键与视觉效果未经真机操作验证；单元测试在无头 Broadway 后端下通过。
- **未验证**：PDF 导出代码路径完整，但未实际打开生成的 PDF 检查排版与分页效果。
- **功能缺口**：HTML 导出尚未实现（见[路线图](#路线图)）。
- **CI 只在 tag 时构建**：推送 `v*` tag 会触发 GitHub Actions 构建四个包并挂到 Release；普通提交与 PR **不会**触发测试。
- **依赖外部服务**：README 的 DeepSeek 徽章图片依赖 shields.io，离线环境不显示。这不影响应用本身的离线特性。
- **边界行为**：未打开任何文件时切到编辑模式，编辑器为空且无法保存（会提示「没有打开的文件」）；打开文件后正常。

## 路线图

- [ ] 多标签页
- [ ] 全文搜索
- [ ] 自定义主题切换
- [x] 导出 PDF（右上角打印按钮）
- [ ] 导出 HTML

## 项目文档

| 文档 | 内容 |
|---|---|
| [`PROJECT.md`](PROJECT.md) | 项目现状与架构（交接入口） |
| [`docs/DEVELOPMENT.md`](docs/DEVELOPMENT.md) | 构建、打包、发布流程与环境约束 |
| [`AGENTS.md`](AGENTS.md) | 面向 AI agent 的上手契约与项目红线 |
| [`docs/welcome.md`](docs/welcome.md) | 应用内示例笔记，可作为渲染效果参考 |
| [`THIRD_PARTY.md`](THIRD_PARTY.md) | 第三方组件与许可 |

## 贡献 / 反馈

本仓库由单人维护。问题与建议请开 [Issue](https://github.com/SyntaxEntity/Glance/issues)；提交 PR 前请先确认改动方向，避免大改动被拒。

## 许可证

MIT。详见 [LICENSE](LICENSE)。SPDX 标识：`MIT`。

第三方组件（均为 MIT）：

- [marked.js](https://github.com/markedjs/marked)
- [KaTeX](https://github.com/KaTeX/KaTeX)（含字体）

## DeepSeek Assisted Development

![DeepSeek Assisted Development](https://img.shields.io/badge/DeepSeek%20Assisted-Development-brightgreen?style=for-the-badge&logo=deepseek)

本仓库代码由 **DeepSeek** 辅助编写（vibecoding）。AI 生成的代码可能包含缺陷、过时用法或不完整实现，
请自行评估后使用与集成。
